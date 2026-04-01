use anyhow::Result;
use bijux_dna_api::v1::api::run::CategorizedError;
use bijux_dna_api::v1::api::run::ErrorCategory;

/// Run the CLI process entrypoint and terminate with the categorized operator exit code.
pub fn run_and_exit(run: impl FnOnce() -> Result<()>) {
    if let Err(err) = run() {
        print_refusal_if_present(&err);
        let failure = bijux_dna_api::v1::api::run::classify_operator_failure(&err);
        eprintln!(
            "operator_failure category={:?} message={}",
            failure.category, failure.message
        );
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
    let msg = err.to_string();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&msg) else {
        return;
    };
    let code = value.get("code").and_then(serde_json::Value::as_str);
    let what = value.get("what").and_then(serde_json::Value::as_str);
    let why = value.get("why").and_then(serde_json::Value::as_str);
    let how = value.get("how").and_then(serde_json::Value::as_str);
    if code.is_none() || what.is_none() {
        return;
    }
    eprintln!("refusal: {}", code.unwrap_or("unknown"));
    eprintln!("reason: {}", what.unwrap_or("unspecified"));
    if let Some(why) = why {
        eprintln!("why: {why}");
    }
    if let Some(how) = how {
        eprintln!("how: {how}");
    }
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
    } else if msg.contains("invalid") || msg.contains("missing") || msg.contains("not found") {
        3
    } else if msg.contains("tool") && msg.contains("failed") {
        4
    } else if msg.contains("contract") || msg.contains("invariant") {
        5
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
