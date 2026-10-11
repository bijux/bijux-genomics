//! Owner: benchmark local campaign planning.
//! Campaign resources owned by the workflow boundary.
use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{CorpusSizeScalePolicy, ResourceTemplate};

pub(crate) fn parse_walltime_seconds(value: &str) -> Option<u64> {
    let mut parts = value.split(':');
    let hours = parts.next()?.parse::<u64>().ok()?;
    let minutes = parts.next()?.parse::<u64>().ok()?;
    let seconds = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() || minutes >= 60 || seconds >= 60 {
        return None;
    }
    Some(
        hours
            .saturating_mul(3600)
            .saturating_add(minutes.saturating_mul(60))
            .saturating_add(seconds),
    )
}

pub(crate) fn format_walltime_seconds(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub(crate) fn scale_u32(value: u32, percent: u32) -> Result<u32> {
    if percent == 0 {
        return Err(anyhow!("scale percentage must be > 0"));
    }
    let scaled = (u64::from(value) * u64::from(percent)).div_ceil(100);
    Ok(u32::try_from(scaled).unwrap_or(u32::MAX).max(1))
}

pub(crate) fn scale_walltime(value: &str, percent: u32) -> Result<String> {
    if percent == 0 {
        return Err(anyhow!("scale percentage must be > 0"));
    }
    let seconds =
        parse_walltime_seconds(value).ok_or_else(|| anyhow!("invalid walltime `{value}`"))?;
    let scaled = seconds.saturating_mul(u64::from(percent)).div_ceil(100).max(1);
    Ok(format_walltime_seconds(scaled))
}

pub(crate) fn apply_corpus_size_scaling(
    base: &ResourceTemplate,
    policy: Option<&CorpusSizeScalePolicy>,
) -> Result<ResourceTemplate> {
    let Some(policy) = policy else {
        return Ok(base.clone());
    };
    Ok(ResourceTemplate {
        cpus: scale_u32(base.cpus, policy.cpu_percent)?,
        mem_gb: scale_u32(base.mem_gb, policy.mem_percent)?,
        walltime: scale_walltime(&base.walltime, policy.walltime_percent)?,
        scratch_gb: scale_u32(base.scratch_gb, policy.scratch_percent)?,
    })
}
