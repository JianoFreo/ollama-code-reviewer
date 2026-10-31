
import { ENV } from "../config/env.js";

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

async function githubRequest(
    endpoint: string,
    options: RequestInit = {}
) {
    if (!ENV.GITHUB_TOKEN) {
        throw new Error("GITHUB_TOKEN is not configured.");
    }

    if (!ENV.GITHUB_OWNER || !ENV.GITHUB_REPO) {
        throw new Error("GitHub repository configuration is missing.");
    }

    const response = await fetch(`${ENV.GITHUB_API_URL}${endpoint}`, {
        ...options,
        headers: {
            Accept: "application/vnd.github+json",
            Authorization: `Bearer ${ENV.GITHUB_TOKEN}`,
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
        `/repos/${ENV.GITHUB_OWNER}/${ENV.GITHUB_REPO}/pulls/${pullNumber}/files?per_page=100`
    );

    return response.json() as Promise<GitHubFile[]>;
}

export async function getPullRequestDiff(
    pullNumber: number
): Promise<string> {
    const files = await getPullRequestFiles(pullNumber);

    return files
        .map(file => {
            return [
                `File: ${file.filename}`,
                `Status: ${file.status}`,
                `Additions: ${file.additions}`,
                `Deletions: ${file.deletions}`,
                file.patch ?? "Diff unavailable."
            ].join("\n");
        })
        .join("\n\n");
}

export async function postReviewComments(
    pullNumber: number,
    comments: PullRequestReviewComment[]
): Promise<void> {
    await githubRequest(
        `/repos/${ENV.GITHUB_OWNER}/${ENV.GITHUB_REPO}/pulls/${pullNumber}/reviews`,
        {
            method: "POST",
            body: JSON.stringify({
                event: "COMMENT",
                comments
            })
        }
    );
}

export async function postReviewSummary(
    pullNumber: number,
    body: string
): Promise<void> {
    await githubRequest(
        `/repos/${ENV.GITHUB_OWNER}/${ENV.GITHUB_REPO}/pulls/${pullNumber}/reviews`,
        {
            method: "POST",
            body: JSON.stringify({
                event: "COMMENT",
                body
            })
        }
    );
}
