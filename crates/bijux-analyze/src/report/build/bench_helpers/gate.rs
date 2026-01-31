use crate::failure::BenchmarkFailure;

pub fn gate_payload(failures: &[BenchmarkFailure]) -> serde_json::Value {
    let rationale: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason,
                "kind": format!("{:?}", failure.kind),
            })
        })
        .collect();
    serde_json::json!({
        "passes": failures.is_empty(),
        "rationale": rationale
    })
}
