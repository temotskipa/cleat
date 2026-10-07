//! Types as the checker sees them: a class with arguments or a type variable, with
//! qualifiers. Chapters 5, 7 and 8 define the rules.

use std::collections::HashMap;

pub type ClassId = u32;

/// A method or constructor of a class, by its index in the class's list.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MethodRef {
    pub class: ClassId,
    pub index: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FieldRef {
    pub class: ClassId,
    pub index: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TvOwner {
    Class(ClassId),
    Method(MethodRef),
    /// The unknown type that a wildcard stands for in one expression.
    Capture(u32),
    /// A type parameter being solved by inference.
    Infer(u32),
}

/// A type variable: a type parameter, a captured wildcard, or an inference variable.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tv {
    pub owner: TvOwner,
    pub index: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ty {
    Class(ClassId, Vec<Arg>),
    Var(Tv),
    /// The type of the literal `null`.
    Null,
    /// The type of an expression that was already reported. It is compatible with
    /// everything, so one mistake gives one message.
    Error,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Type {
    pub ty: Ty,
    pub nullable: bool,
    /// The qualifiers other than `@Nullable`, sorted.
    pub quals: Vec<ClassId>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Arg {
    Ty(Type),
    /// `? extends U` is `(Some(U), None)`, `? super L` is `(None, Some(L))`, and `?` is
    /// `(None, None)`.
    Wild(Option<Box<Type>>, Option<Box<Type>>),
}

impl Type {
    pub fn class(id: ClassId, args: Vec<Arg>) -> Type {
        Type { ty: Ty::Class(id, args), nullable: false, quals: Vec::new() }
    }

    pub fn simple(id: ClassId) -> Type {
        Type::class(id, Vec::new())
    }

    pub fn var(tv: Tv) -> Type {
        Type { ty: Ty::Var(tv), nullable: false, quals: Vec::new() }
    }

    pub fn error() -> Type {
        Type { ty: Ty::Error, nullable: false, quals: Vec::new() }
    }

    pub fn null() -> Type {
        Type { ty: Ty::Null, nullable: true, quals: Vec::new() }
    }

    pub fn is_error(&self) -> bool {
        matches!(self.ty, Ty::Error)
    }

    pub fn is_null_literal(&self) -> bool {
        matches!(self.ty, Ty::Null)
    }

    pub fn class_id(&self) -> Option<ClassId> {
        match &self.ty {
            Ty::Class(id, _) => Some(*id),
            _ => None,
        }
    }

    pub fn args(&self) -> &[Arg] {
        match &self.ty {
            Ty::Class(_, args) => args,
            _ => &[],
        }
    }

    pub fn nullable(mut self) -> Type {
        self.nullable = true;
        self
    }

    pub fn non_null(mut self) -> Type {
        self.nullable = false;
        self
    }

    pub fn with_nullable(mut self, nullable: bool) -> Type {
        self.nullable = nullable;
        self
    }

    /// The type with every qualifier removed.
    pub fn bare(&self) -> Type {
        Type { ty: self.ty.clone(), nullable: false, quals: Vec::new() }
    }

    pub fn add_qual(&mut self, q: ClassId) {
        if let Err(at) = self.quals.binary_search(&q) {
            self.quals.insert(at, q);
        }
    }

    pub fn has_wildcard_arg(&self) -> bool {
        self.args().iter().any(|a| matches!(a, Arg::Wild(..)))
    }

    /// Whether the type mentions the variable anywhere.
    pub fn mentions(&self, pred: &dyn Fn(Tv) -> bool) -> bool {
        match &self.ty {
            Ty::Var(tv) => pred(*tv),
            Ty::Class(_, args) => args.iter().any(|a| match a {
                Arg::Ty(t) => t.mentions(pred),
                Arg::Wild(e, s) => {
                    e.as_ref().map(|t| t.mentions(pred)).unwrap_or(false)
                        || s.as_ref().map(|t| t.mentions(pred)).unwrap_or(false)
                }
            }),
            _ => false,
        }
    }
}

impl Arg {
    pub fn as_type(&self) -> Option<&Type> {
        match self {
            Arg::Ty(t) => Some(t),
            _ => None,
        }
    }
}

/// A substitution of types for type variables.
#[derive(Clone, Debug, Default)]
pub struct Subst {
    pub map: HashMap<Tv, Arg>,
}

impl Subst {
    pub fn new() -> Subst {
        Subst { map: HashMap::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn bind(&mut self, tv: Tv, t: Type) {
        self.map.insert(tv, Arg::Ty(t));
    }

    /// Binds the parameters of a class to the arguments of one of its instantiations.
    pub fn for_class(class: ClassId, args: &[Arg]) -> Subst {
        let mut s = Subst::new();
        for (i, a) in args.iter().enumerate() {
            s.map.insert(Tv { owner: TvOwner::Class(class), index: i as u32 }, a.clone());
        }
        s
    }

    pub fn apply(&self, t: &Type) -> Type {
        if self.map.is_empty() {
            return t.clone();
        }
        match &t.ty {
            Ty::Var(tv) => match self.map.get(tv) {
                Some(Arg::Ty(r)) => {
                    // The qualifiers written on the variable's use join those of its argument.
                    let mut out = r.clone();
                    out.nullable |= t.nullable;
                    for q in &t.quals {
                        out.add_qual(*q);
                    }
                    out
                }
                // A wildcard reaches here only for a variable in argument position,
                // which `apply_arg` handles. Standing alone it is read as its bound.
                Some(Arg::Wild(Some(u), _)) => {
                    let mut out = (**u).clone();
                    out.nullable |= t.nullable;
                    out
                }
                Some(Arg::Wild(None, _)) | None => t.clone(),
            },
            Ty::Class(id, args) => Type {
                ty: Ty::Class(*id, args.iter().map(|a| self.apply_arg(a)).collect()),
                nullable: t.nullable,
                quals: t.quals.clone(),
            },
            _ => t.clone(),
        }
    }

    pub fn apply_arg(&self, a: &Arg) -> Arg {
        match a {
            Arg::Ty(t) => {
                if let Ty::Var(tv) = &t.ty {
                    if let Some(w @ Arg::Wild(..)) = self.map.get(tv) {
                        return w.clone();
                    }
                }
                Arg::Ty(self.apply(t))
            }
            Arg::Wild(e, s) => Arg::Wild(
                e.as_ref().map(|t| Box::new(self.apply(t))),
                s.as_ref().map(|t| Box::new(self.apply(t))),
            ),
        }
    }

    /// `self` applied after `inner`: the result maps a variable as `inner` does and then
    /// applies `self` to what it finds.
    pub fn compose(&self, inner: &Subst) -> Subst {
        let mut out = self.clone();
        for (k, v) in &inner.map {
            out.map.insert(*k, self.apply_arg(v));
        }
        out
    }
}
