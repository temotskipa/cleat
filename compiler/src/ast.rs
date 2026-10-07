//! The syntax tree that the grammar of chapter 11 derives.

use crate::lex::Pos;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Unit {
    pub file: PathBuf,
    pub text: String,
    pub package: Vec<String>,
    pub imports: Vec<Import>,
    pub types: Vec<TypeDecl>,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub path: Vec<String>,
    pub star: bool,
    pub pos: Pos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Audience {
    Private,
    Package,
    Protected,
    Public,
}

#[derive(Clone, Debug, Default)]
pub struct Mods {
    pub annotations: Vec<AnnotationUse>,
    pub audience: Option<Audience>,
    pub only: Option<Vec<TypeRef>>,
    pub is_static: bool,
    pub is_final: bool,
    pub is_open: bool,
    pub is_abstract: bool,
    pub is_sealed: bool,
    pub is_foreign: bool,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct AnnotationUse {
    pub name: Vec<String>,
    pub args: AnnArgs,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub enum AnnArgs {
    None,
    Positional(Vec<AnnValue>),
    Named(Vec<(String, AnnValue)>),
}

#[derive(Clone, Debug)]
pub enum AnnValue {
    Expr(Expr),
    Ann(AnnotationUse),
    List(Vec<AnnValue>),
}

/// A type as written: annotations, a qualified name with type arguments, and array
/// brackets. `dims[0]` is the outermost array.
#[derive(Clone, Debug)]
pub struct TypeRef {
    pub annotations: Vec<AnnotationUse>,
    pub name: Vec<String>,
    pub args: Option<Vec<TypeArg>>,
    pub dims: Vec<Vec<AnnotationUse>>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub enum TypeArg {
    Type(TypeRef),
    /// `?`, `? extends T` (false) or `? super T` (true).
    Wildcard(Option<(bool, TypeRef)>, Pos),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variance {
    Invariant,
    In,
    Out,
}

#[derive(Clone, Debug)]
pub struct TypeParam {
    pub variance: Variance,
    pub annotations: Vec<AnnotationUse>,
    pub name: String,
    pub bounds: Vec<TypeRef>,
    pub pos: Pos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeKind {
    Class,
    ValueClass,
    Interface,
    Enum,
    Annotation,
}

#[derive(Clone, Debug)]
pub struct TypeDecl {
    pub kind: TypeKind,
    pub mods: Mods,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    /// The superclass of a class, or the superinterfaces of an interface.
    pub extends: Vec<TypeRef>,
    pub implements: Vec<TypeRef>,
    pub permits: Option<Vec<TypeRef>>,
    pub members: Vec<Member>,
    pub constants: Vec<EnumConstant>,
    pub elements: Vec<Element>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct EnumConstant {
    pub annotations: Vec<AnnotationUse>,
    pub name: String,
    pub args: Vec<Expr>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Element {
    pub ty: TypeRef,
    pub name: String,
    pub default: Option<AnnValue>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub enum Member {
    Field(FieldDecl),
    Method(MethodDecl),
    Ctor(CtorDecl),
    StaticInit(Block),
}

#[derive(Clone, Debug)]
pub struct FieldDecl {
    pub mods: Mods,
    pub ty: TypeRef,
    pub name: String,
    pub init: Option<Expr>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub mods: Mods,
    pub ty: TypeRef,
    pub varargs: bool,
    pub name: String,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct MethodDecl {
    pub mods: Mods,
    pub type_params: Vec<TypeParam>,
    /// `None` is `void`.
    pub result: Option<TypeRef>,
    pub name: String,
    /// The type written for a receiver parameter `T this`.
    pub receiver: Option<TypeRef>,
    pub params: Vec<Param>,
    pub body: Option<Block>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct CtorDecl {
    pub mods: Mods,
    /// `None` for the compact form of a value class.
    pub params: Option<Vec<Param>>,
    /// `None` when the declaration ends in `;`.
    pub body: Option<Block>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub pos: Pos,
    pub end: Pos,
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Local {
    pub mods: Mods,
    /// `None` is `var`.
    pub ty: Option<TypeRef>,
    pub name: String,
    pub init: Option<Expr>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub enum ArmHead {
    Constants(Vec<Expr>),
    Type(TypeRef, String),
    Default,
}

#[derive(Clone, Debug)]
pub struct Catch {
    pub ty: TypeRef,
    pub name: String,
    pub block: Block,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Block(Block),
    Local(Local),
    Expr(Expr),
    Empty,
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    For { init: Vec<Stmt>, cond: Option<Expr>, update: Vec<Expr>, body: Box<Stmt> },
    ForEach { local: Local, iter: Expr, body: Box<Stmt> },
    Switch { selector: Expr, arms: Vec<(ArmHead, Stmt, Pos)> },
    Try { block: Block, catches: Vec<Catch>, finally: Option<Block> },
    Using { resources: Vec<Local>, block: Block },
    Return(Option<Expr>),
    Break(Option<String>),
    Continue(Option<String>),
    Throw(Expr),
    Assert(Expr, Option<Expr>),
    Labeled(String, Box<Stmt>),
    /// `super(args);` when `true`, `this(args);` otherwise.
    CtorCall(bool, Vec<Expr>),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub pos: Pos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl BinOp {
    /// The method an operator spells, as section 4.6 lists them.
    pub fn method(self) -> &'static str {
        match self {
            BinOp::Add => "plus",
            BinOp::Sub => "minus",
            BinOp::Mul => "times",
            BinOp::Div => "div",
            BinOp::Rem => "mod",
            BinOp::BitAnd => "and",
            BinOp::BitOr => "or",
            BinOp::BitXor => "xor",
            BinOp::Shl => "shiftLeft",
            BinOp::Shr => "shiftRight",
            BinOp::Lt => "lessThan",
            BinOp::Le => "atMost",
            BinOp::Gt => "greaterThan",
            BinOp::Ge => "atLeast",
            BinOp::Eq | BinOp::Ne => "equals",
        }
    }

    pub fn spelling(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    BitNot,
    Not,
}

#[derive(Clone, Debug)]
pub struct LambdaParam {
    pub ty: Option<TypeRef>,
    pub name: String,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(Block),
}

#[derive(Clone, Debug)]
pub enum ArmValue {
    Expr(Expr),
    Throw(Expr),
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Int(String, u32),
    Dec(String),
    Char(char),
    Str(String),
    Bool(bool),
    Null,
    Name(String),
    This,
    Paren(Box<Expr>),
    /// `target.name`
    Field(Box<Expr>, String),
    /// `target.<targs>name(args)`, or `name(args)` with no target.
    Call { target: Option<Box<Expr>>, type_args: Vec<TypeArg>, name: String, args: Vec<Expr> },
    SuperCall { name: String, args: Vec<Expr> },
    Index(Box<Expr>, Box<Expr>),
    New { ty: TypeRef, args: Vec<Expr> },
    /// `new T[len]` followed by further brackets: `elem` is the element type.
    NewArray { elem: TypeRef, len: Box<Expr> },
    /// `new T[] { ... }`: `ty` is the array type.
    ArrayLit { ty: TypeRef, elems: Vec<Expr> },
    ClassLit(TypeRef),
    /// `target::name`. The target is an expression when it could be one.
    MethodRef { target: RefTarget, name: String },
    Lambda { params: Vec<LambdaParam>, body: LambdaBody },
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    Coalesce(Box<Expr>, Box<Expr>),
    Assign { op: Option<BinOp>, target: Box<Expr>, value: Box<Expr> },
    IncDec { inc: bool, prefix: bool, target: Box<Expr> },
    Cast(TypeRef, Box<Expr>),
    InstanceOf(Box<Expr>, TypeRef),
    Switch { selector: Box<Expr>, arms: Vec<(ArmHead, ArmValue, Pos)> },
}

#[derive(Clone, Debug)]
pub enum RefTarget {
    Expr(Box<Expr>),
    Type(TypeRef),
}
