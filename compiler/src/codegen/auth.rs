//! AID std.auth → Rust code generation
//!
//! Maps AID auth operations to jsonwebtoken, bcrypt, and Axum middleware.

use crate::ast::*;

/// Tracks which auth features are used to determine dependencies.
#[derive(Debug, Default)]
pub struct AuthUsage {
    pub uses_auth: bool,
    pub uses_jwt: bool,
    pub uses_bcrypt: bool,
    pub uses_api_key: bool,
    pub uses_middleware: bool,
}

/// Check if a program imports std.auth
pub fn has_auth_import(program: &Program) -> bool {
    program.imports.iter().any(|imp| imp.path == vec!["std", "auth"])
}

/// Scan a program for auth usage patterns and return what's needed.
pub fn scan_auth_usage(program: &Program) -> AuthUsage {
    let mut usage = AuthUsage::default();
    if !has_auth_import(program) {
        return usage;
    }
    usage.uses_auth = true;

    for decl in &program.declarations {
        if let Declaration::Function(f) = decl {
            scan_function_for_auth(&f.body, &mut usage);
        }
    }
    usage
}

fn scan_function_for_auth(body: &FunctionBody, usage: &mut AuthUsage) {
    match body {
        FunctionBody::Block(stmts) => {
            for stmt in stmts {
                scan_stmt_for_auth(stmt, usage);
            }
        }
        FunctionBody::Expression(expr) => scan_expr_for_auth(expr, usage),
    }
}

fn scan_stmt_for_auth(stmt: &Statement, usage: &mut AuthUsage) {
    match stmt {
        Statement::VarDecl { value, .. } => scan_expr_for_auth(value, usage),
        Statement::Assignment { value, .. } => scan_expr_for_auth(value, usage),
        Statement::Expression { expr, .. } => scan_expr_for_auth(expr, usage),
        Statement::Return { value: Some(expr), .. } => scan_expr_for_auth(expr, usage),
        Statement::If { condition, then_body, else_if_branches, else_body, .. } => {
            scan_expr_for_auth(condition, usage);
            for s in then_body { scan_stmt_for_auth(s, usage); }
            for branch in else_if_branches {
                scan_expr_for_auth(&branch.condition, usage);
                for s in &branch.body { scan_stmt_for_auth(s, usage); }
            }
            if let Some(eb) = else_body {
                for s in eb { scan_stmt_for_auth(s, usage); }
            }
        }
        Statement::For { body, iterable, .. } => {
            scan_expr_for_auth(iterable, usage);
            for s in body { scan_stmt_for_auth(s, usage); }
        }
        Statement::While { condition, body, .. } => {
            scan_expr_for_auth(condition, usage);
            for s in body { scan_stmt_for_auth(s, usage); }
        }
        _ => {}
    }
}

fn scan_expr_for_auth(expr: &Expression, usage: &mut AuthUsage) {
    match expr {
        Expression::Call { callee, args, .. } => {
            if let Some(method) = get_auth_method(callee) {
                match method.as_str() {
                    "jwt_sign" | "jwt_verify" => usage.uses_jwt = true,
                    "hash_password" | "verify_password" => usage.uses_bcrypt = true,
                    "api_key" => usage.uses_api_key = true,
                    "middleware" => usage.uses_middleware = true,
                    _ => {}
                }
            }
            scan_expr_for_auth(callee, usage);
            for arg in args { scan_expr_for_auth(&arg.value, usage); }
        }
        Expression::BinaryOp { left, right, .. } => {
            scan_expr_for_auth(left, usage);
            scan_expr_for_auth(right, usage);
        }
        Expression::MemberAccess { object, .. } => scan_expr_for_auth(object, usage),
        Expression::UnaryOp { operand, .. } => scan_expr_for_auth(operand, usage),
        Expression::Lambda { body, .. } => scan_function_for_auth(body, usage),
        _ => {}
    }
}

/// Check if an expression is an auth method call, return the method name.
fn get_auth_method(callee: &Expression) -> Option<String> {
    if let Expression::MemberAccess { object, member, .. } = callee {
        if let Expression::Identifier { name, .. } = object.as_ref() {
            if name == "auth" {
                return Some(member.clone());
            }
        }
    }
    None
}

/// Generate Rust code for an auth method call expression.
/// Returns Some(rust_code) if it's an auth call, None otherwise.
pub fn generate_auth_call(callee: &Expression, args: &[Argument]) -> Option<String> {
    let method = get_auth_method(callee)?;

    match method.as_str() {
        "jwt_sign" => {
            let (claims, claims_is_literal) = extract_arg_info(args, 0).unwrap_or(("claims_json".to_string(), false));
            let secret = extract_string_arg(args, 1)?;
            let claims_expr = if claims_is_literal {
                format!("\"{}\"", claims.replace('"', "\\\""))
            } else {
                claims.clone()
            };
            Some(format!(
                r#"{{
        let key = jsonwebtoken::EncodingKey::from_secret("{secret}".as_bytes());
        let header = jsonwebtoken::Header::default();
        let claims_val: serde_json::Value = serde_json::from_str({claims_expr}).unwrap_or(serde_json::json!({{}}));
        let mut claims_map = std::collections::HashMap::new();
        if let serde_json::Value::Object(m) = claims_val {{
            for (k, v) in m {{
                claims_map.insert(k, v);
            }}
        }}
        // Add exp claim if not present (1 hour from now)
        if !claims_map.contains_key("exp") {{
            let exp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600;
            claims_map.insert("exp".to_string(), serde_json::json!(exp));
        }}
        jsonwebtoken::encode(&header, &claims_map, &key).expect("JWT encoding failed")
    }}"#,
                secret = secret,
                claims_expr = claims_expr,
            ))
        }
        "jwt_verify" => {
            let (token, token_is_literal) = extract_arg_info(args, 0).unwrap_or(("token".to_string(), false));
            let secret = extract_string_arg(args, 1)?;
            let token_expr = if token_is_literal {
                format!("\"{}\"", token.replace('"', "\\\""))
            } else {
                format!("&{}", token)
            };
            Some(format!(
                r#"{{
        let key = jsonwebtoken::DecodingKey::from_secret("{secret}".as_bytes());
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_exp = true;
        validation.required_spec_claims.clear();
        match jsonwebtoken::decode::<std::collections::HashMap<String, serde_json::Value>>({token_expr}, &key, &validation) {{
            Ok(data) => Ok(serde_json::to_string(&data.claims).unwrap_or_default()),
            Err(e) => Err(format!("JWT verification failed: {{}}", e)),
        }}
    }}"#,
                secret = secret,
                token_expr = token_expr,
            ))
        }
        "hash_password" => {
            let (password, is_literal) = extract_arg_info(args, 0).unwrap_or(("password".to_string(), false));
            let pw_expr = if is_literal {
                format!("\"{}\"", password.replace('"', "\\\""))
            } else {
                format!("&{}", password)
            };
            Some(format!(
                r#"bcrypt::hash({pw_expr}, bcrypt::DEFAULT_COST).expect("bcrypt hash failed")"#,
                pw_expr = pw_expr,
            ))
        }
        "verify_password" => {
            let (password, pw_lit) = extract_arg_info(args, 0).unwrap_or(("password".to_string(), false));
            let (hash, hash_lit) = extract_arg_info(args, 1).unwrap_or(("hash".to_string(), false));
            let pw_expr = if pw_lit { format!("\"{}\"", password) } else { format!("&{}", password) };
            let hash_expr = if hash_lit { format!("\"{}\"", hash) } else { format!("&{}", hash) };
            Some(format!(
                r#"bcrypt::verify({pw_expr}, {hash_expr}).unwrap_or(false)"#,
                pw_expr = pw_expr,
                hash_expr = hash_expr,
            ))
        }
        "api_key" => {
            let header_name = extract_string_arg(args, 0)?;
            Some(format!(
                r#"req.headers().get("{header}").and_then(|v| v.to_str().ok()).map(|s| s.to_string())"#,
                header = header_name,
            ))
        }
        "middleware" => {
            // Generates a middleware layer for JWT auth
            Some("axum::middleware::from_fn(auth_middleware)".to_string())
        }
        _ => None,
    }
}

/// Generate Rust code for an auth call used as a statement (semicolon terminated).
pub fn generate_auth_statement(callee: &Expression, args: &[Argument]) -> Option<String> {
    let code = generate_auth_call(callee, args)?;
    Some(format!("    {};", code))
}

/// Generate the JWT auth middleware function.
pub fn generate_auth_middleware(secret: &str) -> String {
    format!(
        r#"async fn auth_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {{
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let auth_header = req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {{
        Some(header) if header.starts_with("Bearer ") => {{
            let token = &header[7..];
            let key = jsonwebtoken::DecodingKey::from_secret("{secret}".as_bytes());
            let mut validation = jsonwebtoken::Validation::default();
            validation.validate_exp = true;
            validation.required_spec_claims.clear();
            match jsonwebtoken::decode::<std::collections::HashMap<String, serde_json::Value>>(token, &key, &validation) {{
                Ok(_) => next.run(req).await,
                Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
            }}
        }}
        _ => (StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header").into_response(),
    }}
}}"#,
        secret = secret,
    )
}

/// Generate the API key auth middleware function.
pub fn generate_api_key_middleware(header_name: &str, expected_key: &str) -> String {
    format!(
        r#"async fn api_key_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {{
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let api_key = req.headers()
        .get("{header}")
        .and_then(|v| v.to_str().ok());

    match api_key {{
        Some(key) if key == "{expected}" => next.run(req).await,
        _ => (StatusCode::UNAUTHORIZED, "Invalid or missing API key").into_response(),
    }}
}}"#,
        header = header_name,
        expected = expected_key,
    )
}

/// Extract Cargo.toml dependencies needed for auth usage.
pub fn auth_dependencies(usage: &AuthUsage) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    if usage.uses_jwt {
        deps.push(("jsonwebtoken".to_string(), "9".to_string()));
    }
    if usage.uses_bcrypt {
        deps.push(("bcrypt".to_string(), "0.15".to_string()));
    }
    deps
}

fn extract_string_arg(args: &[Argument], index: usize) -> Option<String> {
    args.get(index).and_then(|a| {
        if let Expression::Literal { value: Literal::String(s), .. } = &a.value {
            Some(s.clone())
        } else if let Expression::Identifier { name, .. } = &a.value {
            Some(name.clone())
        } else {
            None
        }
    })
}

/// Extract a string arg and whether it's a literal (true) or identifier (false).
fn extract_arg_info(args: &[Argument], index: usize) -> Option<(String, bool)> {
    args.get(index).and_then(|a| {
        match &a.value {
            Expression::Literal { value: Literal::String(s), .. } => Some((s.clone(), true)),
            Expression::Identifier { name, .. } => Some((name.clone(), false)),
            _ => None,
        }
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_auth_call(method: &str, args: Vec<&str>) -> (Expression, Vec<Argument>) {
        let callee = Expression::MemberAccess {
            object: Box::new(Expression::Identifier {
                name: "auth".to_string(),
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
    fn test_jwt_sign() {
        let (callee, args) = make_auth_call("jwt_sign", vec!["{\"sub\":\"user1\"}", "my_secret"]);
        let result = generate_auth_call(&callee, &args).unwrap();
        assert!(result.contains("EncodingKey::from_secret"));
        assert!(result.contains("my_secret"));
        assert!(result.contains("jsonwebtoken::encode"));
    }

    #[test]
    fn test_jwt_verify() {
        let (callee, args) = make_auth_call("jwt_verify", vec!["some.jwt.token", "my_secret"]);
        let result = generate_auth_call(&callee, &args).unwrap();
        assert!(result.contains("DecodingKey::from_secret"));
        assert!(result.contains("my_secret"));
        assert!(result.contains("jsonwebtoken::decode"));
    }

    #[test]
    fn test_hash_password() {
        let (callee, args) = make_auth_call("hash_password", vec!["mypassword"]);
        let result = generate_auth_call(&callee, &args).unwrap();
        assert!(result.contains("bcrypt::hash"));
    }

    #[test]
    fn test_verify_password() {
        let (callee, args) = make_auth_call("verify_password", vec!["mypassword", "somehash"]);
        let result = generate_auth_call(&callee, &args).unwrap();
        assert!(result.contains("bcrypt::verify"));
    }

    #[test]
    fn test_api_key() {
        let (callee, args) = make_auth_call("api_key", vec!["x-api-key"]);
        let result = generate_auth_call(&callee, &args).unwrap();
        assert!(result.contains("x-api-key"));
        assert!(result.contains("headers()"));
    }

    #[test]
    fn test_middleware() {
        let (callee, args) = make_auth_call("middleware", vec![]);
        let result = generate_auth_call(&callee, &args).unwrap();
        assert!(result.contains("auth_middleware"));
    }

    #[test]
    fn test_auth_dependencies_jwt() {
        let usage = AuthUsage {
            uses_auth: true,
            uses_jwt: true,
            uses_bcrypt: false,
            uses_api_key: false,
            uses_middleware: false,
        };
        let deps = auth_dependencies(&usage);
        assert!(deps.iter().any(|(name, _)| name == "jsonwebtoken"));
        assert!(!deps.iter().any(|(name, _)| name == "bcrypt"));
    }

    #[test]
    fn test_auth_dependencies_bcrypt() {
        let usage = AuthUsage {
            uses_auth: true,
            uses_jwt: false,
            uses_bcrypt: true,
            uses_api_key: false,
            uses_middleware: false,
        };
        let deps = auth_dependencies(&usage);
        assert!(deps.iter().any(|(name, _)| name == "bcrypt"));
        assert!(!deps.iter().any(|(name, _)| name == "jsonwebtoken"));
    }

    #[test]
    fn test_auth_dependencies_all() {
        let usage = AuthUsage {
            uses_auth: true,
            uses_jwt: true,
            uses_bcrypt: true,
            uses_api_key: true,
            uses_middleware: true,
        };
        let deps = auth_dependencies(&usage);
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_no_auth_no_deps() {
        let usage = AuthUsage::default();
        let deps = auth_dependencies(&usage);
        assert!(deps.is_empty());
    }

    #[test]
    fn test_has_auth_import() {
        let program = Program {
            module: "test".to_string(),
            imports: vec![Import {
                path: vec!["std".to_string(), "auth".to_string()],
                kind: ImportKind::Module,
                span: Span::default(),
            }],
            declarations: vec![],
            span: Span::default(),
        };
        assert!(has_auth_import(&program));
    }

    #[test]
    fn test_no_auth_import() {
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
        assert!(!has_auth_import(&program));
    }

    #[test]
    fn test_generate_auth_middleware() {
        let code = generate_auth_middleware("test_secret");
        assert!(code.contains("auth_middleware"));
        assert!(code.contains("Bearer"));
        assert!(code.contains("test_secret"));
        assert!(code.contains("UNAUTHORIZED"));
    }

    #[test]
    fn test_generate_api_key_middleware() {
        let code = generate_api_key_middleware("x-api-key", "secret123");
        assert!(code.contains("api_key_middleware"));
        assert!(code.contains("x-api-key"));
        assert!(code.contains("secret123"));
    }
}
