//! Expressions: chapters 4, 5 and 6, with the narrowing facts of section 12.4.

use super::call::{ArgIn, CallSite};
use super::konst::{parse_decimal, parse_int, Fold};
use super::*;
use crate::ast::{BinOp, ExprKind, UnOp};
use crate::sema::decl;
use num_rational::BigRational;

/// A checked condition with what it tells about locals on each outcome.
pub struct Cond {
    pub e: TExpr,
    pub t: Facts,
    pub f: Facts,
}

/// What stands before a `.`: a value, a type, a type parameter or a package.
pub enum Target {
    Value(TExpr),
    Type(ClassId),
    TypeVar(Tv),
    Package(String),
}

/// A location that an assignment stores into.
enum Place {
    Local(usize, LocalId),
    Field(TExpr, FieldRef, Type),
    Static(FieldRef, Type),
    /// The array or collection and its index, each already in a temporary.
    Index(TExpr, TExpr),
}

/// Section 6.3: a numeric literal, in parentheses or under arithmetic operators.
pub fn is_literal_expr(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Int(..) | ExprKind::Dec(_) => true,
        ExprKind::Paren(x) | ExprKind::Unary(UnOp::Neg, x) => is_literal_expr(x),
        ExprKind::Binary(op, a, b) => {
            matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem)
                && is_literal_expr(a)
                && is_literal_expr(b)
        }
        _ => false,
    }
}

fn has_decimal(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Dec(_) => true,
        ExprKind::Paren(x) | ExprKind::Unary(_, x) => has_decimal(x),
        ExprKind::Binary(_, a, b) => has_decimal(a) || has_decimal(b),
        _ => false,
    }
}

/// Whether an argument can be checked only once the type it is given for is known.
pub fn needs_expected(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Lambda { .. } | ExprKind::MethodRef { .. } | ExprKind::Switch { .. } => true,
        ExprKind::Paren(x) => needs_expected(x),
        ExprKind::Cond(_, a, b) => needs_expected(a) || needs_expected(b),
        ExprKind::Coalesce(_, b) => needs_expected(b),
        _ => is_literal_expr(e),
    }
}

fn is_null(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Null => true,
        ExprKind::Paren(x) => is_null(x),
        _ => false,
    }
}

fn strip(e: &TExpr) -> &TExpr {
    match &e.kind {
        TKind::Coerce(inner) => strip(inner),
        _ => e,
    }
}

fn intersect(a: &Facts, b: &Facts) -> Facts {
    a.iter().filter(|x| b.contains(x)).cloned().collect()
}

impl<'p> Checker<'p> {
    pub fn texpr(&self, kind: TKind, ty: Type) -> TExpr {
        TExpr { kind, ty }
    }

    pub fn bool_expr(&self, b: bool) -> TExpr {
        TExpr { kind: TKind::Bool(b), ty: self.boolean() }
    }

    /// The type of the enclosing class as its own code sees it.
    pub fn self_type(&self) -> Type {
        let c = self.p.class(self.class);
        let args = (0..c.tparams.len())
            .map(|i| Arg::Ty(Type::var(Tv { owner: TvOwner::Class(self.class), index: i as u32 })))
            .collect();
        Type::class(self.class, args)
    }

    // ---- this and locals ----

    /// The value of `this` where the object is complete.
    pub fn this_value(&mut self, pos: Pos) -> TExpr {
        if self.is_static || self.frames[0].this_ty.is_none() {
            return self.error_expr(pos, "`this` is not available in a static context");
        }
        if self.frames[0].ctor == CtorPhase::Before {
            return self.error_expr(
                pos,
                "before `super(...)` is called, `this` is used only to assign a field of the class and to read one that is assigned",
            );
        }
        let declared = self.frames[0].this_ty.clone().unwrap();
        let ty = self.frames[0].flow.facts.get(&THIS).cloned().unwrap_or_else(|| declared.clone());
        let at = self.frames.len() - 1;
        if at == 0 {
            let base = TExpr { kind: TKind::This, ty: declared.clone() };
            return if ty == declared { base } else { TExpr { kind: TKind::Coerce(Box::new(base)), ty } };
        }
        self.captured(0, at, CapSource::This, ty)
    }

    /// `this` for a field of the class in the part of a constructor before `super(...)`.
    fn raw_this(&self) -> TExpr {
        TExpr { kind: TKind::This, ty: self.frames[0].this_ty.clone().unwrap_or_else(Type::error) }
    }

    pub fn local_type(&self, fi: usize, id: LocalId) -> Type {
        let f = &self.frames[fi];
        f.flow.facts.get(&id).cloned().unwrap_or_else(|| f.locals[id as usize].ty.clone())
    }

    pub fn local_read(&mut self, fi: usize, id: LocalId, pos: Pos) -> TExpr {
        let declared = self.frames[fi].locals[id as usize].ty.clone();
        if !self.frames[fi].flow.assigned.contains(&id) {
            let name = self.frames[fi].locals[id as usize].name.clone();
            self.err(pos, format!("`{name}` is used where it may not have been assigned"));
        }
        let ty = self.local_type(fi, id);
        let at = self.frames.len() - 1;
        if fi == at {
            let base = TExpr { kind: TKind::Local(id), ty: declared.clone() };
            return if ty == declared { base } else { TExpr { kind: TKind::Coerce(Box::new(base)), ty } };
        }
        let m = &mut self.frames[fi].meta[id as usize];
        if m.captured_at.is_none() {
            m.captured_at = Some(pos);
        }
        self.captured(fi, at, CapSource::Local(id), ty)
    }

    /// The value a lambda's instance holds for one capture, computed in the frame that
    /// creates the lambda.
    pub fn capture_value(&mut self, c: &Capture) -> TExpr {
        let at = self.frames.len() - 1;
        if c.owner == at {
            let (kind, declared) = match &c.source {
                CapSource::This => (TKind::This, self.frames[at].this_ty.clone().unwrap_or_else(Type::error)),
                CapSource::Local(id) => (TKind::Local(*id), self.frames[at].locals[*id as usize].ty.clone()),
            };
            let base = TExpr { kind, ty: declared.clone() };
            return if declared == c.ty { base } else { TExpr { kind: TKind::Coerce(Box::new(base)), ty: c.ty.clone() } };
        }
        self.captured(c.owner, at, c.source.clone(), c.ty.clone())
    }

    // ---- assignability ----

    /// The `@Implicit` method that converts a value of one type to another (section 4.5).
    pub fn conversion(&mut self, from: &Type, to: &Type) -> Option<MethodRef> {
        if from.nullable {
            return None;
        }
        let (Ty::Class(s, sa), Ty::Class(t, ta)) = (&from.ty, &to.ty) else { return None };
        if s == t || !sa.is_empty() || !ta.is_empty() {
            return None;
        }
        // The result of a conversion has no refinement.
        if to.quals.iter().any(|q| self.p.class(*q).qualifier == Qualifier::Refines) {
            return None;
        }
        let c = self.p.class(*t);
        for (i, m) in c.methods.iter().enumerate() {
            if m.implicit && m.is_static && m.params.len() == 1 && m.params[0].ty.class_id() == Some(*s) {
                return Some(MethodRef { class: *t, index: i as u32 });
            }
        }
        None
    }

    pub fn assignable(&mut self, from: &Type, to: &Type, conv: bool) -> bool {
        self.p.is_subtype(from, to) || (conv && self.conversion(from, to).is_some())
    }

    /// Makes a value fit a location, converting it when section 4.5 allows.
    pub fn coerce(&mut self, e: TExpr, to: &Type, pos: Pos) -> TExpr {
        if e.ty.is_error() || to.is_error() {
            return TExpr { kind: e.kind, ty: Type::error() };
        }
        if self.p.is_subtype(&e.ty, to) {
            return if e.ty == *to { e } else { TExpr { kind: TKind::Coerce(Box::new(e)), ty: to.clone() } };
        }
        if let Some(m) = self.conversion(&e.ty, to) {
            let ret = Type::simple(m.class);
            let call = Call { recv: None, method: m, class_args: Vec::new(), targs: Vec::new(), args: vec![e], dispatch: Dispatch::Direct };
            let out = self.fold(TExpr { kind: TKind::Call(Box::new(call)), ty: ret.clone() }, pos);
            return if ret == *to { out } else { TExpr { kind: TKind::Coerce(Box::new(out)), ty: to.clone() } };
        }
        let (a, b) = (self.show(&e.ty), self.show(to));
        if e.ty.is_null_literal() {
            self.error_expr(pos, format!("`null` is not a value of `{b}`; a type that contains `null` is written `@Nullable`"))
        } else {
            self.error_expr(pos, format!("`{a}` is not assignable to `{b}`"))
        }
    }

    /// Replaces a call of a prelude method on constants with its value (section 12.1).
    pub fn fold(&mut self, e: TExpr, pos: Pos) -> TExpr {
        let TKind::Call(c) = &e.kind else { return e };
        let class = c.method.class;
        let w = &self.p.wk;
        if self.p.num_kind(class).is_none() && class != w.boolean && class != w.char_ && class != w.string {
            return e;
        }
        let recv = match &c.recv {
            Some(r) => match self.as_const(r) {
                Some(k) => Some(k),
                None => return e,
            },
            None => None,
        };
        let mut args = Vec::new();
        for a in &c.args {
            match self.as_const(a) {
                Some(k) => args.push(k),
                None => return e,
            }
        }
        let name = self.p.method(c.method).name.clone();
        match self.fold_method(class, &name, recv.as_ref(), &args, &e.ty) {
            Fold::Value(v) => {
                let k = self.const_expr(&v);
                if k.ty == e.ty {
                    k
                } else {
                    TExpr { kind: k.kind, ty: e.ty }
                }
            }
            Fold::Raises(x) => self.error_expr(pos, format!("this constant expression would raise `{x}`")),
            Fold::NotConst => e,
        }
    }

    /// Section 4.5: whether no value can belong to both types.
    pub fn disjoint(&mut self, a: &Type, b: &Type) -> bool {
        if a.is_error() || b.is_error() || (self.p.may_be_null(a) && self.p.may_be_null(b)) {
            return false;
        }
        let (Ty::Class(x, _), Ty::Class(y, _)) = (&a.ty, &b.ty) else { return false };
        let (xi, yi) = (self.p.class(*x).is_interface(), self.p.class(*y).is_interface());
        let (ab, bb) = (a.bare(), b.bare());
        match (xi, yi) {
            (true, true) => false,
            (false, false) => !(self.p.is_subtype(&ab, &bb) || self.p.is_subtype(&bb, &ab)),
            (false, true) => self.p.class(*x).is_final() && !self.p.is_subtype(&ab, &bb),
            (true, false) => self.p.class(*y).is_final() && !self.p.is_subtype(&bb, &ab),
        }
    }

    // ---- literals ----

    fn default_literal_class(&self, e: &ast::Expr) -> ClassId {
        if has_decimal(e) {
            self.p.wk.rational
        } else {
            self.p.wk.int
        }
    }

    /// A literal expression with every literal in the numeric class `class`.
    pub fn literal_at(&mut self, e: &ast::Expr, class: ClassId) -> TExpr {
        let lit = |ck: &mut Self, value: BigRational, dec: bool, pos: Pos| -> TExpr {
            match ck.literal_const(&value, dec, class) {
                Ok(c) => ck.const_expr(&c),
                Err(why) => ck.error_expr(pos, why),
            }
        };
        match &e.kind {
            ExprKind::Int(digits, radix) => lit(self, BigRational::from_integer(parse_int(digits, *radix)), false, e.pos),
            ExprKind::Dec(text) => lit(self, parse_decimal(text), true, e.pos),
            ExprKind::Paren(x) => self.literal_at(x, class),
            ExprKind::Unary(UnOp::Neg, x) => match &x.kind {
                // A `-` written directly before a literal is part of the literal.
                ExprKind::Int(digits, radix) => lit(self, -BigRational::from_integer(parse_int(digits, *radix)), false, e.pos),
                ExprKind::Dec(text) => lit(self, -parse_decimal(text), true, e.pos),
                _ => {
                    let v = self.literal_at(x, class);
                    self.call_method(v, "negate", CallSite::new(Vec::new(), e.pos))
                }
            },
            ExprKind::Binary(op, a, b) => {
                let l = self.literal_at(a, class);
                let r = self.literal_at(b, class);
                if l.ty.is_error() || r.ty.is_error() {
                    return TExpr { kind: TKind::Null, ty: Type::error() };
                }
                self.operator(*op, l, r, e.pos)
            }
            _ => unreachable!("not a literal expression"),
        }
    }

    fn literal(&mut self, e: &ast::Expr, expected: Option<&Type>) -> TExpr {
        let class = expected.and_then(|t| self.p.numeric_class(t)).unwrap_or_else(|| self.default_literal_class(e));
        self.literal_at(e, class)
    }

    /// Whether a literal expression denotes a value of the numeric class.
    pub fn literal_fits(&mut self, e: &ast::Expr, class: ClassId) -> bool {
        let snap = self.snapshot();
        self.literal_at(e, class);
        let ok = !self.failed_since(&snap);
        self.restore(snap);
        ok
    }

    // ---- the main entry ----

    pub fn check_expr(&mut self, e: &ast::Expr, expected: Option<&Type>) -> TExpr {
        let pos = e.pos;
        if is_literal_expr(e) {
            return self.literal(e, expected);
        }
        match &e.kind {
            ExprKind::Int(..) | ExprKind::Dec(_) => unreachable!(),
            ExprKind::Char(c) => TExpr { kind: TKind::Char(*c), ty: Type::simple(self.p.wk.char_) },
            ExprKind::Str(s) => TExpr { kind: TKind::Str(s.clone()), ty: self.string() },
            ExprKind::Bool(b) => self.bool_expr(*b),
            ExprKind::Null => TExpr { kind: TKind::Null, ty: Type::null() },
            ExprKind::Name(n) => self.name_expr(n, pos),
            ExprKind::This => self.this_value(pos),
            ExprKind::Paren(x) => self.check_expr(x, expected),
            ExprKind::Field(..) => match self.target(e) {
                Target::Value(v) => v,
                Target::Type(_) | Target::TypeVar(_) => self.error_expr(pos, "a type stands here where a value is needed"),
                Target::Package(_) => self.error_expr(pos, "a package stands here where a value is needed"),
            },
            ExprKind::Call { target, type_args, name, args } => {
                let targs = self.explicit_targs(type_args);
                let mut site = CallSite::new(args.iter().map(ArgIn::Ast).collect(), pos);
                site.targs = targs;
                site.expected = expected.cloned();
                match target {
                    None => self.call_unqualified(name, site),
                    Some(t) => match self.target(t) {
                        Target::Value(recv) => {
                            if recv.ty.is_error() {
                                return recv;
                            }
                            self.call_method(recv, name, site)
                        }
                        Target::Type(cid) => self.call_static(cid, name, site),
                        Target::TypeVar(tv) => self.call_static_req(tv, name, site),
                        Target::Package(_) => self.error_expr(pos, "a package has no methods"),
                    },
                }
            }
            ExprKind::SuperCall { name, args } => {
                let mut site = CallSite::new(args.iter().map(ArgIn::Ast).collect(), pos);
                site.expected = expected.cloned();
                self.call_super(name, site)
            }
            ExprKind::Index(a, i) => {
                let recv = self.check_expr(a, None);
                if recv.ty.is_error() {
                    return recv;
                }
                let mut site = CallSite::new(vec![ArgIn::Ast(i)], pos);
                site.operator = Some("[]");
                self.call_method(recv, "get", site)
            }
            ExprKind::New { ty, args } => self.new_expr(ty, args, pos),
            ExprKind::NewArray { elem, len } => {
                let et = self.resolve(elem);
                let n = self.check_expr(len, Some(&self.int()));
                let n = self.coerce(n, &self.int(), len.pos);
                if !et.is_error() && !self.has_default(&et, 0) {
                    let s = self.show(&et);
                    self.err(pos, format!("`new {s}[n]` is rejected because `{s}` has no default value; write `new @Nullable {s}[n]`, list the elements, or use `Array.build`"));
                }
                let ty = self.p.array_of(et.clone());
                TExpr { kind: TKind::NewArray { elem: et, len: Box::new(n) }, ty }
            }
            ExprKind::ArrayLit { ty, elems } => {
                let at = self.resolve(ty);
                let Some(et) = self.p.array_elem(&at) else {
                    return TExpr { kind: TKind::Null, ty: Type::error() };
                };
                let mut out = Vec::new();
                for x in elems {
                    let v = self.check_expr(x, Some(&et));
                    out.push(self.coerce(v, &et, x.pos));
                }
                TExpr { kind: TKind::ArrayLit { elem: et, elems: out }, ty: at }
            }
            ExprKind::ClassLit(t) => {
                let ty = self.resolve_no_wildcards(t, "a class literal");
                if ty.nullable || !ty.quals.is_empty() {
                    self.err(pos, "a class literal names a class, without a qualifier");
                }
                TExpr { kind: TKind::ClassLit(ty), ty: Type::simple(self.p.wk.class) }
            }
            ExprKind::MethodRef { .. } => self.method_ref(e, expected),
            ExprKind::Lambda { .. } => match expected {
                Some(t) if !t.is_error() => self.lambda(e, t),
                Some(_) => TExpr { kind: TKind::Null, ty: Type::error() },
                None => self.error_expr(pos, "a lambda has no type of its own; it stands where a functional interface is expected"),
            },
            ExprKind::Unary(UnOp::Not, _) | ExprKind::And(..) | ExprKind::Or(..) | ExprKind::InstanceOf(..) => self.check_cond(e).e,
            ExprKind::Binary(BinOp::Eq | BinOp::Ne, a, b) if is_null(a) || is_null(b) => self.check_cond(e).e,
            ExprKind::Unary(op, x) => {
                let v = self.check_expr(x, None);
                if v.ty.is_error() {
                    return v;
                }
                let name = if *op == UnOp::Neg { "negate" } else { "complement" };
                let mut site = CallSite::new(Vec::new(), pos);
                site.operator = Some(if *op == UnOp::Neg { "-" } else { "~" });
                self.call_method(v, name, site)
            }
            ExprKind::Binary(op, a, b) => self.binary(*op, a, b, pos),
            ExprKind::Cond(c, a, b) => self.conditional(c, a, b, expected, pos),
            ExprKind::Coalesce(a, b) => self.coalesce(a, b, expected, pos),
            ExprKind::Assign { op, target, value } => self.assign(*op, target, value, pos, true),
            ExprKind::IncDec { inc, prefix, target } => self.inc_dec(*inc, *prefix, target, pos, true),
            ExprKind::Cast(t, x) => self.cast(t, x, pos),
            ExprKind::Switch { selector, arms } => self.switch_expr(selector, arms, expected, pos),
        }
    }

    /// Checks a condition and gathers the facts of section 12.4.
    pub fn check_cond(&mut self, e: &ast::Expr) -> Cond {
        let pos = e.pos;
        let boolean = self.boolean();
        match &e.kind {
            ExprKind::Paren(x) => self.check_cond(x),
            ExprKind::Unary(UnOp::Not, x) => {
                let c = self.check_cond(x);
                if c.e.ty.is_error() {
                    return c;
                }
                if c.e.ty == boolean {
                    let e = match c.e.kind {
                        TKind::Bool(b) => self.bool_expr(!b),
                        _ => TExpr { kind: TKind::Not(Box::new(c.e)), ty: boolean },
                    };
                    return Cond { e, t: c.f, f: c.t };
                }
                let mut site = CallSite::new(Vec::new(), pos);
                site.operator = Some("!");
                let e = self.call_method(c.e, "not", site);
                Cond { e, t: Vec::new(), f: Vec::new() }
            }
            ExprKind::And(a, b) | ExprKind::Or(a, b) => {
                let is_and = matches!(e.kind, ExprKind::And(..));
                let ca = self.check_cond(a);
                let ea = self.coerce(ca.e, &boolean, a.pos);
                let after_a = self.frame().flow.clone();
                self.apply_facts(if is_and { &ca.t } else { &ca.f });
                let cb = self.check_cond(b);
                let eb = self.coerce(cb.e, &boolean, b.pos);
                // Only what the left operand assigns is certain after the whole expression.
                let now = std::mem::replace(&mut self.frame().flow, after_a);
                let f = self.frame();
                f.flow.maybe.extend(now.maybe);
                f.flow.maybe_fields.extend(now.maybe_fields);
                let (t, fl) = if is_and {
                    let mut t = ca.t.clone();
                    t.extend(cb.t.clone());
                    (t, intersect(&ca.f, &cb.f))
                } else {
                    let mut fl = ca.f.clone();
                    fl.extend(cb.f.clone());
                    (intersect(&ca.t, &cb.t), fl)
                };
                let folded = match (&ea.kind, &eb.kind) {
                    (TKind::Bool(x), TKind::Bool(y)) => Some(if is_and { *x && *y } else { *x || *y }),
                    _ => None,
                };
                let e = match folded {
                    Some(v) => self.bool_expr(v),
                    None if is_and => TExpr { kind: TKind::And(Box::new(ea), Box::new(eb)), ty: boolean },
                    None => TExpr { kind: TKind::Or(Box::new(ea), Box::new(eb)), ty: boolean },
                };
                Cond { e, t, f: fl }
            }
            ExprKind::Binary(op @ (BinOp::Eq | BinOp::Ne), a, b) if is_null(a) || is_null(b) => {
                let other = if is_null(a) { b } else { a };
                let is_eq = *op == BinOp::Eq;
                let x = self.check_expr(other, None);
                if x.ty.is_error() {
                    return Cond { e: x, t: Vec::new(), f: Vec::new() };
                }
                if !self.p.may_be_null(&x.ty) {
                    let s = self.show(&x.ty);
                    self.err(pos, format!("a null test of a `{s}` is rejected, because it cannot be `null`"));
                }
                let mut facts = Vec::new();
                if let Some(id) = self.subject(&x) {
                    facts.push((id, x.ty.clone().non_null()));
                }
                let e = TExpr { kind: TKind::IsNull(Box::new(x), is_eq), ty: boolean };
                if is_eq {
                    Cond { e, t: Vec::new(), f: facts }
                } else {
                    Cond { e, t: facts, f: Vec::new() }
                }
            }
            ExprKind::InstanceOf(x, t) => {
                let v = self.check_expr(x, None);
                let ty = self.resolve(t);
                if v.ty.is_error() || ty.is_error() {
                    return Cond { e: TExpr { kind: TKind::Null, ty: Type::error() }, t: Vec::new(), f: Vec::new() };
                }
                if ty.nullable {
                    self.err(t.pos, "`@Nullable` is rejected after `instanceof`: `null` is an instance of no type");
                }
                if !ty.quals.is_empty() {
                    self.err(t.pos, "a value does not carry its qualifiers at run time, so `instanceof` does not test one");
                }
                let ty = ty.bare();
                if self.disjoint(&v.ty.clone().non_null(), &ty) {
                    let (a, b) = (self.show(&v.ty), self.show(&ty));
                    self.err(pos, format!("`{a}` and `{b}` are disjoint, so this test could never be `true`"));
                }
                let mut facts = Vec::new();
                if let Some(id) = self.subject(&v) {
                    // Keep what is already known when it says more than the test does.
                    let known = v.ty.clone().non_null();
                    let narrowed = if self.p.is_subtype(&known, &ty) { known } else { ty.clone() };
                    facts.push((id, narrowed));
                }
                let e = TExpr { kind: TKind::InstanceOf(Box::new(v), ty), ty: boolean };
                Cond { e, t: facts, f: Vec::new() }
            }
            _ => {
                let e = self.check_expr(e, Some(&boolean));
                let t = self.narrows_facts(&e);
                Cond { e, t, f: Vec::new() }
            }
        }
    }

    /// The local that a test narrows, when its operand is one.
    fn subject(&self, e: &TExpr) -> Option<LocalId> {
        match &strip(e).kind {
            TKind::Local(id) => Some(*id),
            TKind::This if self.frames.len() == 1 => Some(THIS),
            _ => None,
        }
    }

    /// The facts a call of a `@Narrows` method gives when it returns `true`.
    fn narrows_facts(&mut self, e: &TExpr) -> Facts {
        let TKind::Call(c) = &e.kind else { return Vec::new() };
        let m = self.p.method(c.method);
        let Some(q) = m.narrows else { return Vec::new() };
        let subject = if m.is_static { c.args.first() } else { c.recv.as_ref() };
        let Some(subject) = subject else { return Vec::new() };
        let Some(id) = self.subject(subject) else { return Vec::new() };
        let mut ty = if id == THIS {
            match self.this_value(0).ty {
                t if !t.is_error() => t,
                _ => return Vec::new(),
            }
        } else {
            let fi = self.frames.len() - 1;
            self.local_type(fi, id)
        };
        if q == self.p.wk.nullable {
            ty.nullable = false;
        } else if self.p.class(q).qualifier == Qualifier::Refines {
            let below = ty.quals.iter().any(|have| self.p.refines(*have, q));
            if !below {
                ty.add_qual(q);
            }
        } else {
            ty.quals.retain(|have| *have != q);
        }
        vec![(id, ty)]
    }

    // ---- names and members ----

    fn find_field(&mut self, class: ClassId, name: &str) -> Option<FieldRef> {
        let mut classes = vec![class];
        classes.extend(decl::all_supertypes(self.p, class).iter().filter_map(|t| t.class_id()));
        for cid in classes {
            if let Some(i) = self.p.class(cid).fields.iter().position(|f| f.name == name) {
                return Some(FieldRef { class: cid, index: i as u32 });
            }
        }
        None
    }

    fn own_instance_field(&mut self, name: &str) -> Option<FieldRef> {
        if self.is_static {
            return None;
        }
        self.find_field(self.class, name).filter(|f| !self.p.field(*f).is_static)
    }

    fn name_expr(&mut self, name: &str, pos: Pos) -> TExpr {
        if let Some((fi, id)) = self.find_local(name) {
            return self.local_read(fi, id, pos);
        }
        if let Some(f) = self.find_field(self.class, name) {
            if self.p.field(f).is_static {
                return self.static_field(f, pos);
            }
            return self.own_field(f, pos);
        }
        self.error_expr(pos, format!("`{name}` is not a variable or a field in scope"))
    }

    /// A field of the enclosing class named without a receiver.
    fn own_field(&mut self, f: FieldRef, pos: Pos) -> TExpr {
        if !self.is_static && self.frames[0].ctor == CtorPhase::Before && self.frames[0].this_ty.is_some() {
            let name = self.p.field(f).name.clone();
            if f.class != self.class {
                return self.error_expr(pos, format!("the inherited field `{name}` is not used before `super(...)` is called"));
            }
            if self.frames.len() > 1 {
                return self.error_expr(pos, "a lambda does not capture `this` before `super(...)` is called");
            }
            if !self.frames[0].flow.fields.contains(&f.index) {
                self.err(pos, format!("the field `{name}` is read before it is assigned"));
            }
            let ty = self.p.field(f).ty.clone();
            return TExpr { kind: TKind::Field(Box::new(self.raw_this()), f), ty };
        }
        let this = self.this_value(pos);
        if this.ty.is_error() {
            return this;
        }
        self.field_of(this, f, pos)
    }

    pub fn can_access(&mut self, decl_class: ClassId, aud: Aud, only: &Option<Vec<ClassId>>, recv: Option<&Type>) -> bool {
        let me = self.class;
        if me == decl_class {
            return true;
        }
        if let Some(list) = only {
            return list.contains(&me);
        }
        match aud {
            Aud::Private => false,
            Aud::File => self.p.class(me).unit == self.p.class(decl_class).unit,
            Aud::Package => self.p.class(me).package == self.p.class(decl_class).package,
            Aud::Protected => {
                if !self.p.is_subclass(me, decl_class) {
                    return false;
                }
                match recv {
                    None => true,
                    Some(t) => match t.class_id() {
                        Some(rc) => self.p.is_subclass(rc, me),
                        None => false,
                    },
                }
            }
            Aud::Public => true,
        }
    }

    pub fn deprecation(&mut self, decl_class: ClassId, what: &str, message: &Option<String>, pos: Pos) {
        let Some(m) = message else { return };
        if self.p.class(decl_class).unit == self.unit {
            return;
        }
        let u = &self.p.units[self.unit];
        let text = if m.is_empty() { format!("{what} is deprecated") } else { format!("{what} is deprecated: {m}") };
        let d = crate::lex::diag(&u.file, &u.text, pos, text);
        self.p.warnings.push(d);
    }

    /// Reads the field `f` of the value `recv`.
    fn field_of(&mut self, recv: TExpr, f: FieldRef, pos: Pos) -> TExpr {
        let fd = self.p.field(f);
        let (name, aud, only, fty) = (fd.name.clone(), fd.aud, fd.only.clone(), fd.ty.clone());
        if !self.can_access(f.class, aud, &only, Some(&recv.ty)) {
            let c = self.p.class(f.class).name.clone();
            return self.error_expr(pos, format!("the field `{name}` of `{c}` is outside its audience here"));
        }
        if self.p.may_be_null(&recv.ty) {
            let s = self.show(&recv.ty);
            return self.error_expr(pos, format!("the field `{name}` is read from a `{s}`, which may be `null`"));
        }
        let cap = self.capture(&recv.ty);
        let ty = match self.p.supertype_at(&cap, f.class) {
            Some(up) => Subst::for_class(f.class, up.args()).apply(&fty),
            None => fty,
        };
        let ty = self.project(&ty);
        TExpr { kind: TKind::Field(Box::new(recv), f), ty }
    }

    fn static_field(&mut self, f: FieldRef, pos: Pos) -> TExpr {
        let fd = self.p.field(f);
        let (name, aud, only, ty, constant) = (fd.name.clone(), fd.aud, fd.only.clone(), fd.ty.clone(), fd.constant.clone());
        if !self.can_access(f.class, aud, &only, None) {
            let c = self.p.class(f.class).name.clone();
            return self.error_expr(pos, format!("the field `{name}` of `{c}` is outside its audience here"));
        }
        if let Some(c) = constant {
            return self.const_expr(&c);
        }
        // A static initializer reads a field of its own class only once it is assigned.
        if self.frames[0].in_static_init && f.class == self.class && self.frames.len() == 1 && !self.frames[0].flow.fields.contains(&f.index) {
            self.err(pos, format!("the static field `{name}` is read before it is assigned"));
        }
        TExpr { kind: TKind::StaticField(f), ty }
    }

    /// Resolves what stands before a `.`.
    pub fn target(&mut self, e: &ast::Expr) -> Target {
        let pos = e.pos;
        match &e.kind {
            ExprKind::Name(n) => {
                if self.find_local(n).is_some() || self.find_field(self.class, n).is_some() {
                    return Target::Value(self.check_expr(e, None));
                }
                if let Some((_, tv)) = self.cx.tvars.iter().rev().find(|(m, _)| m == n) {
                    return Target::TypeVar(*tv);
                }
                if let Some(id) = decl::lookup_simple(self.p, self.unit, n, pos, false) {
                    return Target::Type(id);
                }
                let prefix = format!("{n}.");
                if self.p.packages.iter().any(|p| p == n || p.starts_with(&prefix)) {
                    return Target::Package(n.clone());
                }
                Target::Value(self.error_expr(pos, format!("`{n}` is not a variable, a field or a type in scope")))
            }
            ExprKind::Field(t, n) if matches!(t.kind, ExprKind::This) && self.own_instance_field(n).is_some() => {
                // `this.field`, which is legal even before `super(...)` is called.
                let f = self.own_instance_field(n).unwrap();
                Target::Value(self.own_field(f, pos))
            }
            ExprKind::Field(t, n) => match self.target(t) {
                Target::Package(pk) => {
                    let mut path: Vec<String> = pk.split('.').map(|s| s.to_string()).collect();
                    path.push(n.clone());
                    if let Some(id) = decl::lookup_class(self.p, self.unit, &path, pos, false) {
                        return Target::Type(id);
                    }
                    let q = format!("{pk}.{n}");
                    let prefix = format!("{q}.");
                    if self.p.packages.iter().any(|p| *p == q || p.starts_with(&prefix)) {
                        return Target::Package(q);
                    }
                    Target::Value(self.error_expr(pos, format!("the package `{pk}` has no type named `{n}`")))
                }
                Target::Type(cid) => match self.find_field(cid, n) {
                    Some(f) if self.p.field(f).is_static => Target::Value(self.static_field(f, pos)),
                    Some(_) => Target::Value(self.error_expr(pos, format!("`{n}` is an instance field and is read from a value"))),
                    None => {
                        let c = self.p.class(cid).name.clone();
                        Target::Value(self.error_expr(pos, format!("`{c}` has no static field named `{n}`")))
                    }
                },
                Target::TypeVar(_) => Target::Value(self.error_expr(pos, "only a static requirement is reached through a type parameter")),
                Target::Value(v) => {
                    if v.ty.is_error() {
                        return Target::Value(v);
                    }
                    let found = self.field_on_type(&v.ty, n);
                    match found {
                        Some(f) if !self.p.field(f).is_static => Target::Value(self.field_of(v, f, pos)),
                        Some(_) => Target::Value(self.error_expr(pos, format!("the static field `{n}` is named through its class"))),
                        None => {
                            let s = self.show(&v.ty);
                            Target::Value(self.error_expr(pos, format!("`{s}` has no field named `{n}`")))
                        }
                    }
                }
            },
            ExprKind::This if self.frames[0].ctor == CtorPhase::Before && !self.is_static => {
                // Only `this.field` is legal here, and `Field` above handles it.
                Target::Value(self.this_value(pos))
            }
            _ => Target::Value(self.check_expr(e, None)),
        }
    }

    fn field_on_type(&mut self, t: &Type, name: &str) -> Option<FieldRef> {
        match &t.ty {
            Ty::Class(id, _) => self.find_field(*id, name),
            Ty::Var(tv) => {
                for b in self.p.bounds_of(*tv) {
                    if let Some(f) = self.field_on_type(&b, name) {
                        return Some(f);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn explicit_targs(&mut self, targs: &[ast::TypeArg]) -> Option<Vec<Type>> {
        if targs.is_empty() {
            return None;
        }
        let mut out = Vec::new();
        for a in targs {
            match a {
                ast::TypeArg::Type(t) => out.push(self.resolve(t)),
                ast::TypeArg::Wildcard(_, wpos) => {
                    self.err(*wpos, "a wildcard is not written as an explicit type argument of a call; a type is needed");
                    out.push(Type::error());
                }
            }
        }
        Some(out)
    }

    /// Resolves a type whose own arguments must be types (section 7.4).
    pub fn resolve_no_wildcards(&mut self, t: &ast::TypeRef, what: &str) -> Type {
        let ty = self.resolve(t);
        if t.dims.is_empty() && ty.has_wildcard_arg() {
            self.err(t.pos, format!("a wildcard is not written among the arguments of {what}; a type is needed"));
            return Type::error();
        }
        ty
    }

    fn new_expr(&mut self, t: &ast::TypeRef, args: &[ast::Expr], pos: Pos) -> TExpr {
        let ty = self.resolve_no_wildcards(t, "the class in `new`");
        if ty.is_error() {
            for a in args {
                if !needs_expected(a) {
                    self.check_expr(a, None);
                }
            }
            return TExpr { kind: TKind::Null, ty };
        }
        if ty.nullable || !ty.quals.is_empty() {
            self.err(t.pos, "the result of `new` has no qualifier");
        }
        let Some(cid) = ty.class_id() else {
            return self.error_expr(pos, "`new` names a class, not a type parameter");
        };
        let c = self.p.class(cid);
        let name = c.name.clone();
        let problem = if cid == self.p.wk.null {
            Some("`new Null()` is rejected: `null` is the one instance")
        } else if c.is_interface() {
            Some("an interface has no instances of its own")
        } else if c.is_abstract {
            Some("an abstract class has no instances of its own")
        } else if c.is_enum() {
            Some("no expression creates an instance of an enum")
        } else if c.is_annotation() {
            Some("an annotation value comes only from a use of the annotation")
        } else if cid == self.p.wk.array {
            Some("an array is created with `new T[n]`, `new T[] { ... }` or `Array.build`")
        } else {
            None
        };
        if let Some(why) = problem {
            return self.error_expr(pos, format!("`new {name}` is rejected: {why}"));
        }
        self.construct(ty.bare(), CallSite::new(args.iter().map(ArgIn::Ast).collect(), pos))
    }

    /// Whether `new T[n]` has a default element for `T` (section 6.10).
    fn has_default(&mut self, t: &Type, depth: u32) -> bool {
        if t.nullable {
            return true;
        }
        let Ty::Class(id, args) = &t.ty else { return false };
        let w = &self.p.wk;
        if self.p.num_kind(*id).is_some() || *id == w.char_ || *id == w.boolean {
            return true;
        }
        let c = self.p.class(*id).clone();
        if c.kind != ast::TypeKind::ValueClass || c.package == "cleat" || depth > 8 {
            return false;
        }
        let s = Subst::for_class(*id, args);
        c.fields.iter().filter(|f| !f.is_static).all(|f| {
            let ft = s.apply(&f.ty);
            self.has_default(&ft, depth + 1)
        })
    }

    // ---- operators ----

    /// An operator on two checked operands.
    pub fn operator(&mut self, op: BinOp, l: TExpr, r: TExpr, pos: Pos) -> TExpr {
        let mut site = CallSite::new(vec![ArgIn::Done(r)], pos);
        site.operator = Some(op.spelling());
        self.call_method(l, op.method(), site)
    }

    /// Section 4.6: converts one operand to the class of the other when exactly one converts.
    fn balance(&mut self, l: TExpr, r: TExpr, pos: Pos) -> (TExpr, TExpr) {
        if l.ty.nullable || r.ty.nullable || l.ty.class_id() == r.ty.class_id() || l.ty.class_id().is_none() || r.ty.class_id().is_none() {
            return (l, r);
        }
        let (lt, rt) = (l.ty.bare(), r.ty.bare());
        let l_to_r = self.conversion(&lt, &rt).is_some();
        let r_to_l = self.conversion(&rt, &lt).is_some();
        match (l_to_r, r_to_l) {
            (true, false) => (self.coerce(l, &rt, pos), r),
            (false, true) => {
                let r = self.coerce(r, &lt, pos);
                (l, r)
            }
            _ => (l, r),
        }
    }

    fn unboxed_class(&self, t: &Type) -> Option<ClassId> {
        if t.nullable {
            return None;
        }
        let id = t.class_id()?;
        let w = &self.p.wk;
        match self.p.num_kind(id) {
            Some(NumKind::Rational) => None,
            Some(_) => Some(id),
            None if id == w.boolean || id == w.char_ => Some(id),
            None => None,
        }
    }

    fn binary(&mut self, op: BinOp, a: &ast::Expr, b: &ast::Expr, pos: Pos) -> TExpr {
        let is_shift = matches!(op, BinOp::Shl | BinOp::Shr);
        let is_eq = matches!(op, BinOp::Eq | BinOp::Ne);
        // A literal operand takes its class from the other operand (section 6.3, rule 2).
        let (l, r): (TExpr, Option<TExpr>) = if is_literal_expr(a) {
            let r = self.check_expr(b, None);
            let l = match self.p.numeric_class(&r.ty) {
                Some(c) if !r.ty.nullable && !is_shift => self.literal_at(a, c),
                _ => self.literal(a, None),
            };
            (l, Some(r))
        } else {
            let l = self.check_expr(a, None);
            if needs_expected(b) {
                (l, None)
            } else {
                let r = self.check_expr(b, None);
                (l, Some(r))
            }
        };
        if l.ty.is_error() || r.as_ref().map(|r| r.ty.is_error()).unwrap_or(false) {
            return TExpr { kind: TKind::Null, ty: Type::error() };
        }
        let Some(r) = r else {
            // The right operand needs the parameter's type: the call supplies it.
            let hint = if l.ty.nullable { None } else { self.p.numeric_class(&l.ty) };
            if is_eq {
                if let (Some(c), true) = (hint, is_literal_expr(b)) {
                    let r = self.literal_at(b, c);
                    return self.equality(op, l, r, pos);
                }
            }
            let mut site = CallSite::new(vec![ArgIn::Ast(b)], pos);
            site.operator = Some(op.spelling());
            site.lit_hint = hint;
            let e = self.call_method(l, op.method(), site);
            return if op == BinOp::Ne { self.negate_bool(e) } else { e };
        };
        let (l, r) = if is_shift { (l, r) } else { self.balance(l, r, pos) };
        if is_eq {
            return self.equality(op, l, r, pos);
        }
        self.operator(op, l, r, pos)
    }

    fn negate_bool(&mut self, e: TExpr) -> TExpr {
        if e.ty.is_error() {
            return e;
        }
        match e.kind {
            TKind::Bool(b) => self.bool_expr(!b),
            _ => TExpr { kind: TKind::Not(Box::new(e)), ty: self.boolean() },
        }
    }

    fn equality(&mut self, op: BinOp, l: TExpr, r: TExpr, pos: Pos) -> TExpr {
        if l.ty.is_error() || r.ty.is_error() {
            return TExpr { kind: TKind::Null, ty: Type::error() };
        }
        if self.disjoint(&l.ty, &r.ty) {
            let (a, b) = (self.show(&l.ty), self.show(&r.ty));
            return self.error_expr(pos, format!("`{}` between `{a}` and `{b}` is rejected: the two types are disjoint, so no pair of values is equal", op.spelling()));
        }
        if let (Some(x), Some(y)) = (self.unboxed_class(&l.ty), self.unboxed_class(&r.ty)) {
            if x == y {
                if let (Some(ca), Some(cb)) = (self.as_const(&l), self.as_const(&r)) {
                    if let Some(v) = self.fold_equals(&ca, &cb) {
                        return self.bool_expr(v == (op == BinOp::Eq));
                    }
                }
                return TExpr { kind: TKind::Prim(op, Box::new(l), Box::new(r)), ty: self.boolean() };
            }
        }
        let mut site = CallSite::new(vec![ArgIn::Done(r)], pos);
        site.operator = Some(op.spelling());
        let e = self.call_method(l, "equals", site);
        if op == BinOp::Ne {
            self.negate_bool(e)
        } else {
            e
        }
    }

    /// The type of `?:` and `??` without an expected type: one arm's, with the other
    /// assignable to it.
    fn unify_arms(&mut self, a: TExpr, b: TExpr, pos: Pos) -> (TExpr, TExpr, Type) {
        if a.ty.is_error() || b.ty.is_error() {
            return (a, b, Type::error());
        }
        let ty = if a.ty.is_null_literal() {
            b.ty.clone().nullable()
        } else if b.ty.is_null_literal() {
            a.ty.clone().nullable()
        } else if self.assignable(&b.ty, &a.ty, true) {
            a.ty.clone()
        } else if self.assignable(&a.ty, &b.ty, true) {
            b.ty.clone()
        } else {
            let (x, y) = (self.show(&a.ty), self.show(&b.ty));
            self.err(pos, format!("the arms have the types `{x}` and `{y}`, and neither is assignable to the other"));
            return (a, b, Type::error());
        };
        let a = self.coerce(a, &ty, pos);
        let b = self.coerce(b, &ty, pos);
        (a, b, ty)
    }

    fn conditional(&mut self, c: &ast::Expr, a: &ast::Expr, b: &ast::Expr, expected: Option<&Type>, pos: Pos) -> TExpr {
        let boolean = self.boolean();
        let cond = self.check_cond(c);
        let ce = self.coerce(cond.e, &boolean, c.pos);
        let base = self.frame().flow.clone();
        self.apply_facts(&cond.t);
        // An arm that is a literal expression takes its class from the other arm.
        let (la, lb) = (is_literal_expr(a), is_literal_expr(b));
        let numeric_expected = expected.map(|t| self.p.numeric_class(t).is_some()).unwrap_or(false);
        let defer_a = la && !lb && !numeric_expected;
        let mut ea = if defer_a { None } else { Some(self.check_expr(a, expected)) };
        let flow_a = std::mem::replace(&mut self.frame().flow, base);
        self.apply_facts(&cond.f);
        let eb = if lb && !la && !numeric_expected {
            let other = ea.as_ref().unwrap();
            match self.p.numeric_class(&other.ty) {
                Some(k) => self.literal_at(b, k),
                None => self.check_expr(b, expected),
            }
        } else {
            self.check_expr(b, expected)
        };
        if defer_a {
            ea = Some(match self.p.numeric_class(&eb.ty) {
                Some(k) => self.literal_at(a, k),
                None => self.check_expr(a, expected),
            });
        }
        let ea = ea.unwrap();
        let flow_b = std::mem::take(&mut self.frame().flow);
        self.frame().flow = Flow::join(flow_a, flow_b);
        let (ea, eb, ty) = match expected {
            Some(t) if !t.is_error() => {
                let x = self.coerce(ea, t, a.pos);
                let y = self.coerce(eb, t, b.pos);
                (x, y, t.clone())
            }
            _ => self.unify_arms(ea, eb, pos),
        };
        if let TKind::Bool(v) = ce.kind {
            if self.as_const(&ea).is_some() && self.as_const(&eb).is_some() {
                return if v { ea } else { eb };
            }
        }
        TExpr { kind: TKind::Cond(Box::new(ce), Box::new(ea), Box::new(eb)), ty }
    }

    fn coalesce(&mut self, a: &ast::Expr, b: &ast::Expr, expected: Option<&Type>, pos: Pos) -> TExpr {
        let nullable_expected = expected.map(|t| t.clone().nullable());
        let ea = self.check_expr(a, nullable_expected.as_ref());
        if ea.ty.is_error() {
            return ea;
        }
        if !self.p.may_be_null(&ea.ty) {
            let s = self.show(&ea.ty);
            self.err(a.pos, format!("the left operand of `??` is a `{s}`, which cannot be `null`"));
        }
        let left = ea.ty.clone().non_null();
        let base = self.frame().flow.clone();
        let eb = if is_literal_expr(b) {
            match self.p.numeric_class(&left) {
                Some(k) => self.literal_at(b, k),
                None => self.check_expr(b, expected),
            }
        } else {
            self.check_expr(b, expected.or(Some(&left)))
        };
        // The right operand may not run.
        let now = std::mem::replace(&mut self.frame().flow, base);
        self.frame().flow.maybe.extend(now.maybe);
        if eb.ty.is_error() {
            return eb;
        }
        let ty = if eb.ty.is_null_literal() {
            left.clone().nullable()
        } else if self.assignable(&eb.ty, &left, true) {
            left.clone()
        } else if self.p.is_subtype(&left, &eb.ty) {
            eb.ty.clone()
        } else {
            let (x, y) = (self.show(&left), self.show(&eb.ty));
            return self.error_expr(pos, format!("the operands of `??` have the types `{x}` and `{y}`, and neither is assignable to the other"));
        };
        let eb = self.coerce(eb, &ty, b.pos);
        // The left value keeps its own type until the test; the result has `ty`.
        let ea = if ea.ty == ty.clone().nullable() { ea } else { self.coerce(ea, &ty.clone().nullable(), a.pos) };
        TExpr { kind: TKind::Coalesce(Box::new(ea), Box::new(eb)), ty }
    }

    fn cast(&mut self, t: &ast::TypeRef, x: &ast::Expr, pos: Pos) -> TExpr {
        let ty = self.resolve(t);
        let v = self.check_expr(x, Some(&ty));
        if ty.is_error() || v.ty.is_error() {
            return TExpr { kind: TKind::Null, ty: Type::error() };
        }
        if !ty.quals.is_empty() {
            self.err(t.pos, "a value does not carry its qualifiers at run time, so a cast does not name one outside a type argument");
        }
        if self.p.is_subtype(&v.ty, &ty) {
            return if v.ty == ty { v } else { TExpr { kind: TKind::Coerce(Box::new(v)), ty } };
        }
        if self.disjoint(&v.ty, &ty) {
            let (a, b) = (self.show(&v.ty), self.show(&ty));
            return self.error_expr(pos, format!("a cast from `{a}` to `{b}` is rejected: the types are disjoint, and a cast never converts a value"));
        }
        TExpr { kind: TKind::Cast(Box::new(v)), ty }
    }

    // ---- assignment ----

    fn place(&mut self, target: &ast::Expr, binds: &mut Vec<(LocalId, TExpr)>, compound: bool) -> Option<Place> {
        let pos = target.pos;
        match &target.kind {
            ExprKind::Paren(x) => self.place(x, binds, compound),
            ExprKind::Name(n) => {
                if let Some((fi, id)) = self.find_local(n) {
                    return Some(Place::Local(fi, id));
                }
                let Some(f) = self.find_field(self.class, n) else {
                    self.err(pos, format!("`{n}` is not a variable or a field in scope"));
                    return None;
                };
                self.field_place(None, f, binds, compound, pos)
            }
            ExprKind::Field(t, n) => {
                if matches!(t.kind, ExprKind::This) {
                    if let Some(f) = self.find_field(self.class, n) {
                        if !self.p.field(f).is_static {
                            return self.field_place(None, f, binds, compound, pos);
                        }
                    }
                }
                match self.target(t) {
                    Target::Value(v) => {
                        if v.ty.is_error() {
                            return None;
                        }
                        let Some(f) = self.field_on_type(&v.ty, n) else {
                            let s = self.show(&v.ty);
                            self.err(pos, format!("`{s}` has no field named `{n}`"));
                            return None;
                        };
                        if self.p.field(f).is_static {
                            self.err(pos, format!("the static field `{n}` is named through its class"));
                            return None;
                        }
                        self.field_place(Some(v), f, binds, compound, pos)
                    }
                    Target::Type(cid) => match self.find_field(cid, n) {
                        Some(f) if self.p.field(f).is_static => self.field_place(None, f, binds, compound, pos),
                        _ => {
                            let c = self.p.class(cid).name.clone();
                            self.err(pos, format!("`{c}` has no static field named `{n}`"));
                            None
                        }
                    },
                    _ => {
                        self.err(pos, "this is not a location that can be assigned");
                        None
                    }
                }
            }
            ExprKind::Index(a, i) => {
                let recv = self.check_expr(a, None);
                if recv.ty.is_error() {
                    return None;
                }
                let ta = self.temp(recv.ty.clone());
                let ra = TExpr { kind: TKind::Local(ta), ty: recv.ty.clone() };
                binds.push((ta, recv));
                // The index is checked as the argument of `get`, which gives a literal its class.
                let index_ty = self.index_type(&ra.ty);
                let idx = self.check_expr(i, index_ty.as_ref());
                if idx.ty.is_error() {
                    return None;
                }
                let idx = match &index_ty {
                    Some(t) => self.coerce(idx, t, i.pos),
                    None => idx,
                };
                let ti = self.temp(idx.ty.clone());
                let ri = TExpr { kind: TKind::Local(ti), ty: idx.ty.clone() };
                binds.push((ti, idx));
                Some(Place::Index(ra, ri))
            }
            _ => {
                self.err(pos, "this is not a location that can be assigned");
                None
            }
        }
    }

    /// The first parameter type of the one-argument `get` of a type, when it has exactly one.
    fn index_type(&mut self, t: &Type) -> Option<Type> {
        let cands = self.method_candidates(t, "get");
        let mut found = None;
        for (m, s) in cands {
            let md = self.p.method(m);
            if md.is_static || md.params.len() != 1 || !md.tparams.is_empty() {
                continue;
            }
            if found.is_some() {
                return None;
            }
            found = Some(s.apply(&md.params[0].ty));
        }
        found
    }

    fn field_place(&mut self, recv: Option<TExpr>, f: FieldRef, binds: &mut Vec<(LocalId, TExpr)>, compound: bool, pos: Pos) -> Option<Place> {
        let fd = self.p.field(f);
        let (name, aud, only, is_static, is_final, fty) = (fd.name.clone(), fd.aud, fd.only.clone(), fd.is_static, fd.is_final, fd.ty.clone());
        let recv_ty = recv.as_ref().map(|r| r.ty.clone());
        if !self.can_access(f.class, aud, &only, recv_ty.as_ref()) {
            let c = self.p.class(f.class).name.clone();
            self.err(pos, format!("the field `{name}` of `{c}` is outside its audience here"));
            return None;
        }
        if is_static {
            let in_init = self.frames[0].in_static_init && f.class == self.class && self.frames.len() == 1;
            if is_final {
                if !in_init {
                    self.err(pos, format!("the static final field `{name}` is assigned only by its initializer or a static initializer block"));
                    return None;
                }
                if self.frames[0].flow.maybe_fields.contains(&f.index) {
                    self.err(pos, format!("the final field `{name}` is assigned at most once"));
                }
            }
            if in_init && !compound {
                let fl = &mut self.frames[0].flow;
                fl.fields.insert(f.index);
                fl.maybe_fields.insert(f.index);
            }
            return Some(Place::Static(f, fty));
        }
        let own_before = recv.is_none() && !self.is_static && self.frames[0].ctor == CtorPhase::Before && self.frames[0].this_ty.is_some();
        if own_before {
            if f.class != self.class {
                self.err(pos, format!("the inherited field `{name}` is not assigned before `super(...)` is called"));
                return None;
            }
            if self.frames.len() > 1 {
                self.err(pos, "a lambda does not capture `this` before `super(...)` is called");
                return None;
            }
            if self.p.class(self.class).is_value() {
                self.err(pos, format!("the field `{name}` of a value class is bound by the constructor's argument and is not assigned"));
                return None;
            }
            if compound && !self.frames[0].flow.fields.contains(&f.index) {
                self.err(pos, format!("the field `{name}` is read before it is assigned"));
            }
            if is_final && self.frames[0].flow.maybe_fields.contains(&f.index) {
                self.err(pos, format!("the final field `{name}` is assigned at most once"));
            }
            let fl = &mut self.frames[0].flow;
            fl.fields.insert(f.index);
            fl.maybe_fields.insert(f.index);
            return Some(Place::Field(self.raw_this(), f, fty));
        }
        if is_final {
            let why = if self.p.class(f.class).is_value() { "a field of a value class is final" } else { "it is final" };
            self.err(pos, format!("the field `{name}` is not assigned here: {why}"));
            return None;
        }
        let recv = match recv {
            Some(r) => r,
            None => {
                let t = self.this_value(pos);
                if t.ty.is_error() {
                    return None;
                }
                t
            }
        };
        if self.p.may_be_null(&recv.ty) {
            let s = self.show(&recv.ty);
            self.err(pos, format!("the field `{name}` is assigned through a `{s}`, which may be `null`"));
            return None;
        }
        // Read the field's type as section 7.4 reads a member's.
        let cap = self.capture(&recv.ty);
        let ty = match self.p.supertype_at(&cap, f.class) {
            Some(up) => Subst::for_class(f.class, up.args()).apply(&fty),
            None => fty,
        };
        let recv = if compound {
            let t = self.temp(recv.ty.clone());
            let r = TExpr { kind: TKind::Local(t), ty: recv.ty.clone() };
            binds.push((t, recv));
            r
        } else {
            recv
        };
        Some(Place::Field(recv, f, ty))
    }

    fn place_type(&mut self, p: &Place) -> Option<Type> {
        match p {
            Place::Local(fi, id) => Some(self.frames[*fi].locals[*id as usize].ty.clone()),
            Place::Field(_, _, t) | Place::Static(_, t) => Some(t.clone()),
            Place::Index(..) => None,
        }
    }

    fn place_read(&mut self, p: &Place, pos: Pos) -> TExpr {
        match p {
            Place::Local(fi, id) => self.local_read(*fi, *id, pos),
            Place::Field(r, f, t) => {
                let ty = self.project(t);
                TExpr { kind: TKind::Field(Box::new(r.clone()), *f), ty }
            }
            Place::Static(f, t) => TExpr { kind: TKind::StaticField(*f), ty: t.clone() },
            Place::Index(a, i) => {
                let mut site = CallSite::new(vec![ArgIn::Done(i.clone())], pos);
                site.operator = Some("[]");
                self.call_method(a.clone(), "get", site)
            }
        }
    }

    /// Stores a checked value. The result is the assignment expression.
    fn place_write(&mut self, p: Place, value: TExpr, pos: Pos) -> TExpr {
        match p {
            Place::Local(fi, id) => {
                let at = self.frames.len() - 1;
                let name = self.frames[fi].locals[id as usize].name.clone();
                if fi != at {
                    return self.error_expr(pos, format!("a lambda does not assign `{name}`, a variable of the enclosing method"));
                }
                let f = self.frame();
                let seen = f.flow.maybe.contains(&id);
                if seen {
                    f.meta[id as usize].reassigned = true;
                }
                if seen && f.meta[id as usize].is_final {
                    self.err(pos, format!("`{name}` is not assignable here: it is final and may already have a value"));
                }
                let f = self.frame();
                f.flow.assigned.insert(id);
                f.flow.maybe.insert(id);
                f.flow.facts.remove(&id);
                let ty = f.locals[id as usize].ty.clone();
                TExpr { kind: TKind::AssignLocal(id, Box::new(value)), ty }
            }
            Place::Field(r, f, t) => TExpr { kind: TKind::AssignField(Box::new(r), f, Box::new(value)), ty: t },
            Place::Static(f, t) => TExpr { kind: TKind::AssignStatic(f, Box::new(value)), ty: t },
            Place::Index(a, i) => {
                let mut site = CallSite::new(vec![ArgIn::Done(i), ArgIn::Done(value)], pos);
                site.operator = Some("[]=");
                self.call_method(a, "set", site)
            }
        }
    }

    fn wrap(&self, binds: Vec<(LocalId, TExpr)>, body: TExpr) -> TExpr {
        let mut out = body;
        for (id, init) in binds.into_iter().rev() {
            let ty = out.ty.clone();
            out = TExpr { kind: TKind::Let(id, Box::new(init), Box::new(out)), ty };
        }
        out
    }

    /// `target = value` and `target op= value`. `want_value` is false where the
    /// expression is a statement and its value is discarded.
    pub fn assign(&mut self, op: Option<BinOp>, target: &ast::Expr, value: &ast::Expr, pos: Pos, want_value: bool) -> TExpr {
        // `a[i] = e` with nothing compound is one call of `set`.
        if let (None, ExprKind::Index(a, i), false) = (op, &target.kind, want_value) {
            let recv = self.check_expr(a, None);
            if recv.ty.is_error() {
                return recv;
            }
            let mut site = CallSite::new(vec![ArgIn::Ast(i), ArgIn::Ast(value)], pos);
            site.operator = Some("[]=");
            return self.call_method(recv, "set", site);
        }
        let mut binds = Vec::new();
        let Some(place) = self.place(target, &mut binds, op.is_some()) else {
            if !needs_expected(value) {
                self.check_expr(value, None);
            }
            return TExpr { kind: TKind::Null, ty: Type::error() };
        };
        let pty = self.place_type(&place);
        let v = match op {
            None => {
                let v = self.check_expr(value, pty.as_ref());
                match &pty {
                    Some(t) => self.coerce(v, t, value.pos),
                    None => v,
                }
            }
            Some(op) => {
                let cur = self.place_read(&place, pos);
                if cur.ty.is_error() {
                    return cur;
                }
                let is_shift = matches!(op, BinOp::Shl | BinOp::Shr);
                let hint = if cur.ty.nullable { None } else { self.p.numeric_class(&cur.ty) };
                let r = if is_literal_expr(value) || needs_expected(value) {
                    let mut site = CallSite::new(vec![ArgIn::Ast(value)], pos);
                    site.operator = Some(op.spelling());
                    site.lit_hint = if is_shift { None } else { hint };
                    self.call_method(cur, op.method(), site)
                } else {
                    let r = self.check_expr(value, None);
                    if r.ty.is_error() {
                        return r;
                    }
                    let (l, r) = if is_shift { (cur, r) } else { self.balance(cur, r, pos) };
                    self.operator(op, l, r, pos)
                };
                match &pty {
                    Some(t) => self.coerce(r, t, pos),
                    None => r,
                }
            }
        };
        if v.ty.is_error() {
            // Still record the assignment, so one mistake gives one message.
            if let Place::Local(fi, id) = place {
                if fi == self.frames.len() - 1 {
                    let f = self.frame();
                    f.flow.assigned.insert(id);
                    f.flow.maybe.insert(id);
                }
            }
            return v;
        }
        if let (Place::Index(..), true) = (&place, want_value) {
            // The value of the expression is the value stored.
            let tv = self.temp(v.ty.clone());
            let rv = TExpr { kind: TKind::Local(tv), ty: v.ty.clone() };
            binds.push((tv, v));
            let set = self.place_write(place, rv.clone(), pos);
            let ty = rv.ty.clone();
            let body = TExpr { kind: TKind::Seq(vec![set], Box::new(rv)), ty };
            return self.wrap(binds, body);
        }
        let out = self.place_write(place, v, pos);
        self.wrap(binds, out)
    }

    pub fn inc_dec(&mut self, inc: bool, prefix: bool, target: &ast::Expr, pos: Pos, want_value: bool) -> TExpr {
        let mut binds = Vec::new();
        let Some(place) = self.place(target, &mut binds, true) else {
            return TExpr { kind: TKind::Null, ty: Type::error() };
        };
        let cur = self.place_read(&place, pos);
        if cur.ty.is_error() {
            return cur;
        }
        let pty = self.place_type(&place);
        let one = ast::Expr { kind: ExprKind::Int("1".into(), 10), pos };
        let op = if inc { BinOp::Add } else { BinOp::Sub };
        let hint = if cur.ty.nullable { None } else { self.p.numeric_class(&cur.ty) };
        // The postfix form yields the value from before the update.
        let (cur, old) = if !prefix && want_value {
            let t = self.temp(cur.ty.clone());
            let r = TExpr { kind: TKind::Local(t), ty: cur.ty.clone() };
            binds.push((t, cur));
            (r.clone(), Some(r))
        } else {
            (cur, None)
        };

        let mut site = CallSite::new(vec![ArgIn::Ast(&one)], pos);
        site.operator = Some(op.spelling());
        site.lit_hint = hint;
        let next = self.call_method(cur, op.method(), site);
        if next.ty.is_error() {
            return next;
        }
        let next = match &pty {
            Some(t) => self.coerce(next, t, pos),
            None => next,
        };
        let write = self.place_write(place, next, pos);
        let body = match old {
            Some(o) => {
                let ty = o.ty.clone();
                TExpr { kind: TKind::Seq(vec![write], Box::new(o)), ty }
            }
            None => write,
        };
        self.wrap(binds, body)
    }
}
