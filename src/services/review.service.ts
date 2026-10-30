import { reviewWithOllama } from "./ollama.service.js";
import { getPullRequestFiles, postReviewSummary } from "./github.service.js";

const SKIP_PATTERNS = [
    /package-lock\.json$/,
    /yarn\.lock$/,
    /pnpm-lock\.yaml$/,
    /\.min\.(js|css)$/,
    /\.(png|jpe?g|gif|svg|ico|pdf|woff2?)$/i,
    /^(dist|build|node_modules)\//
];

// Small local models struggle with huge inputs, so cap each file's patch.
const MAX_PATCH_CHARS = 12000;

export async function reviewDiff(diff: string): Promise<string> {
    console.log("Sending code changes to Ollama...");
    return reviewWithOllama(diff);
}

export async function reviewPullRequest(
    pullNumber: number,
    post = false
): Promise<string> {
    console.log(`Fetching files for PR #${pullNumber}...`);
    const files = await getPullRequestFiles(pullNumber);

    const reviewable = files.filter(
        f => f.patch && !SKIP_PATTERNS.some(re => re.test(f.filename))
    );

    if (reviewable.length === 0) {
        return "No reviewable changes found in this pull request.";
    }

    const sections: string[] = [];

    for (const [i, file] of reviewable.entries()) {
        console.log(`Reviewing ${file.filename} (${i + 1}/${reviewable.length})...`);

        let patch = file.patch!;
        let note = "";
        if (patch.length > MAX_PATCH_CHARS) {
            patch = patch.slice(0, MAX_PATCH_CHARS);
            note = "\n\n_(Diff truncated; only the first part was reviewed.)_";
        }

        const input = `File: ${file.filename}\nStatus: ${file.status}\n\n${patch}`;

        try {
            const review = await reviewWithOllama(input);
            sections.push(`### \`${file.filename}\`\n\n${review.trim()}${note}`);
        } catch (error) {
            sections.push(`### \`${file.filename}\`\n\nReview failed: ${error}`);
        }
    }

    const result = `## Automated Code Review\n\n${sections.join("\n\n---\n\n")}`;

    if (post) {
        console.log("Posting review to GitHub...");
        await postReviewSummary(pullNumber, result);
        console.log("Review posted.");
    }

    return result;
}
