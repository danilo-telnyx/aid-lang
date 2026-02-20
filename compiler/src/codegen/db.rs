//! AID std.db → rusqlite code generation
//!
//! Maps AID database constructs to rusqlite Rust code.

/// Generates Rust code for database operations.
pub struct DbCodegen;

impl DbCodegen {
    /// Generate the connection setup code.
    /// `db.connect("sqlite://data.db")` → `rusqlite::Connection::open("data.db")`
    pub fn generate_connect(db_path: &str) -> String {
        // Strip "sqlite://" prefix if present
        let path = db_path.strip_prefix("sqlite://").unwrap_or(db_path);
        format!(
            r#"let db = rusqlite::Connection::open("{}").expect("failed to open database");"#,
            path
        )
    }

    /// Generate execute code.
    /// `db.execute("SQL")` → `db.execute_batch("SQL")`
    pub fn generate_execute(sql: &str) -> String {
        format!(
            r#"db.execute_batch("{}").expect("failed to execute SQL");"#,
            sql.replace('"', r#"\""#)
        )
    }

    /// Generate query code that returns JSON array.
    /// `db.query("SELECT * FROM table")` → prepare + query_map → Vec<serde_json::Value>
    pub fn generate_query(var_name: &str, sql: &str) -> String {
        format!(
            r#"let {var} = {{
        let mut stmt = db.prepare("{sql}").expect("failed to prepare query");
        let column_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();
        let rows = stmt.query_map([], |row| {{
            let mut map = serde_json::Map::new();
            for (i, col) in column_names.iter().enumerate() {{
                let val: rusqlite::types::Value = row.get_unwrap(i);
                let json_val = match val {{
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(n) => serde_json::json!(n),
                    rusqlite::types::Value::Real(f) => serde_json::json!(f),
                    rusqlite::types::Value::Text(s) => serde_json::json!(s),
                    rusqlite::types::Value::Blob(b) => serde_json::json!(format!("{{:?}}", b)),
                }};
                map.insert(col.clone(), json_val);
            }}
            Ok(serde_json::Value::Object(map))
        }}).expect("query failed");
        rows.filter_map(|r| r.ok()).collect::<Vec<serde_json::Value>>()
    }};"#,
            var = var_name,
            sql = sql.replace('"', r#"\""#)
        )
    }

    /// Generate migrate code.
    /// `db.migrate("migrations/")` → read .sql files, execute in order
    pub fn generate_migrate(dir: &str) -> String {
        format!(
            r#"{{
        let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir("{dir}")
            .expect("failed to read migrations directory")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|ext| ext == "sql").unwrap_or(false))
            .collect();
        entries.sort();
        for path in entries {{
            let sql = std::fs::read_to_string(&path)
                .expect(&format!("failed to read migration {{:?}}", path));
            db.execute_batch(&sql)
                .expect(&format!("failed to run migration {{:?}}", path));
            println!("  ✓ Migrated: {{:?}}", path.file_name().unwrap_or_default());
        }}
    }}"#,
            dir = dir
        )
    }

    /// Cargo dependencies required by generated db code.
    pub fn required_dependencies() -> Vec<(&'static str, &'static str)> {
        vec![
            ("rusqlite", r#"{ version = "0.31", features = ["bundled"] }"#),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_connect() {
        let code = DbCodegen::generate_connect("sqlite://data.db");
        assert!(code.contains("Connection::open"));
        assert!(code.contains("data.db"));
        assert!(!code.contains("sqlite://"));
    }

    #[test]
    fn test_generate_connect_no_prefix() {
        let code = DbCodegen::generate_connect("mydb.db");
        assert!(code.contains("mydb.db"));
    }

    #[test]
    fn test_generate_execute() {
        let code = DbCodegen::generate_execute("CREATE TABLE t (id INTEGER)");
        assert!(code.contains("execute_batch"));
        assert!(code.contains("CREATE TABLE"));
    }

    #[test]
    fn test_generate_query() {
        let code = DbCodegen::generate_query("results", "SELECT * FROM users");
        assert!(code.contains("let results"));
        assert!(code.contains("prepare"));
        assert!(code.contains("query_map"));
        assert!(code.contains("SELECT * FROM users"));
    }

    #[test]
    fn test_generate_migrate() {
        let code = DbCodegen::generate_migrate("migrations/");
        assert!(code.contains("read_dir"));
        assert!(code.contains("migrations/"));
        assert!(code.contains("sql"));
    }
}
