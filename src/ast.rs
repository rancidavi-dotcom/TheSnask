use crate::types::Type;
use crate::span::{Span, Position};

// Moved from parser.rs to be a central part of the AST
#[derive(Debug, PartialEq, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn to_span(&self) -> Span {
        let p = Position::from_line_col(self.line, self.column);
        Span::single(p)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    IntDivide,
    Equals,
    StrictEquals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Negative,
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Nil,
}

// Wrapper struct for Expression, including location info
#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub loc: Location,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, loc: Location) -> Self {
        let span = loc.to_span();
        Expr { kind, loc, span }
    }

    pub fn with_span(kind: ExprKind, loc: Location, span: Span) -> Self {
        Expr { kind, loc, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Literal(LiteralValue),
    Variable(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FunctionCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    PropertyAccess {
        target: Box<Expr>,
        property: String,
    },
    IndexAccess {
        target: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDecl {
    pub name: String,
    pub var_type: Option<Type>,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MutDecl {
    pub name: String,
    pub var_type: Option<Type>,
    pub value: Expr,
}

impl MutDecl {
    pub fn to_var_decl(&self) -> VarDecl {
        VarDecl {
            name: self.name.clone(),
            var_type: self.var_type.clone(),
            value: self.value.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub var_type: Option<Type>,
    pub value: Expr,
}

impl ConstDecl {
    pub fn to_var_decl(&self) -> VarDecl {
        VarDecl {
            name: self.name.clone(),
            var_type: self.var_type.clone(),
            value: self.value.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarSet {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub properties: Vec<VarDecl>,
    pub methods: Vec<FuncDecl>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfBlock {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConditionalStmt {
    pub if_block: IfBlock,
    pub elif_blocks: Vec<IfBlock>,
    pub else_block: Option<Vec<Stmt>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LoopStmt {
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        iterator: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct ListDecl {
    pub name: String,
    pub var_type: Option<Type>,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ListPush {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DictDecl {
    pub name: String,
    pub var_type: Option<Type>,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DictSet {
    pub name: String,
    pub key: Expr,
    pub value: Expr,
}

// Wrapper struct for Statement, including location info
#[derive(Debug, PartialEq, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub loc: Location,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, loc: Location) -> Self {
        let span = loc.to_span();
        Stmt { kind, loc, span }
    }

    pub fn with_span(kind: StmtKind, loc: Location, span: Span) -> Self {
        Stmt { kind, loc, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtKind {
    Expression(Expr),
    VarDeclaration(VarDecl),
    MutDeclaration(MutDecl),
    ConstDeclaration(ConstDecl),
    VarAssignment(VarSet),
    Print(Vec<Expr>),
    Input {
        name: String,
        var_type: Type,
    },
    FuncDeclaration(FuncDecl),
    ClassDeclaration(ClassDecl),
    FuncCall(Expr),
    Return(Expr),
    Conditional(ConditionalStmt),
    Loop(LoopStmt),
    ListDeclaration(ListDecl),
    ListPush(ListPush),
    DictDeclaration(DictDecl),
    DictSet(DictSet),
    Import(String),
    FromImport {
        from: Vec<String>,
        is_current_dir: bool,
        module: String,
    },
}

pub type Program = Vec<Stmt>;
