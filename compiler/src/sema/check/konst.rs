//! Constant expressions (section 12.1) and the evaluation of annotations (chapter 8).

use super::Checker;
use crate::ast::{self, BinOp, ExprKind};
use crate::lex::Pos;
use crate::sema::decl::lookup_class;
use crate::sema::program::*;
use crate::sema::tir::*;
use crate::sema::types::*;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

pub enum Fold {
    Value(Const),
    /// Evaluating the expression would raise the named exception.
    Raises(&'static str),
    NotConst,
}

pub fn int_range(kind: NumKind) -> Option<(i128, i128)> {
    match kind {
        NumKind::Signed(b) => Some((-(1i128 << (b - 1)), (1i128 << (b - 1)) - 1)),
        NumKind::Unsigned(b) => Some((0, (1i128 << b) - 1)),
        _ => None,
    }
}

/// The exact value of a decimal literal such as `19.99` or `5.97e24`.
pub fn parse_decimal(text: &str) -> BigRational {
    let (mantissa, exp) = match text.split_once('e') {
        Some((m, e)) => (m, e.parse::<i64>().unwrap_or(0)),
        None => (text, 0),
    };
    let (whole, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits: BigInt = format!("{whole}{frac}").parse().unwrap_or_else(|_| BigInt::zero());
    let scale = exp - frac.len() as i64;
    let ten = BigInt::from(10);
    if scale >= 0 {
        BigRational::from_integer(digits * num_traits::pow(ten, scale as usize))
    } else {
        BigRational::new(digits, num_traits::pow(ten, (-scale) as usize))
    }
}

pub fn parse_int(digits: &str, radix: u32) -> BigInt {
    BigInt::parse_bytes(digits.as_bytes(), radix).unwrap_or_else(BigInt::zero)
}

pub fn rational_text(r: &BigRational) -> String {
    format!("{}/{}", r.numer(), r.denom())
}

pub fn rational_to_string(r: &BigRational) -> String {
    if r.is_integer() {
        r.numer().to_string()
    } else {
        format!("{}/{}", r.numer(), r.denom())
    }
}

fn rational_to_f64(r: &BigRational) -> f64 {
    // Correctly rounded: format exactly enough digits and let the standard parser round.
    let n = r.numer();
    let d = r.denom();
    if let (Some(a), Some(b)) = (n.to_i64(), d.to_i64()) {
        if a.unsigned_abs() < (1u64 << 53) && (b as u64) < (1u64 << 53) {
            return a as f64 / b as f64;
        }
    }
    let scale = num_traits::pow(BigInt::from(10), 40);
    let q: BigInt = (n * &scale) / d;
    format!("{q}e-40").parse::<f64>().unwrap_or(f64::NAN)
}

fn f64_to_rational(f: f64) -> Option<BigRational> {
    BigRational::from_float(f)
}

impl<'p> Checker<'p> {
    /// The constant a checked expression denotes, when it is one.
    pub fn as_const(&mut self, e: &TExpr) -> Option<Const> {
        match &e.kind {
            TKind::Int(v) => Some(Const::Int(*v, e.ty.class_id()?)),
            TKind::Float(f) => Some(Const::Float(*f, e.ty.class_id()?)),
            TKind::Rational(s) => {
                let (n, d) = s.split_once('/')?;
                Some(Const::Rational(BigRational::new(n.parse().ok()?, d.parse().ok()?)))
            }
            TKind::Bool(b) => Some(Const::Bool(*b)),
            TKind::Char(c) => Some(Const::Char(*c)),
            TKind::Str(s) => Some(Const::Str(s.clone())),
            TKind::Coerce(inner) => self.as_const(inner),
            TKind::StaticField(f) => {
                let field = self.p.field(*f);
                if field.enum_ordinal.is_some() {
                    return Some(Const::Enum(*f));
                }
                field.constant.clone()
            }
            TKind::ClassLit(t) => Some(Const::Class(t.clone())),
            _ => None,
        }
    }

    pub fn const_expr(&self, c: &Const) -> TExpr {
        let w = &self.p.wk;
        match c {
            Const::Int(v, class) => TExpr { kind: TKind::Int(*v), ty: Type::simple(*class) },
            Const::Float(f, class) => TExpr { kind: TKind::Float(*f), ty: Type::simple(*class) },
            Const::Rational(r) => TExpr { kind: TKind::Rational(rational_text(r)), ty: Type::simple(w.rational) },
            Const::Bool(b) => TExpr { kind: TKind::Bool(*b), ty: Type::simple(w.boolean) },
            Const::Char(c) => TExpr { kind: TKind::Char(*c), ty: Type::simple(w.char_) },
            Const::Str(s) => TExpr { kind: TKind::Str(s.clone()), ty: Type::simple(w.string) },
            Const::Enum(f) => TExpr { kind: TKind::StaticField(*f), ty: Type::simple(f.class) },
            Const::Class(t) => TExpr { kind: TKind::ClassLit(t.clone()), ty: Type::simple(w.class) },
            Const::Ann(_) | Const::Array(_) => TExpr { kind: TKind::Null, ty: Type::error() },
        }
    }

    /// A numeric literal value as a constant of the numeric class `class`, or the
    /// reason it does not fit.
    pub fn literal_const(&self, value: &BigRational, is_decimal: bool, class: ClassId) -> Result<Const, String> {
        let kind = self.p.num_kind(class).expect("a numeric class");
        let name = self.p.class(class).name.clone();
        match kind {
            NumKind::Signed(_) | NumKind::Unsigned(_) => {
                if is_decimal {
                    return Err(format!("a decimal literal is not a value of `{name}`"));
                }
                let (lo, hi) = int_range(kind).unwrap();
                match value.numer().to_i128() {
                    Some(v) if v >= lo && v <= hi => Ok(Const::Int(v, class)),
                    _ => Err(format!("the literal is outside the range of `{name}`")),
                }
            }
            NumKind::Float(bits) => {
                let f = rational_to_f64(value);
                let f = if bits == 32 { (f as f32) as f64 } else { f };
                if f.is_infinite() {
                    return Err(format!("the literal is too large for `{name}`"));
                }
                Ok(Const::Float(f, class))
            }
            NumKind::Rational => Ok(Const::Rational(value.clone())),
        }
    }

    /// Folds a call of a prelude method on constants.
    pub fn fold_method(&self, class: ClassId, name: &str, recv: Option<&Const>, args: &[Const], ret: &Type) -> Fold {
        let w = self.p.wk.clone();
        let int_result = |v: i128, class: ClassId| -> Fold {
            let (lo, hi) = int_range(self.p.num_kind(class).unwrap()).unwrap();
            if v < lo || v > hi {
                Fold::Raises("ArithmeticException")
            } else {
                Fold::Value(Const::Int(v, class))
            }
        };
        // Conversions: static `from` and `nearest` on a numeric class.
        if recv.is_none() && args.len() == 1 && (name == "from" || name == "nearest") {
            if class == w.char_ {
                if let Const::Int(v, _) = &args[0] {
                    return match u32::try_from(*v).ok().and_then(char::from_u32) {
                        Some(c) => Fold::Value(Const::Char(c)),
                        None => Fold::Raises("ArithmeticException"),
                    };
                }
                return Fold::NotConst;
            }
            let Some(kind) = self.p.num_kind(class) else { return Fold::NotConst };
            let exact: Option<BigRational> = match &args[0] {
                Const::Int(v, _) => Some(BigRational::from_integer(BigInt::from(*v))),
                Const::Rational(r) => Some(r.clone()),
                Const::Float(f, _) => f64_to_rational(*f),
                _ => return Fold::NotConst,
            };
            return match kind {
                NumKind::Signed(_) | NumKind::Unsigned(_) => {
                    if name == "nearest" {
                        return Fold::NotConst;
                    }
                    match exact {
                        Some(r) if r.is_integer() => match r.numer().to_i128() {
                            Some(v) => int_result(v, class),
                            None => Fold::Raises("ArithmeticException"),
                        },
                        _ => Fold::Raises("ArithmeticException"),
                    }
                }
                NumKind::Float(bits) => {
                    let Some(r) = exact else {
                        // NaN or an infinity converts to itself between float classes.
                        return match &args[0] {
                            Const::Float(f, _) => Fold::Value(Const::Float(*f, class)),
                            _ => Fold::NotConst,
                        };
                    };
                    let f = rational_to_f64(&r);
                    let f = if bits == 32 { (f as f32) as f64 } else { f };
                    if name == "from" && f64_to_rational(f).as_ref() != Some(&r) {
                        return Fold::Raises("ArithmeticException");
                    }
                    Fold::Value(Const::Float(f, class))
                }
                NumKind::Rational => match exact {
                    Some(r) => Fold::Value(Const::Rational(r)),
                    None => Fold::Raises("ArithmeticException"),
                },
            };
        }
        let Some(recv) = recv else { return Fold::NotConst };
        match (recv, args) {
            // Integer arithmetic.
            (Const::Int(a, c), _) if *c == class && self.p.num_kind(class).is_some() => {
                let kind = self.p.num_kind(class).unwrap();
                let bits = match kind {
                    NumKind::Signed(b) | NumKind::Unsigned(b) => b,
                    _ => return Fold::NotConst,
                };
                let wrap = |v: i128| -> i128 {
                    let m = 1i128 << bits;
                    let mut r = v.rem_euclid(m);
                    if matches!(kind, NumKind::Signed(_)) && r >= m / 2 {
                        r -= m;
                    }
                    r
                };
                let a = *a;
                let b = match args.first() {
                    Some(Const::Int(b, _)) => Some(*b),
                    _ => None,
                };
                let boolean = |v: bool| Fold::Value(Const::Bool(v));
                match (name, b) {
                    ("negate", None) => int_result(-a, class),
                    ("complement", None) => Fold::Value(Const::Int(wrap(!a), class)),
                    ("plus", Some(b)) => int_result(a + b, class),
                    ("minus", Some(b)) => int_result(a - b, class),
                    ("times", Some(b)) => match a.checked_mul(b) {
                        Some(v) => int_result(v, class),
                        None => Fold::Raises("ArithmeticException"),
                    },
                    ("wrappingPlus", Some(b)) => Fold::Value(Const::Int(wrap(a.wrapping_add(b)), class)),
                    ("wrappingMinus", Some(b)) => Fold::Value(Const::Int(wrap(a.wrapping_sub(b)), class)),
                    ("wrappingTimes", Some(b)) => Fold::Value(Const::Int(wrap(a.wrapping_mul(b)), class)),
                    ("floorDiv", Some(b)) | ("mod", Some(b)) | ("truncatingDiv", Some(b)) | ("truncatingRem", Some(b)) => {
                        if b == 0 {
                            return Fold::Raises("ArithmeticException");
                        }
                        let v = match name {
                            "floorDiv" => a.div_euclid(b) - if b < 0 && a.rem_euclid(b) != 0 { 1 } else { 0 },
                            "mod" => {
                                let r = a % b;
                                if r != 0 && ((r < 0) != (b < 0)) {
                                    r + b
                                } else {
                                    r
                                }
                            }
                            "truncatingDiv" => a / b,
                            _ => a % b,
                        };
                        // `floorDiv` above is corrected to the floor for a negative divisor.
                        let v = if name == "floorDiv" {
                            let q = a / b;
                            if (a % b != 0) && ((a < 0) != (b < 0)) {
                                q - 1
                            } else {
                                q
                            }
                        } else {
                            v
                        };
                        int_result(v, class)
                    }
                    ("and", Some(b)) => Fold::Value(Const::Int(wrap(a & b), class)),
                    ("or", Some(b)) => Fold::Value(Const::Int(wrap(a | b), class)),
                    ("xor", Some(b)) => Fold::Value(Const::Int(wrap(a ^ b), class)),
                    ("shiftLeft", Some(b)) | ("shiftRight", Some(b)) => {
                        if b < 0 || b >= bits as i128 {
                            return Fold::Raises("ArithmeticException");
                        }
                        let v = if name == "shiftLeft" { wrap(a << b) } else { a >> b };
                        Fold::Value(Const::Int(v, class))
                    }
                    ("lessThan", Some(b)) => boolean(a < b),
                    ("atMost", Some(b)) => boolean(a <= b),
                    ("greaterThan", Some(b)) => boolean(a > b),
                    ("atLeast", Some(b)) => boolean(a >= b),
                    ("compare", Some(b)) => Fold::Value(Const::Int((a > b) as i128 - (a < b) as i128, w.int)),
                    _ => Fold::NotConst,
                }
            }
            (Const::Float(a, c), _) if *c == class => {
                let single = class == w.float32;
                let fix = |v: f64| Fold::Value(Const::Float(if single { (v as f32) as f64 } else { v }, class));
                let a = *a;
                let b = match args.first() {
                    Some(Const::Float(b, _)) => Some(*b),
                    _ => None,
                };
                let boolean = |v: bool| Fold::Value(Const::Bool(v));
                match (name, b) {
                    ("negate", None) => fix(-a),
                    ("plus", Some(b)) => fix(a + b),
                    ("minus", Some(b)) => fix(a - b),
                    ("times", Some(b)) => fix(a * b),
                    ("div", Some(b)) => fix(a / b),
                    ("lessThan", Some(b)) => boolean(a < b),
                    ("atMost", Some(b)) => boolean(a <= b),
                    ("greaterThan", Some(b)) => boolean(a > b),
                    ("atLeast", Some(b)) => boolean(a >= b),
                    _ => Fold::NotConst,
                }
            }
            (Const::Rational(a), _) if class == w.rational => {
                let b = match args.first() {
                    Some(Const::Rational(b)) => Some(b.clone()),
                    _ => None,
                };
                let boolean = |v: bool| Fold::Value(Const::Bool(v));
                let val = |r: BigRational| Fold::Value(Const::Rational(r));
                match (name, b) {
                    ("negate", None) => val(-a),
                    ("plus", Some(b)) => val(a + b),
                    ("minus", Some(b)) => val(a - b),
                    ("times", Some(b)) => val(a * b),
                    ("div", Some(b)) => {
                        if b.is_zero() {
                            Fold::Raises("ArithmeticException")
                        } else {
                            val(a / b)
                        }
                    }
                    ("mod", Some(b)) | ("floorDiv", Some(b)) => {
                        if b.is_zero() {
                            return Fold::Raises("ArithmeticException");
                        }
                        let q = (a / &b).floor();
                        if name == "floorDiv" {
                            val(q)
                        } else {
                            val(a - q * b)
                        }
                    }
                    ("lessThan", Some(b)) => boolean(*a < b),
                    ("atMost", Some(b)) => boolean(*a <= b),
                    ("greaterThan", Some(b)) => boolean(*a > b),
                    ("atLeast", Some(b)) => boolean(*a >= b),
                    _ => Fold::NotConst,
                }
            }
            (Const::Bool(a), _) if class == w.boolean => {
                let b = match args.first() {
                    Some(Const::Bool(b)) => Some(*b),
                    _ => None,
                };
                let boolean = |v: bool| Fold::Value(Const::Bool(v));
                match (name, b) {
                    ("not", None) => boolean(!a),
                    ("and", Some(b)) => boolean(*a && b),
                    ("or", Some(b)) => boolean(*a || b),
                    ("xor", Some(b)) => boolean(*a != b),
                    _ => Fold::NotConst,
                }
            }
            (Const::Str(a), [other]) if class == w.string && name == "plus" => {
                let text = match other {
                    Const::Str(s) => s.clone(),
                    Const::Int(v, _) => v.to_string(),
                    Const::Bool(b) => b.to_string(),
                    Const::Char(c) => c.to_string(),
                    Const::Rational(r) => rational_to_string(r),
                    _ => return Fold::NotConst,
                };
                let _ = ret;
                Fold::Value(Const::Str(format!("{a}{text}")))
            }
            _ => Fold::NotConst,
        }
    }

    /// `a == b` on two constants of one class.
    pub fn fold_equals(&self, a: &Const, b: &Const) -> Option<bool> {
        Some(match (a, b) {
            (Const::Int(x, c1), Const::Int(y, c2)) if c1 == c2 => x == y,
            (Const::Float(x, c1), Const::Float(y, c2)) if c1 == c2 => x == y || (x.is_nan() && y.is_nan()),
            (Const::Rational(x), Const::Rational(y)) => x == y,
            (Const::Bool(x), Const::Bool(y)) => x == y,
            (Const::Char(x), Const::Char(y)) => x == y,
            (Const::Str(x), Const::Str(y)) => x == y,
            (Const::Enum(x), Const::Enum(y)) => x == y,
            _ => return None,
        })
    }
}

// ---- constant fields ----

/// Whether an initializer is built only from the forms a constant expression has.
fn looks_constant(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Int(..) | ExprKind::Dec(_) | ExprKind::Char(_) | ExprKind::Str(_) | ExprKind::Bool(_) => true,
        ExprKind::Name(_) => true,
        ExprKind::Field(t, _) => looks_constant(t),
        ExprKind::Paren(x) | ExprKind::Unary(_, x) => looks_constant(x),
        ExprKind::Binary(_, a, b) | ExprKind::And(a, b) | ExprKind::Or(a, b) => looks_constant(a) && looks_constant(b),
        ExprKind::Cond(c, a, b) => looks_constant(c) && looks_constant(a) && looks_constant(b),
        ExprKind::Call { target: Some(t), name, args, .. } => {
            (name == "from" || name == "nearest") && looks_constant(t) && args.iter().all(looks_constant)
        }
        _ => false,
    }
}

fn const_type_ok(p: &Program, t: &Type) -> bool {
    if t.nullable || !t.quals.is_empty() {
        return false;
    }
    let Some(id) = t.class_id() else { return false };
    p.num_kind(id).is_some() || id == p.wk.boolean || id == p.wk.char_ || id == p.wk.string
}

/// Finds the constant fields of section 12.1. A constant field may be defined by
/// another one, so the search repeats until it finds nothing new.
fn constant_fields(p: &mut Program) {
    loop {
        let mut progress = false;
        for id in 0..p.classes.len() as ClassId {
            for fi in 0..p.class(id).fields.len() {
                let f = p.class(id).fields[fi].clone();
                if !f.is_static || !f.is_final || f.constant.is_some() || !const_type_ok(p, &f.ty) {
                    continue;
                }
                let Some(init) = &f.init else { continue };
                if !looks_constant(init) {
                    continue;
                }
                // Evaluate quietly: the real check of the initializer reports its mistakes.
                let saved = p.diags.len();
                let value = {
                    let mut ck = Checker::new(p, id, true);
                    ck.push_frame(None, None);
                    let e = ck.check_expr(init, Some(&f.ty));
                    let e = ck.coerce(e, &f.ty, init.pos);
                    ck.as_const(&e)
                };
                p.diags.truncate(saved);
                if let Some(v) = value {
                    p.classes[id as usize].fields[fi].constant = Some(v);
                    progress = true;
                }
            }
        }
        if !progress {
            break;
        }
    }
}

// ---- annotations ----

impl<'p> Checker<'p> {
    /// Evaluates one use of a declaration annotation written at `site`.
    pub fn eval_annotation(&mut self, a: &ast::AnnotationUse, site: Option<Site>) -> Option<AnnValue> {
        let class = lookup_class(self.p, self.unit, &a.name, a.pos, true)?;
        let c = self.p.class(class).clone();
        if !c.is_annotation() {
            self.err(a.pos, format!("`{}` is not an annotation", c.name));
            return None;
        }
        if let (Some(site), Some(targets)) = (site, &c.targets) {
            if !targets.contains(&site) {
                self.err(a.pos, format!("`@{}` may not be written here: its `@Target` does not list this site", c.name));
            }
        }
        let n = c.elements.len();
        let mut sources: Vec<Option<ast::AnnValue>> = vec![None; n];
        match &a.args {
            ast::AnnArgs::None => {}
            ast::AnnArgs::Positional(list) => {
                if list.len() > n {
                    self.err(a.pos, format!("`@{}` has {n} elements, and {} arguments are written", c.name, list.len()));
                }
                for (i, v) in list.iter().enumerate().take(n) {
                    sources[i] = Some(v.clone());
                }
            }
            ast::AnnArgs::Named(list) => {
                for (name, v) in list {
                    match c.elements.iter().position(|e| e.name == *name) {
                        Some(i) if sources[i].is_none() => sources[i] = Some(v.clone()),
                        Some(_) => self.err(a.pos, format!("the element `{name}` is given twice")),
                        None => self.err(a.pos, format!("`@{}` has no element named `{name}`", c.name)),
                    }
                }
            }
        }
        let mut values = Vec::new();
        for (i, e) in c.elements.iter().enumerate() {
            let v = match &sources[i] {
                Some(src) => self.eval_ann_value(&e.ty, src, a.pos),
                None => match &e.default {
                    Some(d) => Some(d.clone()),
                    None => {
                        self.err(a.pos, format!("the element `{}` of `@{}` has no default and must be supplied", e.name, c.name));
                        None
                    }
                },
            };
            values.push(v?);
        }
        Some(AnnValue { class, values })
    }

    pub fn eval_ann_value(&mut self, ty: &Type, src: &ast::AnnValue, pos: Pos) -> Option<Const> {
        if let Some(elem) = self.p.array_elem(ty) {
            let items: Vec<ast::AnnValue> = match src {
                ast::AnnValue::List(l) => l.clone(),
                single => vec![single.clone()],
            };
            let mut out = Vec::new();
            for it in &items {
                out.push(self.eval_ann_value(&elem, it, pos)?);
            }
            return Some(Const::Array(out));
        }
        match src {
            ast::AnnValue::List(_) => {
                self.err(pos, "a list in braces is the value of an array element");
                None
            }
            ast::AnnValue::Ann(use_) => {
                let v = self.eval_annotation(use_, None)?;
                if Some(v.class) != ty.class_id() {
                    let want = self.show(ty);
                    self.err(use_.pos, format!("this element takes a `{want}`"));
                    return None;
                }
                Some(Const::Ann(v))
            }
            ast::AnnValue::Expr(e) => {
                let checked = self.check_expr(e, Some(ty));
                let checked = self.coerce(checked, ty, e.pos);
                if checked.ty.is_error() {
                    return None;
                }
                match self.as_const(&checked) {
                    Some(c) => Some(c),
                    None => {
                        self.err(e.pos, "an annotation argument is a constant expression, a class literal, or a use of an annotation");
                        None
                    }
                }
            }
        }
    }
}

fn site_of(p: &Program, c: &Const) -> Option<Site> {
    let Const::Enum(f) = c else { return None };
    let name = &p.field(*f).name;
    SITE_NAMES.iter().find(|(n, _)| n == name).map(|(_, s)| *s)
}

fn string_arg(v: &AnnValue, i: usize) -> Option<String> {
    match v.values.get(i) {
        Some(Const::Str(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Evaluates every annotation of the program and applies the ones the compiler knows.
pub fn evaluate_annotations(p: &mut Program) {
    constant_fields(p);
    let n = p.classes.len() as ClassId;
    let wk = p.wk.clone();

    // Element defaults first: every later use of an annotation may need them.
    for id in 0..n {
        if !p.class(id).is_annotation() {
            continue;
        }
        for ei in 0..p.class(id).elements.len() {
            let e = p.class(id).elements[ei].clone();
            if let Some(src) = &e.default_src {
                let mut ck = Checker::new(p, id, true);
                ck.push_frame(None, None);
                let v = ck.eval_ann_value(&e.ty, src, e.pos);
                drop(ck);
                p.classes[id as usize].elements[ei].default = v;
            }
        }
    }

    // Annotation declarations first: their targets govern every later use.
    for round in 0..2 {
        for id in 0..n {
            if p.class(id).is_annotation() != (round == 0) {
                continue;
            }
            let uses = p.class(id).ann_uses.clone();
            let site = if p.class(id).is_annotation() { Site::Annotation } else { Site::Type };
            let mut ck = Checker::new(p, id, true);
            ck.push_frame(None, None);
            let mut anns = Vec::new();
            for a in &uses {
                // Qualifier marks and declaration annotations share the modifier position.
                if let Some(v) = ck.eval_annotation(a, Some(site)) {
                    if anns.iter().any(|o: &AnnValue| o.class == v.class) {
                        ck.err(a.pos, "an annotation is written at most once in one position");
                        continue;
                    }
                    anns.push(v);
                }
            }
            drop(ck);
            let unit = p.class(id).unit;
            let pos = p.class(id).pos;
            for v in &anns {
                if v.class == wk.target {
                    if p.class(id).qualifier != Qualifier::No {
                        p.error(unit, pos, "a qualifier is written on types, and `@Target` on its declaration is rejected");
                    }
                    if let Some(Const::Array(items)) = v.values.first() {
                        let sites: Vec<Site> = items.iter().filter_map(|c| site_of(p, c)).collect();
                        p.classes[id as usize].targets = Some(sites);
                    }
                } else if v.class == wk.refines {
                    if let Some(Const::Array(items)) = v.values.first() {
                        for c in items {
                            let Const::Class(t) = c else { continue };
                            match t.class_id() {
                                Some(above) if p.class(above).qualifier == Qualifier::Refines => {
                                    p.classes[id as usize].refines_above.push(above);
                                }
                                _ => p.error(unit, pos, "`@Refines` lists refinements"),
                            }
                        }
                    }
                } else if v.class == wk.deprecated {
                    p.classes[id as usize].deprecated = string_arg(v, 0);
                } else if v.class == wk.inherited && !p.class(id).is_annotation() {
                    p.error(unit, pos, "`@Inherited` is written on an annotation declaration");
                }
            }
            p.classes[id as usize].anns = anns;
        }
    }
    // A refinement must not sit below itself.
    for id in 0..n {
        if p.class(id).qualifier == Qualifier::Refines {
            let mut seen = vec![id];
            let mut work = p.class(id).refines_above.clone();
            while let Some(a) = work.pop() {
                if a == id {
                    let (unit, pos) = (p.class(id).unit, p.class(id).pos);
                    p.error(unit, pos, "this refinement is placed below itself");
                    p.classes[id as usize].refines_above.clear();
                    break;
                }
                if !seen.contains(&a) {
                    seen.push(a);
                    work.extend(p.class(a).refines_above.clone());
                }
            }
        }
    }

    // Members.
    for id in 0..n {
        let eval_list = |p: &mut Program, uses: &[ast::AnnotationUse], site: Site| -> Vec<AnnValue> {
            let mut ck = Checker::new(p, id, true);
            ck.push_frame(None, None);
            let mut anns: Vec<AnnValue> = Vec::new();
            for a in uses {
                if let Some(v) = ck.eval_annotation(a, Some(site)) {
                    if anns.iter().any(|o| o.class == v.class) {
                        ck.err(a.pos, "an annotation is written at most once in one position");
                        continue;
                    }
                    anns.push(v);
                }
            }
            anns
        };
        for fi in 0..p.class(id).fields.len() {
            let uses = p.class(id).fields[fi].ann_uses.clone();
            if uses.is_empty() {
                continue;
            }
            let anns = eval_list(p, &uses, Site::Field);
            p.classes[id as usize].fields[fi].anns = anns;
        }
        for mi in 0..p.class(id).methods.len() {
            let uses = p.class(id).methods[mi].ann_uses.clone();
            let anns = eval_list(p, &uses, Site::Method);
            let unit = p.class(id).unit;
            let m = p.class(id).methods[mi].clone();
            for v in &anns {
                if v.class == wk.narrows {
                    let q = match v.values.first() {
                        Some(Const::Class(t)) => t.class_id(),
                        _ => None,
                    };
                    match q {
                        Some(q) if p.class(q).qualifier != Qualifier::No => {
                            if p.class(q).package != p.class(id).package {
                                p.error(unit, m.pos, "a `@Narrows` method is declared in the package that declares its qualifier");
                            }
                            if m.ret != Type::simple(wk.boolean) {
                                p.error(unit, m.pos, "a `@Narrows` method returns `Boolean`");
                            }
                            if m.is_static && m.params.is_empty() {
                                p.error(unit, m.pos, "a static `@Narrows` method tests its first parameter");
                            }
                            p.classes[id as usize].methods[mi].narrows = Some(q);
                        }
                        _ => p.error(unit, m.pos, "`@Narrows` names a qualifier"),
                    }
                } else if v.class == wk.symbol {
                    if !m.foreign {
                        p.error(unit, m.pos, "`@Symbol` is written on a `foreign` method");
                    }
                    p.classes[id as usize].methods[mi].symbol = string_arg(v, 0);
                } else if v.class == wk.deprecated {
                    p.classes[id as usize].methods[mi].deprecated = string_arg(v, 0);
                } else if v.class == wk.implicit {
                    let c = p.class(id).clone();
                    let self_ty = Type::simple(id);
                    let shape_ok = m.is_static
                        && c.is_value()
                        && c.tparams.is_empty()
                        && m.tparams.is_empty()
                        && m.params.len() == 1
                        && m.ret == self_ty
                        && m.params[0].ty.class_id().map(|s| s != id && p.class(s).is_value()).unwrap_or(false)
                        && !m.params[0].ty.nullable;
                    if !shape_ok {
                        p.error(unit, m.pos, "an `@Implicit` method is a static method of a value class `T` with one parameter of another value class, the result `T`, and no type parameters");
                        p.classes[id as usize].methods[mi].implicit = false;
                    }
                }
            }
            p.classes[id as usize].methods[mi].anns = anns;
            for pi in 0..p.class(id).methods[mi].params.len() {
                let uses = p.class(id).methods[mi].params[pi].ann_uses.clone();
                if uses.is_empty() {
                    continue;
                }
                let anns = eval_list(p, &uses, Site::Parameter);
                p.classes[id as usize].methods[mi].params[pi].anns = anns;
            }
        }
        for ci in 0..p.class(id).ctors.len() {
            let uses = p.class(id).ctors[ci].ann_uses.clone();
            let anns = eval_list(p, &uses, Site::Constructor);
            for v in &anns {
                if v.class == wk.deprecated {
                    p.classes[id as usize].ctors[ci].deprecated = string_arg(v, 0);
                }
            }
            p.classes[id as usize].ctors[ci].anns = anns;
            for pi in 0..p.class(id).ctors[ci].params.len() {
                let uses = p.class(id).ctors[ci].params[pi].ann_uses.clone();
                if uses.is_empty() {
                    continue;
                }
                let anns = eval_list(p, &uses, Site::Parameter);
                p.classes[id as usize].ctors[ci].params[pi].anns = anns;
            }
        }
    }
    let _ = (BigInt::one(), BinOp::Add, BigRational::one().abs());
}
