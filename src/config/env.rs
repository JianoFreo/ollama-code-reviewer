//! env.ts  ->  env.rs
//!
//! Reads configuration from environment variables (and your `.env` file) ONE
//! time, and exposes it as a global called `ENV`, exactly like your TS file.
//!
//! Usage anywhere else in the project:
//!     use crate::config::env::ENV;
//!     ENV.github_token      // instead of ENV.GITHUB_TOKEN
//!
//! (Rust style is snake_case for fields, so GITHUB_TOKEN -> github_token.
//!  The variable names inside your .env file are UNCHANGED.)

use std::collections::HashMap;
use std::sync::LazyLock;

// TS:  export const ENV = { ... };
//
// In TS the object's shape is inferred. In Rust we declare it up front.
//
// `Option<String>`  ==  `string | undefined`.
//   Some("abc")  -> the variable exists and is not empty
//   None         -> missing (or empty -- same as how `||` / `!x` treat "" in JS)
// Rust will not let you use an Option as if it were a String until you've
// handled the None case. That's the compiler catching "undefined" bugs.
//
// The allow(dead_code) is because a few fields (ai_provider, node_env, port)
// are loaded but not read anywhere yet -- the same is true in the TS version.
#[allow(dead_code)]
pub struct Env {
    // AI Provider
    pub ai_provider: String,
    pub ai_base_url: Option<String>,
    pub ai_model: Option<String>,
    pub ai_api_key: Option<String>,

    // GitHub
    pub github_api_url: String,
    pub github_token: Option<String>,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    //pub github_owner: var("GITHUB_OWNER").or_else(|| Some("JianoFreo".to_string())),
    //pub github_repo: var("GITHUB_REPO").or_else(|| Some("your-repo-name".to_string())),
    ///// or if you want to hardcode them, you can do that too:
    //pub github_owner: Some("JianoFreo".to_string()),
    //pub github_repo: Some("your-repo-name".to_string()),
    // Application
    pub node_env: String,
    pub port: u16,
}

// ---------------------------------------------------------------------
// .env loading   (TS:  import "dotenv/config";)
// ---------------------------------------------------------------------
// Node's `dotenv` reads the `.env` file in the current folder and copies its
// lines into process.env. We do the same job here with a small hand-written
// loader instead of a crate.
//
// WHY not the `dotenvy` crate? It is stricter than Node's dotenv: it STOPS at
// the first line it can't understand. Your README's example .env contains one
// (`AI_API_KEY=Leave it empty. Local Ollama doesn't need a key.`), so dotenvy
// would silently ignore every line after it (GITHUB_TOKEN, GITHUB_OWNER, ...).
// This loader follows dotenv's lenient rules, so your existing .env keeps
// working unchanged:
//   - blank lines and `# comments` are ignored
//   - `export KEY=value` is allowed
//   - unquoted values end at the first `#` (inline comment) and are trimmed
//   - "double", 'single' or `backtick` quoted values keep everything inside
//   - lines it can't understand are skipped instead of aborting
//   - if a key appears twice, the LAST one wins
//   - (not supported: values that span several lines)

/// The parsed contents of `.env`, loaded once (empty if the file doesn't exist).
static DOTENV: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    std::fs::read_to_string(".env")
        .map(|text| parse_dotenv(&text))
        .unwrap_or_default() // no .env file? fine, use an empty map
});

/// Turns the text of a .env file into key -> value pairs.
fn parse_dotenv(text: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    // `.lines()` handles both "\n" and Windows "\r\n" line endings.
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue; // blank line or comment
        }
        let line = line.strip_prefix("export ").unwrap_or(line);

        // Split at the FIRST '='.  "A=b=c" -> ("A", "b=c")
        let Some((key, value)) = line.split_once('=') else {
            continue; // no '=' at all, e.g. a stray sentence: skip it
        };

        // Valid names are letters, digits, `_`, `.`, `-` (same as dotenv).
        // This is what rejects "Paste it in: GITHUB_TOKEN" from your README.
        let key = key.trim();
        let valid = !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'));
        if !valid {
            continue;
        }

        // `insert` overwrites an existing key -> "last one wins".
        vars.insert(key.to_string(), parse_value(value));
    }

    vars
}

/// Cleans up the part after `=`.
fn parse_value(raw: &str) -> String {
    let raw = raw.trim_start();

    // Quoted value: take everything between the quotes.
    // (The quote is a 1-byte ASCII char, so slicing at 1 is safe.)
    if let Some(quote) = raw.chars().next().filter(|c| matches!(c, '"' | '\'' | '`')) {
        if let Some(end) = raw[1..].find(quote) {
            let inner = &raw[1..1 + end];
            return if quote == '"' {
                // like dotenv, "\n" inside double quotes becomes a real newline
                inner.replace("\\n", "\n").replace("\\r", "\r")
            } else {
                inner.to_string()
            };
        }
    }

    // Unquoted value: stop at the first `#`, then trim spaces.
    raw.split('#').next().unwrap_or("").trim().to_string()
}

/// Helper: `process.env.NAME`  ->  `var("NAME")`
///
/// Like dotenv, a REAL environment variable (set in your shell) always beats
/// the same name in the .env file. Returns None when the final value is
/// missing OR empty (the same cases where `||` / `!x` is truthy-checked in JS).
fn var(name: &str) -> Option<String> {
    let value = match std::env::var(name) {
        Ok(from_shell) => Some(from_shell),
        Err(_) => DOTENV.get(name).cloned(),
    };
    value.filter(|v| !v.is_empty())
}

// `LazyLock` = "run this block the first time somebody touches ENV, then
// remember the result". That's the closest thing to a TS module that runs
// its top-level code once, on first import.
pub static ENV: LazyLock<Env> = LazyLock::new(|| {
    Env {
        // AI Provider
        // TS:  process.env.AI_PROVIDER || "ollama"
        ai_provider: var("AI_PROVIDER").unwrap_or_else(|| "ollama".to_string()),
        ai_base_url: var("AI_BASE_URL"),
        ai_model: var("AI_MODEL"),
        ai_api_key: var("AI_API_KEY"),

        // GitHub
        github_api_url: var("GITHUB_API_URL")
            .unwrap_or_else(|| "https://api.github.com".to_string()),
        github_token: var("GITHUB_TOKEN"),
        github_owner: var("GITHUB_OWNER"),
        github_repo: var("GITHUB_REPO"),

        // Application
        node_env: var("NODE_ENV").unwrap_or_else(|| "development".to_string()),
        // parse() can fail (like Number("abc") -> NaN), so it returns a Result;
        // .ok() turns "failed" into None, and unwrap_or(3000) is the `|| 3000`.
        port: var("PORT")
            .and_then(|p| p.parse::<u16>().ok())
            .filter(|p| *p != 0)
            .unwrap_or(3000),
    }
});

// ---------------------------------------------------------------------
// Tests  (run them with:  cargo test)
// ---------------------------------------------------------------------
// There was no test setup in the TS project; this one is here to prove the
// loader reads your README's example .env the same way Node's dotenv does.
// `#[cfg(test)]` = "only compile this part when running `cargo test`".
#[cfg(test)]
mod tests {
    use super::parse_dotenv;

    // The exact block from your README.
    const README_ENV: &str = "# AI Configuration -- locally
AI_PROVIDER=ollama
AI_BASE_URL=http://localhost:11434
AI_MODEL=qwen2.5-coder:7b
AI_API_KEY=Leave it empty. Local Ollama doesn\u{2019}t need a key.

# AI Configuration -- cloud based
AI_PROVIDER=openai
AI_BASE_URL=https://api.openai.com/v1
AI_MODEL=gpt-6
AI_API_KEY=your_openai_api_key

# GitHub Configuration
GITHUB_API_URL=https://api.github.com #Leave it as is. Only change it if you use GitHub Enterprise.
GITHUB_TOKEN=
GITHUB_OWNER=JianoFreo #The first part of the repo URL: github.com/owner/repo
GITHUB_REPO=# The second part of the repo URL: github.com/owner/repo

Paste it in: GITHUB_TOKEN=github_pat_...
# Application
NODE_ENV=development
PORT=3000
";

    #[test]
    fn readme_env_loads_every_line_even_after_odd_ones() {
        let vars = parse_dotenv(README_ENV);
        assert_eq!(vars["GITHUB_API_URL"], "https://api.github.com"); // inline comment cut off
        assert_eq!(vars["GITHUB_OWNER"], "JianoFreo");
        assert_eq!(vars["GITHUB_REPO"], ""); // `=# comment` -> empty
        assert_eq!(vars["GITHUB_TOKEN"], ""); // the stray "Paste it in:" line is ignored
        assert_eq!(vars["NODE_ENV"], "development"); // still loaded after the odd lines
        assert_eq!(vars["PORT"], "3000");
    }

    #[test]
    fn last_duplicate_wins_like_node_dotenv() {
        let vars = parse_dotenv(README_ENV);
        assert_eq!(vars["AI_PROVIDER"], "openai");
        assert_eq!(vars["AI_MODEL"], "gpt-6");
    }

    #[test]
    fn quotes_export_and_windows_line_endings() {
        let vars = parse_dotenv("export A=1\r\nB=\"two words # kept\"\r\nC='x=y'\r\nD = spaced \r\n");
        assert_eq!(vars["A"], "1");
        assert_eq!(vars["B"], "two words # kept");
        assert_eq!(vars["C"], "x=y");
        assert_eq!(vars["D"], "spaced");
    }
}
