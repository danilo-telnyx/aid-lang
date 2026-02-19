use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

// ─── Configuration ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    #[serde(default)]
    pub cortex: CortexConfig,
    #[serde(default)]
    pub build: BuildConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct CortexConfig {
    #[serde(default = "default_cortex_mode")]
    pub mode: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default = "default_target")]
    pub target: String,
}

fn default_version() -> String { "0.1.0".into() }
fn default_entry() -> String { "src/main.aid".into() }
fn default_cortex_mode() -> String { "local".into() }
fn default_confidence() -> f64 { 0.85 }
fn default_target() -> String { "wasm32-wasi".into() }

impl Config {
    pub fn load() -> Option<Config> {
        let content = fs::read_to_string("aid.toml").ok()?;
        toml::from_str(&content).ok()
    }

    pub fn load_or_default() -> Config {
        Self::load().unwrap_or(Config {
            project: ProjectConfig {
                name: "unnamed".into(),
                version: default_version(),
                entry: default_entry(),
            },
            cortex: CortexConfig::default(),
            build: BuildConfig::default(),
        })
    }
}

// ─── CLI Definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "aid", version = "0.1.0", about = "AID — Auto-Intelligent Development Language Compiler")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new AID project
    New {
        /// Project name
        name: String,
    },

    /// Compile the project
    Build {
        /// Source file (defaults to entry in aid.toml or src/main.aid)
        file: Option<PathBuf>,

        /// Optimized release build
        #[arg(long)]
        release: bool,

        /// Skip auto-documentation generation
        #[arg(long)]
        no_docs: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Build and execute the project
    Run {
        /// Source file (defaults to entry in aid.toml or src/main.aid)
        file: Option<PathBuf>,

        /// Port for the HTTP server
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Watch for file changes and rebuild
        #[arg(short, long)]
        watch: bool,
    },

    /// Run tests
    Test {
        /// Test reason blocks specifically
        #[arg(long)]
        reason: bool,
    },

    /// Remove build artifacts
    Clean,

    /// Generate documentation
    Docs {
        /// Serve docs locally after generation
        #[arg(long)]
        serve: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "html")]
        format: DocsFormat,
    },

    /// Format source files
    Fmt {
        /// Files to format (defaults to all .aid files)
        files: Vec<PathBuf>,
    },

    /// Run the linter
    Lint {
        /// Files to lint (defaults to all .aid files)
        files: Vec<PathBuf>,
    },

    /// Cortex engine commands
    Cortex {
        #[command(subcommand)]
        command: CortexCommands,
    },

    /// Rollback an evolved reason block
    Rollback {
        /// Reason block name
        name: String,

        /// Revert to a specific version
        #[arg(long)]
        to: Option<u32>,
    },

    /// Evolution tracking commands
    Evolve {
        #[command(subcommand)]
        command: EvolveCommands,
    },
}

#[derive(Subcommand)]
pub enum CortexCommands {
    /// Show Cortex engine status and model info
    Status,
    /// Test a specific reason block interactively
    Test {
        /// Reason block name
        block: String,
    },
}

#[derive(Subcommand)]
pub enum EvolveCommands {
    /// Show evolution status for all tracked blocks
    Status,
    /// Show evolution history for a specific block
    History {
        /// Reason block name
        block: String,
    },
}

#[derive(Clone, ValueEnum)]
pub enum DocsFormat {
    Markdown,
    Html,
}

// ─── Banner ──────────────────────────────────────────────────────────────────

fn print_banner() {
    println!();
    println!("  {}", "AID Compiler v0.1.0".bold().cyan());
    println!("  {}", "────────────────────".dimmed());
    println!();
}

// ─── HTTP Server Extraction & Code Generation ───────────────────────────────

use crate::ast::*;

struct HttpRoute {
    method: String,
    path: String,
    handler_name: String,
    handler_code: String,
}

struct HttpServerInfo {
    port: u16,
    routes: Vec<HttpRoute>,
}

fn extract_http_server(program: &Program) -> Option<HttpServerInfo> {
    let main_fn = program.declarations.iter().find_map(|d| {
        if let Declaration::Function(f) = d {
            if f.name == "main" { Some(f) } else { None }
        } else {
            None
        }
    })?;

    let stmts = match &main_fn.body {
        FunctionBody::Block(stmts) => stmts,
        _ => return None,
    };

    let mut port = 8080u16;
    let mut server_var = String::new();
    let mut routes = Vec::new();

    for stmt in stmts {
        match stmt {
            Statement::VarDecl { name, value, .. } => {
                if let Some(p) = extract_port_from_http_new(value) {
                    server_var = name.clone();
                    port = p;
                }
            }
            Statement::Expression { expr, .. } => {
                if let Some(route) = extract_route(expr, &server_var) {
                    routes.push(route);
                }
            }
            _ => {}
        }
    }

    if routes.is_empty() {
        return None;
    }
    Some(HttpServerInfo { port, routes })
}

fn extract_port_from_http_new(expr: &Expression) -> Option<u16> {
    if let Expression::Call { callee, args, .. } = expr {
        if let Expression::MemberAccess { object, member, .. } = callee.as_ref() {
            if let Expression::Identifier { name, .. } = object.as_ref() {
                if name == "http" && member == "new" {
                    for arg in args {
                        if arg.name.as_deref() == Some("port") {
                            if let Expression::Literal { value: Literal::Int(p), .. } = &arg.value {
                                return Some(*p as u16);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_route(expr: &Expression, server_var: &str) -> Option<HttpRoute> {
    if let Expression::BinaryOp {
        left,
        op: BinaryOperator::Arrow,
        right,
        ..
    } = expr
    {
        if let Expression::Call { callee, args, .. } = left.as_ref() {
            if let Expression::MemberAccess { object, member, .. } = callee.as_ref() {
                if let Expression::Identifier { name, .. } = object.as_ref() {
                    if name == server_var {
                        let method = member.clone();
                        let path = args.first().and_then(|a| {
                            if let Expression::Literal {
                                value: Literal::String(s),
                                ..
                            } = &a.value
                            {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })?;

                        let handler_code = extract_handler_body(right)?;
                        let handler_name = {
                            let slug = path
                                .trim_matches('/')
                                .replace('/', "_")
                                .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
                            if slug.is_empty() {
                                "handle_root".to_string()
                            } else {
                                format!("handle_{}", slug)
                            }
                        };

                        return Some(HttpRoute {
                            method,
                            path,
                            handler_name,
                            handler_code,
                        });
                    }
                }
            }
        }
    }
    None
}

fn extract_handler_body(expr: &Expression) -> Option<String> {
    if let Expression::Lambda { body, .. } = expr {
        match body {
            FunctionBody::Block(stmts) => {
                let last = stmts.last()?;
                if let Statement::Expression { expr, .. } = last {
                    return Some(expr_to_handler_rust(expr));
                }
            }
            FunctionBody::Expression(expr) => {
                return Some(expr_to_handler_rust(expr));
            }
        }
    }
    None
}

fn expr_to_handler_rust(expr: &Expression) -> String {
    if let Expression::Call { callee, args, .. } = expr {
        if let Expression::MemberAccess { object, member, .. } = callee.as_ref() {
            if let Expression::Identifier { name, .. } = object.as_ref() {
                if name == "Response" {
                    match member.as_str() {
                        "text" => {
                            if let Some(arg) = args.first() {
                                if let Expression::Literal {
                                    value: Literal::String(s),
                                    ..
                                } = &arg.value
                                {
                                    return format!("    \"{}\".into_response()", s);
                                }
                            }
                        }
                        "json" => {
                            if let Some(arg) = args.first() {
                                let json_str = expr_to_json(&arg.value);
                                return format!(
                                    "    Json(serde_json::json!({})).into_response()",
                                    json_str
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    "    todo!()".to_string()
}

fn expr_to_json(expr: &Expression) -> String {
    match expr {
        Expression::MapLiteral { entries, .. } => {
            let fields: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    let key = match k {
                        Expression::Identifier { name, .. } => format!("\"{}\"", name),
                        Expression::Literal {
                            value: Literal::String(s),
                            ..
                        } => format!("\"{}\"", s),
                        _ => "\"?\"".to_string(),
                    };
                    let val = expr_to_json(v);
                    format!("{}: {}", key, val)
                })
                .collect();
            format!("{{{}}}", fields.join(", "))
        }
        Expression::Literal {
            value: Literal::String(s),
            ..
        } => format!("\"{}\"", s),
        Expression::Literal {
            value: Literal::Int(n),
            ..
        } => format!("{}", n),
        Expression::Literal {
            value: Literal::Float(f),
            ..
        } => format!("{}", f),
        Expression::Literal {
            value: Literal::Bool(b),
            ..
        } => format!("{}", b),
        _ => "null".to_string(),
    }
}

// ─── Reason Block Code Generation ───────────────────────────────────────────

struct ReasonBlockInfo {
    name: String,
    params: Vec<(String, String)>, // (name, type)
    return_type: String,
    goal: String,
    constraints: Vec<String>,
    examples: Vec<(String, String)>, // (input_text, output_text)
    fallback: Option<String>,
}

fn extract_reason_blocks(program: &Program) -> Vec<ReasonBlockInfo> {
    let mut blocks = Vec::new();
    for decl in &program.declarations {
        if let Declaration::ReasonBlock(rb) = decl {
            let params: Vec<(String, String)> = rb.params.iter().map(|p| {
                let ty = match &p.ty {
                    AidType::String => "string".to_string(),
                    AidType::Int => "int".to_string(),
                    AidType::Float => "float".to_string(),
                    AidType::Bool => "bool".to_string(),
                    _ => "string".to_string(),
                };
                (p.name.clone(), ty)
            }).collect();

            let return_type = match &rb.return_type {
                AidType::String => "String".to_string(),
                AidType::Int => "i64".to_string(),
                AidType::Float => "f64".to_string(),
                AidType::Bool => "bool".to_string(),
                _ => "String".to_string(),
            };

            let examples: Vec<(String, String)> = rb.examples.iter().filter_map(|ex| {
                let input = extract_string_literal(&ex.input)?;
                let output = extract_string_literal(&ex.output)?;
                Some((input, output))
            }).collect();

            let fallback = rb.fallback.as_ref().and_then(|f| extract_string_literal(f));

            blocks.push(ReasonBlockInfo {
                name: rb.name.clone(),
                params,
                return_type,
                goal: rb.goal.clone(),
                constraints: rb.constraints.clone(),
                examples,
                fallback,
            });
        }
    }
    blocks
}

fn extract_string_literal(expr: &Expression) -> Option<String> {
    if let Expression::Literal { value: Literal::String(s), .. } = expr {
        Some(s.clone())
    } else {
        None
    }
}

// ─── Evolve Block Extraction ─────────────────────────────────────────────────

struct EvolveBlockInfo {
    target: String,
    track: bool,
    retrain_every: Option<i64>,
    min_accuracy: Option<f64>,
    approve: Option<bool>,
}

fn extract_evolve_blocks(program: &Program) -> Vec<EvolveBlockInfo> {
    let mut blocks = Vec::new();
    for decl in &program.declarations {
        if let Declaration::EvolveBlock(eb) = decl {
            blocks.push(EvolveBlockInfo {
                target: eb.target.clone(),
                track: eb.track,
                retrain_every: eb.retrain_every,
                min_accuracy: eb.min_accuracy,
                approve: eb.approve,
            });
        }
    }
    blocks
}

/// Read telemetry JSONL file and return (call_count, distribution map)
fn read_telemetry_stats(name: &str) -> Option<(usize, Vec<(String, usize)>)> {
    let path = format!(".cortex/telemetry/{}.jsonl", name);
    let content = fs::read_to_string(&path).ok()?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return None;
    }
    let mut dist: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in &lines {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(output) = v.get("output").and_then(|o| o.as_str()) {
                *dist.entry(output.to_string()).or_insert(0) += 1;
            }
        }
    }
    let total = lines.len();
    let mut sorted: Vec<(String, usize)> = dist.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    Some((total, sorted))
}

use serde_json;

/// Extract keywords from a text string (words >= 3 chars, lowercased, no stop words)
fn extract_keywords(text: &str) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "can", "shall", "for", "and", "nor", "but",
        "or", "yet", "so", "in", "on", "at", "to", "of", "by", "with", "from",
        "my", "your", "his", "her", "its", "our", "their", "this", "that",
        "what", "how", "not", "you", "it", "i", "we", "they", "me",
    ];
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !stop_words.contains(w))
        .map(|w| w.to_string())
        .collect()
}

/// Parse constraint rules like "Tickets mentioning X or Y are always Z"
fn parse_constraint_rules(constraints: &[String]) -> Vec<(Vec<String>, String)> {
    let mut rules = Vec::new();
    for constraint in constraints {
        let lower = constraint.to_lowercase();
        // Pattern: "mentioning X or Y are always Z" or "mentions of X or Y are always Z"
        if let Some(always_idx) = lower.find("always ") {
            let result_word = lower[always_idx + 7..].trim().to_string();
            // Extract trigger words between "mentioning/mentions of" and "are always"
            let trigger_start = if let Some(idx) = lower.find("mentioning ") {
                Some(idx + 11)
            } else if let Some(idx) = lower.find("mentions of ") {
                Some(idx + 12)
            } else {
                None
            };
            if let Some(start) = trigger_start {
                if let Some(are_idx) = lower.find(" are always") {
                    let trigger_text = &lower[start..are_idx];
                    let keywords: Vec<String> = trigger_text
                        .split(" or ")
                        .flat_map(|s| s.split(" and "))
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !keywords.is_empty() {
                        rules.push((keywords, result_word));
                    }
                }
            }
        }
    }
    rules
}

fn generate_reason_function(block: &ReasonBlockInfo) -> String {
    let mut code = String::new();

    // Doc comment
    code.push_str(&format!("/// Reason block: {}\n", block.name));
    code.push_str(&format!("/// Goal: {}\n", block.goal));
    for c in &block.constraints {
        code.push_str(&format!("/// Constraint: {}\n", c));
    }
    if !block.examples.is_empty() {
        code.push_str("/// Examples:\n");
        for (input, output) in &block.examples {
            code.push_str(&format!("///   \"{}\" → \"{}\"\n", input, output));
        }
    }

    // Function signature
    let params_str: Vec<String> = block.params.iter().map(|(name, ty)| {
        match ty.as_str() {
            "string" => format!("{}: &str", name),
            "int" => format!("{}: i64", name),
            "float" => format!("{}: f64", name),
            "bool" => format!("{}: bool", name),
            _ => format!("{}: &str", name),
        }
    }).collect();
    code.push_str(&format!("fn {}({}) -> {} {{\n", block.name, params_str.join(", "), block.return_type));

    // Assume first param is the text input
    let input_param = block.params.first().map(|(n, _)| n.as_str()).unwrap_or("text");
    code.push_str(&format!("    let text_lower = {}.to_lowercase();\n\n", input_param));

    // 1. Constraint-based rules first (highest priority)
    let constraint_rules = parse_constraint_rules(&block.constraints);
    for (keywords, result) in &constraint_rules {
        let conditions: Vec<String> = keywords.iter()
            .map(|k| format!("text_lower.contains(\"{}\")", k))
            .collect();
        code.push_str(&format!("    // From constraint rule\n"));
        code.push_str(&format!("    if {} {{\n", conditions.join(" || ")));
        code.push_str(&format!("        return \"{}\".to_string();\n", result));
        code.push_str("    }\n\n");
    }

    // 2. Example-based keyword matching
    // Group examples by output category
    let mut categories: Vec<(String, Vec<String>)> = Vec::new();
    for (input, output) in &block.examples {
        let keywords = extract_keywords(input);
        if let Some(existing) = categories.iter_mut().find(|(cat, _)| cat == output) {
            existing.1.extend(keywords);
        } else {
            categories.push((output.clone(), keywords));
        }
    }

    // Also add the category name itself as a keyword
    for (cat, keywords) in &mut categories {
        if !keywords.contains(cat) {
            keywords.push(cat.clone());
        }
        // Deduplicate
        keywords.sort();
        keywords.dedup();
    }

    // Skip categories that are already handled by constraint rules
    let constraint_outputs: Vec<&String> = constraint_rules.iter().map(|(_, r)| r).collect();

    for (category, keywords) in &categories {
        if constraint_outputs.contains(&category) {
            // Still emit for non-constraint keywords
            let constraint_kws: Vec<&String> = constraint_rules.iter()
                .filter(|(_, r)| r == category)
                .flat_map(|(kws, _)| kws)
                .collect();
            let extra_kws: Vec<&String> = keywords.iter()
                .filter(|k| !constraint_kws.contains(k))
                .collect();
            if extra_kws.is_empty() {
                continue;
            }
            let conditions: Vec<String> = extra_kws.iter()
                .map(|k| format!("text_lower.contains(\"{}\")", k))
                .collect();
            code.push_str(&format!("    // From examples — keyword matching for \"{}\"\n", category));
            code.push_str(&format!("    if {} {{\n", conditions.join(" || ")));
            code.push_str(&format!("        return \"{}\".to_string();\n", category));
            code.push_str("    }\n\n");
        } else {
            let conditions: Vec<String> = keywords.iter()
                .map(|k| format!("text_lower.contains(\"{}\")", k))
                .collect();
            code.push_str(&format!("    // From examples — keyword matching for \"{}\"\n", category));
            code.push_str(&format!("    if {} {{\n", conditions.join(" || ")));
            code.push_str(&format!("        return \"{}\".to_string();\n", category));
            code.push_str("    }\n\n");
        }
    }

    // 3. Fallback
    let fallback_val = block.fallback.as_deref().unwrap_or("unknown");
    code.push_str(&format!("    // Fallback\n"));
    code.push_str(&format!("    \"{}\".to_string()\n", fallback_val));
    code.push_str("}\n");

    code
}

/// Try to match a POST route path to a reason block name
fn reason_block_for_route(path: &str, blocks: &[ReasonBlockInfo]) -> Option<String> {
    let slug = path.trim_matches('/').replace('-', "_");
    // Match /classify -> classify_ticket, /priority -> detect_priority
    for block in blocks {
        if block.name.contains(&slug) || slug.contains(&block.name) {
            return Some(block.name.clone());
        }
        // Also match partial: /classify -> classify_ticket
        let name_parts: Vec<&str> = block.name.split('_').collect();
        if name_parts.iter().any(|p| *p == slug) {
            return Some(block.name.clone());
        }
    }
    None
}

fn generate_telemetry_wrapper(block_name: &str, input_param: &str) -> String {
    let mut code = String::new();
    code.push_str(&format!("fn {name}({param}: &str) -> String {{\n", name = block_name, param = input_param));
    code.push_str(&format!("    let result = {name}_logic({param});\n", name = block_name, param = input_param));
    code.push_str("\n    // Telemetry logging\n");
    code.push_str(&format!("    if let Ok(json) = serde_json::to_string(&serde_json::json!({{\n"));
    code.push_str(&format!("        \"function\": \"{}\",\n", block_name));
    code.push_str(&format!("        \"input\": {},\n", input_param));
    code.push_str("        \"output\": &result,\n");
    code.push_str("        \"timestamp\": chrono::Utc::now().to_rfc3339()\n");
    code.push_str("    })) {\n");
    code.push_str("        let _ = std::fs::create_dir_all(\".cortex/telemetry\");\n");
    code.push_str(&format!("        if let Ok(mut f) = std::fs::OpenOptions::new()\n"));
    code.push_str("            .create(true)\n");
    code.push_str("            .append(true)\n");
    code.push_str(&format!("            .open(\".cortex/telemetry/{}.jsonl\")\n", block_name));
    code.push_str("        {\n");
    code.push_str("            use std::io::Write;\n");
    code.push_str("            let _ = writeln!(f, \"{}\", json);\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");
    code.push_str("    result\n");
    code.push_str("}\n");
    code
}

fn generate_telemetry_endpoint(evolve_blocks: &[EvolveBlockInfo]) -> String {
    let mut code = String::new();
    code.push_str("async fn handle_telemetry() -> impl IntoResponse {\n");
    code.push_str("    let mut stats = serde_json::Map::new();\n");
    for eb in evolve_blocks {
        code.push_str(&format!("    // Telemetry for {}\n", eb.target));
        code.push_str(&format!("    if let Ok(content) = std::fs::read_to_string(\".cortex/telemetry/{}.jsonl\") {{\n", eb.target));
        code.push_str("        let mut dist: std::collections::HashMap<String, usize> = std::collections::HashMap::new();\n");
        code.push_str("        let mut count = 0usize;\n");
        code.push_str("        for line in content.lines().filter(|l| !l.trim().is_empty()) {\n");
        code.push_str("            count += 1;\n");
        code.push_str("            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {\n");
        code.push_str("                if let Some(output) = v.get(\"output\").and_then(|o| o.as_str()) {\n");
        code.push_str("                    *dist.entry(output.to_string()).or_insert(0) += 1;\n");
        code.push_str("                }\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str(&format!("        stats.insert(\"{}\".to_string(), serde_json::json!({{\"calls\": count, \"distribution\": dist}}));\n", eb.target));
        code.push_str("    }\n");
    }
    code.push_str("    Json(serde_json::Value::Object(stats)).into_response()\n");
    code.push_str("}\n");
    code
}

fn generate_http_project(project_name: &str, info: &HttpServerInfo, reason_blocks: &[ReasonBlockInfo]) -> (String, String) {
    generate_http_project_with_evolve(project_name, info, reason_blocks, &[], &[], &Program {
        module: String::new(), imports: vec![], declarations: vec![], span: Span::default(),
    })
}

fn generate_http_project_with_evolve(project_name: &str, info: &HttpServerInfo, reason_blocks: &[ReasonBlockInfo], evolve_blocks: &[EvolveBlockInfo], contracts: &[ContractInfo], program: &Program) -> (String, String) {
    let has_evolve = !evolve_blocks.is_empty();
    let evolved_targets: Vec<&str> = evolve_blocks.iter().map(|e| e.target.as_str()).collect();

    let mut main_rs = String::new();
    main_rs.push_str("// Generated by the AID compiler — do not edit.\n\n");
    main_rs.push_str("use axum::{Router, routing, Json, response::IntoResponse};\n");
    let has_contracts = !contracts.is_empty();
    if !reason_blocks.is_empty() || has_evolve || has_contracts {
        main_rs.push_str("use serde_json;\n");
    }
    main_rs.push_str("\n");

    // Emit entity structs needed by contracts
    if has_contracts {
        // Collect entity names used by contracts
        let mut emitted_entities = std::collections::HashSet::new();
        for contract in contracts {
            if let Some(entity_name) = &contract.entity_name {
                if emitted_entities.insert(entity_name.clone()) {
                    // Find entity in program and emit its struct
                    for decl in &program.declarations {
                        if let Declaration::Entity(e) = decl {
                            if &e.name == entity_name {
                                main_rs.push_str("#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\n");
                                main_rs.push_str(&format!("struct {} {{\n", e.name));
                                for field in &e.fields {
                                    let rust_type = match &field.ty {
                                        AidType::Int => "i64",
                                        AidType::Float => "f64",
                                        AidType::Bool => "bool",
                                        AidType::String => "String",
                                        _ => "String",
                                    };
                                    main_rs.push_str(&format!("    {}: {},\n", field.name, rust_type));
                                }
                                main_rs.push_str("}\n\n");
                            }
                        }
                    }
                }
            }
        }

        // Emit validation code for each contract
        for contract in contracts {
            if let Some(entity_name) = &contract.entity_name {
                let fields = extract_entity_fields(program, entity_name);
                main_rs.push_str(&generate_contract_validation_code(contract, &fields));
                main_rs.push_str("\n");
            }
        }
    }

    // Emit reason block functions (with telemetry wrappers if evolved)
    for block in reason_blocks {
        let is_evolved = evolved_targets.contains(&block.name.as_str());
        if is_evolved {
            // Rename the function to _logic
            let mut logic_code = generate_reason_function(block);
            logic_code = logic_code.replace(
                &format!("fn {}(", block.name),
                &format!("fn {}_logic(", block.name),
            );
            main_rs.push_str(&logic_code);
            main_rs.push_str("\n");
            // Add the telemetry wrapper
            let input_param = block.params.first().map(|(n, _)| n.as_str()).unwrap_or("text");
            main_rs.push_str(&generate_telemetry_wrapper(&block.name, input_param));
            main_rs.push_str("\n");
        } else {
            main_rs.push_str(&generate_reason_function(block));
            main_rs.push_str("\n");
        }
    }

    // Telemetry endpoint
    if has_evolve {
        main_rs.push_str(&generate_telemetry_endpoint(evolve_blocks));
        main_rs.push_str("\n");
    }

    for route in &info.routes {
        // POST handlers that call reason blocks get special treatment
        if route.method == "post" {
            if let Some(reason_name) = reason_block_for_route(&route.path, reason_blocks) {
                main_rs.push_str(&format!(
                    "async fn {}(body: String) -> impl IntoResponse {{\n",
                    route.handler_name
                ));
                main_rs.push_str(&format!(
                    "    let result = {}(&body);\n",
                    reason_name
                ));
                main_rs.push_str(
                    "    Json(serde_json::json!({\"result\": result})).into_response()\n"
                );
                main_rs.push_str("}\n\n");
                continue;
            }
        }
        main_rs.push_str(&format!(
            "async fn {}() -> impl IntoResponse {{\n{}\n}}\n\n",
            route.handler_name, route.handler_code
        ));
    }

    // Generate /validate handler if contracts present
    if has_contracts {
        if let Some(contract) = contracts.first() {
            if let Some(entity_name) = &contract.entity_name {
                let var_name = entity_name.to_lowercase();
                main_rs.push_str(&format!(
                    "async fn handle_validate(Json(payload): Json<{}>) -> impl IntoResponse {{\n",
                    entity_name
                ));
                main_rs.push_str(&format!(
                    "    let errors = validate_{}(&payload);\n",
                    var_name
                ));
                main_rs.push_str("    if errors.is_empty() {\n");
                main_rs.push_str("        Json(serde_json::json!({\"valid\": true})).into_response()\n");
                main_rs.push_str("    } else {\n");
                main_rs.push_str("        Json(serde_json::json!({\"valid\": false, \"errors\": errors})).into_response()\n");
                main_rs.push_str("    }\n");
                main_rs.push_str("}\n\n");
            }
        }
    }

    main_rs.push_str("#[tokio::main]\nasync fn main() {\n");
    main_rs.push_str("    let app = Router::new()\n");
    for route in &info.routes {
        main_rs.push_str(&format!(
            "        .route(\"{}\", routing::{}({}))\n",
            route.path, route.method, route.handler_name
        ));
    }
    if has_evolve {
        main_rs.push_str("        .route(\"/telemetry\", routing::get(handle_telemetry))\n");
    }
    if has_contracts {
        main_rs.push_str("        .route(\"/validate\", routing::post(handle_validate))\n");
    }
    main_rs.push_str("    ;\n\n");
    main_rs.push_str(&format!(
        "    let listener = tokio::net::TcpListener::bind(\"0.0.0.0:{}\").await.unwrap();\n",
        info.port
    ));
    main_rs.push_str(&format!(
        "    println!(\"🚀 AID server listening on http://0.0.0.0:{}\");\n",
        info.port
    ));
    main_rs.push_str("    axum::serve(listener, app).await.unwrap();\n");
    main_rs.push_str("}\n");

    let chrono_dep = if has_evolve { "\nchrono = \"0.4\"\n" } else { "" };
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
{}"#,
        project_name, chrono_dep
    );

    (main_rs, cargo_toml)
}

// ─── Contract Extraction & Validation Code Generation ────────────────────────

struct ContractInfo {
    name: String,
    rules: Vec<String>,
    /// The first parameter's entity type name (e.g. "User") — used to find matching entity fields
    entity_name: Option<String>,
}

struct EntityFieldInfo {
    name: String,
    typ: String, // "int", "string", "float", "bool"
}

fn extract_contracts(program: &Program) -> Vec<ContractInfo> {
    let mut contracts = Vec::new();
    for decl in &program.declarations {
        if let Declaration::Contract(c) = decl {
            // Find the entity type from the first method's first param
            let entity_name = c.methods.first().and_then(|m| {
                m.params.first().and_then(|p| {
                    match &p.ty {
                        AidType::Entity(name) => Some(name.clone()),
                        _ => None,
                    }
                })
            });
            contracts.push(ContractInfo {
                name: c.name.clone(),
                rules: c.rules.clone(),
                entity_name,
            });
        }
    }
    contracts
}

fn extract_entity_fields(program: &Program, entity_name: &str) -> Vec<EntityFieldInfo> {
    for decl in &program.declarations {
        if let Declaration::Entity(e) = decl {
            if e.name == entity_name {
                return e.fields.iter().map(|f| {
                    let typ = match &f.ty {
                        AidType::Int => "int",
                        AidType::Float => "float",
                        AidType::Bool => "bool",
                        AidType::String => "string",
                        _ => "string",
                    };
                    EntityFieldInfo { name: f.name.clone(), typ: typ.to_string() }
                }).collect();
            }
        }
    }
    Vec::new()
}

/// Parse a contract rule and generate a Rust validation check.
/// Returns (field_name, rust_code) or None if unrecognized.
fn parse_validation_rule(rule: &str, fields: &[EntityFieldInfo], struct_var: &str) -> Option<String> {
    let lower = rule.to_lowercase();

    // Pattern: "X must be a positive integer"
    if lower.contains("must be a positive integer") {
        let field = find_field_in_rule(&lower, fields)?;
        return Some(format!(
            r#"    // "{rule}"
    if {var}.{field} <= 0 {{
        errors.push(ValidationError {{
            field: "{field}".to_string(),
            rule: "{rule}".to_string(),
            message: format!("{field} must be positive, got {{}}", {var}.{field}),
        }});
    }}"#,
            rule = rule, var = struct_var, field = field
        ));
    }

    // Pattern: "X must be between N and M characters"
    if let Some(_) = lower.find("must be between") {
        if lower.contains("characters") || lower.contains("chars") {
            let field = find_field_in_rule(&lower, fields)?;
            let (min, max) = extract_two_numbers(&lower)?;
            return Some(format!(
                r#"    // "{rule}"
    if {var}.{field}.len() < {min} || {var}.{field}.len() > {max} {{
        errors.push(ValidationError {{
            field: "{field}".to_string(),
            rule: "{rule}".to_string(),
            message: format!("{field} length {{}} is out of range [{min}, {max}]", {var}.{field}.len()),
        }});
    }}"#,
                rule = rule, var = struct_var, field = field, min = min, max = max
            ));
        }

        // Pattern: "X must be between N and M" (numeric)
        let field = find_field_in_rule(&lower, fields)?;
        let (min, max) = extract_two_numbers(&lower)?;
        return Some(format!(
            r#"    // "{rule}"
    if {var}.{field} < {min} || {var}.{field} > {max} {{
        errors.push(ValidationError {{
            field: "{field}".to_string(),
            rule: "{rule}".to_string(),
            message: format!("{field} {{}} is out of range [{min}, {max}]", {var}.{field}),
        }});
    }}"#,
            rule = rule, var = struct_var, field = field, min = min, max = max
        ));
    }

    // Pattern: "X must contain exactly one @"
    if lower.contains("must contain exactly one") {
        let field = find_field_in_rule(&lower, fields)?;
        // Extract the character after "one "
        let char_to_match = if lower.contains("@ symbol") || lower.contains("@") {
            '@'
        } else {
            return None;
        };
        return Some(format!(
            r#"    // "{rule}"
    if {var}.{field}.matches('{ch}').count() != 1 {{
        errors.push(ValidationError {{
            field: "{field}".to_string(),
            rule: "{rule}".to_string(),
            message: "{field} must contain exactly one {ch}".to_string(),
        }});
    }}"#,
            rule = rule, var = struct_var, field = field, ch = char_to_match
        ));
    }

    // Pattern: "X must be one of: a, b, c"
    if lower.contains("must be one of:") {
        let field = find_field_in_rule(&lower, fields)?;
        let colon_idx = lower.find("one of:").unwrap() + 7;
        let values_str = &rule[colon_idx..];
        let values: Vec<String> = values_str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let arr_items: Vec<String> = values.iter().map(|v| format!("\"{}\"", v)).collect();
        return Some(format!(
            r#"    // "{rule}"
    if ![{items}].contains(&{var}.{field}.as_str()) {{
        errors.push(ValidationError {{
            field: "{field}".to_string(),
            rule: "{rule}".to_string(),
            message: format!("invalid {field}: {{}}", {var}.{field}),
        }});
    }}"#,
            rule = rule, var = struct_var, field = field, items = arr_items.join(", ")
        ));
    }

    None
}

fn find_field_in_rule(lower_rule: &str, fields: &[EntityFieldInfo]) -> Option<String> {
    // Try to match field names in the rule text
    // First, check if a field name appears directly
    for field in fields {
        if lower_rule.contains(&field.name.to_lowercase()) {
            return Some(field.name.clone());
        }
    }
    // Try common mappings: "User ID" -> "id", "Name" -> "name", etc.
    for field in fields {
        let upper_field = field.name.to_uppercase();
        // Check for "X ID" pattern matching field "id"
        if field.name == "id" && lower_rule.contains(" id ") || lower_rule.contains(" id\n") || lower_rule.ends_with(" id") {
            return Some(field.name.clone());
        }
        // Check by splitting rule words
        let words: Vec<&str> = lower_rule.split_whitespace().collect();
        for word in &words {
            if word.to_lowercase() == field.name.to_lowercase() {
                return Some(field.name.clone());
            }
        }
    }
    None
}

fn extract_two_numbers(text: &str) -> Option<(i64, i64)> {
    let nums: Vec<i64> = text.split(|c: char| !c.is_ascii_digit() && c != '-')
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();
    if nums.len() >= 2 {
        Some((nums[0], nums[1]))
    } else {
        None
    }
}

fn generate_contract_validation_code(contract: &ContractInfo, fields: &[EntityFieldInfo]) -> String {
    let entity_name = contract.entity_name.as_deref().unwrap_or("Input");
    let var_name = entity_name.to_lowercase();
    let mut code = String::new();

    // ValidationError struct
    code.push_str("#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\n");
    code.push_str("struct ValidationError {\n");
    code.push_str("    field: String,\n");
    code.push_str("    rule: String,\n");
    code.push_str("    message: String,\n");
    code.push_str("}\n\n");

    // Validator function
    code.push_str(&format!(
        "fn validate_{}({}: &{}) -> Vec<ValidationError> {{\n",
        var_name, var_name, entity_name
    ));
    code.push_str("    let mut errors = Vec::new();\n\n");

    for rule in &contract.rules {
        if let Some(check) = parse_validation_rule(rule, fields, &var_name) {
            code.push_str(&check);
            code.push_str("\n\n");
        } else {
            code.push_str(&format!("    // TODO: unrecognized rule: \"{}\"\n\n", rule));
        }
    }

    code.push_str("    errors\n");
    code.push_str("}\n");
    code
}

fn generate_docs_from_ast(program: &Program) -> String {
    let mut docs = String::new();
    docs.push_str(&format!("# Module `{}`\n\n", program.module));
    for decl in &program.declarations {
        if let Declaration::Function(f) = decl {
            docs.push_str(&format!("## `fn {}`\n\n", f.name));
        }
    }
    docs
}

// ─── Handlers ────────────────────────────────────────────────────────────────

fn handle_build(file: Option<PathBuf>, release: bool, no_docs: bool, verbose: bool) {
    let config = Config::load_or_default();
    let entry = file
        .unwrap_or_else(|| PathBuf::from(&config.project.entry));

    print_banner();

    let mode_str = if release { "release" } else { "debug" };
    println!(
        "  {} {} {} ({})",
        "Building".green().bold(),
        config.project.name.bold(),
        format!("v{}", config.project.version).dimmed(),
        mode_str
    );
    println!();

    let source = match fs::read_to_string(&entry) {
        Ok(s) => s,
        Err(e) => {
            println!(
                "  {} Could not read {}: {}",
                "✗".red().bold(),
                entry.display(),
                e
            );
            std::process::exit(1);
        }
    };

    let start = Instant::now();

    // ── Phase 1: Parse ──────────────────────────────────────────────────
    let program = match crate::parser::parse_file(&source) {
        Ok(p) => p,
        Err(e) => {
            println!("  {} Parse — {}", "✗".red().bold(), e.to_string().red());
            std::process::exit(1);
        }
    };
    println!("  {} Parse", "✓".green().bold());

    // ── Phase 2: Transpile (AST → Rust) ─────────────────────────────────
    let file_stem = entry
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "main".to_string());
    let project_name = format!("aid-{}", file_stem);

    let reason_blocks = extract_reason_blocks(&program);
    if !reason_blocks.is_empty() {
        println!(
            "  {} Reason blocks: {} found",
            "✓".green().bold(),
            reason_blocks.len()
        );
    }

    let evolve_blocks = extract_evolve_blocks(&program);
    if !evolve_blocks.is_empty() {
        println!(
            "  {} Evolve: {} blocks tracked",
            "✓".green().bold(),
            evolve_blocks.len()
        );

        // Check for existing telemetry data
        let mut has_telemetry = false;
        for eb in &evolve_blocks {
            if let Some((count, dist)) = read_telemetry_stats(&eb.target) {
                if !has_telemetry {
                    println!();
                    println!("  {} Evolve telemetry:", "⚡".yellow().bold());
                    has_telemetry = true;
                }
                let dist_str: Vec<String> = dist.iter().map(|(k, v)| {
                    let pct = (*v as f64 / count as f64 * 100.0) as u32;
                    format!("{}: {}%", k, pct)
                }).collect();
                println!(
                    "    {}: {} calls ({})",
                    eb.target,
                    count,
                    dist_str.join(", ")
                );
            }
        }
        if has_telemetry {
            println!();
        }
    }

    let contracts = extract_contracts(&program);
    if !contracts.is_empty() {
        let total_rules: usize = contracts.iter().map(|c| c.rules.len()).sum();
        let total_validators: usize = contracts.iter().map(|c| {
            if let Some(entity_name) = &c.entity_name {
                let fields = extract_entity_fields(&program, entity_name);
                c.rules.iter().filter(|r| parse_validation_rule(r, &fields, "x").is_some()).count()
            } else { 0 }
        }).sum();
        println!(
            "  {} Contracts: {} found ({} rules → {} validators)",
            "✓".green().bold(),
            contracts.len(),
            total_rules,
            total_validators,
        );
    }

    let (main_rs, cargo_toml) = if let Some(info) = extract_http_server(&program) {
        generate_http_project_with_evolve(&project_name, &info, &reason_blocks, &evolve_blocks, &contracts, &program)
    } else {
        println!(
            "  {} Transpile — no supported pattern found in source",
            "✗".red().bold()
        );
        std::process::exit(1);
    };
    println!("  {} Transpile", "✓".green().bold());

    if verbose {
        println!("  {} Generated main.rs ({} bytes)", "→".dimmed(), main_rs.len());
    }

    // ── Phase 3: Write generated project ────────────────────────────────
    let build_dir = PathBuf::from("build/rust-gen");
    fs::create_dir_all(build_dir.join("src")).unwrap_or_else(|e| {
        println!("  {} Could not create build dir: {}", "✗".red().bold(), e);
        std::process::exit(1);
    });
    fs::write(build_dir.join("src/main.rs"), &main_rs).unwrap();
    fs::write(build_dir.join("Cargo.toml"), &cargo_toml).unwrap();
    println!("  {} Codegen", "✓".green().bold());

    // ── Phase 4: Compile with cargo ─────────────────────────────────────
    let cargo_args = if release {
        vec!["build", "--release"]
    } else {
        vec!["build"]
    };

    let output = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(&build_dir)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            println!("  {} Compile", "✓".green().bold());
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            println!("  {} Compile — cargo build failed", "✗".red().bold());
            if verbose || true {
                eprintln!("{}", stderr);
            }
            std::process::exit(1);
        }
        Err(e) => {
            println!("  {} Compile — {}", "✗".red().bold(), e);
            std::process::exit(1);
        }
    }

    // ── Phase 5: Copy binary ────────────────────────────────────────────
    let profile_dir = if release { "release" } else { "debug" };
    let src_binary = build_dir
        .join("target")
        .join(profile_dir)
        .join(&project_name);
    let dest_binary = PathBuf::from("build").join(&project_name);

    if src_binary.exists() {
        fs::copy(&src_binary, &dest_binary).ok();
        // Make executable on unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dest_binary, fs::Permissions::from_mode(0o755)).ok();
        }
        println!(
            "  {} Binary → {}",
            "✓".green().bold(),
            dest_binary.display()
        );
    }

    // ── Phase 6: Docs ───────────────────────────────────────────────────
    if !no_docs {
        let docs = generate_docs_from_ast(&program);
        let docs_path = PathBuf::from("build/docs.md");
        fs::write(&docs_path, &docs).ok();
        println!("  {} Docs", "✓".green().bold());
    }

    let elapsed = start.elapsed();
    println!();
    println!(
        "  {} in {:.2}s",
        "Build complete".green().bold(),
        elapsed.as_secs_f64()
    );
    println!();
}

fn handle_run(file: Option<PathBuf>, port: u16, watch: bool) {
    let config = Config::load_or_default();

    // Build first
    handle_build(file.clone(), false, true, false);

    let bin_name = &config.project.name;
    let bin_path = PathBuf::from("target").join("debug").join(bin_name);

    if watch {
        println!(
            "  {} Watching for changes (port {})...",
            "⟳".yellow().bold(),
            port
        );
        println!("  {}", "Watch mode not yet implemented".yellow());
        return;
    }

    println!(
        "  {} {} on port {}",
        "Running".green().bold(),
        bin_name.bold(),
        port
    );
    println!();

    let status = Command::new(&bin_path)
        .env("PORT", port.to_string())
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            let code = s.code().unwrap_or(1);
            println!(
                "  {} Process exited with code {}",
                "✗".red().bold(),
                code
            );
            std::process::exit(code);
        }
        Err(e) => {
            println!(
                "  {} Failed to execute {}: {}",
                "✗".red().bold(),
                bin_path.display(),
                e
            );
            std::process::exit(1);
        }
    }
}

fn handle_new(name: &str) {
    print_banner();
    println!("  {} Creating project '{}'...", "→".dimmed(), name.bold());
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_clean() {
    print_banner();
    println!("  {} Cleaning build artifacts...", "→".dimmed());
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_docs(serve: bool, format: &DocsFormat) {
    print_banner();
    let fmt = match format {
        DocsFormat::Markdown => "markdown",
        DocsFormat::Html => "html",
    };
    println!("  {} Generating docs (format: {})...", "→".dimmed(), fmt);
    if serve {
        println!("  {} Serving docs locally...", "→".dimmed());
    }
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_fmt(files: &[PathBuf]) {
    print_banner();
    if files.is_empty() {
        println!("  {} Formatting all .aid files...", "→".dimmed());
    } else {
        println!("  {} Formatting {} file(s)...", "→".dimmed(), files.len());
    }
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_lint(files: &[PathBuf]) {
    print_banner();
    if files.is_empty() {
        println!("  {} Linting all .aid files...", "→".dimmed());
    } else {
        println!("  {} Linting {} file(s)...", "→".dimmed(), files.len());
    }
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_test(reason: bool) {
    print_banner();
    if reason {
        println!("  {} Testing reason blocks...", "→".dimmed());
    } else {
        println!("  {} Running tests...", "→".dimmed());
    }
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_cortex_status() {
    print_banner();
    let config = Config::load_or_default();
    println!("  {}", "Cortex Engine Status".bold());
    println!("  Mode:       {}", config.cortex.mode);
    println!("  Confidence: {:.0}%", config.cortex.confidence * 100.0);
    println!("  {}", "Not fully implemented yet".yellow());
}

fn handle_cortex_test(block: &str) {
    print_banner();
    println!("  {} Testing reason block '{}'...", "→".dimmed(), block.bold());
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_rollback(name: &str, to: Option<u32>) {
    print_banner();
    match to {
        Some(v) => println!(
            "  {} Rolling back '{}' to version {}...",
            "→".dimmed(),
            name.bold(),
            v
        ),
        None => println!(
            "  {} Rolling back '{}' to previous version...",
            "→".dimmed(),
            name.bold()
        ),
    }
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_evolve_status() {
    print_banner();
    println!("  {} Evolution status for all tracked blocks:", "→".dimmed());
    println!("  {}", "Not implemented yet".yellow());
}

fn handle_evolve_history(block: &str) {
    print_banner();
    println!(
        "  {} Evolution history for '{}':",
        "→".dimmed(),
        block.bold()
    );
    println!("  {}", "Not implemented yet".yellow());
}

// ─── Entry Point ─────────────────────────────────────────────────────────────

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => handle_new(&name),
        Commands::Build {
            file,
            release,
            no_docs,
            verbose,
        } => handle_build(file, release, no_docs, verbose),
        Commands::Run { file, port, watch } => handle_run(file, port, watch),
        Commands::Test { reason } => handle_test(reason),
        Commands::Clean => handle_clean(),
        Commands::Docs { serve, format } => handle_docs(serve, &format),
        Commands::Fmt { files } => handle_fmt(&files),
        Commands::Lint { files } => handle_lint(&files),
        Commands::Cortex { command } => match command {
            CortexCommands::Status => handle_cortex_status(),
            CortexCommands::Test { block } => handle_cortex_test(&block),
        },
        Commands::Rollback { name, to } => handle_rollback(&name, to),
        Commands::Evolve { command } => match command {
            EvolveCommands::Status => handle_evolve_status(),
            EvolveCommands::History { block } => handle_evolve_history(&block),
        },
    }
}
