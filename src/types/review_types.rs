//! Just data shapes, no logic. In TypeScript an `interface` only exists at
//! compile time. In Rust a `struct` is a real type, and with serde's derive
//! macros it can also be turned into / built from JSON automatically.
//!
//! In Rust, you must explicitly declare which types can be serialized and
//! deserialized. The `#[derive(Serialize, Deserialize)]` attribute tells the
//! compiler to generate the necessary code for this.

// but never imported anywhere). Without this line the compiler would warn
// "never used" -- this tells it "I know, it's for later".
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

//  severity: "low" | "medium" | "high" | "critical";
// A string-union becomes a Rust `enum`. `rename_all = "lowercase"` makes
// JSON say "low" instead of "Low", so it matches your original strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

// export interface ReviewFinding { ... }
// `#[derive(...)]` auto-writes boilerplate for us:
//   Debug       -> lets you print it with {:?}
//   Clone       -> lets you .clone() it
//   Serialize   -> struct -> JSON      (like JSON.stringify)
//   Deserialize -> JSON -> struct      (like JSON.parse, but it VALIDATES)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub file: String,
    pub line: u32, // `number` ; Rust makes you choose a number type. u32 = whole number >= 0
    pub severity: Severity,
    pub title: String,
    pub description: String,
    // suggestion?: string;
    // `?` (optional field) becomes `Option<String>`: either Some("text") or None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}
