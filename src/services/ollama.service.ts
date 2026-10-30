export async function reviewWithOllama(diff: string) {
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
                    content: "You are a senior software engineer reviewing code changes."
                },
                {
                    role: "user",
                    content: diff
                }
            ]
        })
    });

    const result = await response.json();

    return result.message.content;
}