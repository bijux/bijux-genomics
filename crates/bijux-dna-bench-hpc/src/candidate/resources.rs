use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::BenchmarkLocalHpcAssetStagingInput;
pub(super) fn parse_time_limit_minutes(time_limit: &str) -> Result<u32> {
    let mut parts = time_limit.split(':');
    let hours = parts
        .next()
        .ok_or_else(|| anyhow!("time limit `{time_limit}` is missing hours"))?
        .parse::<u32>()
        .with_context(|| format!("parse hours from `{time_limit}`"))?;
    let minutes = parts
        .next()
        .ok_or_else(|| anyhow!("time limit `{time_limit}` is missing minutes"))?
        .parse::<u32>()
        .with_context(|| format!("parse minutes from `{time_limit}`"))?;
    let seconds = parts
        .next()
        .ok_or_else(|| anyhow!("time limit `{time_limit}` is missing seconds"))?
        .parse::<u32>()
        .with_context(|| format!("parse seconds from `{time_limit}`"))?;
    if parts.next().is_some() {
        return Err(anyhow!("time limit `{time_limit}` has too many segments"));
    }
    if seconds != 0 {
        return Err(anyhow!(
            "time limit `{time_limit}` must use whole minutes for the governed candidate ceiling"
        ));
    }
    if minutes >= 60 {
        return Err(anyhow!("time limit `{time_limit}` has a minute component outside 0..60"));
    }
    hours.checked_mul(60).and_then(|value| value.checked_add(minutes)).with_context(|| {
        format!("time limit `{time_limit}` exceeds the governed minute representation")
    })
}
/// Sum admitted observations without allowing release-mode integer wraparound.
pub(super) fn staged_input_bytes(inputs: &[BenchmarkLocalHpcAssetStagingInput]) -> Result<u64> {
    inputs.iter().try_fold(0_u64, |total, input| {
        total
            .checked_add(input.size_bytes)
            .context("candidate staged input footprint exceeds the byte representation")
    })
}
