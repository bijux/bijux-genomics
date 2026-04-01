use std::collections::BTreeMap;

use bijux_dna_bench_model::BenchmarkObservation;

use super::SummaryGroupKey;

pub(super) fn collect_summary_groups<'a>(
    observations: &'a [BenchmarkObservation],
) -> BTreeMap<SummaryGroupKey, Vec<&'a BenchmarkObservation>> {
    let mut groups: BTreeMap<SummaryGroupKey, Vec<&BenchmarkObservation>> = BTreeMap::new();
    for obs in observations {
        groups
            .entry((
                obs.dataset_id.clone(),
                obs.stage_id.clone(),
                obs.stage_instance_id.clone(),
                obs.lineage_id.clone(),
                obs.tool_id.clone(),
                obs.params_hash.clone(),
            ))
            .or_default()
            .push(obs);
    }
    groups
}
