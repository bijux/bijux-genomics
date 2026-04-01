use std::collections::BTreeMap;

use bijux_dna_bench_model::{SummaryRow, SummaryStratum};

use super::super::summary_scope::SummaryStratumKey;

pub(super) fn build_summary_strata(rows: &mut [SummaryRow]) -> Vec<SummaryStratum> {
    rows.sort_by(|a, b| {
        (
            &a.dataset_id,
            &a.stage_id,
            &a.stage_instance_id,
            &a.lineage_id,
            &a.tool_id,
            &a.params_hash,
        )
            .cmp(&(
                &b.dataset_id,
                &b.stage_id,
                &b.stage_instance_id,
                &b.lineage_id,
                &b.tool_id,
                &b.params_hash,
            ))
    });

    let mut strata_map: BTreeMap<SummaryStratumKey, (usize, usize)> = BTreeMap::new();
    for row in rows.iter() {
        let entry = strata_map
            .entry((
                row.stage_id.clone(),
                row.stage_instance_id.clone(),
                row.lineage_id.clone(),
                row.dataset_class.clone(),
            ))
            .or_insert((0, 0));
        entry.0 += 1;
        if row.low_power {
            entry.1 += 1;
        }
    }

    let mut strata = Vec::new();
    for ((stage_id, stage_instance_id, lineage_id, dataset_class), (row_count, low_power_count)) in
        strata_map
    {
        strata.push(SummaryStratum {
            stage_id,
            stage_instance_id,
            lineage_id,
            dataset_class,
            row_count,
            low_power_count,
        });
    }
    strata
}
