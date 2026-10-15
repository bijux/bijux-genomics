//! Compare complete expected and observed typed records by declared sample identity.
use super::model::{BamCoverageSampleTruth, BamGcBiasSampleTruth};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

pub(super) fn validate_coverage_sample_truths(
    expected: &[BamCoverageSampleTruth],
    actual: &[BamCoverageSampleTruth],
) -> Result<()> {
    validate_sample_truths_by_key(expected, actual, "coverage")
}

pub(super) fn validate_gc_bias_sample_truths(
    expected: &[BamGcBiasSampleTruth],
    actual: &[BamGcBiasSampleTruth],
) -> Result<()> {
    validate_sample_truths_by_key(expected, actual, "GC-bias")
}

fn validate_sample_truths_by_key<T>(expected: &[T], actual: &[T], label: &str) -> Result<()>
where
    T: PartialEq + SampleTruthIdentity,
{
    let expected_by_id = expected
        .iter()
        .map(|sample| (sample.sample_id().to_string(), sample))
        .collect::<BTreeMap<_, _>>();
    let actual_by_id = actual
        .iter()
        .map(|sample| (sample.sample_id().to_string(), sample))
        .collect::<BTreeMap<_, _>>();
    if expected_by_id.len() != actual_by_id.len() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth {label} sample count drifted: expected {}, observed {}",
            expected_by_id.len(),
            actual_by_id.len()
        ));
    }
    for (sample_id, expected_sample) in &expected_by_id {
        let actual_sample = actual_by_id.get(sample_id).ok_or_else(|| {
            anyhow!("observed BAM GC-bias/coverage truth is missing {label} sample `{sample_id}`")
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth {label} sample drifted for `{sample_id}`"
            ));
        }
    }
    Ok(())
}

trait SampleTruthIdentity {
    fn sample_id(&self) -> &str;
}

impl SampleTruthIdentity for BamCoverageSampleTruth {
    fn sample_id(&self) -> &str {
        &self.sample_id
    }
}

impl SampleTruthIdentity for BamGcBiasSampleTruth {
    fn sample_id(&self) -> &str {
        &self.sample_id
    }
}
