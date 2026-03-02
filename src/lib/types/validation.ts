export interface ValidationIssue {
    severity: "error" | "warning";
    category: string;
    message: string;
}

export interface ValidationReport {
    valid: boolean;
    issues: ValidationIssue[];
    warnings_count: number;
    errors_count: number;
}

export interface RouteResult {
    validation_report: ValidationReport;
    archived: boolean;
    validation_path: string | null;
    archive_path: string | null;
    override_used: boolean;
}
