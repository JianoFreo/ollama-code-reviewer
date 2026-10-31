//!
//! The "brain" of the app. It decides WHAT to review (which files, how much of
//! each) and glues the GitHub service and the Ollama service together.
//!
//! Flow for a PR:
//!   GitHub (get changed files) -> filter junk -> Ollama (review each file)
//!   -> combine into one markdown report -> optionally post it back to GitHub

use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::services::github_service::{get_pull_request_files, post_review_summary};
use crate::services::ollama_service::review_with_ollama;

// SKIP_PATTERNS = [ /package-lock\.json$/, ... ];
//
// Regex literals (/.../) don't exist in Rust; we write the pattern as a normal
// string and compile it with `Regex::new`. `r"..."` (raw string) means we
// don't have to double every backslash.
//   /\.(png|jpe?g|...)$/i   ->   (?i) at the start means "case-insensitive"
//
// LazyLock: compile the patterns once, the first time they're needed.
static SKIP_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"package-lock\.json$",
        r"yarn\.lock$",
        r"pnpm-lock\.yaml$",
        r"\.min\.(js|css)$",
        r"(?i)\.(png|jpe?g|gif|svg|ico|pdf|woff2?)$",
        r"^(dist|build|node_modules)/",
    ]
    .iter()
    // `.expect()` = "crash with this message if the pattern is invalid".
    // These are hard-coded, so that can only happen from a typo while editing.
    .map(|pattern| Regex::new(pattern).expect("invalid skip pattern"))
    .collect()
});

// Small local models struggle with huge inputs, so cap each file's patch.
const MAX_PATCH_CHARS: usize = 12000;

// export async function reviewDiff(diff: string): Promise<string>
pub async fn review_diff(diff: &str) -> Result<String> {
    println!("Sending code changes to Ollama...");
    review_with_ollama(diff).await
}

// export async function reviewPullRequest(pullNumber: number, post = false)
//
// Rust has no default parameter values (`post = false`), so the caller
// (main.rs) always passes `post` explicitly.
pub async fn review_pull_request(pull_number: u64, post: bool) -> Result<String> {
    println!("Fetching files for PR #{pull_number}...");
    let files = get_pull_request_files(pull_number).await?;

    //files.filter(f => f.patch && !SKIP_PATTERNS.some(re => re.test(f.filename)))
    //
    // `into_iter()` hands over ownership of each file; `.filter(...)` keeps the
    // ones where the closure returns true; `.collect()` gathers them in a Vec.
    // |f| ... is Rust's arrow function  (f => ...).
    let reviewable: Vec<_> = files
        .into_iter()
        .filter(|f| {
            // `f.patch &&` in JS is "exists and isn't an empty string"
            let has_patch = f.patch.as_deref().is_some_and(|p| !p.is_empty());
            has_patch && !SKIP_PATTERNS.iter().any(|re| re.is_match(&f.filename))
        })
        .collect();

    if reviewable.is_empty() {
        return Ok("No reviewable changes found in this pull request.".to_string());
    }

    // const sections: string[] = [];
    // `mut` = "this variable can be changed". Variables are read-only by default!
    let mut sections: Vec<String> = Vec::new();

    // for (const [i, file] of reviewable.entries())
    for (i, file) in reviewable.iter().enumerate() {
        println!(
            "Reviewing {} ({}/{})...",
            file.filename,
            i + 1,
            reviewable.len()
        );

        // The `!` meant "trust me, it's not undefined"
        // The filter above guarantees it's Some, so unwrap_or("") never actually hits "".
        let patch = file.patch.as_deref().unwrap_or("");

        //if (patch.length > MAX_PATCH_CHARS) { patch = patch.slice(0, MAX_PATCH_CHARS); note = "..." }
        //
        // We count/cut by CHARACTERS (`.chars()`), not bytes. Rust strings are
        // UTF-8, so slicing at a raw byte position could land in the middle of
        // an emoji or accented letter, and Rust would crash instead of
        // silently producing garbage.
        let (patch, note) = if patch.chars().count() > MAX_PATCH_CHARS {
            (
                patch.chars().take(MAX_PATCH_CHARS).collect::<String>(),
                "\n\n_(Diff truncated; only the first part was reviewed.)_",
            )
        } else {
            (patch.to_string(), "")
        };

        let input = format!("File: {}\nStatus: {}\n\n{}", file.filename, file.status, patch);

        //  try { ... } catch (error) { ... }
        //
        // Rust has no try/catch. Instead the function returns a Result and we
        // `match` on it: Ok(...) is the "try" path, Err(...) is the "catch".
        match review_with_ollama(&input).await {
            Ok(review) => {
                sections.push(format!(
                    "### `{}`\n\n{}{}",
                    file.filename,
                    review.trim(),
                    note
                ));
            }
            Err(error) => {
                sections.push(format!(
                    "### `{}`\n\nReview failed: {}",
                    file.filename, error
                ));
            }
        }
    }

    let result = format!(
        "## Automated Code Review\n\n{}",
        sections.join("\n\n---\n\n")
    );

    if post {
        println!("Posting review to GitHub...");
        post_review_summary(pull_number, &result).await?;
        println!("Review posted.");
    }

    Ok(result)
}
