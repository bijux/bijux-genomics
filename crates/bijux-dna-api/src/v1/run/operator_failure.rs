pub use bijux_dna_core::prelude::errors::{CategorizedError, ErrorCategory};
pub use bijux_dna_core::prelude::{ErrorHintV1, HintSeverity};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
/// Stable operator-facing failure envelope surfaced by CLI/API.
///
/// Stability: v1 (stable).
pub struct OperatorFailureV1 {
    pub schema_version: String,
    pub category: ErrorCategory,
    pub message: String,
    pub hints: Vec<ErrorHintV1>,
}

#[must_use]
pub fn classify_operator_failure(err: &anyhow::Error) -> OperatorFailureV1 {
    let category = if let Some(categorized) = err.downcast_ref::<CategorizedError>() {
        categorized.category
    } else {
        err.chain()
            .find_map(|cause| {
                cause.downcast_ref::<CategorizedError>().map(|categorized| categorized.category)
            })
            .unwrap_or(ErrorCategory::InfraError)
    };
    let hints = default_hints_for_category(category);
    OperatorFailureV1 {
        schema_version: "bijux.operator_failure.v1".to_string(),
        category,
        message: err.to_string(),
        hints,
    }
}

fn default_hints_for_category(category: ErrorCategory) -> Vec<ErrorHintV1> {
    let (id, message, action, docs_link_key) = match category {
        ErrorCategory::PlanError => (
            "plan.inputs",
            "input/run configuration is invalid or incomplete",
            "check required args, profile selection, and input file paths",
            Some("docs.plan_inputs".to_string()),
        ),
        ErrorCategory::ContractError => (
            "contract.violation",
            "a contract validation failed",
            "re-run with --dry-run and inspect manifest/contract diagnostics",
            Some("docs.contracts".to_string()),
        ),
        ErrorCategory::ParseError => (
            "observer.parse",
            "tool output could not be parsed by observer contract",
            "inspect stage logs and compare output format against fixture contracts",
            Some("docs.observer_parsers".to_string()),
        ),
        ErrorCategory::ToolError => (
            "tool.exit",
            "a tool invocation failed",
            "inspect tool stderr/stdout artifacts and adjust tool params or resources",
            Some("docs.tool_failures".to_string()),
        ),
        ErrorCategory::InfraError => (
            "infra.runtime",
            "runtime/environment failure during execution",
            "verify runner availability, image catalog, and filesystem permissions",
            Some("docs.runtime_failures".to_string()),
        ),
    };
    vec![ErrorHintV1 {
        id: id.to_string(),
        category,
        severity: HintSeverity::High,
        message: message.to_string(),
        suggested_action: action.to_string(),
        docs_link_key,
    }]
}
