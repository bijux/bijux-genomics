use crate::inputs::VcfAdapterInputs;
use anyhow::{ensure, Result};
use bijux_dna_bench_model::VcfStageMatrixRow;
use bijux_dna_bench_vcf_evidence::{
    model::expectations::VcfExpectedBenchmarkResultRow, snapshot::VcfStageReadinessBindingKey,
};
use std::collections::{BTreeMap, BTreeSet};

type BindingMatrix<'a> = BTreeMap<&'a str, BTreeMap<&'a str, &'a VcfStageMatrixRow>>;

pub(super) fn admitted_matrix<'a>(
    inputs: &'a VcfAdapterInputs,
    active: &[VcfStageReadinessBindingKey],
    expected: &[VcfExpectedBenchmarkResultRow],
) -> Result<BindingMatrix<'a>> {
    let mut pairs = BindingMatrix::new();
    let mut keys = BTreeSet::new();
    for row in inputs.bindings() {
        ensure!(
            pairs
                .entry(row.stage_id.as_str())
                .or_default()
                .insert(row.tool_id.as_str(), row)
                .is_none(),
            "retained VCF pair has ambiguous corpus/asset authority"
        );
        keys.insert((
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.corpus_id.as_str(),
            row.asset_profile_id.as_str(),
        ));
    }
    let mut active_keys = BTreeSet::new();
    for row in active {
        let key = (
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.corpus_id.as_str(),
            row.asset_profile_id.as_str(),
        );
        ensure!(
            keys.contains(&key) && active_keys.insert(key),
            "retained VCF scope contains a foreign or repeated active binding"
        );
    }
    let mut expected_keys = BTreeSet::new();
    for row in expected {
        let key = (
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.corpus_id.as_str(),
            row.asset_profile_id.as_str(),
        );
        ensure!(
            row.domain == "vcf" && keys.contains(&key) && expected_keys.insert(key),
            "retained VCF scope contains a foreign or repeated expected binding"
        );
    }
    Ok(pairs)
}
