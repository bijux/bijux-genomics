//! Prepare governed VCF inputs and input observations.
use crate::filter::fixtures::inputs::write_governed_filter_input_vcf;
use crate::filter::model::DEFAULT_INPUT_VCF_NAME;
use crate::filter::observations::variants::parse_vcf_record_count;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct PreparedInputs {
    pub(crate) input_vcf: PathBuf,
    pub(crate) stage_root: PathBuf,
    pub(crate) input_variants: u64,
}

pub(crate) fn prepare_inputs(output_root: &Path) -> Result<PreparedInputs> {
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    bijux_dna_infra::ensure_dir(&input_root)
        .with_context(|| format!("create {}", input_root.display()))?;
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;

    let input_vcf = input_root.join(DEFAULT_INPUT_VCF_NAME);
    write_governed_filter_input_vcf(&input_vcf)?;
    let input_variants = parse_vcf_record_count(&input_vcf)?;

    Ok(PreparedInputs { input_vcf, stage_root, input_variants })
}
