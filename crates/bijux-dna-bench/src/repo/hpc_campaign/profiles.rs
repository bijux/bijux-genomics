//! Owner: benchmark campaign repository acquisition.
//! Campaign profiles owned by the repo boundary.
use anyhow::Context;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn write_campaign_profiles(out_dir: &Path) -> Result<Vec<PathBuf>> {
    let resolved_out_dir = out_dir;
    bijux_dna_infra::ensure_dir(resolved_out_dir)
        .with_context(|| format!("create {}", resolved_out_dir.display()))?;

    let lunarc_path = resolved_out_dir.join("lunarc-small.toml");
    let lunarc_local_ready_path = resolved_out_dir.join("lunarc-fastq-bam-local-ready.toml");
    let lunarc_local_ready_vcf_path =
        resolved_out_dir.join("lunarc-fastq-bam-vcf-local-ready.toml");
    let generic_path = resolved_out_dir.join("generic-small.toml");
    let cross_path = resolved_out_dir.join("cross-mini.toml");

    let lunarc = include_str!("../../../bench/campaign_profiles/lunarc.toml");

    let lunarc_local_ready =
        include_str!("../../../bench/campaign_profiles/lunarc_local_ready.toml");

    let lunarc_local_ready_vcf =
        include_str!("../../../bench/campaign_profiles/lunarc_local_ready_vcf.toml");

    let generic = include_str!("../../../bench/campaign_profiles/generic.toml");

    let cross = include_str!("../../../bench/campaign_profiles/cross.toml");

    bijux_dna_infra::atomic_write_bytes(&lunarc_path, lunarc.as_bytes())
        .with_context(|| format!("write {}", lunarc_path.display()))?;
    bijux_dna_infra::atomic_write_bytes(&lunarc_local_ready_path, lunarc_local_ready.as_bytes())
        .with_context(|| format!("write {}", lunarc_local_ready_path.display()))?;
    bijux_dna_infra::atomic_write_bytes(
        &lunarc_local_ready_vcf_path,
        lunarc_local_ready_vcf.as_bytes(),
    )
    .with_context(|| format!("write {}", lunarc_local_ready_vcf_path.display()))?;
    bijux_dna_infra::atomic_write_bytes(&generic_path, generic.as_bytes())
        .with_context(|| format!("write {}", generic_path.display()))?;
    bijux_dna_infra::atomic_write_bytes(&cross_path, cross.as_bytes())
        .with_context(|| format!("write {}", cross_path.display()))?;

    Ok(vec![
        lunarc_path,
        lunarc_local_ready_path,
        lunarc_local_ready_vcf_path,
        generic_path,
        cross_path,
    ])
}
