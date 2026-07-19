use crate::span::{Position, Span};
use crate::types::Type;

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
    Modulo,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Negative,
    Not,
    BitNot,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

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
    IndexAccess {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Deref {
        ptr: Box<Expr>,
        type_hint: Type,
    },
    Inb(Box<Expr>),
    AddrOf(String),
    SizeOf(Box<Expr>),
    AlignOf(Box<Expr>),
    OffsetOf {
        expr: Box<Expr>,
        field: String,
    },
    VolatileLoad {
        ptr: Box<Expr>,
        type_hint: Type,
    },
    VolatileStore {
        ptr: Box<Expr>,
        value: Box<Expr>,
    },
    IntToPtr {
        expr: Box<Expr>,
        type_hint: Type,
    },
    PtrToInt {
        expr: Box<Expr>,
        type_hint: Type,
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
pub struct IndexAssignment {
    pub target: Expr,
    pub index: Expr,
    pub value: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
    pub is_unsafe: bool,
    pub is_interrupt: bool,
    pub is_extern: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructMember {
    pub name: String,
    pub var_type: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDecl {
    pub name: String,
    pub members: Vec<StructMember>,
    pub repr_c: bool,
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
    IndexAssignment(IndexAssignment),
    FuncDeclaration(FuncDecl),
    FuncCall(Expr),
    Return(Expr),
    Conditional(ConditionalStmt),
    Loop(LoopStmt),
    UnsafeBlock(Vec<Stmt>),
    Asm(String),
    WritePtr {
        ptr: Expr,
        value: Expr,
        type_hint: Type,
    },
    Outb {
        port: Expr,
        value: Expr,
    },
    GlobalAsm(String),
    StructDeclaration(StructDecl),
    Fence {
        ordering: String,
    },
    AtomicRmw {
        ptr: Expr,
        op: String,
        value: Expr,
        ordering: String,
    },
    VolatileStore {
        ptr: Expr,
        value: Expr,
        type_hint: Type,
    },
}

pub type Program = Vec<Stmt>;
