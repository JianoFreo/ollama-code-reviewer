import "dotenv/config";

export const ENV = {
    // AI Provider
    AI_PROVIDER: process.env.AI_PROVIDER || "ollama",
    AI_BASE_URL: process.env.AI_BASE_URL,
    AI_MODEL: process.env.AI_MODEL,
    AI_API_KEY: process.env.AI_API_KEY,

    // GitHub
    GITHUB_API_URL: process.env.GITHUB_API_URL || "https://api.github.com",
    GITHUB_TOKEN: process.env.GITHUB_TOKEN,
    GITHUB_OWNER: process.env.GITHUB_OWNER,
    GITHUB_REPO: process.env.GITHUB_REPO,

    // Application
    NODE_ENV: process.env.NODE_ENV || "development",
    PORT: Number(process.env.PORT) || 3000,
};

