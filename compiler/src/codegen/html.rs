//! AID std.html → Rust code generation
//!
//! Maps AID HTML constructs to Rust code for templating, static file serving,
//! HTML responses, and redirects.

use crate::ast::*;

/// Tracks which html features are used to determine dependencies.
#[derive(Debug, Default)]
pub struct HtmlUsage {
    pub uses_html: bool,
    pub uses_template: bool,
    pub uses_serve_static: bool,
    pub uses_render: bool,
    pub uses_redirect: bool,
}

/// Check if a program imports std.html
pub fn has_html_import(program: &Program) -> bool {
    program.imports.iter().any(|imp| {
        imp.path == vec!["std", "html"]
    })
}

/// Scan a program for html usage patterns and return what's needed.
pub fn scan_html_usage(program: &Program) -> HtmlUsage {
    let mut usage = HtmlUsage::default();
    if !has_html_import(program) {
        return usage;
    }
    usage.uses_html = true;

    for decl in &program.declarations {
        match decl {
            Declaration::Function(f) => scan_function_for_html(&f.body, &mut usage),
            _ => {}
        }
    }
    usage
}

fn scan_function_for_html(body: &FunctionBody, usage: &mut HtmlUsage) {
    match body {
        FunctionBody::Block(stmts) => {
            for stmt in stmts {
                scan_stmt_for_html(stmt, usage);
            }
        }
        FunctionBody::Expression(expr) => scan_expr_for_html(expr, usage),
    }
}

fn scan_stmt_for_html(stmt: &Statement, usage: &mut HtmlUsage) {
    match stmt {
        Statement::VarDecl { value, .. } => scan_expr_for_html(value, usage),
        Statement::Assignment { value, .. } => scan_expr_for_html(value, usage),
        Statement::Expression { expr, .. } => scan_expr_for_html(expr, usage),
        Statement::Return { value: Some(expr), .. } => scan_expr_for_html(expr, usage),
        Statement::If { condition, then_body, else_if_branches, else_body, .. } => {
            scan_expr_for_html(condition, usage);
            for s in then_body { scan_stmt_for_html(s, usage); }
            for branch in else_if_branches {
                scan_expr_for_html(&branch.condition, usage);
                for s in &branch.body { scan_stmt_for_html(s, usage); }
            }
            if let Some(eb) = else_body {
                for s in eb { scan_stmt_for_html(s, usage); }
            }
        }
        Statement::For { body, iterable, .. } => {
            scan_expr_for_html(iterable, usage);
            for s in body { scan_stmt_for_html(s, usage); }
        }
        Statement::While { condition, body, .. } => {
            scan_expr_for_html(condition, usage);
            for s in body { scan_stmt_for_html(s, usage); }
        }
        _ => {}
    }
}

fn scan_expr_for_html(expr: &Expression, usage: &mut HtmlUsage) {
    match expr {
        Expression::Call { callee, args, .. } => {
            if let Some(method) = get_html_method(callee) {
                match method.as_str() {
                    "template" => usage.uses_template = true,
                    "serve_static" => usage.uses_serve_static = true,
                    "render" => usage.uses_render = true,
                    "redirect" => usage.uses_redirect = true,
                    _ => {}
                }
            }
            scan_expr_for_html(callee, usage);
            for arg in args { scan_expr_for_html(&arg.value, usage); }
        }
        Expression::BinaryOp { left, right, .. } => {
            scan_expr_for_html(left, usage);
            scan_expr_for_html(right, usage);
        }
        Expression::MemberAccess { object, .. } => scan_expr_for_html(object, usage),
        Expression::UnaryOp { operand, .. } => scan_expr_for_html(operand, usage),
        Expression::Lambda { body, .. } => scan_function_for_html(body, usage),
        _ => {}
    }
}

/// Check if an expression is an html method call, return the method name.
fn get_html_method(callee: &Expression) -> Option<String> {
    if let Expression::MemberAccess { object, member, .. } = callee {
        if let Expression::Identifier { name, .. } = object.as_ref() {
            if name == "html" {
                return Some(member.clone());
            }
        }
    }
    None
}

/// Generate Rust code for an html method call expression.
/// Returns Some(rust_code) if it's an html call, None otherwise.
pub fn generate_html_call(callee: &Expression, args: &[Argument]) -> Option<String> {
    let method = get_html_method(callee)?;

    match method.as_str() {
        "template" => {
            let path = extract_string_arg(args, 0)?;
            let data_arg = args.get(1).map(|a| format_expr_as_rust(&a.value))
                .unwrap_or_else(|| "serde_json::json!({})".to_string());
            Some(format!(
                "aid_html_render_template(\"{path}\", &{data})",
                path = path,
                data = data_arg,
            ))
        }
        "serve_static" => {
            let dir = extract_string_arg(args, 0)?;
            Some(format!(
                "tower_http::services::ServeDir::new(\"{}\")",
                dir
            ))
        }
        "render" => {
            // args[0] is the HTML content expression
            let content = args.get(0).map(|a| format_expr_as_rust(&a.value))
                .unwrap_or_else(|| "\"\"".to_string());
            Some(format!(
                "axum::response::Html({}.to_string())",
                content
            ))
        }
        "redirect" => {
            let url = extract_string_arg(args, 0)?;
            Some(format!(
                "axum::response::Redirect::to(\"{}\")",
                url
            ))
        }
        _ => None,
    }
}

/// Generate Rust code for an html call used as a statement (semicolon terminated).
pub fn generate_html_statement(callee: &Expression, args: &[Argument]) -> Option<String> {
    let code = generate_html_call(callee, args)?;
    Some(format!("    {};", code))
}

/// Generate the template engine helper function included in the output.
pub fn generate_template_engine() -> String {
    r#"/// AID built-in template engine.
/// Supports: {{variable}}, {{#each items}}...{{/each}}, {{#if condition}}...{{/if}}
fn aid_html_render_template(path: &str, data: &serde_json::Value) -> String {
    let template = std::fs::read_to_string(path)
        .unwrap_or_else(|e| format!("<h1>Template Error</h1><p>Could not read {}: {}</p>", path, e));
    aid_html_process_template(&template, data)
}

fn aid_html_process_template(template: &str, data: &serde_json::Value) -> String {
    let mut result = template.to_string();

    // Process {{#each key}}...{{/each}} blocks
    let each_re = regex::Regex::new(r"\{\{#each\s+(\w+)\}\}([\s\S]*?)\{\{/each\}\}").unwrap();
    result = each_re.replace_all(&result, |caps: &regex::Captures| {
        let key = &caps[1];
        let body = &caps[2];
        if let Some(arr) = data.get(key).and_then(|v| v.as_array()) {
            arr.iter().map(|item| {
                aid_html_process_template(body, item)
            }).collect::<Vec<_>>().join("")
        } else {
            String::new()
        }
    }).to_string();

    // Process {{#if key}}...{{/if}} blocks
    let if_re = regex::Regex::new(r"\{\{#if\s+(\w+)\}\}([\s\S]*?)\{\{/if\}\}").unwrap();
    result = if_re.replace_all(&result, |caps: &regex::Captures| {
        let key = &caps[1];
        let body = &caps[2];
        let truthy = match data.get(key) {
            Some(serde_json::Value::Bool(b)) => *b,
            Some(serde_json::Value::Null) => false,
            Some(serde_json::Value::String(s)) => !s.is_empty(),
            Some(serde_json::Value::Array(a)) => !a.is_empty(),
            Some(_) => true,
            None => false,
        };
        if truthy { aid_html_process_template(body, data) } else { String::new() }
    }).to_string();

    // Process {{variable}} substitutions
    let var_re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
    result = var_re.replace_all(&result, |caps: &regex::Captures| {
        let key = &caps[1];
        match data.get(key) {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => format!("{{{{{}}}}}", key),
        }
    }).to_string();

    result
}"#.to_string()
}

/// Generate the static file serving route setup.
/// Returns the route line to nest into the Axum router.
pub fn generate_static_route(dir: &str) -> String {
    format!(
        r#".nest_service("/static", tower_http::services::ServeDir::new("{}"))"#,
        dir
    )
}

/// Extract Cargo.toml dependencies needed for html usage.
pub fn html_dependencies(usage: &HtmlUsage) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    if usage.uses_template {
        deps.push(("regex".to_string(), "1".to_string()));
    }
    if usage.uses_serve_static {
        deps.push(("tower-http".to_string(), r#"{ version = "0.6", features = ["fs"] }"#.to_string()));
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

/// Simple expression-to-Rust formatter for data arguments.
fn format_expr_as_rust(expr: &Expression) -> String {
    match expr {
        Expression::Literal { value: Literal::String(s), .. } => format!("\"{}\"", s),
        Expression::Literal { value: Literal::Int(n), .. } => format!("{}", n),
        Expression::Literal { value: Literal::Float(f), .. } => format!("{}", f),
        Expression::Literal { value: Literal::Bool(b), .. } => format!("{}", b),
        Expression::Identifier { name, .. } => name.clone(),
        Expression::MapLiteral { entries, .. } => {
            let pairs: Vec<String> = entries.iter().map(|(k, v)| {
                let key = format_expr_as_rust(k);
                let val = format_expr_as_rust(v);
                format!("{}: {}", key, val)
            }).collect();
            format!("serde_json::json!({{ {} }})", pairs.join(", "))
        }
        Expression::Call { callee, args, .. } => {
            let func = format_expr_as_rust(callee);
            let arg_strs: Vec<String> = args.iter().map(|a| format_expr_as_rust(&a.value)).collect();
            format!("{}({})", func, arg_strs.join(", "))
        }
        Expression::MemberAccess { object, member, .. } => {
            format!("{}.{}", format_expr_as_rust(object), member)
        }
        _ => "serde_json::json!({})".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_html_call(method: &str, args: Vec<&str>) -> (Expression, Vec<Argument>) {
        let callee = Expression::MemberAccess {
            object: Box::new(Expression::Identifier {
                name: "html".to_string(),
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
    fn test_html_template() {
        let (callee, args) = make_html_call("template", vec!["templates/index.html"]);
        let result = generate_html_call(&callee, &args).unwrap();
        assert!(result.contains("aid_html_render_template"));
        assert!(result.contains("templates/index.html"));
    }

    #[test]
    fn test_html_serve_static() {
        let (callee, args) = make_html_call("serve_static", vec!["public/"]);
        let result = generate_html_call(&callee, &args).unwrap();
        assert!(result.contains("ServeDir"));
        assert!(result.contains("public/"));
    }

    #[test]
    fn test_html_render() {
        let (callee, args) = make_html_call("render", vec!["<h1>Hello</h1>"]);
        let result = generate_html_call(&callee, &args).unwrap();
        assert!(result.contains("axum::response::Html"));
    }

    #[test]
    fn test_html_redirect() {
        let (callee, args) = make_html_call("redirect", vec!["https://example.com"]);
        let result = generate_html_call(&callee, &args).unwrap();
        assert!(result.contains("Redirect::to"));
        assert!(result.contains("https://example.com"));
    }

    #[test]
    fn test_html_dependencies_template() {
        let usage = HtmlUsage {
            uses_html: true,
            uses_template: true,
            uses_serve_static: false,
            uses_render: false,
            uses_redirect: false,
        };
        let deps = html_dependencies(&usage);
        assert!(deps.iter().any(|(name, _)| name == "regex"));
    }

    #[test]
    fn test_html_dependencies_static() {
        let usage = HtmlUsage {
            uses_html: true,
            uses_template: false,
            uses_serve_static: true,
            uses_render: false,
            uses_redirect: false,
        };
        let deps = html_dependencies(&usage);
        assert!(deps.iter().any(|(name, _)| name == "tower-http"));
    }

    #[test]
    fn test_html_no_deps_when_unused() {
        let usage = HtmlUsage {
            uses_html: true,
            uses_template: false,
            uses_serve_static: false,
            uses_render: true,
            uses_redirect: true,
        };
        let deps = html_dependencies(&usage);
        assert!(deps.is_empty());
    }

    #[test]
    fn test_has_html_import() {
        let program = Program {
            module: "test".to_string(),
            imports: vec![Import {
                path: vec!["std".to_string(), "html".to_string()],
                kind: ImportKind::Module,
                span: Span::default(),
            }],
            declarations: vec![],
            span: Span::default(),
        };
        assert!(has_html_import(&program));
    }

    #[test]
    fn test_no_html_import() {
        let program = Program {
            module: "test".to_string(),
            imports: vec![Import {
                path: vec!["std".to_string(), "http".to_string()],
                kind: ImportKind::Module,
                span: Span::default(),
            }],
            declarations: vec![],
            span: Span::default(),
        };
        assert!(!has_html_import(&program));
    }

    #[test]
    fn test_template_engine_output() {
        let engine = generate_template_engine();
        assert!(engine.contains("aid_html_render_template"));
        assert!(engine.contains("aid_html_process_template"));
        assert!(engine.contains("#each"));
        assert!(engine.contains("#if"));
    }

    #[test]
    fn test_static_route() {
        let route = generate_static_route("public/");
        assert!(route.contains("nest_service"));
        assert!(route.contains("ServeDir"));
        assert!(route.contains("public/"));
    }
}
