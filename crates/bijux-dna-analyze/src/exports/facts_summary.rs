use bijux_dna_runtime::FactsRowV1;

use crate::model::FactsSummary;

#[must_use]
pub fn summarize_facts(rows: &[FactsRowV1]) -> FactsSummary {
    let stages = rows.len();
    let mut run_ids = std::collections::BTreeSet::new();
    let mut total_runtime_s = 0.0;
    for row in rows {
        run_ids.insert(row.run_id.clone());
        total_runtime_s += row.runtime_s;
    }
    let runs = run_ids.len();
    let avg_runtime_s = if stages == 0 {
        0.0
    } else {
        let denom = f64::from(u32::try_from(stages).unwrap_or(u32::MAX));
        total_runtime_s / denom
    };
    FactsSummary {
        runs,
        stages,
        total_runtime_s,
        avg_runtime_s,
    }
}
