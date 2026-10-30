
import { reviewDiff } from "./services/review.service.js";

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
    try {
        console.log("Starting Ollama Code Reviewer...\n");

        const result = await reviewDiff(sampleDiff);

        console.log("=== CODE REVIEW RESULTS ===\n");
        console.log(result);
    } catch (error) {
        console.error("Code review failed:", error);
    }
}

main();
