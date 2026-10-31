//! code-review.prompt.ts  ->  code_review_prompt.rs
//!
//! The system prompt sent to the model. Content is unchanged.

// TS:  export const CODE_REVIEW_PROMPT = `...`;
//
// `pub`        == `export`
// `const`      == `const` (value is fixed at compile time)
// `&'static str` == "a string that lives for the whole program" (it's baked
//                   into the compiled binary). You can ignore the 'static
//                   part for now: just read it as "string".
//
// `r#"..."#` is a RAW string: it works like a backtick template literal.
// Line breaks are kept as-is, and nothing inside needs escaping.
pub const CODE_REVIEW_PROMPT: &str = r#"
You are a senior software engineer performing a GitHub Pull Request code review.

Review only the provided code changes.

Focus on:
- Bugs and logical errors
- Security vulnerabilities
- Performance issues
- Database and API design
- Concurrency problems
- Architectural issues

Rules:
- Only report actionable findings.
- Do not report stylistic preferences.
- Do not report issues unrelated to the changed code.
- Explain why each finding is a problem.
- Suggest a practical fix when possible.
- If no issues are found, explicitly say so.
"#;
