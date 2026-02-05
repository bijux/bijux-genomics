use std::path::Path;

use bijux_core::{Cardinality, PortSpec};
use bijux_runtime::manifests::load_manifests;

fn domain_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain")
}

fn stage_port_matches(ports: &[PortSpec], data_type: &str, cardinality: Cardinality) -> bool {
    ports.iter().any(|port| {
        port.data_type == data_type
            && matches!(
                (port.cardinality, cardinality),
                (Cardinality::One, Cardinality::One) | (Cardinality::Many, Cardinality::Many)
            )
    })
}

#[test]
fn trim_outputs_are_compatible_with_merge_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let trim = registry
        .stages()
        .get("fastq.trim")
        .ok_or("missing fastq.trim")?;
    let merge = registry
        .stages()
        .get("fastq.merge")
        .ok_or("missing fastq.merge")?;
    assert!(
        stage_port_matches(&trim.outputs, "fastq", Cardinality::Many)
            && stage_port_matches(&merge.inputs, "fastq", Cardinality::Many),
        "trim outputs must satisfy merge input type"
    );
    Ok(())
}

#[test]
fn filter_outputs_are_compatible_with_stats_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let filter = registry
        .stages()
        .get("fastq.filter")
        .ok_or("missing fastq.filter")?;
    let stats = registry
        .stages()
        .get("fastq.stats_neutral")
        .ok_or("missing fastq.stats_neutral")?;
    assert!(
        stage_port_matches(&filter.outputs, "fastq", Cardinality::Many)
            && stage_port_matches(&stats.inputs, "fastq", Cardinality::Many),
        "filter outputs must satisfy stats input type"
    );
    Ok(())
}

#[test]
fn validate_trim_filter_chain_is_type_safe() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let validate = registry
        .stages()
        .get("fastq.validate_pre")
        .ok_or("missing fastq.validate_pre")?;
    let trim = registry
        .stages()
        .get("fastq.trim")
        .ok_or("missing fastq.trim")?;
    let filter = registry
        .stages()
        .get("fastq.filter")
        .ok_or("missing fastq.filter")?;

    assert!(
        stage_port_matches(&validate.inputs, "fastq", Cardinality::Many)
            && stage_port_matches(&trim.inputs, "fastq", Cardinality::Many),
        "validate inputs must match trim inputs"
    );
    assert!(
        stage_port_matches(&trim.outputs, "fastq", Cardinality::Many)
            && stage_port_matches(&filter.inputs, "fastq", Cardinality::Many),
        "trim outputs must match filter inputs"
    );
    Ok(())
}

#[test]
fn trim_outputs_are_compatible_with_correct_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let trim = registry
        .stages()
        .get("fastq.trim")
        .ok_or("missing fastq.trim")?;
    let correct = registry
        .stages()
        .get("fastq.correct")
        .ok_or("missing fastq.correct")?;
    assert!(
        stage_port_matches(&trim.outputs, "fastq", Cardinality::Many)
            && stage_port_matches(&correct.inputs, "fastq", Cardinality::Many),
        "trim outputs must satisfy correct input type"
    );
    Ok(())
}

#[test]
fn correct_outputs_are_compatible_with_filter_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let correct = registry
        .stages()
        .get("fastq.correct")
        .ok_or("missing fastq.correct")?;
    let filter = registry
        .stages()
        .get("fastq.filter")
        .ok_or("missing fastq.filter")?;
    assert!(
        stage_port_matches(&correct.outputs, "fastq", Cardinality::Many)
            && stage_port_matches(&filter.inputs, "fastq", Cardinality::Many),
        "correct outputs must satisfy filter input type"
    );
    Ok(())
}

#[test]
fn preprocess_outputs_are_compatible_with_qc_post_inputs() -> Result<(), Box<dyn std::error::Error>>
{
    let registry = load_manifests(&domain_root())?;
    let preprocess = registry
        .stages()
        .get("fastq.preprocess")
        .ok_or("missing fastq.preprocess")?;
    let qc_post = registry
        .stages()
        .get("fastq.qc_post")
        .ok_or("missing fastq.qc_post")?;
    assert!(
        stage_port_matches(&preprocess.outputs, "fastq", Cardinality::Many)
            && stage_port_matches(&qc_post.inputs, "fastq", Cardinality::Many),
        "preprocess outputs must satisfy qc_post input type"
    );
    Ok(())
}
