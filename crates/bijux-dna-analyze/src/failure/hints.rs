use bijux_dna_core::prelude::errors::{ErrorCategory, ErrorHintV1, HintSeverity};
use bijux_dna_core::prelude::RawFailure;

#[must_use]
pub(crate) fn remediation_hints_for_failure(raw: &RawFailure) -> Vec<ErrorHintV1> {
    let msg = raw.reason.to_lowercase();
    let mut hints = Vec::new();
    if msg.contains("adapter") || msg.contains("adapter preset") {
        hints.push(ErrorHintV1 {
            id: "adapter_preset_missing".to_string(),
            category: ErrorCategory::ContractError,
            severity: HintSeverity::Medium,
            message: "Adapter preset missing or invalid".to_string(),
            suggested_action: "Configure a valid adapter preset or supply an adapter file"
                .to_string(),
            docs_link_key: Some("adapters".to_string()),
        });
    }
    if msg.contains("polyg") || msg.contains("poly-g") {
        hints.push(ErrorHintV1 {
            id: "polyg_artifact".to_string(),
            category: ErrorCategory::ContractError,
            severity: HintSeverity::Low,
            message: "Poly-G artifact suspected".to_string(),
            suggested_action: "Enable illumina_twocolor or configure polyG filtering".to_string(),
            docs_link_key: Some("polyg".to_string()),
        });
    }
    if raw.stage == "fastq.screen_taxonomy" || msg.contains("contaminant") {
        hints.push(ErrorHintV1 {
            id: "contamination_screen".to_string(),
            category: ErrorCategory::ContractError,
            severity: HintSeverity::Medium,
            message: "Potential contaminant signal detected".to_string(),
            suggested_action: "Review contaminant screen output and adjust contaminant bank"
                .to_string(),
            docs_link_key: Some("contamination".to_string()),
        });
    }
    if msg.contains("missing outputs") {
        hints.push(ErrorHintV1 {
            id: "missing_outputs".to_string(),
            category: ErrorCategory::ToolError,
            severity: HintSeverity::High,
            message: "Expected outputs missing".to_string(),
            suggested_action: "Check tool output paths, permissions, and working directory"
                .to_string(),
            docs_link_key: Some("outputs".to_string()),
        });
    }
    hints
}
