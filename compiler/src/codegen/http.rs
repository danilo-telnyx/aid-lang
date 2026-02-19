//! AID std.http → Axum code generation
//!
//! Maps AID HTTP server constructs (§17) to Axum/Tower Rust code.

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// How a handler returns its response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    Text,
    Json,
    Error,
    Redirect,
    Empty,
}

/// A single route definition.
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub method: String,
    pub path: String,
    pub handler_name: String,
}

/// Information about a handler used for intent-based routing.
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    pub name: String,
    pub params: Vec<(String, String)>, // (name, type)
    pub return_type: String,
}

/// Middleware attachment point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiddlewarePosition {
    Before,
    After,
}

/// Middleware descriptor.
#[derive(Debug, Clone)]
pub struct MiddlewareInfo {
    pub name: String,
    pub position: MiddlewarePosition,
}

// ---------------------------------------------------------------------------
// HttpCodegen
// ---------------------------------------------------------------------------

/// Generates Axum Rust source fragments from AID HTTP constructs.
pub struct HttpCodegen;

impl HttpCodegen {
    pub fn new() -> Self {
        Self
    }

    // -- server setup -------------------------------------------------------

    /// Generate the Axum server boot-up code.
    pub fn generate_server_setup(&self, port: u16, host: &str, tls: bool) -> String {
        let bind = format!("{}:{}", host, port);
        if tls {
            format!(
                r#"    let addr = "{bind}";
    let tls_config = RustlsConfig::from_pem_file("cert.pem", "key.pem")
        .await
        .expect("failed to load TLS config");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    axum_server::bind_rustls(addr.parse().unwrap(), tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();"#,
                bind = bind,
            )
        } else {
            format!(
                r#"    let listener = tokio::net::TcpListener::bind("{bind}")
        .await
        .expect("failed to bind");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();"#,
                bind = bind,
            )
        }
    }

    // -- single route -------------------------------------------------------

    /// Generate an Axum `.route()` call.
    pub fn generate_route(&self, method: &str, path: &str, handler_name: &str) -> String {
        let axum_method = Self::method_fn(method);
        // Convert AID :param to Axum :param (same syntax) and *param likewise
        let axum_path = path.to_string();
        format!(
            r#"        .route("{path}", {method}({handler}))"#,
            path = axum_path,
            method = axum_method,
            handler = handler_name,
        )
    }

    // -- intent routing -----------------------------------------------------

    /// Analyse handler names and produce routes automatically.
    ///
    /// Naming conventions:
    /// - `create_*` → POST  `/<entities>`
    /// - `get_*` / `find_*` → GET  `/<entities>/:id`
    /// - `list_*` → GET  `/<entities>`
    /// - `update_*` → PATCH `/<entities>/:id`
    /// - `delete_*` / `remove_*` → DELETE `/<entities>/:id`
    pub fn generate_intent_route(
        &self,
        base_path: &str,
        handlers: &[HandlerInfo],
    ) -> String {
        let mut routes = Vec::new();
        for h in handlers {
            if let Some((method, path)) = Self::infer_route(&h.name, base_path) {
                routes.push(self.generate_route(&method, &path, &h.name));
            }
        }
        routes.join("\n")
    }

    // -- handler ------------------------------------------------------------

    /// Generate an `async fn` handler compatible with Axum extractors.
    pub fn generate_handler(
        &self,
        name: &str,
        body: &str,
        response_type: ResponseType,
    ) -> String {
        let return_type = Self::axum_return_type(response_type);
        let indented_body = body
            .lines()
            .map(|l| format!("    {}", l))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "async fn {name}(\n    req: AidRequest,\n) -> {ret} {{\n{body}\n}}",
            name = name,
            ret = return_type,
            body = indented_body,
        )
    }

    // -- middleware ----------------------------------------------------------

    /// Wrap AID middleware as a Tower layer.
    pub fn generate_middleware(&self, name: &str, body: &str) -> String {
        let indented_body = body
            .lines()
            .map(|l| format!("        {}", l))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"async fn {name}_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {{
    // --- AID middleware body ---
{body}
    // --- end ---
    next.run(req).await
}}"#,
            name = name,
            body = indented_body,
        )
    }

    // -- route group --------------------------------------------------------

    /// Generate a nested `Router` with an optional prefix.
    pub fn generate_route_group(&self, prefix: &str, routes: &[RouteInfo]) -> String {
        let route_lines: Vec<String> = routes
            .iter()
            .map(|r| self.generate_route(&r.method, &r.path, &r.handler_name))
            .collect();

        format!(
            r#"    Router::new()
{routes}
        .with_state(state.clone())
        // nest under prefix
        ;
    // Attach: app = app.nest("{prefix}", group);"#,
            routes = route_lines.join("\n"),
            prefix = prefix,
        )
    }

    // -- response -----------------------------------------------------------

    /// Generate the Axum expression for an AID Response factory.
    pub fn generate_response(&self, response_type: ResponseType, content: &str) -> String {
        match response_type {
            ResponseType::Json => format!(
                r#"axum::Json(serde_json::json!({content}))"#,
                content = content,
            ),
            ResponseType::Text => format!(
                r#"axum::response::Html({content}.to_string())"#,
                content = content,
            ),
            ResponseType::Error => format!(
                r#"(axum::http::StatusCode::INTERNAL_SERVER_ERROR, {content}.to_string())"#,
                content = content,
            ),
            ResponseType::Redirect => format!(
                r#"axum::response::Redirect::to({content})"#,
                content = content,
            ),
            ResponseType::Empty => {
                "axum::http::StatusCode::NO_CONTENT".to_string()
            }
        }
    }

    // -- entity structs -----------------------------------------------------

    /// Rust struct that mirrors AID's `Request` entity, backed by Axum extractors.
    pub fn generate_request_entity() -> String {
        r#"/// AID Request entity — wraps Axum extractors into a single value.
#[derive(Debug)]
pub struct AidRequest {
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub query: std::collections::HashMap<String, String>,
    pub body: AidBody,
    params: std::collections::HashMap<String, String>,
}

#[derive(Debug)]
pub struct AidBody {
    pub raw: String,
}

impl AidBody {
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.raw)
    }

    pub fn text(&self) -> &str {
        &self.raw
    }

    pub fn bytes(&self) -> &[u8] {
        self.raw.as_bytes()
    }
}

impl AidRequest {
    pub fn param(&self, name: &str) -> String {
        self.params.get(name).cloned().unwrap_or_default()
    }

    pub fn param_int(&self, name: &str) -> Result<i64, std::num::ParseIntError> {
        self.param(name).parse::<i64>()
    }

    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    pub fn has_auth(&self) -> bool {
        self.headers.contains_key("authorization")
    }
}

/// Extract an `AidRequest` from an incoming Axum request.
#[axum::async_trait]
impl<S: Send + Sync> axum::extract::FromRequest<S> for AidRequest {
    type Rejection = axum::response::Response;

    async fn from_request(
        req: axum::extract::Request,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        use axum::body::Body;
        use axum::http::Request;

        let (parts, body) = req.into_parts();

        let method = parts.method.to_string();
        let path = parts.uri.path().to_string();

        let headers: std::collections::HashMap<String, String> = parts
            .headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let query: std::collections::HashMap<String, String> = parts
            .uri
            .query()
            .map(|q| {
                url::form_urlencoded::parse(q.as_bytes())
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let params = std::collections::HashMap::new(); // filled by Axum Path extractor layer

        let raw = String::from_utf8(
            axum::body::to_bytes(body, usize::MAX)
                .await
                .unwrap_or_default()
                .to_vec(),
        )
        .unwrap_or_default();

        Ok(AidRequest {
            method,
            path,
            headers,
            query,
            body: AidBody { raw },
            params,
        })
    }
}"#
        .to_string()
    }

    /// Helpers that map to AID `Response.*` factories.
    pub fn generate_response_entity() -> String {
        r#"/// AID Response helpers — mirrors Response.json / .text / .error / .redirect / .empty
pub struct AidResponse;

impl AidResponse {
    pub fn json<T: serde::Serialize>(data: T) -> axum::response::Response {
        let body = serde_json::to_string(&data).unwrap_or_default();
        axum::response::Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn json_status<T: serde::Serialize>(data: T, status: u16) -> axum::response::Response {
        let body = serde_json::to_string(&data).unwrap_or_default();
        axum::response::Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn text(content: &str) -> axum::response::Response {
        axum::response::Response::builder()
            .status(200)
            .header("content-type", "text/plain; charset=utf-8")
            .body(axum::body::Body::from(content.to_string()))
            .unwrap()
    }

    pub fn error(message: &str, status: u16) -> axum::response::Response {
        let body = serde_json::json!({ "error": message }).to_string();
        axum::response::Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn redirect(url: &str) -> axum::response::Response {
        axum::response::Response::builder()
            .status(302)
            .header("location", url)
            .body(axum::body::Body::empty())
            .unwrap()
    }

    pub fn empty(status: u16) -> axum::response::Response {
        axum::response::Response::builder()
            .status(status)
            .body(axum::body::Body::empty())
            .unwrap()
    }
}"#
        .to_string()
    }

    // -- dependencies -------------------------------------------------------

    /// Cargo dependencies required by the generated code.
    pub fn required_dependencies() -> Vec<(String, String)> {
        vec![
            ("axum".into(), "0.8".into()),
            ("axum-server".into(), "0.7".into()),
            ("tokio".into(), "1"),
            ("serde".into(), "1"),
            ("serde_json".into(), "1"),
            ("tower".into(), "0.5".into()),
            ("tower-http".into(), "0.6".into()),
            ("url".into(), "2"),
        ]
        .into_iter()
        .map(|(k, v): (&str, &str)| (k.to_string(), v.to_string()))
        .collect()
    }

    // -- private helpers ----------------------------------------------------

    /// Map an HTTP method string to the axum routing function.
    fn method_fn(method: &str) -> &'static str {
        match method.to_uppercase().as_str() {
            "GET" => "get",
            "POST" => "post",
            "PUT" => "put",
            "PATCH" => "patch",
            "DELETE" => "delete",
            "HEAD" => "head",
            "OPTIONS" => "options",
            _ => "any",
        }
    }

    /// Map an Axum response type to a Rust return type string.
    fn axum_return_type(rt: ResponseType) -> &'static str {
        match rt {
            ResponseType::Json => "axum::Json<serde_json::Value>",
            ResponseType::Text => "String",
            ResponseType::Error => "(axum::http::StatusCode, String)",
            ResponseType::Redirect => "axum::response::Redirect",
            ResponseType::Empty => "axum::http::StatusCode",
        }
    }

    /// Infer HTTP method + path from a handler function name.
    ///
    /// Returns `(METHOD, path)` or `None` if the name doesn't match a pattern.
    fn infer_route(name: &str, base: &str) -> Option<(String, String)> {
        let base = base.trim_end_matches('/');

        // Try each prefix pattern
        let prefixes: &[(&[&str], &str, bool)] = &[
            (&["create_"], "POST", false),
            (&["list_"], "GET", false),
            (&["get_", "find_"], "GET", true),
            (&["update_"], "PATCH", true),
            (&["delete_", "remove_"], "DELETE", true),
        ];

        for (patterns, method, with_id) in prefixes {
            for prefix in *patterns {
                if let Some(entity) = name.strip_prefix(prefix) {
                    let plural = Self::pluralize(entity);
                    let path = if *with_id {
                        format!("{}/{}/:id", base, plural)
                    } else {
                        format!("{}/{}", base, plural)
                    };
                    return Some((method.to_string(), path));
                }
            }
        }

        None
    }

    /// Naïve English pluralisation (good enough for code-gen).
    fn pluralize(word: &str) -> String {
        if word.ends_with('s') || word.ends_with('x') || word.ends_with("sh") || word.ends_with("ch") {
            format!("{}es", word)
        } else if word.ends_with('y') && !word.ends_with("ey") && !word.ends_with("ay") && !word.ends_with("oy") {
            format!("{}ies", &word[..word.len() - 1])
        } else {
            format!("{}s", word)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_create() {
        let (method, path) = HttpCodegen::infer_route("create_user", "/api").unwrap();
        assert_eq!(method, "POST");
        assert_eq!(path, "/api/users");
    }

    #[test]
    fn test_infer_get() {
        let (method, path) = HttpCodegen::infer_route("get_user", "/api").unwrap();
        assert_eq!(method, "GET");
        assert_eq!(path, "/api/users/:id");
    }

    #[test]
    fn test_infer_list() {
        let (method, path) = HttpCodegen::infer_route("list_user", "/api").unwrap();
        assert_eq!(method, "GET");
        assert_eq!(path, "/api/users");
    }

    #[test]
    fn test_infer_delete() {
        let (method, path) = HttpCodegen::infer_route("remove_post", "").unwrap();
        assert_eq!(method, "DELETE");
        assert_eq!(path, "/posts/:id");
    }

    #[test]
    fn test_generate_route() {
        let cg = HttpCodegen::new();
        let out = cg.generate_route("GET", "/users/:id", "get_user");
        assert!(out.contains("get(get_user)"));
        assert!(out.contains("/users/:id"));
    }

    #[test]
    fn test_required_deps() {
        let deps = HttpCodegen::required_dependencies();
        let names: Vec<_> = deps.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"axum"));
        assert!(names.contains(&"tokio"));
        assert!(names.contains(&"serde"));
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(HttpCodegen::pluralize("user"), "users");
        assert_eq!(HttpCodegen::pluralize("category"), "categories");
        assert_eq!(HttpCodegen::pluralize("bus"), "buses");
    }
}
