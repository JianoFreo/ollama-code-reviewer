
import { CODE_REVIEW_PROMPT } from "../prompts/code-review.prompt.js";

interface OllamaResponse {
    message: {
        role: string;
        content: string;
    };
}

export async function reviewWithOllama(diff: string): Promise<string> {
    const response = await fetch("http://localhost:11434/api/chat", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            model: "qwen2.5-coder:7b",
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
        throw new Error(`Ollama API error: ${response.status}`);
    }

    const result = await response.json() as OllamaResponse;

    return result.message.content;
}
