use crate::inputs::VcfAdapterInputs;
use anyhow::{ensure, Result};
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use bijux_dna_domain_vcf::{contracts::stage_metrics_contract, VcfDomainStage};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate_expected_rows(
    inputs: &VcfAdapterInputs,
    rows: &[VcfExpectedBenchmarkResultRow],
) -> Result<()> {
    let ready = crate::evidence::serving::collect_vcf_tool_serving_map_rows(inputs)?
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (row.stage_id, row.tool_id))
        .collect::<BTreeSet<_>>();
    let expected = inputs
        .bindings()
        .iter()
        .filter(|row| ready.contains(&(row.stage_id.clone(), row.tool_id.clone())))
        .map(|row| ((&row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id), row))
        .collect::<BTreeMap<_, _>>();
    let catalog =
        inputs.catalog().iter().map(|row| (&row.stage_id, row)).collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for row in rows {
        let key = (&row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id);
        ensure!(seen.insert(key), "VCF report map repeats an expected binding");
        let binding = expected.get(&key).ok_or_else(|| {
            anyhow::anyhow!("VCF report map contains an unknown expected binding")
        })?;
        let stage = VcfDomainStage::try_from(row.stage_id.as_str())?;
        let section = catalog
            .get(&row.stage_id)
            .ok_or_else(|| anyhow::anyhow!("VCF report map is missing catalog stage"))?;
        ensure!(
            row.domain == "vcf"
                && row.expected_outputs == binding.expected_outputs
                && row.report_section == section.benchmark_category,
            "VCF report map expected declaration disagrees with admitted inputs"
        );
        ensure!(
            row.expected_metrics
                .iter()
                .map(String::as_str)
                .eq(stage_metrics_contract(stage).required_metrics.iter().copied()),
            "VCF report map expected metrics disagree with the stage contract"
        );
    }
    ensure!(
        seen == expected.keys().copied().collect(),
        "VCF report map must retain every benchmark-ready binding"
    );
    Ok(())
}
