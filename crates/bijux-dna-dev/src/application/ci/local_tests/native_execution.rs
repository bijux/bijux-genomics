//! Source-sealed native worker controls for the complete local catalog.

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

pub(crate) const NATIVE_CONTROL_NAMES: [&str; 4] = [
    "BIJUX_CI_FULL_WORKER_BUDGET",
    "BIJUX_CI_FULL_SHARD_JOBS",
    "BIJUX_CI_FULL_SHARD_TEST_THREADS",
    "CARGO_BUILD_JOBS",
];

#[derive(Debug, Default)]
pub(crate) struct NativeExecutionControls {
    values: [Option<String>; 4],
}

impl NativeExecutionControls {
    pub(crate) fn read(mut read: impl FnMut(&str) -> Result<Option<String>>) -> Result<Self> {
        let mut values = [const { None }; 4];
        for (name, value) in NATIVE_CONTROL_NAMES.iter().zip(&mut values) {
            *value = read(name)?;
        }
        Ok(Self { values })
    }

    pub(crate) fn admit(
        &self,
        complete_workers: usize,
        configured_compiler_jobs: usize,
        partitions: usize,
    ) -> Result<NativeExecutionPlan> {
        let values = self
            .values
            .iter()
            .zip(NATIVE_CONTROL_NAMES)
            .map(|(value, name)| positive(name, value.as_deref()))
            .collect::<Result<Vec<_>>>()?;
        let worker_units = values[0].unwrap_or(8);
        let case_threads = values[2].unwrap_or(2);
        let command_units = case_threads.max(2);
        let default_jobs = if values[0].is_some() {
            4.min(partitions).min(worker_units / command_units)
        } else {
            4
        };
        let plan = NativeExecutionPlan {
            worker_units,
            process_jobs: values[1].unwrap_or(default_jobs),
            case_threads,
            compiler_jobs: values[3].unwrap_or(configured_compiler_jobs),
            configured_compiler_jobs,
            partitions,
        };
        plan.validate(complete_workers)?;
        Ok(plan)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeExecutionPlan {
    pub worker_units: usize,
    /// Parent ceiling inside the weighted native executor, not an unconditional launch count.
    pub process_jobs: usize,
    /// Requested per-chunk ceiling; the executor may reduce it for exact singleton chunks.
    pub case_threads: usize,
    pub compiler_jobs: usize,
    pub configured_compiler_jobs: usize,
    pub partitions: usize,
}

impl NativeExecutionPlan {
    pub(crate) fn validate(&self, complete_workers: usize) -> Result<()> {
        ensure!(
            self.partitions > 0 && self.worker_units >= 2 && self.worker_units <= complete_workers,
            "native worker allocation exceeds complete capacity or minimum discovery demand"
        );
        ensure!(
            self.process_jobs > 0 && self.process_jobs <= self.partitions && self.case_threads > 0,
            "native command allocation or partition count invalid"
        );
        ensure!(
            self.process_jobs.checked_mul(self.case_threads.max(2)).is_some(),
            "native command worker demand overflow"
        );
        // The fixed sharded backend reserves paired listing demand before each startup and
        // retains each chunk's actual case demand. Parent ceilings do not launch unweighted work.
        ensure!(
            self.case_threads.max(2) <= self.worker_units,
            "native single-command discovery/case allocation exceeds native worker budget"
        );
        ensure!(
            (1..=16).contains(&self.configured_compiler_jobs)
                && self.compiler_jobs > 0
                && self.compiler_jobs <= self.configured_compiler_jobs,
            "compiler jobs exceed configured authority or admitted native allocation"
        );
        super::admit_archive_build_jobs(u64::try_from(self.compiler_jobs)?, self.worker_units)?;
        Ok(())
    }

    pub(crate) fn environment(&self) -> Vec<(String, String)> {
        NATIVE_CONTROL_NAMES
            .into_iter()
            .zip([self.worker_units, self.process_jobs, self.case_threads, self.compiler_jobs])
            .map(|(name, value)| (name.into(), value.to_string()))
            .collect()
    }
}

pub(crate) fn admit_complete_worker_budget(
    hardware: usize,
    control: Option<&str>,
) -> Result<usize> {
    let budget = positive("BIJUX_CI_COMPLETE_WORKER_BUDGET", control)?
        .unwrap_or_else(|| hardware.saturating_sub(1).max(1));
    ensure!(budget < hardware, "complete CPU budget must reserve a foreground hardware worker");
    Ok(budget)
}

fn positive(name: &str, value: Option<&str>) -> Result<Option<usize>> {
    value
        .map(|value| {
            ensure!(
                !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()),
                "{name} must be a positive decimal integer"
            );
            let value = value.parse::<usize>().with_context(|| format!("parse {name}"))?;
            ensure!(value > 0, "{name} must be positive");
            Ok(value)
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controls(values: [Option<&str>; 4]) -> NativeExecutionControls {
        NativeExecutionControls { values: values.map(|value| value.map(str::to_string)) }
    }

    #[test]
    fn complete_capacity_requires_strict_positive_budget_and_foreground_reserve() -> Result<()> {
        assert_eq!(admit_complete_worker_budget(10, None)?, 9);
        assert_eq!(admit_complete_worker_budget(10, Some("6"))?, 6);
        for control in
            ["0", "10", "11", "-1", "+6", " 6", "6 ", "", "99999999999999999999999999999"]
        {
            assert!(admit_complete_worker_budget(10, Some(control)).is_err(), "{control}");
        }
        assert!(admit_complete_worker_budget(1, None).is_err());
        Ok(())
    }

    #[test]
    fn default_and_explicit_capacity_preserve_all_partitions() -> Result<()> {
        let default = NativeExecutionControls::default().admit(9, 8, 5)?;
        assert_eq!(
            (
                default.worker_units,
                default.process_jobs,
                default.case_threads,
                default.compiler_jobs,
                default.partitions
            ),
            (8, 4, 2, 8, 5)
        );
        let reduced = controls([Some("4"), Some("2"), Some("2"), Some("2")]).admit(6, 8, 5)?;
        assert_eq!(
            (
                reduced.worker_units,
                reduced.process_jobs,
                reduced.case_threads,
                reduced.compiler_jobs,
                reduced.partitions
            ),
            (4, 2, 2, 2, 5)
        );
        let derived = controls([Some("4"), None, None, Some("2")]).admit(6, 8, 5)?;
        assert_eq!(reduced, derived);
        assert_eq!(reduced.worker_units + 1 + 1, 6);
        assert!(NativeExecutionControls::default().admit(6, 8, 5).is_err());
        let adaptive = controls([Some("4"), Some("3"), Some("2"), Some("2")]).admit(6, 8, 5)?;
        assert_eq!(
            (adaptive.worker_units, adaptive.process_jobs, adaptive.case_threads),
            (4, 3, 2)
        );
        assert_eq!(adaptive.environment()[1], ("BIJUX_CI_FULL_SHARD_JOBS".into(), "3".into()));
        // A parent ceiling may wait for weighted startup capacity rather than requiring all parents at once.
        assert!(controls([Some("2"), Some("2"), Some("1"), Some("2")]).admit(6, 8, 5).is_ok());
        Ok(())
    }

    #[test]
    fn malformed_zero_overflow_and_inherited_conflicts_are_refused() {
        for index in 0..4 {
            for malformed in
                ["", "0", "-1", "+2", "2.0", " 2", "2 ", "999999999999999999999999999999999999999"]
            {
                let mut values = [Some("4"), Some("2"), Some("2"), Some("2")];
                values[index] = Some(malformed);
                assert!(controls(values).admit(6, 8, 5).is_err(), "{index}: {malformed}");
            }
        }
        for values in [
            [Some("8"), Some("2"), Some("2"), Some("2")],
            [Some("4"), Some("4"), Some("5"), Some("2")],
            [Some("4"), Some("2"), Some("5"), Some("2")],
            [Some("4"), Some("6"), Some("1"), Some("2")],
            [Some("4"), Some("2"), Some("2"), Some("5")],
            [Some("4"), Some("2"), Some("2"), Some("9")],
            [Some("4"), Some("2"), Some("2"), None],
        ] {
            assert!(controls(values).admit(6, 8, 5).is_err(), "{values:?}");
        }
        assert!(controls([Some("4"), Some("2"), Some("1"), Some("2")]).admit(6, 8, 5).is_ok());
        assert!(controls([Some("1"), Some("2"), Some("1"), Some("1")]).admit(6, 8, 5).is_err());
        assert!(controls([Some("4"), Some("2"), Some("2"), Some("2")]).admit(6, 17, 5).is_err());
    }

    #[test]
    fn decoded_plan_revalidates_arithmetic_and_native_forwarding() -> Result<()> {
        let plan = controls([Some("4"), Some("2"), Some("2"), Some("2")]).admit(6, 8, 5)?;
        let decoded: NativeExecutionPlan = serde_json::from_slice(&serde_json::to_vec(&plan)?)?;
        decoded.validate(6)?;
        let forwarded = NativeExecutionControls::read(|name| {
            Ok(decoded
                .environment()
                .into_iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value))
        })?
        .admit(6, 8, 5)?;
        assert_eq!(forwarded, plan);
        let mut invalid = plan;
        invalid.process_jobs = usize::MAX;
        invalid.partitions = usize::MAX;
        invalid.case_threads = usize::MAX;
        assert!(invalid.validate(usize::MAX).is_err());
        invalid.case_threads = 1;
        assert!(invalid.validate(usize::MAX).is_err(), "paired-startup product overflow passed");
        assert!(NativeExecutionControls::read(|_| anyhow::bail!("non-Unicode control")).is_err());
        Ok(())
    }
}
