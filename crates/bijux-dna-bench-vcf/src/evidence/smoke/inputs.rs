use crate::inputs::VcfAdapterInputs;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
/// Explicit declaration authorities for retained bindings, defaults and external delivery.
/// Host composition identifies an available wrapper; it does not claim the wrapper was run.
pub struct VcfSmokeInputs<'a> {
    pub bindings: &'a VcfAdapterInputs,
    pub retained: &'a [VcfActiveStageToolMatrixRow],
    pub stage_defaults: &'a BTreeMap<String, String>,
    pub external_tool_ids: &'a BTreeSet<String>,
    pub host_support: fn(&Path, &str) -> Result<Option<String>>,
}
