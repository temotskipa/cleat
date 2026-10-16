//! The class table: every type of the program with its resolved signatures.

use super::tir::Body;
use super::types::*;
use crate::Diagnostic;
use crate::ast;
use crate::lex::{self, Pos};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aud {
    Private,
    /// A type with no audience written: the file that declares it.
    File,
    Package,
    Protected,
    Public,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qualifier {
    No,
    Refines,
    Widens,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    Type,
    Annotation,
    Field,
    Method,
    Constructor,
    Parameter,
}

pub const SITE_NAMES: &[(&str, Site)] = &[
    ("TYPE", Site::Type),
    ("ANNOTATION", Site::Annotation),
    ("FIELD", Site::Field),
    ("METHOD", Site::Method),
    ("CONSTRUCTOR", Site::Constructor),
    ("PARAMETER", Site::Parameter),
];

/// An exact rational of any size.
pub type BigRational = num_rational::Ratio<num_bigint::BigInt>;

/// A value the compiler has computed: chapter 12's constants and annotation arguments.
#[derive(Clone, Debug, PartialEq)]
pub enum Const {
    /// An integer of the given numeric class.
    Int(i128, ClassId),
    Float(f64, ClassId),
    /// An exact rational, as numerator and denominator in lowest terms.
    Rational(BigRational),
    Bool(bool),
    Char(char),
    Str(String),
    Enum(FieldRef),
    Class(Type),
    Ann(AnnValue),
    Array(Vec<Const>),
}

/// One use of an annotation with its arguments evaluated, in element order.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnValue {
    pub class: ClassId,
    pub values: Vec<Const>,
}

#[derive(Clone, Debug)]
pub struct TParam {
    pub name: String,
    pub variance: ast::Variance,
    pub bounds: Vec<Type>,
    /// The tags a type argument must carry.
    pub tags: Vec<ClassId>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub is_static: bool,
    pub is_final: bool,
    pub aud: Aud,
    pub only: Option<Vec<ClassId>>,
    pub init: Option<ast::Expr>,
    pub ann_uses: Vec<ast::AnnotationUse>,
    pub anns: Vec<AnnValue>,
    pub enum_ordinal: Option<u32>,
    /// The arguments an enum constant passes to its constructor.
    pub enum_args: Vec<ast::Expr>,
    /// Set when the field is a constant field of section 12.1.
    pub constant: Option<Const>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub varargs: bool,
    pub is_final: bool,
    pub ann_uses: Vec<ast::AnnotationUse>,
    pub anns: Vec<AnnValue>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub name: String,
    pub tparams: Vec<TParam>,
    pub params: Vec<Param>,
    pub ret: Type,
    pub recv_nullable: bool,
    pub recv_quals: Vec<ClassId>,
    pub is_static: bool,
    pub is_open: bool,
    pub is_abstract: bool,
    pub is_final_written: bool,
    pub foreign: bool,
    pub intrinsic: bool,
    pub discardable: bool,
    pub has_override: bool,
    pub implicit: bool,
    pub narrows: Option<ClassId>,
    pub symbol: Option<String>,
    pub deprecated: Option<String>,
    pub aud: Aud,
    pub only: Option<Vec<ClassId>>,
    pub body: Option<ast::Block>,
    pub ann_uses: Vec<ast::AnnotationUse>,
    pub anns: Vec<AnnValue>,
    /// The methods of supertypes that this one overrides or implements.
    pub overrides: Vec<MethodRef>,
    /// A static method of an interface with no body: a requirement on implementers.
    pub static_requirement: bool,
    pub pos: Pos,
    pub checked: Option<Body>,
}

impl Method {
    /// Whether a call of the method selects its body by the receiver's class.
    pub fn is_virtual(&self, in_interface: bool) -> bool {
        !self.is_static
            && (self.is_open || self.is_abstract || (in_interface && !self.is_final_written))
    }
}

#[derive(Clone, Debug)]
pub struct Ctor {
    pub params: Vec<Param>,
    pub aud: Aud,
    pub only: Option<Vec<ClassId>>,
    pub body: Option<ast::Block>,
    /// The compact form of a value class, or the constructor a value class has without one.
    pub of_value: bool,
    /// Declared by the compiler because the class writes none.
    pub implicit: bool,
    pub intrinsic: bool,
    pub ann_uses: Vec<ast::AnnotationUse>,
    pub anns: Vec<AnnValue>,
    pub deprecated: Option<String>,
    pub pos: Pos,
    pub checked: Option<Body>,
}

#[derive(Clone, Debug)]
pub struct Element {
    pub name: String,
    pub ty: Type,
    pub default_src: Option<ast::AnnValue>,
    pub default: Option<Const>,
    pub pos: Pos,
}

/// What a lambda expression contributes when it becomes a class.
#[derive(Clone, Debug)]
pub struct LambdaClass {
    /// The class whose method contains the lambda. Access is checked as if from there.
    pub host: ClassId,
    /// The type variables of the enclosing context, in the order of the class's own
    /// type parameters.
    pub env: Vec<Tv>,
    pub captures_this: Option<Type>,
}

#[derive(Clone, Debug)]
pub struct Class {
    pub id: ClassId,
    pub unit: usize,
    pub name: String,
    pub package: String,
    pub qname: String,
    pub kind: ast::TypeKind,
    pub aud: Aud,
    pub is_open: bool,
    pub is_sealed: bool,
    pub is_abstract: bool,
    pub tparams: Vec<TParam>,
    pub superclass: Option<Type>,
    pub interfaces: Vec<Type>,
    pub permits: Option<Vec<ClassId>>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub ctors: Vec<Ctor>,
    pub static_inits: Vec<ast::Block>,
    pub static_init_checked: Option<Body>,
    pub elements: Vec<Element>,
    pub qualifier: Qualifier,
    /// For a refinement, the refinements directly above it.
    pub refines_above: Vec<ClassId>,
    pub targets: Option<Vec<Site>>,
    pub inherited: bool,
    pub ann_uses: Vec<ast::AnnotationUse>,
    pub anns: Vec<AnnValue>,
    pub deprecated: Option<String>,
    pub lambda: Option<LambdaClass>,
    pub pos: Pos,
    pub decl_index: usize,
}

impl Class {
    pub fn is_interface(&self) -> bool {
        self.kind == ast::TypeKind::Interface
    }

    pub fn is_value(&self) -> bool {
        matches!(
            self.kind,
            ast::TypeKind::ValueClass | ast::TypeKind::Annotation
        )
    }

    pub fn is_enum(&self) -> bool {
        self.kind == ast::TypeKind::Enum
    }

    pub fn is_annotation(&self) -> bool {
        self.kind == ast::TypeKind::Annotation
    }

    /// Whether no class may extend this one.
    pub fn is_final(&self) -> bool {
        !self.is_interface() && !self.is_open && !self.is_sealed && !self.is_abstract
    }
}

/// The prelude classes the compiler knows by name.
#[derive(Clone, Debug, Default)]
pub struct WellKnown {
    pub object: ClassId,
    pub unit: ClassId,
    pub boolean: ClassId,
    pub char_: ClassId,
    pub string: ClassId,
    pub int8: ClassId,
    pub int16: ClassId,
    pub int32: ClassId,
    pub int: ClassId,
    pub uint8: ClassId,
    pub uint16: ClassId,
    pub uint32: ClassId,
    pub uint64: ClassId,
    pub float32: ClassId,
    pub float64: ClassId,
    pub rational: ClassId,
    pub array: ClassId,
    pub iterable: ClassId,
    pub iterator: ClassId,
    pub throwable: ClassId,
    pub enum_: ClassId,
    pub class: ClassId,
    pub annotation: ClassId,
    pub site: ClassId,
    pub refines: ClassId,
    pub widens: ClassId,
    pub target: ClassId,
    pub inherited: ClassId,
    pub narrows: ClassId,
    pub override_: ClassId,
    pub discardable: ClassId,
    pub implicit: ClassId,
    pub deprecated: ClassId,
    pub intrinsic: ClassId,
    pub symbol: ClassId,
    pub pointer: ClassId,
    pub assertion_exception: ClassId,
    pub illegal_argument_exception: ClassId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumKind {
    Signed(u32),
    Unsigned(u32),
    Float(u32),
    Rational,
}

/// What is known about a captured wildcard or an inference variable.
#[derive(Clone, Debug, Default)]
pub struct VarInfo {
    pub upper: Vec<Type>,
    pub lower: Option<Type>,
}

pub struct Program {
    pub units: Vec<ast::Unit>,
    pub classes: Vec<Class>,
    /// Types by package and simple name. Several file-private types may share a name.
    pub by_name: HashMap<(String, String), Vec<ClassId>>,
    pub packages: HashSet<String>,
    pub wk: WellKnown,
    pub diags: Vec<Diagnostic>,
    /// Reports that do not reject the program: uses of deprecated declarations.
    pub warnings: Vec<Diagnostic>,
    /// The bounds of captured wildcards, by capture number.
    pub captures: Vec<VarInfo>,
    pub supertype_cache: HashMap<(ClassId, ClassId), Option<Type>>,
    /// Set once every class header is resolved, when bounds can be checked.
    pub headers_done: bool,
    pub deferred_type_checks: Vec<(Type, usize, Pos)>,
}

impl Program {
    pub fn error(&mut self, unit: usize, pos: Pos, message: impl Into<String>) {
        let u = &self.units[unit];
        let d = lex::diag(&u.file, &u.text, pos, message);
        // One message for one place: a second mistake found at the same position adds nothing.
        if !self
            .diags
            .iter()
            .any(|e| e.file == d.file && e.line == d.line && e.column == d.column)
        {
            self.diags.push(d);
        }
    }

    pub fn class(&self, id: ClassId) -> &Class {
        &self.classes[id as usize]
    }

    pub fn method(&self, m: MethodRef) -> &Method {
        &self.classes[m.class as usize].methods[m.index as usize]
    }

    pub fn field(&self, f: FieldRef) -> &Field {
        &self.classes[f.class as usize].fields[f.index as usize]
    }

    pub fn num_kind(&self, id: ClassId) -> Option<NumKind> {
        let w = &self.wk;
        Some(if id == w.int8 {
            NumKind::Signed(8)
        } else if id == w.int16 {
            NumKind::Signed(16)
        } else if id == w.int32 {
            NumKind::Signed(32)
        } else if id == w.int {
            NumKind::Signed(64)
        } else if id == w.uint8 {
            NumKind::Unsigned(8)
        } else if id == w.uint16 {
            NumKind::Unsigned(16)
        } else if id == w.uint32 {
            NumKind::Unsigned(32)
        } else if id == w.uint64 {
            NumKind::Unsigned(64)
        } else if id == w.float32 {
            NumKind::Float(32)
        } else if id == w.float64 {
            NumKind::Float(64)
        } else if id == w.rational {
            NumKind::Rational
        } else {
            return None;
        })
    }

    /// The numeric class of a type, when the type is not nullable.
    pub fn numeric_class(&self, t: &Type) -> Option<ClassId> {
        let id = t.class_id()?;
        self.num_kind(id).map(|_| id)
    }

    pub fn array_of(&self, elem: Type) -> Type {
        Type::class(self.wk.array, vec![Arg::Ty(elem)])
    }

    pub fn array_elem(&self, t: &Type) -> Option<Type> {
        match &t.ty {
            Ty::Class(id, args) if *id == self.wk.array => args[0].as_type().cloned(),
            _ => None,
        }
    }

    pub fn object(&self) -> Type {
        Type::simple(self.wk.object)
    }

    pub fn tparam(&self, tv: Tv) -> Option<&TParam> {
        match tv.owner {
            TvOwner::Class(c) => self.classes[c as usize].tparams.get(tv.index as usize),
            TvOwner::Method(m) => self.method(m).tparams.get(tv.index as usize),
            _ => None,
        }
    }

    /// The upper bounds of a type variable. With none written it is `Object?`.
    pub fn bounds_of(&self, tv: Tv) -> Vec<Type> {
        let written: Vec<Type> = match tv.owner {
            TvOwner::Capture(n) => self.captures[n as usize].upper.clone(),
            TvOwner::Infer(_) => Vec::new(),
            _ => self
                .tparam(tv)
                .map(|p| p.bounds.clone())
                .unwrap_or_default(),
        };
        if written.is_empty() {
            vec![self.object().nullable()]
        } else {
            written
        }
    }

    /// Whether a value of the type may be `null`.
    pub fn may_be_null(&self, t: &Type) -> bool {
        if t.nullable {
            return true;
        }
        match &t.ty {
            Ty::Null => true,
            Ty::Var(tv) => self.bounds_of(*tv).iter().all(|b| self.may_be_null(b)),
            _ => false,
        }
    }

    // ---- the supertype relation ----

    /// The direct supertypes of a class, over its own type parameters.
    pub fn direct_supers(&self, id: ClassId) -> Vec<Type> {
        let c = self.class(id);
        let mut out = Vec::new();
        if let Some(s) = &c.superclass {
            out.push(s.clone());
        }
        out.extend(c.interfaces.iter().cloned());
        // An interface with no superinterface still has the members of Object.
        if c.is_interface() && id != self.wk.object {
            out.push(self.object());
        }
        out
    }

    /// The type of a class over its own type parameters.
    pub fn own_type(&self, id: ClassId) -> Type {
        let args = (0..self.class(id).tparams.len())
            .map(|i| {
                Arg::Ty(Type::var(Tv {
                    owner: TvOwner::Class(id),
                    index: i as u32,
                }))
            })
            .collect();
        Type::class(id, args)
    }

    /// Whether a bound of an outer wildcard admits the inner argument's bound on the
    /// same side. An outer wildcard with no bound there admits anything, and one
    /// with a bound needs the inner bound to be within it.
    fn bound_admits(&mut self, outer: &Option<Type>, inner: &Option<Type>, upper: bool) -> bool {
        match (outer, inner) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(o), Some(i)) if upper => self.is_subtype(i, o),
            (Some(o), Some(i)) => self.is_subtype(o, i),
        }
    }

    /// The instantiation of `target` that the class `from` extends or implements, over
    /// the type parameters of `from`.
    fn generic_super(&mut self, from: ClassId, target: ClassId) -> Option<Type> {
        if from == target {
            return Some(self.own_type(from));
        }
        if let Some(hit) = self.supertype_cache.get(&(from, target)) {
            return hit.clone();
        }
        // Guard against a cycle in a broken hierarchy.
        self.supertype_cache.insert((from, target), None);
        let mut found = None;
        for sup in self.direct_supers(from) {
            let Ty::Class(sid, sargs) = &sup.ty else {
                continue;
            };
            if let Some(up) = self.generic_super(*sid, target) {
                found = Some(Subst::for_class(*sid, sargs).apply(&up));
                break;
            }
        }
        self.supertype_cache.insert((from, target), found.clone());
        found
    }

    /// The supertype of `t` that instantiates `target`, without qualifiers. A wildcard
    /// argument of `t` is read as its bound.
    pub fn supertype_at(&mut self, t: &Type, target: ClassId) -> Option<Type> {
        match &t.ty {
            Ty::Class(id, args) => {
                let up = self.generic_super(*id, target)?;
                Some(Subst::for_class(*id, args).apply(&up))
            }
            Ty::Var(tv) => {
                for b in self.bounds_of(*tv) {
                    if let Some(up) = self.supertype_at(&b, target) {
                        return Some(up);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn is_subclass(&mut self, sub: ClassId, sup: ClassId) -> bool {
        self.generic_super(sub, sup).is_some()
    }

    // ---- qualifiers ----

    /// Whether the refinement `low` is `high` or sits below it.
    pub fn refines(&self, low: ClassId, high: ClassId) -> bool {
        if low == high {
            return true;
        }
        self.class(low)
            .refines_above
            .clone()
            .iter()
            .any(|a| self.refines(*a, high))
    }

    fn quals_ok(&self, s: &Type, t: &Type) -> bool {
        // Every refinement on T is on S, or is above a refinement on S.
        for q in &t.quals {
            if self.class(*q).qualifier == Qualifier::Refines
                && !s.quals.iter().any(|sq| {
                    self.class(*sq).qualifier == Qualifier::Refines && self.refines(*sq, *q)
                })
            {
                return false;
            }
        }
        // Every widening on S is also on T.
        for q in &s.quals {
            if self.class(*q).qualifier == Qualifier::Widens && !t.quals.contains(q) {
                return false;
            }
        }
        true
    }

    // ---- subtyping ----

    pub fn is_subtype(&mut self, s: &Type, t: &Type) -> bool {
        if s.is_error() || t.is_error() {
            return true;
        }
        // Section 5.2: the null type is a subtype of `T?` for every `T`, and of no type
        // written without `?`.
        if s.is_null_literal() {
            return t.nullable;
        }
        if t.is_null_literal() {
            return false;
        }
        if !t.nullable {
            // T does not contain null, so S must not either.
            let t_excludes_null = !matches!(&t.ty, Ty::Var(_));
            if s.nullable {
                return false;
            }
            if t_excludes_null && self.may_be_null(s) {
                return false;
            }
        }
        if !self.quals_ok(s, t) {
            return false;
        }
        self.base_subtype(s, t)
    }

    fn base_subtype(&mut self, s: &Type, t: &Type) -> bool {
        match (&s.ty, &t.ty) {
            (Ty::Var(a), Ty::Var(b)) if a == b => true,
            (_, Ty::Var(b)) => {
                // Only a captured `? super L` accepts something that is not itself.
                if let TvOwner::Capture(n) = b.owner
                    && let Some(lower) = self.captures[n as usize].lower.clone()
                {
                    return self.is_subtype(&s.bare(), &lower.bare());
                }
                if let Ty::Var(a) = &s.ty {
                    let bounds = self.bounds_of(*a);
                    return bounds.iter().any(|bd| self.base_subtype(&bd.bare(), t));
                }
                false
            }
            (Ty::Var(a), Ty::Class(..)) => {
                let bounds = self.bounds_of(*a);
                bounds.iter().any(|bd| self.base_subtype(&bd.bare(), t))
            }
            (Ty::Class(_, _), Ty::Class(tid, targs)) => {
                let Some(up) = self.supertype_at(s, *tid) else {
                    return false;
                };
                let sargs = up.args().to_vec();
                let targs = targs.clone();
                for (i, (sa, ta)) in sargs.iter().zip(targs.iter()).enumerate() {
                    let variance = self.class(*tid).tparams[i].variance;
                    if !self.arg_contains(ta, sa, variance, *tid, i) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Whether two types are the same type: each is a subtype of the other.
    pub fn same_type(&mut self, a: &Type, b: &Type) -> bool {
        a == b || (self.is_subtype(a, b) && self.is_subtype(b, a))
    }

    /// The bound that `?` stands for as an argument of this parameter.
    pub fn param_bound(&self, class: ClassId, index: usize) -> Type {
        let p = &self.class(class).tparams[index];
        // A bound that mentions a type parameter of its own class has no closed form,
        // so the widest type stands in for it.
        match p.bounds.first() {
            Some(b) if !b.mentions(&|tv| tv.owner == TvOwner::Class(class)) => b.clone(),
            _ => self.object().nullable(),
        }
    }

    /// Whether the argument `outer` of a supertype contains the argument `inner` of a
    /// subtype, for a parameter of the given variance (sections 7.3 and 7.4).
    pub fn arg_contains(
        &mut self,
        outer: &Arg,
        inner: &Arg,
        variance: ast::Variance,
        class: ClassId,
        index: usize,
    ) -> bool {
        use ast::Variance::*;
        // Read each argument as a range: a lower limit and an upper limit on the type
        // it stands for. A plain type is both of its own limits.
        let bound = self.param_bound(class, index);
        let range = |a: &Arg| -> (Option<Type>, Option<Type>, bool) {
            match a {
                Arg::Ty(t) => (Some(t.clone()), Some(t.clone()), true),
                Arg::Wild(e, s) => (
                    s.as_ref().map(|t| (**t).clone()),
                    Some(
                        e.as_ref()
                            .map(|t| (**t).clone())
                            .unwrap_or_else(|| bound.clone()),
                    ),
                    false,
                ),
            }
        };
        let (olow, ohigh, oexact) = range(outer);
        let (ilow, ihigh, iexact) = range(inner);
        match variance {
            Out => {
                // Only the upper limits matter: G<A> is a subtype of G<B> when A is one of B.
                match (ihigh, ohigh) {
                    (Some(i), Some(o)) => self.is_subtype(&i, &o),
                    _ => false,
                }
            }
            In => {
                // `?` on an `in` parameter is the type of every instantiation.
                if matches!(outer, Arg::Wild(None, None)) {
                    return true;
                }
                if matches!(inner, Arg::Wild(None, None)) {
                    return false;
                }
                let o = match outer {
                    Arg::Ty(t) => t.clone(),
                    Arg::Wild(_, Some(l)) => (**l).clone(),
                    _ => return false,
                };
                let i = match inner {
                    Arg::Ty(t) => t.clone(),
                    Arg::Wild(_, Some(l)) => (**l).clone(),
                    _ => return false,
                };
                self.is_subtype(&o, &i)
            }
            Invariant => {
                if oexact {
                    // A type contains only itself.
                    if !iexact {
                        return false;
                    }
                    let (Some(o), Some(i)) = (ohigh, ihigh) else {
                        return false;
                    };
                    return self.same_type(&o, &i);
                }
                // The outer wildcard's range must enclose the inner range.
                self.bound_admits(&ohigh, &ihigh, true) && self.bound_admits(&olow, &ilow, false)
            }
        }
    }

    // ---- display ----

    pub fn show(&self, t: &Type) -> String {
        // An array is written as its element type and then its brackets, the outermost
        // first, as section 11.2 reads them.
        let mut levels = Vec::new();
        let mut base = t;
        while let Ty::Class(id, args) = &base.ty
            && *id == self.wk.array
            && args.len() == 1
            && let Arg::Ty(e) = &args[0]
        {
            levels.push(base);
            base = e;
        }
        let mut s = String::new();
        for q in &base.quals {
            s.push('@');
            s.push_str(&self.class(*q).name);
            s.push(' ');
        }
        self.show_base(base, &mut s);
        if base.nullable && !base.is_null_literal() {
            s.push('?');
        }
        for l in levels {
            for q in &l.quals {
                s.push_str(" @");
                s.push_str(&self.class(*q).name);
                s.push(' ');
            }
            s.push_str("[]");
            if l.nullable {
                s.push('?');
            }
        }
        s
    }

    fn show_base(&self, t: &Type, s: &mut String) {
        match &t.ty {
            Ty::Class(id, args) => {
                s.push_str(&self.class(*id).name);
                if !args.is_empty() {
                    s.push('<');
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            s.push_str(", ");
                        }
                        s.push_str(&self.show_arg(a));
                    }
                    s.push('>');
                }
            }
            Ty::Var(tv) => match tv.owner {
                TvOwner::Capture(_) => s.push_str("an unknown type"),
                TvOwner::Infer(_) => s.push('?'),
                _ => s.push_str(self.tparam(*tv).map(|p| p.name.as_str()).unwrap_or("?")),
            },
            Ty::Null => s.push_str("null"),
            Ty::Error => s.push_str("<error>"),
        }
    }

    pub fn show_arg(&self, a: &Arg) -> String {
        match a {
            Arg::Ty(t) => self.show(t),
            Arg::Wild(None, None) => "?".into(),
            Arg::Wild(Some(e), _) => format!("? extends {}", self.show(e)),
            Arg::Wild(None, Some(l)) => format!("? super {}", self.show(l)),
        }
    }
}
