
import { reviewWithOllama } from "./ollama.service.js";

export async function reviewDiff(diff: string): Promise<string> {
    console.log("Sending code changes to Ollama...");

    const review = await reviewWithOllama(diff);

    return review;
}
