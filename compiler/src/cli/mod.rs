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

fn generate_http_project(project_name: &str, info: &HttpServerInfo) -> (String, String) {
    let mut main_rs = String::new();
    main_rs.push_str("// Generated by the AID compiler — do not edit.\n\n");
    main_rs.push_str("use axum::{Router, routing, Json, response::IntoResponse};\n\n");

    for route in &info.routes {
        main_rs.push_str(&format!(
            "async fn {}() -> impl IntoResponse {{\n{}\n}}\n\n",
            route.handler_name, route.handler_code
        ));
    }

    main_rs.push_str("#[tokio::main]\nasync fn main() {\n");
    main_rs.push_str("    let app = Router::new()\n");
    for route in &info.routes {
        main_rs.push_str(&format!(
            "        .route(\"{}\", routing::{}({}))\n",
            route.path, route.method, route.handler_name
        ));
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
"#,
        project_name
    );

    (main_rs, cargo_toml)
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

    let (main_rs, cargo_toml) = if let Some(info) = extract_http_server(&program) {
        generate_http_project(&project_name, &info)
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
