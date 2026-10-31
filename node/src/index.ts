import { reviewDiff, reviewPullRequest } from "./services/review.service.js";

const sampleDiff = `
diff --git a/src/calculator.ts b/src/calculator.ts
--- a/src/calculator.ts
+++ b/src/calculator.ts
@@ -1,3 +1,3 @@
 export function calculateTotal(items: { price: number }[]) {
-    return items.reduce((total, item) => total + item.price, 0);
+    return items.reduce((total, item) => total + item.price);
 }
`;

async function main() {
    const args = process.argv.slice(2);
    const post = args.includes("--post");
    const prArg = args.find(a => !a.startsWith("--"));

    try {
        console.log("Starting Ollama Code Reviewer...\n");

        let result: string;
        if (prArg) {
            const pullNumber = Number(prArg);
            if (!Number.isInteger(pullNumber) || pullNumber <= 0) {
                throw new Error(`Invalid PR number: ${prArg}`);
            }
            result = await reviewPullRequest(pullNumber, post);
        } else {
            console.log("No PR number given, reviewing the sample diff.\n");
            result = await reviewDiff(sampleDiff);
        }

        console.log("\n=== CODE REVIEW RESULTS ===\n");
        console.log(result);
    } catch (error) {
        console.error("Code review failed:", error);
        process.exit(1);
    }
}

main();
