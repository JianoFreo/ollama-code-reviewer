
import "dotenv/config";

const GITHUB_API = "https://api.github.com";

const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
const GITHUB_OWNER = process.env.GITHUB_OWNER;
const GITHUB_REPO = process.env.GITHUB_REPO;

interface GitHubFile {
    filename: string;
    status: string;
    patch?: string;
    additions: number;
    deletions: number;
}

interface PullRequestReviewComment {
    path: string;
    line: number;
    body: string;
}

async function githubRequest(endpoint: string, options: RequestInit = {}) {
    if (!GITHUB_TOKEN) {
        throw new Error("GITHUB_TOKEN is not configured.");
    }

    const response = await fetch(`${GITHUB_API}${endpoint}`, {
        ...options,
        headers: {
            Accept: "application/vnd.github+json",
            Authorization: `Bearer ${GITHUB_TOKEN}`,
            "X-GitHub-Api-Version": "2022-11-28",
            "Content-Type": "application/json",
            ...options.headers
        }
    });

    if (!response.ok) {
        throw new Error(
            `GitHub API error: ${response.status} ${await response.text()}`
        );
    }

    return response;
}

export async function getPullRequestFiles(
    pullNumber: number
): Promise<GitHubFile[]> {
    const response = await githubRequest(
        `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/pulls/${pullNumber}/files`
    );

    return response.json() as Promise<GitHubFile[]>;
}

export async function getPullRequestDiff(
    pullNumber: number
): Promise<string> {
    const files = await getPullRequestFiles(pullNumber);

    return files
        .map(file => {
            return `File: ${file.filename}\nStatus: ${file.status}\n${file.patch ?? "Diff unavailable."}`;
        })
        .join("\n\n");
}

export async function postReviewComments(
    pullNumber: number,
    comments: PullRequestReviewComment[]
): Promise<void> {
    await githubRequest(
        `/repos/${GITHUB_OWNER}/${GITHUB_REPO}/pulls/${pullNumber}/reviews`,
        {
            method: "POST",
            body: JSON.stringify({
                event: "COMMENT",
                comments
            })
        }
    );
}
