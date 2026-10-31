//! github.service.ts  ->  github_service.rs
//!
//! Everything that talks to the GitHub REST API lives here.

use std::sync::LazyLock;

use anyhow::{bail, Result};
use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::env::ENV;

// ---------------------------------------------------------------------
// HTTP client
// ---------------------------------------------------------------------
// In Node, `fetch` is a global that's always there. In Rust we create one
// `Client` and reuse it (it keeps connections open, which is faster).
//
// IMPORTANT difference: Node's fetch automatically sends a `User-Agent`
// header. reqwest does NOT, and GitHub rejects requests without one
// (403 Forbidden). So we set it ourselves.
static HTTP: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent("ollama-code-reviewer")
        .build()
        .expect("failed to build HTTP client")
});

// ---------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------

// TS:  interface GitHubFile { filename: string; status: string; patch?: string; ... }
//
// `Deserialize` is what makes `response.json()` work: serde reads the JSON and
// fills in this struct. Extra fields GitHub sends (sha, blob_url, ...) are
// ignored. If a REQUIRED field is missing, you get a clear error instead of a
// silent `undefined` -- TS's `as GitHubFile[]` would have just trusted it.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubFile {
    pub filename: String,
    pub status: String,
    pub patch: Option<String>, // `patch?: string`  ->  Option<String>
    pub additions: u64,
    pub deletions: u64,
}

// TS:  interface PullRequestReviewComment { path: string; line: number; body: string; }
// `Serialize` is the other direction: struct -> JSON for the request body.
// (Declared and exported in the TS version but never called, so we silence the
// "unused" warning, same story as ReviewFinding.)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct PullRequestReviewComment {
    pub path: String,
    pub line: u32,
    pub body: String,
}

// ---------------------------------------------------------------------
// The shared request helper
// ---------------------------------------------------------------------

// TS:  async function githubRequest(endpoint: string, options: RequestInit = {})
//
// Not `pub` -> private to this file, same as a non-exported function in TS.
//
// `Result<Response>` is how Rust does errors. There is no `throw`. A function
// that can fail returns either Ok(value) or Err(error), and the caller MUST
// deal with it. `bail!("msg")` is the equivalent of `throw new Error("msg")`.
//
// Small change from the TS: `endpoint` here is everything AFTER
// `/repos/{owner}/{repo}`, e.g. "/pulls/12/files". That lets us do the
// owner/repo check once, in here, instead of every caller interpolating
// ENV.GITHUB_OWNER (which in TS could silently become the text "undefined").
async fn github_request(
    method: Method,
    endpoint: &str,
    body: Option<Value>, // `Value` = "any JSON", like TS `any`/`unknown` for JSON
) -> Result<Response> {
    // TS:  if (!ENV.GITHUB_TOKEN) throw new Error("GITHUB_TOKEN is not configured.");
    //
    // `let Some(x) = option else { ... }` reads: "if this is Some, give me the
    // inner value as `x`; otherwise run the else block (which must exit)."
    // After this line, `token` is a plain &str -- no more undefined to worry about.
    let Some(token) = ENV.github_token.as_deref() else {
        bail!("GITHUB_TOKEN is not configured.");
    };

    let (Some(owner), Some(repo)) = (ENV.github_owner.as_deref(), ENV.github_repo.as_deref())
    else {
        bail!("GitHub repository configuration is missing.");
    };

    // TS:  `${ENV.GITHUB_API_URL}${endpoint}`
    let url = format!("{}/repos/{owner}/{repo}{endpoint}", ENV.github_api_url);

    // Build the request step by step (builder pattern).
    // `.bearer_auth(token)` adds   Authorization: Bearer <token>
    let mut request = HTTP
        .request(method, url)
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(token)
        .header("X-GitHub-Api-Version", "2022-11-28");

    // `.json(&body)` sets the body AND the `Content-Type: application/json`
    // header, which the TS set on every request by hand.
    if let Some(body) = body {
        request = request.json(&body);
    }

    // `.await` works like in JS. The `?` at the end means:
    // "if this failed, return the error from MY function right now".
    // It's `try { await ... }` + re-throw, in one character.
    let response = request.send().await?;

    // TS:  if (!response.ok) { throw new Error(`GitHub API error: ${response.status} ${await response.text()}`) }
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        bail!("GitHub API error: {} {}", status.as_u16(), text);
    }

    Ok(response)
}

// ---------------------------------------------------------------------
// Public functions (the ones `export`ed in TS)
// ---------------------------------------------------------------------

pub async fn get_pull_request_files(pull_number: u64) -> Result<Vec<GitHubFile>> {
    // `Vec<T>` == `T[]`   (a growable array)
    let response = github_request(
        Method::GET,
        &format!("/pulls/{pull_number}/files?per_page=100"),
        None, // no request body
    )
    .await?;

    // TS:  return response.json() as Promise<GitHubFile[]>;
    // Same idea, but the type is checked against the real JSON as it's parsed.
    Ok(response.json::<Vec<GitHubFile>>().await?)
}

// Exported in the TS version but not called by anything yet.
#[allow(dead_code)]
pub async fn get_pull_request_diff(pull_number: u64) -> Result<String> {
    let files = get_pull_request_files(pull_number).await?;

    // TS:  files.map(file => [ ... ].join("\n")).join("\n\n")
    // Iterators in Rust are the same idea as array methods: .map() then collect.
    let sections: Vec<String> = files
        .iter()
        .map(|file| {
            [
                format!("File: {}", file.filename),
                format!("Status: {}", file.status),
                format!("Additions: {}", file.additions),
                format!("Deletions: {}", file.deletions),
                // TS:  file.patch ?? "Diff unavailable."
                file.patch
                    .clone()
                    .unwrap_or_else(|| "Diff unavailable.".to_string()),
            ]
            .join("\n")
        })
        .collect();

    Ok(sections.join("\n\n"))
}

// Exported in the TS version but not called by anything yet.
#[allow(dead_code)]
pub async fn post_review_comments(
    pull_number: u64,
    comments: &[PullRequestReviewComment], // `&[T]` = "borrowed, read-only view of a list"
) -> Result<()> {
    // `Result<()>` == `Promise<void>` that can fail. `()` is Rust's "void".
    //
    // `json!({...})` lets you write a JSON object literal inline, almost
    // identical to JSON.stringify({ event: "COMMENT", comments }).
    github_request(
        Method::POST,
        &format!("/pulls/{pull_number}/reviews"),
        Some(json!({
            "event": "COMMENT",
            "comments": comments
        })),
    )
    .await?;

    Ok(())
}

pub async fn post_review_summary(pull_number: u64, body: &str) -> Result<()> {
    github_request(
        Method::POST,
        &format!("/pulls/{pull_number}/reviews"),
        Some(json!({
            "event": "COMMENT",
            "body": body
        })),
    )
    .await?;

    Ok(())
}
