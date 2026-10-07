//! The checked form of a body: every name resolved, every call chosen, every type known.
//! The backend reads this and nothing else of a body.

use super::types::*;
use crate::ast::BinOp;

pub type LocalId = u32;

#[derive(Clone, Debug)]
pub struct LocalVar {
    pub name: String,
    pub ty: Type,
    pub is_param: bool,
}

/// A checked method, constructor, initializer or lambda body.
#[derive(Clone, Debug)]
pub struct Body {
    pub locals: Vec<LocalVar>,
    /// The locals that are the parameters, in order.
    pub params: Vec<LocalId>,
    pub stmts: Vec<TStmt>,
}

#[derive(Clone, Debug)]
pub struct TExpr {
    pub kind: TKind,
    pub ty: Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dispatch {
    /// A static method, or a method whose body is fixed at compile time.
    Direct,
    /// The receiver's class selects the body.
    Virtual,
}

#[derive(Clone, Debug)]
pub struct Call {
    pub recv: Option<TExpr>,
    pub method: MethodRef,
    /// The arguments of the declaring class as the receiver's static type has them.
    pub class_args: Vec<Arg>,
    /// The method's own type arguments.
    pub targs: Vec<Type>,
    pub args: Vec<TExpr>,
    pub dispatch: Dispatch,
}

#[derive(Clone, Debug)]
pub struct Lambda {
    /// The class the lambda expression declares.
    pub class: ClassId,
    /// The values the instance holds, in the order of the class's fields.
    pub captures: Vec<TExpr>,
}

#[derive(Clone, Debug)]
pub enum ArmTest {
    /// The selector equals one of these constants.
    Constants(Vec<TExpr>),
    /// The selector is an instance of the type, bound to the local.
    Type(Type, LocalId),
    Default,
}

#[derive(Clone, Debug)]
pub enum TKind {
    Int(i128),
    Float(f64),
    /// An exact rational, written `numerator/denominator`.
    Rational(String),
    Bool(bool),
    Char(char),
    Str(String),
    Null,
    Unit,
    Local(LocalId),
    This,
    Field(Box<TExpr>, FieldRef),
    StaticField(FieldRef),
    Call(Box<Call>),
    /// A static requirement called through a type parameter: `T.zero()`.
    StaticReq { on: Type, method: MethodRef, args: Vec<TExpr> },
    New { ctor: MethodRef, args: Vec<TExpr> },
    NewArray { elem: Type, len: Box<TExpr> },
    ArrayLit { elem: Type, elems: Vec<TExpr> },
    ClassLit(Type),
    Lambda(Box<Lambda>),
    And(Box<TExpr>, Box<TExpr>),
    Or(Box<TExpr>, Box<TExpr>),
    Not(Box<TExpr>),
    Cond(Box<TExpr>, Box<TExpr>, Box<TExpr>),
    /// `a ?? b`: the left value unless it is null.
    Coalesce(Box<TExpr>, Box<TExpr>),
    AssignLocal(LocalId, Box<TExpr>),
    AssignField(Box<TExpr>, FieldRef, Box<TExpr>),
    AssignStatic(FieldRef, Box<TExpr>),
    /// A checked cast to the expression's type.
    Cast(Box<TExpr>),
    InstanceOf(Box<TExpr>, Type),
    /// `e == null` when the flag is true, `e != null` otherwise.
    IsNull(Box<TExpr>, bool),
    /// The same value seen at another static type. Nothing is checked at run time: the
    /// type rules have shown that the value belongs to the new type.
    Coerce(Box<TExpr>),
    /// An operator on two values of one built-in class, done without a call.
    Prim(BinOp, Box<TExpr>, Box<TExpr>),
    /// Binds a temporary and evaluates the body.
    Let(LocalId, Box<TExpr>, Box<TExpr>),
    Seq(Vec<TExpr>, Box<TExpr>),
    Switch { selector: Box<TExpr>, temp: LocalId, arms: Vec<(ArmTest, TExpr)> },
    /// Raises the value. It appears as the value of a switch arm.
    Throw(Box<TExpr>),
}

#[derive(Clone, Debug)]
pub struct TCatch {
    pub ty: Type,
    pub local: LocalId,
    pub body: Vec<TStmt>,
}

#[derive(Clone, Debug)]
pub enum TStmt {
    Expr(TExpr),
    /// Declares a local, with its initial value when it has one.
    Local(LocalId, Option<TExpr>),
    Block(Vec<TStmt>),
    If(TExpr, Vec<TStmt>, Vec<TStmt>),
    /// A loop. `label` names it for `Break` and `Continue`. `update` runs after the
    /// body and after a `continue`.
    Loop { label: u32, cond: Option<TExpr>, update: Vec<TExpr>, body: Vec<TStmt> },
    /// A statement that a `break label` may leave.
    Labeled { label: u32, body: Vec<TStmt> },
    Switch { label: u32, selector: TExpr, temp: LocalId, arms: Vec<(ArmTest, Vec<TStmt>)> },
    Try { body: Vec<TStmt>, catches: Vec<TCatch>, finally: Option<Vec<TStmt>> },
    /// `using`: the resource is bound, the body runs, and `close` is called. `close`
    /// is the checked call on the resource's local.
    Using { local: LocalId, init: TExpr, close: TExpr, body: Vec<TStmt> },
    Return(Option<TExpr>),
    Break(u32),
    Continue(u32),
    Throw(TExpr),
    /// `super(args)` or `this(args)` in a constructor.
    CtorCall { ctor: MethodRef, args: Vec<TExpr> },
}
