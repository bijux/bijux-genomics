use anyhow::Result;
use bijux_dna_api::v1::api::run::CategorizedError;
use bijux_dna_api::v1::api::run::ErrorCategory;

/// Run the CLI process entrypoint and terminate with the categorized operator exit code.
pub fn run_and_exit(run: impl FnOnce() -> Result<()>) {
    if let Err(err) = run() {
        print_refusal_if_present(&err);
        let failure = bijux_dna_api::v1::api::run::classify_operator_failure(&err);
        eprintln!("operator_failure category={:?} message={}", failure.category, failure.message);
        for hint in &failure.hints {
            eprintln!(
                "hint id={} severity={:?} action={}",
                hint.id, hint.severity, hint.suggested_action
            );
        }
        eprintln!("{err}");
        std::process::exit(exit_code_for_error(&err));
    }
}

fn print_refusal_if_present(err: &anyhow::Error) {
    let Some(refusal) = refusal_payload_from_chain(err) else {
        return;
    };
    eprintln!("refusal: {}", refusal.code);
    eprintln!("reason: {}", refusal.what);
    if let Some(why) = refusal.why {
        eprintln!("why: {why}");
    }
    if let Some(how) = refusal.how {
        eprintln!("how: {how}");
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RefusalPayload {
    code: String,
    what: String,
    why: Option<String>,
    how: Option<String>,
}

fn refusal_payload_from_chain(err: &anyhow::Error) -> Option<RefusalPayload> {
    err.chain().find_map(|cause| refusal_payload_from_message(&cause.to_string()))
}

fn refusal_payload_from_message(msg: &str) -> Option<RefusalPayload> {
    let value = serde_json::from_str::<serde_json::Value>(msg).ok()?;
    let code = value.get("code").and_then(serde_json::Value::as_str)?;
    let what = value.get("what").and_then(serde_json::Value::as_str)?;
    Some(RefusalPayload {
        code: code.to_string(),
        what: what.to_string(),
        why: value.get("why").and_then(serde_json::Value::as_str).map(ToOwned::to_owned),
        how: value.get("how").and_then(serde_json::Value::as_str).map(ToOwned::to_owned),
    })
}

fn exit_code_for_error(err: &anyhow::Error) -> i32 {
    if let Some(category) = error_category_from_chain(err) {
        return match category {
            ErrorCategory::PlanError => 2,
            ErrorCategory::ContractError => 3,
            ErrorCategory::ParseError => 4,
            ErrorCategory::ToolError => 5,
            ErrorCategory::InfraError => 70,
        };
    }
    let msg = err.to_string().to_lowercase();
    if msg.contains("invalid arg") || msg.contains("usage:") {
        2
    } else if msg.contains("parse") {
        4
    } else if msg.contains("tool") && msg.contains("failed") {
        5
    } else if msg.contains("contract") || msg.contains("invariant") {
        3
    } else if msg.contains("invalid") || msg.contains("missing") || msg.contains("not found") {
        3
    } else {
        70
    }
}

fn error_category_from_chain(err: &anyhow::Error) -> Option<ErrorCategory> {
    if let Some(categorized) = err.downcast_ref::<CategorizedError>() {
        return Some(categorized.category);
    }
    for cause in err.chain() {
        if let Some(categorized) = cause.downcast_ref::<CategorizedError>() {
            return Some(categorized.category);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::{exit_code_for_error, refusal_payload_from_chain, RefusalPayload};

    #[test]
    fn fallback_exit_codes_keep_tool_failures_distinct_from_parse_failures() {
        assert_eq!(exit_code_for_error(&anyhow!("tool fastp failed with exit 1")), 5);
        assert_eq!(exit_code_for_error(&anyhow!("parse observer output failed")), 4);
    }

    #[test]
    fn fallback_exit_codes_keep_contract_failures_on_contract_code() {
        assert_eq!(exit_code_for_error(&anyhow!("contract invariant violated")), 3);
    }

    #[test]
    fn refusal_payload_can_be_found_below_context() {
        let err = anyhow!(
            "{}",
            r#"{"code":"policy.refused","what":"blocked","why":"unsafe","how":"retry"}"#
        )
        .context("operator request failed");

        assert_eq!(
            refusal_payload_from_chain(&err),
            Some(RefusalPayload {
                code: "policy.refused".to_string(),
                what: "blocked".to_string(),
                why: Some("unsafe".to_string()),
                how: Some("retry".to_string()),
            })
        );
    }
}
