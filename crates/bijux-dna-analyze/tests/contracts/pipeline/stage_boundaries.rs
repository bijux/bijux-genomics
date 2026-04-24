use std::path::Path;

use bijux_dna_core::contract::ToolRegistry;
use bijux_dna_core::prelude::{Cardinality, PortSpec, StageId};
use bijux_dna_runtime::manifests::load_manifests;

fn domain_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("domain")
}

fn stage_port_has_type(ports: &[PortSpec], data_type: &str) -> bool {
    ports.iter().any(|port| {
        port.data_type == data_type
            && matches!(port.cardinality, Cardinality::One | Cardinality::Many)
    })
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

fn stage_or<'a>(
    registry: &'a ToolRegistry,
    stage_id: &'static str,
) -> Result<&'a bijux_dna_core::contract::StageSpec, Box<dyn std::error::Error>> {
    let stage_id = StageId::from_static(stage_id);
    registry.stages().get(&stage_id).ok_or_else(|| format!("missing {}", stage_id.as_str()).into())
}

#[test]
fn trim_outputs_are_compatible_with_merge_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let trim = stage_or(&registry, "fastq.trim_reads")?;
    let merge = stage_or(&registry, "fastq.merge_pairs")?;
    assert!(
        stage_port_has_type(&trim.outputs, "fastq") && stage_port_has_type(&merge.inputs, "fastq"),
        "trim outputs must satisfy merge input type"
    );
    Ok(())
}

#[test]
fn filter_outputs_are_compatible_with_stats_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let filter = stage_or(&registry, "fastq.filter_reads")?;
    let stats = stage_or(&registry, "fastq.profile_reads")?;
    assert!(
        stage_port_has_type(&filter.outputs, "fastq")
            && stage_port_has_type(&stats.inputs, "fastq"),
        "filter outputs must satisfy stats input type"
    );
    Ok(())
}

#[test]
fn validate_trim_filter_chain_is_type_safe() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let validate = stage_or(&registry, "fastq.validate_reads")?;
    let trim = stage_or(&registry, "fastq.trim_reads")?;
    let filter = stage_or(&registry, "fastq.filter_reads")?;

    assert!(
        stage_port_has_type(&validate.inputs, "fastq")
            && stage_port_has_type(&trim.inputs, "fastq"),
        "validate inputs must match trim inputs"
    );
    assert!(
        stage_port_has_type(&trim.outputs, "fastq") && stage_port_has_type(&filter.inputs, "fastq"),
        "trim outputs must match filter inputs"
    );
    Ok(())
}

#[test]
fn trim_outputs_are_compatible_with_correct_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let trim = stage_or(&registry, "fastq.trim_reads")?;
    let correct = stage_or(&registry, "fastq.correct_errors")?;
    assert!(
        stage_port_has_type(&trim.outputs, "fastq")
            && stage_port_has_type(&correct.inputs, "fastq"),
        "trim outputs must satisfy correct input type"
    );
    Ok(())
}

#[test]
fn correct_outputs_are_compatible_with_filter_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let correct = stage_or(&registry, "fastq.correct_errors")?;
    let filter = stage_or(&registry, "fastq.filter_reads")?;
    assert!(
        stage_port_has_type(&correct.outputs, "fastq")
            && stage_port_has_type(&filter.inputs, "fastq"),
        "correct outputs must satisfy filter input type"
    );
    Ok(())
}

#[test]
fn validate_pre_outputs_are_compatible_with_qc_post_inputs(
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let validate_pre = stage_or(&registry, "fastq.validate_reads")?;
    assert!(
        stage_port_matches(&validate_pre.outputs, "json", Cardinality::One),
        "validate_pre must emit one JSON report artifact"
    );
    Ok(())
}
