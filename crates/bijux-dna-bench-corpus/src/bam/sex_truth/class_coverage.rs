//! Retain every supported sex-inference call class in fixture bundles.
use super::model::BamSexSampleTruth;
use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::metrics::SexConfidenceClass;
use std::path::Path;

pub(super) fn validate_required_call_classes(call_classes: &[String], path: &Path) -> Result<()> {
    let required = ["male", "female", "ambiguous", "insufficient"];
    for call_class in required {
        if !call_classes.iter().any(|value| value == call_class) {
            return Err(anyhow!(
                "BAM sex truth `{}` must cover the `{call_class}` call class",
                path.display()
            ));
        }
    }
    Ok(())
}

pub(super) fn collect_call_classes(sample_truths: &[BamSexSampleTruth]) -> Vec<String> {
    let mut classes = sample_truths
        .iter()
        .map(|sample| sex_call_name(sample.summary.call).to_string())
        .collect::<Vec<_>>();
    classes.sort();
    classes.dedup();
    classes
}

fn sex_call_name(call: SexConfidenceClass) -> &'static str {
    match call {
        SexConfidenceClass::Male => "male",
        SexConfidenceClass::Female => "female",
        SexConfidenceClass::Ambiguous => "ambiguous",
        SexConfidenceClass::Insufficient => "insufficient",
    }
}
