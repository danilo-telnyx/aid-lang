// AID Language — Cortex V1 Code Generation
//
// Generates Rust code for reason blocks that:
// 1. Tries to call the local Cortex sidecar via HTTP (localhost:8090)
// 2. Falls back to V1 keyword matching if sidecar is unavailable
//
// The Cortex sidecar (`aid cortex serve`) loads a GGUF model and provides
// local-only LLM inference. No network calls, no cloud APIs.

/// Configuration for Cortex code generation
#[derive(Debug, Clone)]
pub struct CortexCodegenConfig {
    pub sidecar_port: u16,
    pub timeout_ms: u64,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for CortexCodegenConfig {
    fn default() -> Self {
        Self {
            sidecar_port: 8090,
            timeout_ms: 5000,
            temperature: 0.1,
            max_tokens: 256,
        }
    }
}

/// Information about a reason block needed for Cortex code generation
#[derive(Debug, Clone)]
pub struct ReasonBlockSpec {
    pub name: String,
    pub params: Vec<(String, String)>, // (name, aid_type)
    pub return_type: String,
    pub goal: String,
    pub constraints: Vec<String>,
    pub examples: Vec<(String, String)>, // (input, output)
    pub fallback: Option<String>,
}

/// Generate a reason block function that tries Cortex sidecar first,
/// then falls back to V1 keyword matching.
pub fn generate_cortex_reason_function(block: &ReasonBlockSpec, config: &CortexCodegenConfig) -> String {
    let mut code = String::new();

    // Doc comment
    code.push_str(&format!("/// Reason block: {} (Cortex V1)\n", block.name));
    code.push_str(&format!("/// Goal: {}\n", block.goal));
    for c in &block.constraints {
        code.push_str(&format!("/// Constraint: {}\n", c));
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

    code.push_str(&format!(
        "fn {}({}) -> {} {{\n",
        block.name,
        params_str.join(", "),
        block.return_type
    ));

    let input_param = block.params.first().map(|(n, _)| n.as_str()).unwrap_or("text");

    // Try Cortex sidecar first
    code.push_str(&format!(
        "    // Try Cortex sidecar (local LLM) first\n"
    ));
    code.push_str(&format!(
        "    if let Some(result) = cortex_infer(\"{name}\", {input}, &{prompt}) {{\n",
        name = block.name,
        input = input_param,
        prompt = generate_prompt_builder(block),
    ));
    code.push_str("        return result;\n");
    code.push_str("    }\n\n");

    // V1 keyword matching fallback
    code.push_str("    // Fallback: V1 keyword matching\n");
    code.push_str(&format!("    let text_lower = {}.to_lowercase();\n\n", input_param));

    // Constraint-based rules
    let constraint_rules = parse_constraint_rules(&block.constraints);
    for (keywords, result) in &constraint_rules {
        let conditions: Vec<String> = keywords.iter()
            .map(|k| format!("text_lower.contains(\"{}\")", k))
            .collect();
        code.push_str(&format!("    if {} {{\n", conditions.join(" || ")));
        code.push_str(&format!("        return \"{}\".to_string();\n", result));
        code.push_str("    }\n\n");
    }

    // Example-based keyword matching
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "can",
        "for", "and", "nor", "but", "or", "yet", "so", "in", "on", "at", "to", "of", "by",
        "with", "from", "my", "your", "his", "her", "its", "our", "their", "this", "that",
        "what", "how", "not", "you", "it", "i", "we", "they", "me",
    ];

    let mut categories: Vec<(String, Vec<String>)> = Vec::new();
    for (input, output) in &block.examples {
        let keywords: Vec<String> = input.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 3 && !stop_words.contains(w))
            .map(|w| w.to_string())
            .collect();
        if let Some(existing) = categories.iter_mut().find(|(cat, _)| cat == output) {
            existing.1.extend(keywords);
        } else {
            categories.push((output.clone(), keywords));
        }
    }

    let constraint_outputs: Vec<&String> = constraint_rules.iter().map(|(_, r)| r).collect();

    for (category, mut keywords) in categories {
        if !keywords.contains(&category) {
            keywords.push(category.clone());
        }
        keywords.sort();
        keywords.dedup();

        if constraint_outputs.contains(&&category) {
            continue;
        }

        let conditions: Vec<String> = keywords.iter()
            .map(|k| format!("text_lower.contains(\"{}\")", k))
            .collect();
        code.push_str(&format!("    if {} {{\n", conditions.join(" || ")));
        code.push_str(&format!("        return \"{}\".to_string();\n", category));
        code.push_str("    }\n\n");
    }

    // Fallback
    let fallback_val = block.fallback.as_deref().unwrap_or("unknown");
    code.push_str(&format!("    \"{}\".to_string()\n", fallback_val));
    code.push_str("}\n");

    code
}

/// Generate the prompt builder expression for a reason block
fn generate_prompt_builder(block: &ReasonBlockSpec) -> String {
    let mut prompt_parts = Vec::new();
    prompt_parts.push(format!("Goal: {}", block.goal));

    if !block.constraints.is_empty() {
        prompt_parts.push("Constraints:".to_string());
        for c in &block.constraints {
            prompt_parts.push(format!("- {}", c));
        }
    }

    if !block.examples.is_empty() {
        prompt_parts.push("Examples:".to_string());
        for (input, output) in &block.examples {
            prompt_parts.push(format!("Input: \"{}\" → Output: \"{}\"", input, output));
        }
    }

    let prompt_str = prompt_parts.join("\\n");
    format!("format!(\"{}\\n\\nInput: \\\"{{}}\\\"\\nOutput:\", {})",
        prompt_str.replace('"', "\\\""),
        block.params.first().map(|(n, _)| n.as_str()).unwrap_or("text"))
}

/// Generate the `cortex_infer` helper function that calls the local sidecar
pub fn generate_cortex_infer_function(config: &CortexCodegenConfig) -> String {
    format!(r#"/// Call the local Cortex sidecar for LLM inference.
/// Returns None if sidecar is not running (falls back to V1 keyword matching).
fn cortex_infer(block_name: &str, input: &str, prompt: &str) -> Option<String> {{
    let client = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis({timeout}))
        .build();

    let url = format!("http://127.0.0.1:{port}/v1/reason");
    let payload = serde_json::json!({{
        "block": block_name,
        "input": input,
        "prompt": prompt,
        "temperature": {temperature},
        "max_tokens": {max_tokens}
    }});

    match client.post(&url).send_json(&payload) {{
        Ok(response) => {{
            if let Ok(body) = response.into_string() {{
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {{
                    if let Some(result) = json.get("result").and_then(|r| r.as_str()) {{
                        return Some(result.trim().trim_matches('"').to_string());
                    }}
                }}
            }}
            None
        }}
        Err(_) => None // Sidecar not running — fall back to V1
    }}
}}
"#,
        timeout = config.timeout_ms,
        port = config.sidecar_port,
        temperature = config.temperature,
        max_tokens = config.max_tokens,
    )
}

/// Generate Cargo.toml dependency entries needed for Cortex runtime
pub fn cortex_cargo_dependencies() -> String {
    r#"ureq = { version = "2", features = ["json"] }
"#.to_string()
}

/// Parse constraint rules like "Tickets mentioning X or Y are always Z"
fn parse_constraint_rules(constraints: &[String]) -> Vec<(Vec<String>, String)> {
    let mut rules = Vec::new();
    for constraint in constraints {
        let lower = constraint.to_lowercase();
        if let Some(always_idx) = lower.find("always ") {
            let result_word = lower[always_idx + 7..].trim().to_string();
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

// ─── Cortex Sidecar Server Code ─────────────────────────────────────────────

/// Default model filename
pub const DEFAULT_MODEL: &str = "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf";

/// Default model URL for download
pub const DEFAULT_MODEL_URL: &str = "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf";

/// Default model directory
pub const MODELS_DIR: &str = ".cortex/models";

/// Default sidecar port
pub const SIDECAR_PORT: u16 = 8090;

/// Generate cortex.toml default configuration
pub fn generate_cortex_toml() -> String {
    format!(r#"# Cortex V1 Configuration
# Local LLM engine for AID reason blocks

[model]
# Path to GGUF model file (relative to project root)
path = "{models_dir}/{model}"

# Model parameters
temperature = 0.1
max_tokens = 256
top_p = 0.9

[server]
# Sidecar HTTP server port (localhost only)
port = {port}

# Request timeout in milliseconds
timeout_ms = 5000

[fallback]
# Fall back to V1 keyword matching if Cortex sidecar is unavailable
enabled = true
"#,
        models_dir = MODELS_DIR,
        model = DEFAULT_MODEL,
        port = SIDECAR_PORT,
    )
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_block() -> ReasonBlockSpec {
        ReasonBlockSpec {
            name: "classify_ticket".to_string(),
            params: vec![("text".to_string(), "string".to_string())],
            return_type: "String".to_string(),
            goal: "Classify a support ticket into a category".to_string(),
            constraints: vec![
                "Return one of: billing, technical, general, urgent".to_string(),
                "Tickets mentioning outage or down are always urgent".to_string(),
            ],
            examples: vec![
                ("My credit card was charged twice".to_string(), "billing".to_string()),
                ("The server is completely down".to_string(), "urgent".to_string()),
                ("How do I reset my password?".to_string(), "technical".to_string()),
                ("I have a question about your product".to_string(), "general".to_string()),
            ],
            fallback: Some("general".to_string()),
        }
    }

    #[test]
    fn test_generate_cortex_reason_function() {
        let block = sample_block();
        let config = CortexCodegenConfig::default();
        let code = generate_cortex_reason_function(&block, &config);

        assert!(code.contains("fn classify_ticket("));
        assert!(code.contains("cortex_infer("));
        assert!(code.contains("text_lower"));
        assert!(code.contains("billing"));
        assert!(code.contains("urgent"));
        assert!(code.contains("general"));
    }

    #[test]
    fn test_generate_cortex_infer_function() {
        let config = CortexCodegenConfig::default();
        let code = generate_cortex_infer_function(&config);

        assert!(code.contains("fn cortex_infer("));
        assert!(code.contains("127.0.0.1:8090"));
        assert!(code.contains("ureq"));
        assert!(code.contains("/v1/reason"));
    }

    #[test]
    fn test_cortex_cargo_dependencies() {
        let deps = cortex_cargo_dependencies();
        assert!(deps.contains("ureq"));
    }

    #[test]
    fn test_generate_cortex_toml() {
        let toml = generate_cortex_toml();
        assert!(toml.contains("[model]"));
        assert!(toml.contains("[server]"));
        assert!(toml.contains("port = 8090"));
        assert!(toml.contains("tinyllama"));
    }

    #[test]
    fn test_parse_constraint_rules() {
        let constraints = vec![
            "Return one of: billing, technical, general, urgent".to_string(),
            "Tickets mentioning outage or down are always urgent".to_string(),
        ];
        let rules = parse_constraint_rules(&constraints);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].1, "urgent");
        assert!(rules[0].0.contains(&"outage".to_string()));
        assert!(rules[0].0.contains(&"down".to_string()));
    }

    #[test]
    fn test_fallback_when_no_cortex() {
        let block = sample_block();
        let config = CortexCodegenConfig::default();
        let code = generate_cortex_reason_function(&block, &config);

        // Code should contain both Cortex call and keyword fallback
        assert!(code.contains("cortex_infer("));
        assert!(code.contains("// Fallback: V1 keyword matching"));
        assert!(code.contains("\"general\".to_string()"));
    }
}
