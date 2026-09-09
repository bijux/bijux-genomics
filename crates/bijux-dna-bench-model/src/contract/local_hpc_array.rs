//! Owner: bijux-dna-bench-model
//! Pure admission and envelope construction for local HPC array resources.

use crate::{BenchError, BenchmarkLocalHpcArrayResources};

/// Build the smallest component-wise resource envelope covering every admitted array row.
/// Equal walltimes retain the last row's spelling, matching array report serialization.
///
/// # Errors
/// Rejects empty input, non-positive resources, malformed walltimes and duration overflow.
pub fn build_benchmark_local_hpc_array_resource_envelope<'a>(
    resources: impl IntoIterator<Item = &'a BenchmarkLocalHpcArrayResources>,
) -> Result<BenchmarkLocalHpcArrayResources, BenchError> {
    let mut envelope: Option<(BenchmarkLocalHpcArrayResources, u64)> = None;
    for row in resources {
        validate_resources(row)?;
        let seconds = parse_time(&row.time_limit)?;
        if let Some((current, current_seconds)) = envelope.as_mut() {
            current.cpus_per_task = current.cpus_per_task.max(row.cpus_per_task);
            current.memory_mb = current.memory_mb.max(row.memory_mb);
            current.scratch_gb = current.scratch_gb.max(row.scratch_gb);
            if seconds >= *current_seconds {
                current.time_limit.clone_from(&row.time_limit);
                *current_seconds = seconds;
            }
        } else {
            envelope = Some((row.clone(), seconds));
        }
    }
    envelope.map(|(resources, _)| resources).ok_or_else(|| {
        BenchError::InvalidPolicy(
            "local HPC array cannot compute an empty resource envelope".to_string(),
        )
    })
}

pub(super) fn validate_resources(
    resources: &BenchmarkLocalHpcArrayResources,
) -> Result<(), BenchError> {
    if resources.cpus_per_task == 0
        || resources.memory_mb == 0
        || resources.scratch_gb == 0
        || parse_time(&resources.time_limit)? == 0
    {
        return invalid("local HPC array resources must be positive");
    }
    Ok(())
}

pub(super) fn parse_time(value: &str) -> Result<u64, BenchError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return invalid("local HPC array time_limit must use HH:MM:SS");
    }
    let hours = parts[0].parse::<u64>().map_err(|_| {
        BenchError::InvalidPolicy("local HPC array time_limit has invalid hours".to_string())
    })?;
    let minutes = parts[1].parse::<u64>().map_err(|_| {
        BenchError::InvalidPolicy("local HPC array time_limit has invalid minutes".to_string())
    })?;
    let seconds = parts[2].parse::<u64>().map_err(|_| {
        BenchError::InvalidPolicy("local HPC array time_limit has invalid seconds".to_string())
    })?;
    if minutes >= 60 || seconds >= 60 {
        return invalid("local HPC array time_limit minute or second is out of range");
    }
    hours
        .checked_mul(3600)
        .and_then(|value| value.checked_add(minutes * 60))
        .and_then(|value| value.checked_add(seconds))
        .ok_or_else(|| {
            BenchError::InvalidPolicy(
                "local HPC array time_limit exceeds supported seconds".to_string(),
            )
        })
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resources(time_limit: &str) -> BenchmarkLocalHpcArrayResources {
        BenchmarkLocalHpcArrayResources {
            cpus_per_task: 1,
            memory_mb: 128,
            scratch_gb: 1,
            time_limit: time_limit.to_string(),
        }
    }

    #[test]
    fn envelope_covers_independent_maxima_and_preserves_equal_time_spelling(
    ) -> Result<(), BenchError> {
        let rows = [
            BenchmarkLocalHpcArrayResources { cpus_per_task: 8, ..resources("01:00:00") },
            BenchmarkLocalHpcArrayResources { memory_mb: 4096, ..resources("00:30:00") },
            BenchmarkLocalHpcArrayResources { scratch_gb: 64, ..resources("1:00:00") },
        ];
        let before = rows.clone();
        let envelope = build_benchmark_local_hpc_array_resource_envelope(&rows)?;
        assert_eq!(
            envelope,
            BenchmarkLocalHpcArrayResources {
                cpus_per_task: 8,
                memory_mb: 4096,
                scratch_gb: 64,
                time_limit: "1:00:00".to_string(),
            }
        );
        assert_eq!(rows, before);
        assert_eq!(build_benchmark_local_hpc_array_resource_envelope([&rows[1]])?, rows[1]);
        Ok(())
    }

    #[test]
    fn envelope_rejects_empty_zero_and_invalid_non_maximal_rows() {
        assert!(build_benchmark_local_hpc_array_resource_envelope([]).is_err());
        for time in [
            "00:00:00",
            "1:60:00",
            "1:00:60",
            "1:00",
            "-1:00:00",
            "abc:00:00",
            "18446744073709551615:00:00",
        ] {
            assert!(
                build_benchmark_local_hpc_array_resource_envelope([
                    &resources(time),
                    &resources("100:00:00")
                ])
                .is_err(),
                "accepted {time}"
            );
        }
        for row in [
            BenchmarkLocalHpcArrayResources { cpus_per_task: 0, ..resources("00:01:00") },
            BenchmarkLocalHpcArrayResources { memory_mb: 0, ..resources("00:01:00") },
            BenchmarkLocalHpcArrayResources { scratch_gb: 0, ..resources("00:01:00") },
        ] {
            assert!(build_benchmark_local_hpc_array_resource_envelope([&row]).is_err());
        }
    }

    #[test]
    fn time_admission_handles_the_exact_duration_limit_without_overflow() -> Result<(), BenchError>
    {
        let maximum =
            format!("{}:{:02}:{:02}", u64::MAX / 3600, u64::MAX % 3600 / 60, u64::MAX % 60);
        assert_eq!(parse_time(&maximum)?, u64::MAX);
        let overflow = format!("{}:00:00", u64::MAX / 3600 + 1);
        assert!(parse_time(&overflow).is_err());
        Ok(())
    }
}
