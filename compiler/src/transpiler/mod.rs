// AID Language Transpiler — AST to Rust code generation
//
// TODO: Once the AST module at `src/ast/mod.rs` is finalized, replace the
// local AST type definitions below with `use crate::ast::*;`.

use std::collections::HashSet;
use std::fmt::Write as FmtWrite;

// ─── Local AST type definitions (mirror expected AST module) ─────────────────

/// Top-level program node produced by the parser.
#[derive(Debug, Clone)]
pub struct Program {
    pub module_name: String,
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub items: ImportItems,
}

#[derive(Debug, Clone)]
pub enum ImportItems {
    All,
    Named(Vec<String>),
    Module,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Entity(EntityDecl),
    Function(FunctionDecl),
    Reason(ReasonDecl),
    Evolve(EvolveDecl),
    Contract(ContractDecl),
    Implement(ImplementDecl),
    Const(ConstDecl),
    TypeAlias(TypeAliasDecl),
}

#[derive(Debug, Clone)]
pub struct EntityDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<FunctionDecl>,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub typ: AidType,
    pub default: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: AidType,
    pub body: FunctionBody,
    pub is_async: bool,
    pub is_private: bool,
}

#[derive(Debug, Clone)]
pub enum FunctionBody {
    Block(Vec<Statement>),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub typ: AidType,
    pub default: Option<Expression>,
}

#[derive(Debug, Clone)]
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
    Named(String),
    Function(Vec<AidType>, Box<AidType>),
    Tuple(Vec<AidType>),
    Void,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl {
        name: String,
        mutable: bool,
        typ: Option<AidType>,
        value: Expression,
    },
    Assignment {
        target: Expression,
        value: Expression,
    },
    Expression(Expression),
    Return(Expression),
    If(IfStmt),
    Match(MatchStmt),
    For(ForStmt),
    While(WhileStmt),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expression,
    pub then_block: Vec<Statement>,
    pub else_block: Option<Vec<Statement>>,
    pub else_if: Option<Box<IfStmt>>,
}

#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub subject: Expression,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchArmBody,
}

#[derive(Debug, Clone)]
pub enum MatchArmBody {
    Expression(Expression),
    Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Expression),
    Identifier(String),
    Variant(String, Option<String>),
    Range(Expression, Expression),
    Or(Vec<Pattern>),
    Wildcard,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub variable: String,
    pub index_var: Option<String>,
    pub iterable: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    Identifier(String),
    Array(Vec<Expression>),
    Map(Vec<(Expression, Expression)>),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        args: Vec<CallArg>,
    },
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Lambda {
        params: Vec<Param>,
        return_type: Option<AidType>,
        body: Box<Expression>,
    },
    Try(Box<Expression>),
    Await(Box<Expression>),
    StructInit {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    Some(Box<Expression>),
    None,
    Ok(Box<Expression>),
    Err(Box<Expression>),
    If {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    Match {
        subject: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    /// Server route registration: `server.get("/path") => handler`
    ServerRoute {
        server_name: String,
        method: HttpMethod,
        path: String,
        handler: Box<Expression>,
    },
    /// `Response.text(s)`, `Response.json(obj)`, etc.
    ResponseFactory {
        kind: ResponseKind,
        args: Vec<CallArg>,
    },
}

#[derive(Debug, Clone)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: Expression,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
    Range,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get, Post, Put, Patch, Delete,
}

#[derive(Debug, Clone)]
pub enum ResponseKind {
    Text,
    Json,
    Error,
    Redirect,
    Empty,
}

#[derive(Debug, Clone)]
pub struct ReasonDecl {
    pub name: String,
    pub mode: Option<ReasonMode>,
    pub params: Vec<Param>,
    pub return_type: AidType,
    pub goal: String,
    pub constraints: Vec<String>,
    pub context: Vec<Expression>,
    pub examples: Vec<(Expression, Expression)>,
    pub fallback: Option<Expression>,
}

#[derive(Debug, Clone, Copy)]
pub enum ReasonMode {
    Static,
    Dynamic,
}

#[derive(Debug, Clone)]
pub struct EvolveDecl {
    pub target: String,
    pub fields: Vec<(String, Expression)>,
}

#[derive(Debug, Clone)]
pub struct ContractDecl {
    pub name: String,
    pub rules: Vec<String>,
    pub methods: Vec<FunctionSignature>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: AidType,
}

#[derive(Debug, Clone)]
pub struct ImplementDecl {
    pub contract_name: String,
    pub methods: Vec<FunctionDecl>,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct TypeAliasDecl {
    pub name: String,
    pub typ: AidType,
}

// ─── Transpiler output ──────────────────────────────────────────────────────

/// The result of transpiling an AID program.
pub struct TranspileResult {
    pub main_rs: String,
    pub cargo_toml: String,
    pub docs: String,
}

// ─── CodeWriter — indentation-aware string builder ──────────────────────────

struct CodeWriter {
    buf: String,
    indent: usize,
}

impl CodeWriter {
    fn new() -> Self {
        Self {
            buf: String::with_capacity(8192),
            indent: 0,
        }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str("    ");
        }
    }

    fn line(&mut self, text: &str) {
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push('\n');
    }

    fn blank(&mut self) {
        self.buf.push('\n');
    }

    fn raw(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    fn finish(self) -> String {
        self.buf
    }
}

// ─── Transpiler ─────────────────────────────────────────────────────────────

pub struct Transpiler {
    w: CodeWriter,
    imports: HashSet<String>,
    uses_axum: bool,
    uses_tokio: bool,
    uses_serde: bool,
    uses_serde_json: bool,
    uses_hashmap: bool,
    routes: Vec<RouteInfo>,
    doc_lines: Vec<String>,
}

struct RouteInfo {
    method: HttpMethod,
    path: String,
    handler_name: String,
}

impl Transpiler {
    fn new() -> Self {
        Self {
            w: CodeWriter::new(),
            imports: HashSet::new(),
            uses_axum: false,
            uses_tokio: false,
            uses_serde: false,
            uses_serde_json: false,
            uses_hashmap: false,
            routes: Vec::new(),
            doc_lines: Vec::new(),
        }
    }

    /// Main entry point — transpile an AID `Program` into Rust source, Cargo.toml, and docs.
    pub fn transpile(program: &Program) -> TranspileResult {
        let mut t = Transpiler::new();
        t.doc_lines.push(format!("# Module `{}`\n", program.module_name));

        // First pass: scan declarations to determine required crates / features.
        t.scan_declarations(&program.declarations);

        // Second pass: emit Rust code into CodeWriter.
        t.emit_program(program);

        // Build the final main_rs with imports prepended.
        let cargo_toml = t.generate_cargo_toml(&program.module_name);
        let docs = t.doc_lines.join("\n");

        let mut main_rs = String::with_capacity(t.w.buf.len() + 512);
        t.write_header(&mut main_rs);
        main_rs.push_str(&t.w.finish());

        TranspileResult {
            main_rs,
            cargo_toml,
            docs,
        }
    }

    // ── Scanning pass ───────────────────────────────────────────────────────

    fn scan_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::Entity(_) => {
                    self.uses_serde = true;
                }
                Declaration::Function(f) => {
                    self.scan_function(f);
                }
                Declaration::Reason(_) => {}
                Declaration::Evolve(_) => {}
                Declaration::Contract(_) => {}
                Declaration::Implement(imp) => {
                    for m in &imp.methods {
                        self.scan_function(m);
                    }
                }
                Declaration::Const(_) | Declaration::TypeAlias(_) => {}
            }
        }
    }

    fn scan_function(&mut self, f: &FunctionDecl) {
        if f.is_async {
            self.uses_tokio = true;
        }
        self.scan_type(&f.return_type);
        for p in &f.params {
            self.scan_type(&p.typ);
        }
        match &f.body {
            FunctionBody::Block(stmts) => self.scan_statements(stmts),
            FunctionBody::Expression(e) => self.scan_expression(e),
        }
    }

    fn scan_type(&mut self, typ: &AidType) {
        match typ {
            AidType::Map(k, v) => {
                self.uses_hashmap = true;
                self.scan_type(k);
                self.scan_type(v);
            }
            AidType::Array(inner)
            | AidType::Option(inner)
            | AidType::Stream(inner) => self.scan_type(inner),
            AidType::Result(a, b) => {
                self.scan_type(a);
                self.scan_type(b);
            }
            AidType::Function(params, ret) => {
                for p in params {
                    self.scan_type(p);
                }
                self.scan_type(ret);
            }
            AidType::Tuple(ts) => {
                for t in ts {
                    self.scan_type(t);
                }
            }
            _ => {}
        }
    }

    fn scan_statements(&mut self, stmts: &[Statement]) {
        for s in stmts {
            match s {
                Statement::Expression(e) | Statement::Return(e) => self.scan_expression(e),
                Statement::VarDecl { value, typ, .. } => {
                    self.scan_expression(value);
                    if let Some(t) = typ {
                        self.scan_type(t);
                    }
                }
                Statement::Assignment { value, .. } => self.scan_expression(value),
                Statement::If(if_stmt) => self.scan_if(if_stmt),
                Statement::Match(m) => {
                    self.scan_expression(&m.subject);
                    for arm in &m.arms {
                        match &arm.body {
                            MatchArmBody::Expression(e) => self.scan_expression(e),
                            MatchArmBody::Block(stmts) => self.scan_statements(stmts),
                        }
                    }
                }
                Statement::For(f) => {
                    self.scan_expression(&f.iterable);
                    self.scan_statements(&f.body);
                }
                Statement::While(w) => {
                    self.scan_expression(&w.condition);
                    self.scan_statements(&w.body);
                }
                Statement::Break | Statement::Continue => {}
            }
        }
    }

    fn scan_if(&mut self, if_stmt: &IfStmt) {
        self.scan_expression(&if_stmt.condition);
        self.scan_statements(&if_stmt.then_block);
        if let Some(else_b) = &if_stmt.else_block {
            self.scan_statements(else_b);
        }
        if let Some(else_if) = &if_stmt.else_if {
            self.scan_if(else_if);
        }
    }

    fn scan_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::ServerRoute { .. } => {
                self.uses_axum = true;
                self.uses_tokio = true;
            }
            Expression::ResponseFactory { kind, .. } => {
                self.uses_axum = true;
                match kind {
                    ResponseKind::Json => {
                        self.uses_serde_json = true;
                    }
                    _ => {}
                }
            }
            Expression::Call { function, args, .. } => {
                self.scan_expression(function);
                for a in args {
                    self.scan_expression(&a.value);
                }
            }
            Expression::MemberAccess { object, .. } => self.scan_expression(object),
            Expression::BinaryOp { left, right, .. } => {
                self.scan_expression(left);
                self.scan_expression(right);
            }
            Expression::UnaryOp { operand, .. } => self.scan_expression(operand),
            Expression::Index { object, index } => {
                self.scan_expression(object);
                self.scan_expression(index);
            }
            Expression::Lambda { body, .. } => self.scan_expression(body),
            Expression::Try(inner) | Expression::Await(inner) => self.scan_expression(inner),
            Expression::Array(elems) => {
                for e in elems {
                    self.scan_expression(e);
                }
            }
            Expression::Map(entries) => {
                self.uses_hashmap = true;
                for (k, v) in entries {
                    self.scan_expression(k);
                    self.scan_expression(v);
                }
            }
            Expression::StructInit { fields, .. } => {
                for (_, v) in fields {
                    self.scan_expression(v);
                }
            }
            Expression::Some(inner) | Expression::Ok(inner) | Expression::Err(inner) => {
                self.scan_expression(inner);
            }
            Expression::If { condition, then_expr, else_expr } => {
                self.scan_expression(condition);
                self.scan_expression(then_expr);
                self.scan_expression(else_expr);
            }
            Expression::Match { subject, arms } => {
                self.scan_expression(subject);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Expression(e) => self.scan_expression(e),
                        MatchArmBody::Block(stmts) => self.scan_statements(stmts),
                    }
                }
            }
            _ => {}
        }
    }

    // ── Header / imports ────────────────────────────────────────────────────

    fn write_header(&self, out: &mut String) {
        out.push_str("// Generated by the AID compiler — do not edit.\n\n");

        if self.uses_serde {
            out.push_str("use serde::{Serialize, Deserialize};\n");
        }
        if self.uses_serde_json {
            out.push_str("use serde_json;\n");
        }
        if self.uses_axum {
            out.push_str("use axum::{Router, routing, Json};\n");
            out.push_str("use axum::extract::{Path, Query};\n");
            out.push_str("use axum::response::IntoResponse;\n");
        }
        if self.uses_hashmap {
            out.push_str("use std::collections::HashMap;\n");
        }
        out.push('\n');
    }

    // ── Program emission ────────────────────────────────────────────────────

    fn emit_program(&mut self, program: &Program) {
        for decl in &program.declarations {
            self.emit_declaration(decl);
            self.w.blank();
        }

        // If there are routes, emit the main function with Axum router.
        if !self.routes.is_empty() {
            self.emit_axum_main();
        }
    }

    fn emit_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Entity(e) => self.emit_entity(e),
            Declaration::Function(f) => self.emit_function(f),
            Declaration::Reason(r) => self.emit_reason(r),
            Declaration::Evolve(e) => self.emit_evolve(e),
            Declaration::Contract(c) => self.emit_contract(c),
            Declaration::Implement(i) => self.emit_implement(i),
            Declaration::Const(c) => self.emit_const(c),
            Declaration::TypeAlias(t) => self.emit_type_alias(t),
        }
    }

    // ── Entity → struct ─────────────────────────────────────────────────────

    fn emit_entity(&mut self, entity: &EntityDecl) {
        self.doc_lines.push(format!("## Entity `{}`\n", entity.name));
        self.doc_lines.push("| Field | Type |".to_string());
        self.doc_lines.push("|-------|------|".to_string());

        self.w.line("#[derive(Debug, Clone, Serialize, Deserialize)]");
        self.w.line(&format!("pub struct {} {{", entity.name));
        self.w.indent();
        for field in &entity.fields {
            self.doc_lines.push(format!(
                "| `{}` | `{}` |",
                field.name,
                self.type_to_string(&field.typ)
            ));
            self.w.line(&format!(
                "pub {}: {},",
                field.name,
                self.type_to_rust(&field.typ)
            ));
        }
        self.w.dedent();
        self.w.line("}");

        // Default impl if any field has a default value
        let has_defaults = entity.fields.iter().any(|f| f.default.is_some());
        if has_defaults {
            self.w.blank();
            self.w.line(&format!("impl Default for {} {{", entity.name));
            self.w.indent();
            self.w.line("fn default() -> Self {");
            self.w.indent();
            self.w.line("Self {");
            self.w.indent();
            for field in &entity.fields {
                if let Some(default) = &field.default {
                    self.w.line(&format!(
                        "{}: {},",
                        field.name,
                        self.expr_to_rust(default)
                    ));
                } else {
                    self.w.line(&format!(
                        "{}: Default::default(),",
                        field.name
                    ));
                }
            }
            self.w.dedent();
            self.w.line("}");
            self.w.dedent();
            self.w.line("}");
            self.w.dedent();
            self.w.line("}");
        }

        // Entity methods
        if !entity.methods.is_empty() {
            self.w.blank();
            self.w.line(&format!("impl {} {{", entity.name));
            self.w.indent();
            for method in &entity.methods {
                self.emit_function(method);
                self.w.blank();
            }
            self.w.dedent();
            self.w.line("}");
        }
    }

    // ── Function ────────────────────────────────────────────────────────────

    fn emit_function(&mut self, f: &FunctionDecl) {
        self.doc_lines.push(format!(
            "### `fn {}({}) -> {}`\n",
            f.name,
            f.params.iter().map(|p| format!("{}: {}", p.name, self.type_to_string(&p.typ))).collect::<Vec<_>>().join(", "),
            self.type_to_string(&f.return_type),
        ));

        let visibility = if f.is_private { "" } else { "pub " };
        let async_kw = if f.is_async { "async " } else { "" };

        let params_str = f
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.type_to_rust(&p.typ)))
            .collect::<Vec<_>>()
            .join(", ");

        let ret = self.type_to_rust(&f.return_type);

        match &f.body {
            FunctionBody::Expression(expr) => {
                let expr_str = self.expr_to_rust(expr);
                self.w.line(&format!(
                    "{visibility}{async_kw}fn {}({params_str}) -> {ret} {{",
                    f.name
                ));
                self.w.indent();
                self.w.line(&expr_str);
                self.w.dedent();
                self.w.line("}");
            }
            FunctionBody::Block(stmts) => {
                self.w.line(&format!(
                    "{visibility}{async_kw}fn {}({params_str}) -> {ret} {{",
                    f.name
                ));
                self.w.indent();

                // Check if the function body contains server routes — collect them.
                for stmt in stmts {
                    self.emit_statement(stmt);
                }

                self.w.dedent();
                self.w.line("}");
            }
        }
    }

    // ── Reason → stub function ──────────────────────────────────────────────

    fn emit_reason(&mut self, r: &ReasonDecl) {
        self.doc_lines.push(format!("## Reason Block `{}`\n", r.name));
        self.doc_lines.push(format!("**Goal:** {}\n", r.goal));
        self.doc_lines.push("**Constraints:**".to_string());
        for c in &r.constraints {
            self.doc_lines.push(format!("- {}", c));
        }
        self.doc_lines.push(String::new());

        let params_str = r
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.type_to_rust(&p.typ)))
            .collect::<Vec<_>>()
            .join(", ");

        let ret = self.type_to_rust(&r.return_type);

        self.w.line(&format!(
            "/// Reason block: {}",
            r.name
        ));
        self.w.line(&format!("/// Goal: {}", r.goal));
        for c in &r.constraints {
            self.w.line(&format!("/// Constraint: {}", c));
        }
        self.w.line(&format!(
            "pub fn {name}({params_str}) -> {ret} {{",
            name = r.name,
        ));
        self.w.indent();
        self.w.line("// TODO: Cortex integration — replace this stub with generated decision logic.");
        self.w.line(&format!("// Mode: {:?}", r.mode.unwrap_or(ReasonMode::Static)));

        if !r.examples.is_empty() {
            self.w.line("// Examples provided:");
            for (input, output) in &r.examples {
                self.w.line(&format!(
                    "//   {} => {}",
                    self.expr_to_rust(input),
                    self.expr_to_rust(output)
                ));
            }
        }

        // Emit the fallback value, or a panic.
        if let Some(fallback) = &r.fallback {
            self.w.line(&format!(
                "{}",
                self.expr_to_rust(fallback)
            ));
        } else {
            self.w.line("panic!(\"Reason block '{}' has no fallback and Cortex is not yet integrated\")");
        }

        self.w.dedent();
        self.w.line("}");
    }

    // ── Evolve → comment block (telemetry is a future concern) ──────────────

    fn emit_evolve(&mut self, e: &EvolveDecl) {
        self.w.line(&format!(
            "// evolve {} — telemetry tracking (not yet implemented)",
            e.target
        ));
        for (key, val) in &e.fields {
            self.w.line(&format!(
                "//   {}: {}",
                key,
                self.expr_to_rust(val)
            ));
        }
    }

    // ── Contract → validator stubs ──────────────────────────────────────────

    fn emit_contract(&mut self, c: &ContractDecl) {
        self.doc_lines.push(format!("## Contract `{}`\n", c.name));
        self.doc_lines.push("**Validation Rules:**".to_string());
        for (i, rule) in c.rules.iter().enumerate() {
            self.doc_lines.push(format!("{}. {}", i + 1, rule));
        }
        self.doc_lines.push(String::new());

        self.w.line(&format!("/// Contract: {}", c.name));
        for rule in &c.rules {
            self.w.line(&format!("/// Rule: {}", rule));
        }
        self.w.line(&format!("pub trait {} {{", c.name));
        self.w.indent();
        for method in &c.methods {
            let params_str = method
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, self.type_to_rust(&p.typ)))
                .collect::<Vec<_>>()
                .join(", ");
            let ret = self.type_to_rust(&method.return_type);
            self.w.line(&format!("fn {}({}) -> {};", method.name, params_str, ret));
        }
        self.w.dedent();
        self.w.line("}");

        // Generate a validator stub function for the contract rules.
        self.w.blank();
        let validator_name = format!("validate_{}", to_snake_case(&c.name));
        self.w.line(&format!(
            "/// Auto-generated validation stub for contract `{}`.",
            c.name
        ));
        self.w.line(&format!(
            "pub fn {validator_name}() -> Result<(), String> {{"
        ));
        self.w.indent();
        for rule in &c.rules {
            self.w.line(&format!(
                "// TODO: implement validation for rule: \"{}\"",
                rule
            ));
        }
        self.w.line("Ok(())");
        self.w.dedent();
        self.w.line("}");
    }

    // ── Implement → impl block ──────────────────────────────────────────────

    fn emit_implement(&mut self, imp: &ImplementDecl) {
        // We don't know the concrete struct that implements the contract, so
        // emit the methods as standalone pub functions prefixed with the
        // contract name. A future version can wire this to a real trait impl.
        self.w.line(&format!(
            "// Implementation of contract `{}`",
            imp.contract_name
        ));
        for method in &imp.methods {
            self.emit_function(method);
            self.w.blank();
        }
    }

    // ── Const ───────────────────────────────────────────────────────────────

    fn emit_const(&mut self, c: &ConstDecl) {
        let val = self.expr_to_rust(&c.value);
        // We cannot always infer a Rust const type, so use a static for strings.
        match &c.value {
            Expression::StringLiteral(_) => {
                self.w.line(&format!(
                    "pub const {}: &str = {};",
                    c.name.to_uppercase(),
                    val
                ));
            }
            Expression::IntLiteral(_) => {
                self.w.line(&format!(
                    "pub const {}: i64 = {};",
                    c.name.to_uppercase(),
                    val
                ));
            }
            Expression::FloatLiteral(_) => {
                self.w.line(&format!(
                    "pub const {}: f64 = {};",
                    c.name.to_uppercase(),
                    val
                ));
            }
            Expression::BoolLiteral(_) => {
                self.w.line(&format!(
                    "pub const {}: bool = {};",
                    c.name.to_uppercase(),
                    val
                ));
            }
            _ => {
                // Fallback: lazy_static or just a let
                self.w.line(&format!(
                    "// TODO: const type inference needed",
                ));
                self.w.line(&format!(
                    "pub const {}: () = /* {} */;",
                    c.name.to_uppercase(),
                    val
                ));
            }
        }
    }

    // ── Type alias ──────────────────────────────────────────────────────────

    fn emit_type_alias(&mut self, t: &TypeAliasDecl) {
        self.w.line(&format!(
            "pub type {} = {};",
            t.name,
            self.type_to_rust(&t.typ)
        ));
    }

    // ── Axum main ───────────────────────────────────────────────────────────

    fn emit_axum_main(&mut self) {
        self.w.line("#[tokio::main]");
        self.w.line("async fn main() {");
        self.w.indent();
        self.w.line("let app = Router::new()");
        self.w.indent();
        for route in &self.routes {
            let method = match route.method {
                HttpMethod::Get => "get",
                HttpMethod::Post => "post",
                HttpMethod::Put => "put",
                HttpMethod::Patch => "patch",
                HttpMethod::Delete => "delete",
            };
            self.w.line(&format!(
                ".route(\"{}\", routing::{}({}))",
                route.path, method, route.handler_name,
            ));
        }
        self.w.raw(";\n");
        self.w.dedent();
        self.w.blank();
        self.w.line("let listener = tokio::net::TcpListener::bind(\"0.0.0.0:8080\").await.unwrap();");
        self.w.line("println!(\"Listening on http://0.0.0.0:8080\");");
        self.w.line("axum::serve(listener, app).await.unwrap();");
        self.w.dedent();
        self.w.line("}");
    }

    // ── Statements ──────────────────────────────────────────────────────────

    fn emit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl { name, mutable, typ, value } => {
                let val = self.expr_to_rust(value);
                let let_kw = if *mutable { "let mut" } else { "let" };
                if let Some(t) = typ {
                    let rs_type = self.type_to_rust(t);
                    self.w.line(&format!("{let_kw} {name}: {rs_type} = {val};"));
                } else {
                    self.w.line(&format!("{let_kw} {name} = {val};"));
                }
            }
            Statement::Assignment { target, value } => {
                let t = self.expr_to_rust(target);
                let v = self.expr_to_rust(value);
                self.w.line(&format!("{t} = {v};"));
            }
            Statement::Expression(expr) => {
                // Check for server route registration and collect it.
                if let Expression::ServerRoute { method, path, handler, .. } = expr {
                    let handler_name = self.extract_handler_name(handler);
                    self.routes.push(RouteInfo {
                        method: *method,
                        path: path.clone(),
                        handler_name,
                    });
                    return;
                }
                let e = self.expr_to_rust(expr);
                self.w.line(&format!("{e};"));
            }
            Statement::Return(expr) => {
                let e = self.expr_to_rust(expr);
                self.w.line(&format!("return {e};"));
            }
            Statement::If(if_stmt) => self.emit_if_stmt(if_stmt),
            Statement::Match(m) => {
                let subject = self.expr_to_rust(&m.subject);
                self.w.line(&format!("match {subject} {{"));
                self.w.indent();
                for arm in &m.arms {
                    let pat = self.pattern_to_rust(&arm.pattern);
                    match &arm.body {
                        MatchArmBody::Expression(e) => {
                            let expr_str = self.expr_to_rust(e);
                            self.w.line(&format!("{pat} => {expr_str},"));
                        }
                        MatchArmBody::Block(stmts) => {
                            self.w.line(&format!("{pat} => {{"));
                            self.w.indent();
                            for s in stmts {
                                self.emit_statement(s);
                            }
                            self.w.dedent();
                            self.w.line("}");
                        }
                    }
                }
                self.w.dedent();
                self.w.line("}");
            }
            Statement::For(f) => {
                let iter = self.expr_to_rust(&f.iterable);
                if let Some(idx) = &f.index_var {
                    self.w.line(&format!(
                        "for ({idx}, {}) in ({iter}).iter().enumerate() {{",
                        f.variable
                    ));
                } else {
                    self.w.line(&format!(
                        "for {} in ({iter}).iter() {{",
                        f.variable
                    ));
                }
                self.w.indent();
                for s in &f.body {
                    self.emit_statement(s);
                }
                self.w.dedent();
                self.w.line("}");
            }
            Statement::While(w) => {
                let cond = self.expr_to_rust(&w.condition);
                self.w.line(&format!("while {cond} {{"));
                self.w.indent();
                for s in &w.body {
                    self.emit_statement(s);
                }
                self.w.dedent();
                self.w.line("}");
            }
            Statement::Break => self.w.line("break;"),
            Statement::Continue => self.w.line("continue;"),
        }
    }

    fn emit_if_stmt(&mut self, if_stmt: &IfStmt) {
        let cond = self.expr_to_rust(&if_stmt.condition);
        self.w.line(&format!("if {cond} {{"));
        self.w.indent();
        for s in &if_stmt.then_block {
            self.emit_statement(s);
        }
        self.w.dedent();

        if let Some(else_if) = &if_stmt.else_if {
            self.w.raw("    ".repeat(self.w.indent).as_str());
            self.w.raw("} else ");
            // Re-emit as a nested if (write_indent already done by raw)
            let cond2 = self.expr_to_rust(&else_if.condition);
            self.w.raw(&format!("if {cond2} {{\n"));
            self.w.indent();
            for s in &else_if.then_block {
                self.emit_statement(s);
            }
            self.w.dedent();
            // Recurse for further else-if chains
            if else_if.else_if.is_some() || else_if.else_block.is_some() {
                if let Some(nested) = &else_if.else_if {
                    // This simplified approach doesn't deeply chain; handle the else.
                    self.w.line("} else {");
                    self.w.indent();
                    self.w.line("// TODO: deeper else-if chain");
                    self.w.dedent();
                }
                if let Some(else_b) = &else_if.else_block {
                    self.w.line("} else {");
                    self.w.indent();
                    for s in else_b {
                        self.emit_statement(s);
                    }
                    self.w.dedent();
                }
            }
            self.w.line("}");
        } else if let Some(else_block) = &if_stmt.else_block {
            self.w.line("} else {");
            self.w.indent();
            for s in else_block {
                self.emit_statement(s);
            }
            self.w.dedent();
            self.w.line("}");
        } else {
            self.w.line("}");
        }
    }

    // ── Expression → Rust string ────────────────────────────────────────────

    fn expr_to_rust(&self, expr: &Expression) -> String {
        match expr {
            Expression::IntLiteral(v) => format!("{v}_i64"),
            Expression::FloatLiteral(v) => format!("{v}_f64"),
            Expression::BoolLiteral(v) => format!("{v}"),
            Expression::StringLiteral(s) => format!("\"{s}\".to_string()"),
            Expression::Identifier(name) => name.clone(),
            Expression::Array(elems) => {
                let items: Vec<String> = elems.iter().map(|e| self.expr_to_rust(e)).collect();
                format!("vec![{}]", items.join(", "))
            }
            Expression::Map(entries) => {
                if entries.is_empty() {
                    "HashMap::new()".to_string()
                } else {
                    let items: Vec<String> = entries
                        .iter()
                        .map(|(k, v)| {
                            format!("({}, {})", self.expr_to_rust(k), self.expr_to_rust(v))
                        })
                        .collect();
                    format!("HashMap::from([{}])", items.join(", "))
                }
            }
            Expression::BinaryOp { left, op, right } => {
                let l = self.expr_to_rust(left);
                let r = self.expr_to_rust(right);
                let op_str = match op {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Sub => "-",
                    BinaryOperator::Mul => "*",
                    BinaryOperator::Div => "/",
                    BinaryOperator::Mod => "%",
                    BinaryOperator::Eq => "==",
                    BinaryOperator::Neq => "!=",
                    BinaryOperator::Lt => "<",
                    BinaryOperator::Gt => ">",
                    BinaryOperator::Lte => "<=",
                    BinaryOperator::Gte => ">=",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                    BinaryOperator::Range => "..",
                };
                format!("({l} {op_str} {r})")
            }
            Expression::UnaryOp { op, operand } => {
                let o = self.expr_to_rust(operand);
                match op {
                    UnaryOperator::Neg => format!("(-{o})"),
                    UnaryOperator::Not => format!("(!{o})"),
                }
            }
            Expression::Call { function, args } => {
                let func = self.expr_to_rust(function);
                let args_str: Vec<String> = args.iter().map(|a| self.expr_to_rust(&a.value)).collect();
                format!("{func}({})", args_str.join(", "))
            }
            Expression::MemberAccess { object, member } => {
                let obj = self.expr_to_rust(object);
                format!("{obj}.{member}")
            }
            Expression::Index { object, index } => {
                let obj = self.expr_to_rust(object);
                let idx = self.expr_to_rust(index);
                format!("{obj}[{idx}]")
            }
            Expression::Lambda { params, body, .. } => {
                let params_str: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.type_to_rust(&p.typ)))
                    .collect();
                let body_str = self.expr_to_rust(body);
                format!("|{}| {{ {} }}", params_str.join(", "), body_str)
            }
            Expression::Try(inner) => {
                let e = self.expr_to_rust(inner);
                format!("{e}?")
            }
            Expression::Await(inner) => {
                let e = self.expr_to_rust(inner);
                format!("{e}.await")
            }
            Expression::StructInit { name, fields } => {
                let fields_str: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", self.expr_to_rust(v)))
                    .collect();
                format!("{name} {{ {} }}", fields_str.join(", "))
            }
            Expression::Some(inner) => {
                format!("Some({})", self.expr_to_rust(inner))
            }
            Expression::None => "None".to_string(),
            Expression::Ok(inner) => {
                format!("Ok({})", self.expr_to_rust(inner))
            }
            Expression::Err(inner) => {
                format!("Err({})", self.expr_to_rust(inner))
            }
            Expression::If { condition, then_expr, else_expr } => {
                let c = self.expr_to_rust(condition);
                let t = self.expr_to_rust(then_expr);
                let e = self.expr_to_rust(else_expr);
                format!("if {c} {{ {t} }} else {{ {e} }}")
            }
            Expression::Match { subject, arms } => {
                let subj = self.expr_to_rust(subject);
                let arms_str: Vec<String> = arms
                    .iter()
                    .map(|arm| {
                        let pat = self.pattern_to_rust(&arm.pattern);
                        let body = match &arm.body {
                            MatchArmBody::Expression(e) => self.expr_to_rust(e),
                            MatchArmBody::Block(_) => "{ /* block */ }".to_string(),
                        };
                        format!("{pat} => {body}")
                    })
                    .collect();
                format!("match {subj} {{ {} }}", arms_str.join(", "))
            }
            Expression::ServerRoute { server_name, method, path, handler } => {
                // This is handled at the statement level for route collection.
                // If encountered as an expression, just emit a comment.
                format!(
                    "/* route: {}.{}(\"{}\") => ... */",
                    server_name,
                    match method {
                        HttpMethod::Get => "get",
                        HttpMethod::Post => "post",
                        HttpMethod::Put => "put",
                        HttpMethod::Patch => "patch",
                        HttpMethod::Delete => "delete",
                    },
                    path,
                )
            }
            Expression::ResponseFactory { kind, args } => {
                match kind {
                    ResponseKind::Text => {
                        let arg = args.first().map(|a| self.expr_to_rust(&a.value)).unwrap_or_default();
                        format!("{arg}.into_response()")
                    }
                    ResponseKind::Json => {
                        let arg = args.first().map(|a| self.expr_to_rust(&a.value)).unwrap_or_default();
                        format!("Json(serde_json::json!({arg})).into_response()")
                    }
                    ResponseKind::Error => {
                        let msg = args.iter().find(|a| a.name.as_deref() == Some("message"))
                            .map(|a| self.expr_to_rust(&a.value))
                            .unwrap_or_else(|| "\"error\"".to_string());
                        let status = args.iter().find(|a| a.name.as_deref() == Some("status"))
                            .map(|a| self.expr_to_rust(&a.value))
                            .unwrap_or_else(|| "500_u16".to_string());
                        format!(
                            "(axum::http::StatusCode::from_u16({status}).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), {msg}).into_response()"
                        )
                    }
                    ResponseKind::Redirect => {
                        let url = args.first().map(|a| self.expr_to_rust(&a.value)).unwrap_or_default();
                        format!("axum::response::Redirect::to(&{url}).into_response()")
                    }
                    ResponseKind::Empty => {
                        let status = args.iter().find(|a| a.name.as_deref() == Some("status"))
                            .map(|a| self.expr_to_rust(&a.value))
                            .unwrap_or_else(|| "204_u16".to_string());
                        format!(
                            "axum::http::StatusCode::from_u16({status}).unwrap_or(axum::http::StatusCode::NO_CONTENT).into_response()"
                        )
                    }
                }
            }
        }
    }

    // ── Pattern → Rust ──────────────────────────────────────────────────────

    fn pattern_to_rust(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Literal(e) => self.expr_to_rust(e),
            Pattern::Identifier(name) => name.clone(),
            Pattern::Variant(variant, binding) => {
                if let Some(b) = binding {
                    format!("{variant}({b})")
                } else {
                    variant.clone()
                }
            }
            Pattern::Range(lo, hi) => {
                format!("{}..={}", self.expr_to_rust(lo), self.expr_to_rust(hi))
            }
            Pattern::Or(patterns) => {
                let parts: Vec<String> = patterns.iter().map(|p| self.pattern_to_rust(p)).collect();
                parts.join(" | ")
            }
            Pattern::Wildcard => "_".to_string(),
        }
    }

    // ── AID type → Rust type string ─────────────────────────────────────────

    fn type_to_rust(&self, typ: &AidType) -> String {
        match typ {
            AidType::Int => "i64".to_string(),
            AidType::Float => "f64".to_string(),
            AidType::Bool => "bool".to_string(),
            AidType::String => "String".to_string(),
            AidType::Byte => "u8".to_string(),
            AidType::Array(inner) => format!("Vec<{}>", self.type_to_rust(inner)),
            AidType::Map(k, v) => format!(
                "std::collections::HashMap<{}, {}>",
                self.type_to_rust(k),
                self.type_to_rust(v)
            ),
            AidType::Option(inner) => format!("Option<{}>", self.type_to_rust(inner)),
            AidType::Result(ok, err) => format!(
                "Result<{}, {}>",
                self.type_to_rust(ok),
                self.type_to_rust(err)
            ),
            AidType::Stream(inner) => {
                // Streams map to tokio broadcast or similar; use a boxed stream for now.
                format!(
                    "futures::stream::BoxStream<'static, {}>",
                    self.type_to_rust(inner)
                )
            }
            AidType::Named(name) => name.clone(),
            AidType::Function(params, ret) => {
                let params_str: Vec<String> = params.iter().map(|p| self.type_to_rust(p)).collect();
                format!(
                    "Box<dyn Fn({}) -> {} + Send + Sync>",
                    params_str.join(", "),
                    self.type_to_rust(ret)
                )
            }
            AidType::Tuple(ts) => {
                let parts: Vec<String> = ts.iter().map(|t| self.type_to_rust(t)).collect();
                format!("({})", parts.join(", "))
            }
            AidType::Void => "()".to_string(),
        }
    }

    /// Human-readable AID type name for documentation.
    fn type_to_string(&self, typ: &AidType) -> String {
        match typ {
            AidType::Int => "int".to_string(),
            AidType::Float => "float".to_string(),
            AidType::Bool => "bool".to_string(),
            AidType::String => "string".to_string(),
            AidType::Byte => "byte".to_string(),
            AidType::Array(inner) => format!("array<{}>", self.type_to_string(inner)),
            AidType::Map(k, v) => format!("map<{}, {}>", self.type_to_string(k), self.type_to_string(v)),
            AidType::Option(inner) => format!("option<{}>", self.type_to_string(inner)),
            AidType::Result(ok, err) => format!("result<{}, {}>", self.type_to_string(ok), self.type_to_string(err)),
            AidType::Stream(inner) => format!("stream<{}>", self.type_to_string(inner)),
            AidType::Named(name) => name.clone(),
            AidType::Function(params, ret) => {
                let ps: Vec<String> = params.iter().map(|p| self.type_to_string(p)).collect();
                format!("fn({}) -> {}", ps.join(", "), self.type_to_string(ret))
            }
            AidType::Tuple(ts) => {
                let parts: Vec<String> = ts.iter().map(|t| self.type_to_string(t)).collect();
                format!("({})", parts.join(", "))
            }
            AidType::Void => "void".to_string(),
        }
    }

    // ── Helper: extract handler name from expression ────────────────────────

    fn extract_handler_name(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(name) => name.clone(),
            Expression::Lambda { .. } => {
                // Generate a unique handler name for inline lambdas.
                format!("__inline_handler_{}", self.routes.len())
            }
            _ => format!("__handler_{}", self.routes.len()),
        }
    }

    // ── Cargo.toml generation ───────────────────────────────────────────────

    fn generate_cargo_toml(&self, project_name: &str) -> String {
        let name = to_snake_case(project_name);
        let mut toml = String::with_capacity(512);

        writeln!(toml, "[package]").unwrap();
        writeln!(toml, "name = \"{}\"", name).unwrap();
        writeln!(toml, "version = \"0.1.0\"").unwrap();
        writeln!(toml, "edition = \"2021\"").unwrap();
        writeln!(toml).unwrap();
        writeln!(toml, "[dependencies]").unwrap();

        if self.uses_serde {
            writeln!(toml, "serde = {{ version = \"1\", features = [\"derive\"] }}").unwrap();
        }
        if self.uses_serde_json {
            writeln!(toml, "serde_json = \"1\"").unwrap();
        }
        if self.uses_tokio {
            writeln!(
                toml,
                "tokio = {{ version = \"1\", features = [\"full\"] }}"
            )
            .unwrap();
        }
        if self.uses_axum {
            writeln!(toml, "axum = \"0.7\"").unwrap();
        }

        toml
    }
}

// ─── Utility ────────────────────────────────────────────────────────────────

fn to_snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == '.' {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    out
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_program() -> Program {
        Program {
            module_name: "main".to_string(),
            imports: vec![],
            declarations: vec![
                Declaration::Entity(EntityDecl {
                    name: "User".to_string(),
                    fields: vec![
                        FieldDecl {
                            name: "id".to_string(),
                            typ: AidType::Int,
                            default: None,
                        },
                        FieldDecl {
                            name: "name".to_string(),
                            typ: AidType::String,
                            default: None,
                        },
                        FieldDecl {
                            name: "role".to_string(),
                            typ: AidType::String,
                            default: Some(Expression::StringLiteral("viewer".to_string())),
                        },
                    ],
                    methods: vec![],
                }),
                Declaration::Function(FunctionDecl {
                    name: "greet".to_string(),
                    params: vec![Param {
                        name: "name".to_string(),
                        typ: AidType::String,
                        default: None,
                    }],
                    return_type: AidType::String,
                    body: FunctionBody::Expression(Expression::BinaryOp {
                        left: Box::new(Expression::StringLiteral("Hello, ".to_string())),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::Identifier("name".to_string())),
                    }),
                    is_async: false,
                    is_private: false,
                }),
                Declaration::Reason(ReasonDecl {
                    name: "classify".to_string(),
                    mode: Some(ReasonMode::Static),
                    params: vec![Param {
                        name: "text".to_string(),
                        typ: AidType::String,
                        default: None,
                    }],
                    return_type: AidType::String,
                    goal: "Classify text into categories".to_string(),
                    constraints: vec!["Return one of: A, B, C".to_string()],
                    context: vec![],
                    examples: vec![
                        (
                            Expression::StringLiteral("hello".to_string()),
                            Expression::StringLiteral("A".to_string()),
                        ),
                    ],
                    fallback: Some(Expression::StringLiteral("C".to_string())),
                }),
                Declaration::Contract(ContractDecl {
                    name: "UserAPI".to_string(),
                    rules: vec!["ID must be positive".to_string()],
                    methods: vec![FunctionSignature {
                        name: "get_user".to_string(),
                        params: vec![Param {
                            name: "id".to_string(),
                            typ: AidType::Int,
                            default: None,
                        }],
                        return_type: AidType::Option(Box::new(AidType::Named("User".to_string()))),
                    }],
                }),
            ],
        }
    }

    #[test]
    fn test_transpile_produces_output() {
        let program = make_simple_program();
        let result = Transpiler::transpile(&program);

        assert!(result.main_rs.contains("pub struct User"));
        assert!(result.main_rs.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"));
        assert!(result.main_rs.contains("pub fn greet"));
        assert!(result.main_rs.contains("pub fn classify"));
        assert!(result.main_rs.contains("TODO: Cortex integration"));
        assert!(result.main_rs.contains("pub trait UserAPI"));
        assert!(result.cargo_toml.contains("serde"));
        assert!(!result.docs.is_empty());
    }

    #[test]
    fn test_type_mapping() {
        let t = Transpiler::new();
        assert_eq!(t.type_to_rust(&AidType::Int), "i64");
        assert_eq!(t.type_to_rust(&AidType::Float), "f64");
        assert_eq!(t.type_to_rust(&AidType::Bool), "bool");
        assert_eq!(t.type_to_rust(&AidType::String), "String");
        assert_eq!(t.type_to_rust(&AidType::Byte), "u8");
        assert_eq!(
            t.type_to_rust(&AidType::Array(Box::new(AidType::Int))),
            "Vec<i64>"
        );
        assert_eq!(
            t.type_to_rust(&AidType::Map(Box::new(AidType::String), Box::new(AidType::Int))),
            "std::collections::HashMap<String, i64>"
        );
        assert_eq!(
            t.type_to_rust(&AidType::Option(Box::new(AidType::Named("User".to_string())))),
            "Option<User>"
        );
        assert_eq!(
            t.type_to_rust(&AidType::Result(
                Box::new(AidType::Named("User".to_string())),
                Box::new(AidType::Named("ApiError".to_string()))
            )),
            "Result<User, ApiError>"
        );
    }
}
