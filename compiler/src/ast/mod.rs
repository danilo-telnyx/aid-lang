//! AST type definitions for the AID programming language.

use serde::Serialize;

// ── Span ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Default for Span {
    fn default() -> Self {
        Span { line: 0, column: 0, offset: 0 }
    }
}

// ── Program (root) ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Program {
    pub module: String,
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

// ── Import ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Import {
    /// Dotted module path, e.g. "std.http" or "models.user"
    pub path: Vec<String>,
    pub kind: ImportKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ImportKind {
    /// `use std.http` — import the module itself
    Module,
    /// `use models.user.User` — import a single item (last segment)
    Item(String),
    /// `use utils.{ validate, sanitize }` — import specific items
    Items(Vec<String>),
    /// `use models.user.*` — wildcard import
    Glob,
}

// ── Declaration ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Declaration {
    Entity(EntityDecl),
    Function(Function),
    ReasonBlock(ReasonBlock),
    EvolveBlock(EvolveBlock),
    Contract(Contract),
    Implement(ImplementBlock),
    Const(ConstDecl),
    TypeAlias(TypeAliasDecl),
}

// ── Entity ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EntityDecl {
    pub name: String,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Field {
    pub name: String,
    pub ty: AidType,
    pub default: Option<Expression>,
    pub span: Span,
}

// ── Function ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<AidType>,
    pub body: FunctionBody,
    pub is_async: bool,
    pub is_private: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FunctionBody {
    /// `{ ... }`
    Block(Vec<Statement>),
    /// `=> expr`
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Param {
    pub name: String,
    pub ty: AidType,
    pub default: Option<Expression>,
    pub span: Span,
}

/// Bare function signature (used in contracts).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FnSignature {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: AidType,
    pub span: Span,
}

// ── AidType ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AidType {
    Int,
    Float,
    Bool,
    String,
    Byte,
    Array(Box<AidType>),
    Map(Box<AidType>, Box<AidType>),
    Option(Box<AidType>),
    Result(Box<AidType>, Box<AidType>),
    Stream(Box<AidType>),
    /// A named entity / user-defined type reference.
    Entity(std::string::String),
    /// `fn(A, B) -> C`
    Fn(Vec<AidType>, Box<AidType>),
    /// Tuple type for multiple return values: `(T, U)`
    Tuple(Vec<AidType>),
    /// Placeholder when parser cannot resolve the type yet.
    Inferred,
}

// ── Statement ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Statement {
    /// `let x := expr` or `mut x := expr` or `x: T = expr`
    VarDecl {
        name: std::string::String,
        ty: Option<AidType>,
        value: Expression,
        mutable: bool,
        span: Span,
    },
    /// `lhs = rhs`
    Assignment {
        target: Expression,
        value: Expression,
        span: Span,
    },
    /// Bare expression statement.
    Expression {
        expr: Expression,
        span: Span,
    },
    /// `return expr`
    Return {
        value: Option<Expression>,
        span: Span,
    },
    /// `if cond { ... } else if ... else { ... }`
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_if_branches: Vec<ElseIfBranch>,
        else_body: Option<Vec<Statement>>,
        span: Span,
    },
    /// `match expr { ... }`
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// `for pattern in iterable { ... }`
    For {
        pattern: Pattern,
        iterable: Expression,
        body: Vec<Statement>,
        span: Span,
    },
    /// `while cond { ... }`
    While {
        condition: Expression,
        body: Vec<Statement>,
        span: Span,
    },
    Break { span: Span },
    Continue { span: Span },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElseIfBranch {
    pub condition: Expression,
    pub body: Vec<Statement>,
    pub span: Span,
}

// ── Expression ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expression {
    /// Integer, float, bool, string, None
    Literal {
        value: Literal,
        span: Span,
    },
    /// Variable or name reference.
    Identifier {
        name: std::string::String,
        span: Span,
    },
    /// `lhs op rhs`
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },
    /// `op expr` (e.g. `-x`, `!flag`)
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },
    /// `callee(args)` — supports named arguments.
    Call {
        callee: Box<Expression>,
        args: Vec<Argument>,
        span: Span,
    },
    /// `expr.field` or `expr.method`
    MemberAccess {
        object: Box<Expression>,
        member: std::string::String,
        span: Span,
    },
    /// `expr[index]`
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    /// `fn(params) -> T => expr` or `fn(params) { ... }`
    Lambda {
        params: Vec<Param>,
        return_type: Option<AidType>,
        body: FunctionBody,
        span: Span,
    },
    /// `try expr`
    Try {
        expr: Box<Expression>,
        span: Span,
    },
    /// `if cond { a } else { b }` used as an expression
    IfExpr {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
        span: Span,
    },
    /// `match expr { arms }` used as an expression
    MatchExpr {
        subject: Box<Expression>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// `[a, b, c]`
    ArrayLiteral {
        elements: Vec<Expression>,
        span: Span,
    },
    /// `{ "key": value, ... }`
    MapLiteral {
        entries: Vec<(Expression, Expression)>,
        span: Span,
    },
    /// `Entity { field: value, ... }`
    EntityInit {
        name: std::string::String,
        fields: Vec<FieldInit>,
        span: Span,
    },
    /// `await expr`
    Await {
        expr: Box<Expression>,
        span: Span,
    },
    /// Tuple expression: `(a, b)`
    Tuple {
        elements: Vec<Expression>,
        span: Span,
    },
    /// `Some(expr)`, `Ok(expr)`, `Err(expr)`
    Wrap {
        kind: WrapKind,
        value: Box<Expression>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WrapKind {
    Some,
    Ok,
    Err,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Argument {
    pub name: Option<std::string::String>,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FieldInit {
    pub name: std::string::String,
    pub value: Expression,
    pub span: Span,
}

// ── Literal ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(std::string::String),
    None,
}

// ── Operators ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Range,
    Arrow,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UnaryOperator {
    Neg,
    Not,
}

// ── Pattern (for match / for-in) ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Pattern {
    /// A literal value: `200`, `"admin"`, `true`
    Literal(Literal),
    /// A binding or enum variant: `user`, `UserCreated(user)`
    Identifier {
        name: std::string::String,
        binding: Option<std::string::String>,
    },
    /// `100..599`
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
    },
    /// `_`
    Wildcard,
    /// `401 | 403`
    Multiple(Vec<Pattern>),
    /// Destructuring: `(i, user)` for tuple destructure
    Destructure(Vec<std::string::String>),
}

// ── MatchArm ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchArmBody,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MatchArmBody {
    Expression(Expression),
    Block(Vec<Statement>),
}

// ── ReasonBlock ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReasonBlock {
    pub name: String,
    pub mode: ReasonMode,
    pub params: Vec<Param>,
    pub return_type: AidType,
    pub goal: String,
    pub constraints: Vec<String>,
    pub examples: Vec<ReasonExample>,
    pub context: Vec<Expression>,
    pub fallback: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ReasonMode {
    Static,
    Dynamic,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReasonExample {
    pub input: Expression,
    pub output: Expression,
}

// ── EvolveBlock ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvolveBlock {
    pub target: String,
    pub track: bool,
    pub retrain_every: Option<i64>,
    pub min_accuracy: Option<f64>,
    pub storage: Option<String>,
    pub approve: Option<bool>,
    pub span: Span,
}

// ── Contract ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Contract {
    pub name: String,
    pub rules: Vec<String>,
    pub methods: Vec<FnSignature>,
    pub span: Span,
}

// ── ImplementBlock ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImplementBlock {
    pub contract_name: String,
    pub methods: Vec<Function>,
    pub span: Span,
}

// ── Const & TypeAlias ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConstDecl {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypeAliasDecl {
    pub name: String,
    pub ty: AidType,
    pub span: Span,
}
