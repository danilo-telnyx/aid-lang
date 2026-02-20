//! AID std.env → Rust code generation
//!
//! Maps AID env operations to Rust std::env and dotenvy.

use crate::ast::*;

/// Tracks which env features are used to determine dependencies.
#[derive(Debug, Default)]
pub struct EnvUsage {
    pub uses_env: bool,
    pub uses_dotenv: bool,
    pub uses_env_all: bool,
}

/// Check if a program imports std.env
pub fn has_env_import(program: &Program) -> bool {
    program.imports.iter().any(|imp| {
        imp.path == vec!["std", "env"]
    })
}

/// Scan a program for env usage patterns and return what's needed.
pub fn scan_env_usage(program: &Program) -> EnvUsage {
    let mut usage = EnvUsage::default();
    if !has_env_import(program) {
        return usage;
    }
    usage.uses_env = true;

    for decl in &program.declarations {
        match decl {
            Declaration::Function(f) => scan_function_for_env(&f.body, &mut usage),
            _ => {}
        }
    }
    usage
}

fn scan_function_for_env(body: &FunctionBody, usage: &mut EnvUsage) {
    match body {
        FunctionBody::Block(stmts) => {
            for stmt in stmts {
                scan_stmt_for_env(stmt, usage);
            }
        }
        FunctionBody::Expression(expr) => scan_expr_for_env(expr, usage),
    }
}

fn scan_stmt_for_env(stmt: &Statement, usage: &mut EnvUsage) {
    match stmt {
        Statement::VarDecl { value, .. } => scan_expr_for_env(value, usage),
        Statement::Assignment { value, .. } => scan_expr_for_env(value, usage),
        Statement::Expression { expr, .. } => scan_expr_for_env(expr, usage),
        Statement::Return { value: Some(expr), .. } => scan_expr_for_env(expr, usage),
        Statement::If { condition, then_body, else_if_branches, else_body, .. } => {
            scan_expr_for_env(condition, usage);
            for s in then_body { scan_stmt_for_env(s, usage); }
            for branch in else_if_branches {
                scan_expr_for_env(&branch.condition, usage);
                for s in &branch.body { scan_stmt_for_env(s, usage); }
            }
            if let Some(eb) = else_body {
                for s in eb { scan_stmt_for_env(s, usage); }
            }
        }
        Statement::For { body, iterable, .. } => {
            scan_expr_for_env(iterable, usage);
            for s in body { scan_stmt_for_env(s, usage); }
        }
        Statement::While { condition, body, .. } => {
            scan_expr_for_env(condition, usage);
            for s in body { scan_stmt_for_env(s, usage); }
        }
        _ => {}
    }
}

fn scan_expr_for_env(expr: &Expression, usage: &mut EnvUsage) {
    match expr {
        Expression::Call { callee, args, .. } => {
            if let Some(method) = get_env_method(callee) {
                match method.as_str() {
                    "load_dotenv" => usage.uses_dotenv = true,
                    "all" => usage.uses_env_all = true,
                    _ => {}
                }
            }
            scan_expr_for_env(callee, usage);
            for arg in args { scan_expr_for_env(&arg.value, usage); }
        }
        Expression::BinaryOp { left, right, .. } => {
            scan_expr_for_env(left, usage);
            scan_expr_for_env(right, usage);
        }
        Expression::MemberAccess { object, .. } => scan_expr_for_env(object, usage),
        Expression::UnaryOp { operand, .. } => scan_expr_for_env(operand, usage),
        Expression::Lambda { body, .. } => scan_function_for_env(body, usage),
        _ => {}
    }
}

/// Check if an expression is an env method call, return the method name.
fn get_env_method(callee: &Expression) -> Option<String> {
    if let Expression::MemberAccess { object, member, .. } = callee {
        if let Expression::Identifier { name, .. } = object.as_ref() {
            if name == "env" {
                return Some(member.clone());
            }
        }
    }
    None
}

/// Generate Rust code for an env method call expression.
/// Returns Some(rust_code) if it's an env call, None otherwise.
pub fn generate_env_call(callee: &Expression, args: &[Argument]) -> Option<String> {
    let method = get_env_method(callee)?;
    
    match method.as_str() {
        "get" => {
            let key = extract_string_arg(args, 0)?;
            Some(format!("std::env::var(\"{}\").ok()", key))
        }
        "require" => {
            let key = extract_string_arg(args, 0)?;
            Some(format!(
                "std::env::var(\"{key}\").expect(\"Required env var {key} is not set\")",
                key = key
            ))
        }
        "load_dotenv" => {
            Some("dotenvy::dotenv().ok()".to_string())
        }
        "all" => {
            Some("std::env::vars().collect::<std::collections::HashMap<String, String>>()".to_string())
        }
        _ => None,
    }
}

/// Generate Rust code for an env call used as a statement (semicolon terminated).
pub fn generate_env_statement(callee: &Expression, args: &[Argument]) -> Option<String> {
    let method = get_env_method(callee)?;
    
    match method.as_str() {
        "load_dotenv" => {
            Some("    dotenvy::dotenv().ok();".to_string())
        }
        _ => {
            let code = generate_env_call(callee, args)?;
            Some(format!("    {};", code))
        }
    }
}

/// Extract Cargo.toml dependencies needed for env usage.
pub fn env_dependencies(usage: &EnvUsage) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    if usage.uses_dotenv {
        deps.push(("dotenvy".to_string(), "0.15".to_string()));
    }
    deps
}

fn extract_string_arg(args: &[Argument], index: usize) -> Option<String> {
    args.get(index).and_then(|a| {
        if let Expression::Literal { value: Literal::String(s), .. } = &a.value {
            Some(s.clone())
        } else {
            None
        }
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_env_call(method: &str, args: Vec<&str>) -> (Expression, Vec<Argument>) {
        let callee = Expression::MemberAccess {
            object: Box::new(Expression::Identifier {
                name: "env".to_string(),
                span: Span::default(),
            }),
            member: method.to_string(),
            span: Span::default(),
        };
        let arguments: Vec<Argument> = args.iter().map(|s| Argument {
            name: None,
            value: Expression::Literal {
                value: Literal::String(s.to_string()),
                span: Span::default(),
            },
        }).collect();
        (callee, arguments)
    }

    #[test]
    fn test_env_get() {
        let (callee, args) = make_env_call("get", vec!["API_KEY"]);
        let result = generate_env_call(&callee, &args).unwrap();
        assert!(result.contains("std::env::var(\"API_KEY\").ok()"));
    }

    #[test]
    fn test_env_require() {
        let (callee, args) = make_env_call("require", vec!["DATABASE_URL"]);
        let result = generate_env_call(&callee, &args).unwrap();
        assert!(result.contains("std::env::var(\"DATABASE_URL\").expect("));
    }

    #[test]
    fn test_env_load_dotenv() {
        let (callee, args) = make_env_call("load_dotenv", vec![]);
        let result = generate_env_call(&callee, &args).unwrap();
        assert!(result.contains("dotenvy::dotenv()"));
    }

    #[test]
    fn test_env_all() {
        let (callee, args) = make_env_call("all", vec![]);
        let result = generate_env_call(&callee, &args).unwrap();
        assert!(result.contains("std::env::vars().collect"));
    }

    #[test]
    fn test_env_dependencies() {
        let usage = EnvUsage {
            uses_env: true,
            uses_dotenv: true,
            uses_env_all: false,
        };
        let deps = env_dependencies(&usage);
        assert!(deps.iter().any(|(name, _)| name == "dotenvy"));
    }

    #[test]
    fn test_no_dotenv_no_dep() {
        let usage = EnvUsage {
            uses_env: true,
            uses_dotenv: false,
            uses_env_all: false,
        };
        let deps = env_dependencies(&usage);
        assert!(deps.is_empty());
    }
}
