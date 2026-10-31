//! ollama.service.ts  ->  ollama_service.rs
//!
//! Sends a diff to the AI model (Ollama's `/api/chat` endpoint) and returns
//! the model's reply text.

use std::sync::LazyLock;

use anyhow::{bail, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

// import { CODE_REVIEW_PROMPT } from "../prompts/code-review.prompt.js";
// `crate::` means "start from the root of this project" (main.rs).
use crate::config::env::ENV;
use crate::prompts::code_review_prompt::CODE_REVIEW_PROMPT;

// TS:  interface OllamaResponse { message: { role: string; content: string } }
// Rust wants each nested object to be its own struct.
#[derive(Deserialize)]
struct OllamaMessage {
    #[allow(dead_code)] // we parse `role` but never read it (same in the TS version)
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

// One shared HTTP client (see github_service.rs for why).
static HTTP: LazyLock<Client> = LazyLock::new(Client::new);

//
// `&str` = a borrowed string ("I just want to look at it, not own it").
// Use `&str` for function inputs, `String` when you need to own/return one.
//
// process.env here. In Rust we go through the shared `ENV` instead, so there's
// only one place that reads the environment.
pub async fn review_with_ollama(diff: &str) -> Result<String> {
    // if (!AI_BASE_URL || !AI_MODEL) throw new Error("AI configuration is missing ...");
    let (Some(base_url), Some(model)) = (ENV.ai_base_url.as_deref(), ENV.ai_model.as_deref())
    else {
        bail!("AI configuration is missing from environment variables.");
    };

    // The JSON body. `json!` is the Rust version of writing an object literal.
    let body = json!({
        "model": model,
        "stream": false,
        "messages": [
            { "role": "system", "content": CODE_REVIEW_PROMPT },
            { "role": "user",   "content": diff }
        ]
    });

    // `.json(&body)` also sets `Content-Type: application/json` for us.
    let mut request = HTTP.post(format!("{base_url}/api/chat")).json(&body);

    // ...(AI_API_KEY && { Authorization: `Bearer ${AI_API_KEY}` })
    // Only attach the header when a key was actually provided.
    if let Some(api_key) = ENV.ai_api_key.as_deref() {
        request = request.bearer_auth(api_key);
    }

    let response = request.send().await?;

    //  if (!response.ok) throw new Error(`AI API error: ${response.status}`);
    if !response.status().is_success() {
        bail!("AI API error: {}", response.status().as_u16());
    }

    //  const result = await response.json() as OllamaResponse;
    // The type annotation on `result` tells serde what shape to expect.
    let result: OllamaResponse = response.json().await?;

    // TS:  return result.message.content;
    Ok(result.message.content)
}
