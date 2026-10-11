//! reference panel fixtures inputs.
use crate::reference_panel::model::DEFAULT_RAW_PANEL_NAME;
use crate::reference_panel::model::GOVERNED_VCF_PREPARE_PANEL_PANEL_ID;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

pub(crate) fn write_governed_prepare_reference_panel_input_vcf(output_path: &Path) -> Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| anyhow!("VCF panel smoke input path has no parent directory"))?;
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))?;
    let payload = "##fileformat=VCFv4.2\n\
##contig=<ID=1,length=248956422>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample1\n\
1\t101\t.\tA\tG\t60\tPASS\tDP=8\tGT\t0/1\n\
1\t105\t.\tC\tT\t42\tPASS\tDP=13\tGT\t0/1\n\
1\t111\t.\tG\tGA\t12\tLOWQUAL\tDP=5\tGT\t0/1\n\
1\t140\t.\tT\tC\t85\tPASS\tDP=31\tGT\t0/1\n";
    bijux_dna_infra::atomic_write_bytes(output_path, payload.as_bytes())?;
    Ok(())
}

pub(crate) fn materialize_governed_prepare_reference_panel_raw_fixture(
    input_root: &Path,
) -> Result<PathBuf> {
    let panel_root = input_root
        .join("panel_store")
        .join(GOVERNED_VCF_PREPARE_PANEL_PANEL_ID)
        .join("local-reference-panel");
    let panel_raw = panel_root.join("raw");
    let panel_normalized = panel_root.join("normalized");
    let panel_derived = panel_root.join("derived");
    bijux_dna_infra::ensure_dir(&panel_raw)
        .with_context(|| format!("create {}", panel_raw.display()))?;
    bijux_dna_infra::ensure_dir(&panel_normalized)
        .with_context(|| format!("create {}", panel_normalized.display()))?;
    bijux_dna_infra::ensure_dir(&panel_derived)
        .with_context(|| format!("create {}", panel_derived.display()))?;
    let raw_panel_path = panel_raw.join(DEFAULT_RAW_PANEL_NAME);
    let payload = "##fileformat=VCFv4.2\n\
##contig=<ID=1,length=248956422>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample1\n\
1\t140\t.\tT\tC\t85\tPASS\tDP=31\tGT\t0/1\n\
1\t101\t.\tA\tG\t60\tPASS\tDP=8\tGT\t0/1\n\
1\t105\t.\tC\tT\t42\tPASS\tDP=13\tGT\t0/1\n\
1\t111\t.\tG\tGA\t12\tLOWQUAL\tDP=5\tGT\t0/1\n\
1\t105\t.\tC\tT\t42\tPASS\tDP=13\tGT\t0/1\n";
    bijux_dna_infra::atomic_write_bytes(&raw_panel_path, payload.as_bytes())?;
    Ok(raw_panel_path)
}
