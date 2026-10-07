//! Checks bodies: field initializers, methods, constructors and static initializers.
//! The result is the typed form of `tir`.

mod call;
mod expr;
pub mod konst;
mod stmt;

use super::decl::{self, TypeCx};
use super::program::*;
use super::tir::*;
use super::types::*;
use crate::ast;
use crate::lex::Pos;
use std::collections::{HashMap, HashSet};

/// What narrowing knows about locals: for each, the type it is known to have.
pub type Facts = Vec<(LocalId, Type)>;

#[derive(Clone, Debug, Default)]
pub struct Flow {
    /// The locals that are definitely assigned.
    pub assigned: HashSet<LocalId>,
    /// The locals that some path has assigned.
    pub maybe: HashSet<LocalId>,
    pub facts: HashMap<LocalId, Type>,
    /// The instance fields a constructor has definitely assigned, or the static fields
    /// a static initializer has.
    pub fields: HashSet<u32>,
    pub maybe_fields: HashSet<u32>,
    pub reachable: bool,
}

impl Flow {
    pub fn join(a: Flow, b: Flow) -> Flow {
        if !a.reachable {
            return b;
        }
        if !b.reachable {
            return a;
        }
        let mut facts = HashMap::new();
        for (k, v) in &a.facts {
            if b.facts.get(k) == Some(v) {
                facts.insert(*k, v.clone());
            }
        }
        Flow {
            assigned: a.assigned.intersection(&b.assigned).copied().collect(),
            maybe: a.maybe.union(&b.maybe).copied().collect(),
            facts,
            fields: a.fields.intersection(&b.fields).copied().collect(),
            maybe_fields: a.maybe_fields.union(&b.maybe_fields).copied().collect(),
            reachable: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocalMeta {
    pub is_final: bool,
    pub reassigned: bool,
    /// Where a lambda first used the local, if one did.
    pub captured_at: Option<Pos>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Label {
    pub name: Option<String>,
    pub id: u32,
    pub is_loop: bool,
    /// Whether a plain `break` may target it: a loop or a switch statement.
    pub breakable: bool,
    pub break_flows: Vec<Flow>,
    pub continue_flows: Vec<Flow>,
}

/// Where a lambda's captured value comes from in the enclosing body.
#[derive(Clone, Debug, PartialEq)]
pub enum CapSource {
    Local(LocalId),
    This,
}

#[derive(Clone, Debug)]
pub struct Capture {
    pub source: CapSource,
    pub field: u32,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct LambdaFrame {
    pub class: ClassId,
    pub captures: Vec<Capture>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtorPhase {
    /// Not in a constructor.
    None,
    /// Before `super(...)`: only the class's own fields may be assigned and read.
    Before,
    After,
}

/// One body being checked. A lambda's body is a frame inside its enclosing frame.
pub struct Frame {
    pub locals: Vec<LocalVar>,
    pub meta: Vec<LocalMeta>,
    pub params: Vec<LocalId>,
    pub scopes: Vec<Vec<(String, LocalId)>>,
    pub flow: Flow,
    pub labels: Vec<Label>,
    /// The result type that `return` must supply. `None` where `return` is not legal
    /// with a value: a constructor or an initializer.
    pub ret: Option<Type>,
    /// The type of `this` in this frame's own code, when it has one.
    pub this_ty: Option<Type>,
    pub lambda: Option<LambdaFrame>,
    pub ctor: CtorPhase,
    pub in_static_init: bool,
    /// The result types of `return` statements seen, for a lambda being inferred.
    pub returned: Vec<Type>,
}

pub struct Checker<'p> {
    pub p: &'p mut Program,
    pub unit: usize,
    /// The class whose code this is. Access is checked from here.
    pub class: ClassId,
    pub cx: TypeCx,
    pub is_static: bool,
    pub frames: Vec<Frame>,
    pub next_label: u32,
    pub next_lambda: u32,
    /// The method being checked, for its type parameters.
    pub method: Option<MethodRef>,
}

impl<'p> Checker<'p> {
    pub fn new(p: &'p mut Program, class: ClassId, is_static: bool) -> Checker<'p> {
        let unit = p.class(class).unit;
        let cx = TypeCx::for_class(p, class, !is_static);
        Checker { p, unit, class, cx, is_static, frames: Vec::new(), next_label: 0, next_lambda: 0, method: None }
    }

    pub fn err(&mut self, pos: Pos, message: impl Into<String>) {
        self.p.error(self.unit, pos, message);
    }

    pub fn error_expr(&mut self, pos: Pos, message: impl Into<String>) -> TExpr {
        self.err(pos, message);
        TExpr { kind: TKind::Null, ty: Type::error() }
    }

    pub fn frame(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    pub fn frame_ref(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    pub fn push_frame(&mut self, ret: Option<Type>, this_ty: Option<Type>) {
        self.frames.push(Frame {
            locals: Vec::new(),
            meta: Vec::new(),
            params: Vec::new(),
            scopes: vec![Vec::new()],
            flow: Flow { reachable: true, ..Flow::default() },
            labels: Vec::new(),
            ret,
            this_ty,
            lambda: None,
            ctor: CtorPhase::None,
            in_static_init: false,
            returned: Vec::new(),
        });
    }

    pub fn pop_frame(&mut self, stmts: Vec<TStmt>) -> Body {
        let f = self.frames.pop().unwrap();
        // A lambda may not use a local that is assigned after it is initialized.
        for (i, m) in f.meta.iter().enumerate() {
            if let (Some(at), true) = (m.captured_at, m.reassigned) {
                let name = f.locals[i].name.clone();
                self.err(at, format!("a lambda may use `{name}` only if it is never assigned after it is initialized"));
            }
        }
        Body { locals: f.locals, params: f.params, stmts }
    }

    // ---- locals and scopes ----

    pub fn declare(&mut self, name: &str, ty: Type, is_final: bool, pos: Pos, is_param: bool) -> LocalId {
        if !name.starts_with('$') && self.find_local(name).is_some() {
            self.err(pos, format!("`{name}` is already a variable of this method; a variable does not hide another"));
        }
        let f = self.frame();
        let id = f.locals.len() as LocalId;
        f.locals.push(LocalVar { name: name.to_string(), ty, is_param });
        f.meta.push(LocalMeta { is_final, reassigned: false, captured_at: None, pos });
        f.scopes.last_mut().unwrap().push((name.to_string(), id));
        if is_param {
            f.params.push(id);
            f.flow.assigned.insert(id);
            f.flow.maybe.insert(id);
        }
        id
    }

    /// A local the program did not write: a temporary.
    pub fn temp(&mut self, ty: Type) -> LocalId {
        let f = self.frame();
        let id = f.locals.len() as LocalId;
        f.locals.push(LocalVar { name: format!("$t{id}"), ty, is_param: false });
        f.meta.push(LocalMeta { is_final: true, reassigned: false, captured_at: None, pos: 0 });
        f.flow.assigned.insert(id);
        f.flow.maybe.insert(id);
        id
    }

    pub fn push_scope(&mut self) {
        self.frame().scopes.push(Vec::new());
    }

    pub fn pop_scope(&mut self) {
        self.frame().scopes.pop();
    }

    /// The frame and id of a variable in scope, innermost first.
    pub fn find_local(&self, name: &str) -> Option<(usize, LocalId)> {
        for (fi, f) in self.frames.iter().enumerate().rev() {
            for scope in f.scopes.iter().rev() {
                if let Some((_, id)) = scope.iter().rev().find(|(n, _)| n == name) {
                    return Some((fi, *id));
                }
            }
        }
        None
    }

    /// The type of `this` for the code being checked, whichever frame it is in.
    pub fn this_type(&self) -> Option<Type> {
        if self.is_static {
            return None;
        }
        self.frames.first().and_then(|f| f.this_ty.clone())
    }

    /// An expression for the enclosing method's `this`, from the current frame.
    pub fn this_expr(&mut self, pos: Pos) -> TExpr {
        let Some(ty) = self.this_type() else {
            return self.error_expr(pos, "`this` is not available in a static method");
        };
        let depth = self.frames.len() - 1;
        self.outer_value(depth, CapSource::This, ty)
    }

    /// Reads a value of an enclosing frame from frame `depth`, capturing it through
    /// every lambda in between.
    pub fn outer_value(&mut self, depth: usize, source: CapSource, ty: Type) -> TExpr {
        // Find the frame that owns the value.
        let owner = match &source {
            CapSource::This => 0,
            CapSource::Local(_) => unreachable!("use local_value for locals"),
        };
        self.captured(owner, depth, source, ty)
    }

    /// The value `source` of frame `owner`, as seen from frame `at`.
    pub fn captured(&mut self, owner: usize, at: usize, source: CapSource, ty: Type) -> TExpr {
        if owner == at {
            return match source {
                CapSource::This => TExpr { kind: TKind::This, ty },
                CapSource::Local(id) => TExpr { kind: TKind::Local(id), ty },
            };
        }
        // Frame `at` is a lambda. Its class holds the value in a field.
        let lambda_class = self.frames[at].lambda.as_ref().expect("an inner frame is a lambda").class;
        let existing = self.frames[at]
            .lambda
            .as_ref()
            .unwrap()
            .captures
            .iter()
            .find(|c| c.source == source_key(owner, at, &source))
            .map(|c| c.field);
        let field = match existing {
            Some(f) => f,
            None => {
                let field = self.p.class(lambda_class).fields.len() as u32;
                let name = match &source {
                    CapSource::This => "$this".to_string(),
                    CapSource::Local(id) => format!("${}", self.frames[owner].locals[*id as usize].name),
                };
                self.p.classes[lambda_class as usize].fields.push(Field {
                    name,
                    ty: ty.clone(),
                    is_static: false,
                    is_final: true,
                    aud: Aud::Private,
                    only: None,
                    init: None,
                    ann_uses: Vec::new(),
                    anns: Vec::new(),
                    enum_ordinal: None,
                    enum_args: Vec::new(),
                    constant: None,
                    pos: 0,
                });
                self.frames[at].lambda.as_mut().unwrap().captures.push(Capture {
                    source: source_key(owner, at, &source),
                    field,
                    ty: ty.clone(),
                });
                field
            }
        };
        let this_ty = self.frames[at].this_ty.clone().unwrap();
        TExpr {
            kind: TKind::Field(
                Box::new(TExpr { kind: TKind::This, ty: this_ty }),
                FieldRef { class: lambda_class, index: field },
            ),
            ty,
        }
    }

    // ---- flow ----

    pub fn apply_facts(&mut self, facts: &Facts) {
        for (id, ty) in facts {
            self.frame().flow.facts.insert(*id, ty.clone());
        }
    }

    pub fn new_label(&mut self) -> u32 {
        self.next_label += 1;
        self.next_label
    }

    // ---- types ----

    pub fn resolve(&mut self, t: &ast::TypeRef) -> Type {
        let cx = self.cx.clone();
        decl::resolve_type(self.p, &cx, t)
    }

    pub fn boolean(&self) -> Type {
        Type::simple(self.p.wk.boolean)
    }

    pub fn int(&self) -> Type {
        Type::simple(self.p.wk.int)
    }

    pub fn string(&self) -> Type {
        Type::simple(self.p.wk.string)
    }

    pub fn unit_ty(&self) -> Type {
        Type::simple(self.p.wk.unit)
    }

    pub fn show(&self, t: &Type) -> String {
        self.p.show(t)
    }

    /// A fresh capture variable for one wildcard.
    pub fn fresh_capture(&mut self, upper: Vec<Type>, lower: Option<Type>) -> Tv {
        let n = self.p.captures.len() as u32;
        self.p.captures.push(VarInfo { upper, lower });
        Tv { owner: TvOwner::Capture(n), index: 0 }
    }

    /// Replaces each wildcard argument of a type with an unknown type of its own.
    pub fn capture(&mut self, t: &Type) -> Type {
        let Ty::Class(id, args) = &t.ty else { return t.clone() };
        if !t.has_wildcard_arg() {
            return t.clone();
        }
        let id = *id;
        let mut out = Vec::new();
        for (i, a) in args.iter().enumerate() {
            match a {
                Arg::Ty(_) => out.push(a.clone()),
                Arg::Wild(e, s) => {
                    let mut upper: Vec<Type> = Vec::new();
                    if let Some(e) = e {
                        upper.push((**e).clone());
                    }
                    // The unknown is also within the parameter's own bounds, when those
                    // do not mention the class's parameters.
                    for b in self.p.class(id).tparams[i].bounds.clone() {
                        if !b.mentions(&|tv| tv.owner == TvOwner::Class(id)) {
                            upper.push(b);
                        }
                    }
                    let tv = self.fresh_capture(upper, s.as_ref().map(|t| (**t).clone()));
                    out.push(Arg::Ty(Type::var(tv)));
                }
            }
        }
        Type { ty: Ty::Class(id, out), nullable: t.nullable, quals: t.quals.clone() }
    }

    /// Writes a type without the unknown types of captured wildcards (section 7.4).
    pub fn project(&mut self, t: &Type) -> Type {
        if !t.mentions(&|tv| matches!(tv.owner, TvOwner::Capture(_))) {
            return t.clone();
        }
        match &t.ty {
            Ty::Var(tv) => {
                if let TvOwner::Capture(n) = tv.owner {
                    let info = self.p.captures[n as usize].clone();
                    let mut up = info.upper.first().cloned().unwrap_or_else(|| self.p.object().nullable());
                    up = self.project(&up);
                    up.nullable |= t.nullable;
                    for q in &t.quals {
                        up.add_qual(*q);
                    }
                    return up;
                }
                t.clone()
            }
            Ty::Class(id, args) => {
                let id = *id;
                let mut out = Vec::new();
                for (i, a) in args.iter().enumerate() {
                    let variance = self.p.class(id).tparams[i].variance;
                    out.push(match a {
                        Arg::Ty(at) => match &at.ty {
                            Ty::Var(tv) if matches!(tv.owner, TvOwner::Capture(_)) => {
                                let TvOwner::Capture(n) = tv.owner else { unreachable!() };
                                let info = self.p.captures[n as usize].clone();
                                match variance {
                                    ast::Variance::Out => Arg::Ty(self.project(at)),
                                    ast::Variance::In => match info.lower {
                                        Some(l) => Arg::Ty(l),
                                        None => Arg::Wild(None, None),
                                    },
                                    ast::Variance::Invariant => Arg::Wild(
                                        info.upper.first().map(|u| Box::new(u.clone())),
                                        info.lower.map(Box::new),
                                    ),
                                }
                            }
                            _ => Arg::Ty(self.project(at)),
                        },
                        Arg::Wild(e, s) => {
                            let e = e.as_ref().map(|t| Box::new(self.project(t)));
                            let s = s.as_ref().map(|t| Box::new(self.project(t)));
                            Arg::Wild(e, s)
                        }
                    });
                }
                Type { ty: Ty::Class(id, out), nullable: t.nullable, quals: t.quals.clone() }
            }
            _ => t.clone(),
        }
    }
}

/// Captures are keyed by where the value lives one frame out.
fn source_key(_owner: usize, _at: usize, source: &CapSource) -> CapSource {
    source.clone()
}

// ---- the driver ----

/// Checks every body of the program.
pub fn check_bodies(p: &mut Program) {
    konst::evaluate_annotations(p);
    let n = p.classes.len() as ClassId;
    for id in 0..n {
        stmt::check_class(p, id);
    }
}
