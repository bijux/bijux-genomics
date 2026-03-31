use anyhow::Result;

use super::model::BenchQueryContext;

/// # Errors
/// Returns an error if the governed stage contract hash cannot be computed.
pub fn governed_stage_bench_query_context(stage_id: &str) -> Result<BenchQueryContext> {
    let mut context = BenchQueryContext::new();
    if let Some(contract_hash) = crate::stage_contract_hash(stage_id) {
        context = context.with_stage_contract_hash(contract_hash?);
    }
    Ok(context)
}
