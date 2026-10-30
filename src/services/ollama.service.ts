
import "dotenv/config";
import { CODE_REVIEW_PROMPT } from "../prompts/code-review.prompt.js";

interface OllamaResponse {
    message: {
        role: string;
        content: string;
    };
}

const AI_BASE_URL = process.env.AI_BASE_URL;
const AI_MODEL = process.env.AI_MODEL;
const AI_API_KEY = process.env.AI_API_KEY;

export async function reviewWithOllama(diff: string): Promise<string> {
    if (!AI_BASE_URL || !AI_MODEL) {
        throw new Error("AI configuration is missing from environment variables.");
    }

    const response = await fetch(`${AI_BASE_URL}/api/chat`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            ...(AI_API_KEY && {
                Authorization: `Bearer ${AI_API_KEY}`
            })
        },
        body: JSON.stringify({
            model: AI_MODEL,
            stream: false,
            messages: [
                {
                    role: "system",
                    content: CODE_REVIEW_PROMPT
                },
                {
                    role: "user",
                    content: diff
                }
            ]
        })
    });

    if (!response.ok) {
        throw new Error(`AI API error: ${response.status}`);
    }

    const result = await response.json() as OllamaResponse;

    return result.message.content;
}
