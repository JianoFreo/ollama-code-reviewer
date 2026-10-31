//! index.ts  ->  main.rs
//!
//! The ENTRY POINT. `cargo run` starts executing at `fn main()` below, just
//! like `node dist/index.js` starts at the top of index.js.
//!
//! Project map (your README flowchart, in file form):
//!
//!   main.rs ──> services/review_service.rs ──┬─> services/github_service.rs ──> GitHub PR
//!                                            └─> services/ollama_service.rs ──> Ollama (local model)
//!                                                     ├─ prompts/code_review_prompt.rs
//!                                                     └─ types/review_types.rs

// ---------------------------------------------------------------------
// Module declarations  (this has no equivalent in Node!)
// ---------------------------------------------------------------------
// In Node, a file is part of your app as soon as something imports it.
// In Rust you must first DECLARE every folder from main.rs:
//     `mod config;`  =  "include the folder src/config/ (its mod.rs lists the files)".
// A file that isn't reachable through `mod` is simply not compiled.
mod config;
mod prompts;
mod services;
mod types;

// ---------------------------------------------------------------------
// Imports
// ---------------------------------------------------------------------
//  { reviewDiff, reviewPullRequest } from "./services/review.service.js";
use anyhow::{bail, Result};
use services::review_service::{review_diff, review_pull_request};

// const sampleDiff = `...`;
// Used when you run the program without a PR number, so you can test Ollama
// without touching GitHub. (Raw string: the content is identical to the TS.)
const SAMPLE_DIFF: &str = r#"
diff --git a/src/calculator.ts b/src/calculator.ts
--- a/src/calculator.ts
+++ b/src/calculator.ts
@@ -1,3 +1,3 @@
 export function calculateTotal(items: { price: number }[]) {
-    return items.reduce((total, item) => total + item.price, 0);
+    return items.reduce((total, item) => total + item.price);
 }
"#;

// ---------------------------------------------------------------------
// main
// ---------------------------------------------------------------------
// main();     (top-level, runs an `async function main()`)
//
// Rust's `main` can't be `async` by itself, because futures need a runtime to
// drive them (Node has one built in: the event loop). `#[tokio::main]` is a
// one-line attribute that starts tokio's runtime and runs our async main on it.
#[tokio::main]
async fn main() {
    // try { ... } catch (error) { console.error("Code review failed:", error); process.exit(1); }
    //
    // The actual work is in `run()`, which returns a Result. Here we handle the
    // failure case in one place: print to stderr and exit with code 1.
    if let Err(error) = run().await {
        // `{error:#}` prints the error plus the chain of causes behind it.
        eprintln!("Code review failed: {error:#}");
        std::process::exit(1);
    }
}

// async function main() { ... }   (the body inside the `try`)
async fn run() -> Result<()> {
    // TS:  const args = process.argv.slice(2);
    // Rust's args() already skips node-style noise, but index 0 is the program
    // name, so `.skip(1)` is the equivalent of `.slice(2)` in Node.
    let args: Vec<String> = std::env::args().skip(1).collect();

    // TS:  const post = args.includes("--post");
    let post = args.iter().any(|a| a == "--post");
    // TS:  const prArg = args.find(a => !a.startsWith("--"));
    let pr_arg = args.iter().find(|a| !a.starts_with("--"));

    println!("Starting Ollama Code Reviewer...\n");

    // let result: string;  if (prArg) { ... } else { ... }
    // In Rust `if` is an expression: whichever branch runs, its value is
    // assigned to `result`.
    let result = if let Some(pr_arg) = pr_arg {
        // const pullNumber = Number(prArg);
        //      if (!Number.isInteger(pullNumber) || pullNumber <= 0) throw ...
        // `parse::<u64>()` fails for "abc", "1.5" and "-3", and the guard
        // `n > 0` rejects 0, so one `match` covers all the checks.
        let pull_number = match pr_arg.parse::<u64>() {
            Ok(n) if n > 0 => n,
            _ => bail!("Invalid PR number: {pr_arg}"),
        };
        review_pull_request(pull_number, post).await?
    } else {
        println!("No PR number given, reviewing the sample diff.\n");
        review_diff(SAMPLE_DIFF).await?
    };

    println!("\n=== CODE REVIEW RESULTS ===\n");
    println!("{result}");

    // `Ok(())` = "finished successfully, nothing to return".
    Ok(())
}
