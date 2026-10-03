use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Unit {
    pub file: PathBuf,
    pub source: String,
    pub package: String,
    pub imports: Vec<Import>,
    pub types: Vec<TypeDecl>,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub span: usize,
    pub name: String,
    pub star: bool,
}

#[derive(Clone, Debug)]
pub struct TypeDecl {
    pub span: usize,
    pub audience: Audience,
    pub is_open: bool,
    pub is_sealed: bool,
    pub name: String,
    pub extends: Option<String>,
    pub permits: Vec<String>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub span: usize,
    pub audience: Audience,
    pub is_static: bool,
    pub nullable: bool,
    pub ty: String,
    pub name: String,
    pub init: Option<Expr>,
    pub only: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Audience {
    Private,
    Package,
    Public,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub span: usize,
    pub audience: Audience,
    pub is_static: bool,
    pub is_open: bool,
    pub result: ResultType,
    pub name: String,
    pub params: Vec<Param>,
    pub only: Vec<String>,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultType {
    Void,
    Named { name: String, nullable: bool },
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub nullable: bool,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Local {
        span: usize,
        ty: String,
        name: String,
        nullable: bool,
        init: Option<Expr>,
    },
    Expr {
        span: usize,
        expr: Expr,
    },
    Assign {
        span: usize,
        name: String,
        owner: Option<String>,
        expr: Expr,
    },
    SetField {
        span: usize,
        object: Expr,
        name: String,
        expr: Expr,
    },
    SetIndex {
        span: usize,
        array: Expr,
        index: Expr,
        expr: Expr,
    },
    If {
        span: usize,
        cond: Expr,
        then_body: Block,
        else_body: Option<Block>,
    },
    While {
        span: usize,
        cond: Expr,
        body: Block,
    },
    Return {
        span: usize,
        expr: Option<Expr>,
    },
    Block(Block),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub span: usize,
    pub kind: ExprKind,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Int(i128),
    Bool(bool),
    Null,
    Name(String),
    Select(Box<Expr>, String),
    Call(Box<Expr>, Vec<Expr>),
    UnaryNeg(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    NewUnit,
    NewClass(String),
    NewArray {
        elem: String,
        len: Box<Expr>,
    },
    Index(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}
