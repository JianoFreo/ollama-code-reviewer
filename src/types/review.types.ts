
export interface ReviewFinding {
    file: string;
    line: number;
    severity: "low" | "medium" | "high" | "critical";
    title: string;
    description: string;
    suggestion?: string;
}
