use std::path::Path;

use bijux_core::{load_manifests, Cardinality, PortSpec};

fn domain_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain")
}

fn stage_port_matches(ports: &[PortSpec], data_type: &str, cardinality: &Cardinality) -> bool {
    ports.iter().any(|port| {
        port.data_type == data_type
            && matches!(
                (&port.cardinality, cardinality),
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
        stage_port_matches(&trim.outputs, "fastq", &Cardinality::Many)
            && stage_port_matches(&merge.inputs, "fastq", &Cardinality::Many),
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
        .get("fastq.stats")
        .ok_or("missing fastq.stats")?;
    assert!(
        stage_port_matches(&filter.outputs, "fastq", &Cardinality::Many)
            && stage_port_matches(&stats.inputs, "fastq", &Cardinality::Many),
        "filter outputs must satisfy stats input type"
    );
    Ok(())
}

#[test]
fn validate_trim_filter_chain_is_type_safe() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_manifests(&domain_root())?;
    let validate = registry
        .stages()
        .get("fastq.validate")
        .ok_or("missing fastq.validate")?;
    let trim = registry
        .stages()
        .get("fastq.trim")
        .ok_or("missing fastq.trim")?;
    let filter = registry
        .stages()
        .get("fastq.filter")
        .ok_or("missing fastq.filter")?;

    assert!(
        stage_port_matches(&validate.inputs, "fastq", &Cardinality::Many)
            && stage_port_matches(&trim.inputs, "fastq", &Cardinality::Many),
        "validate inputs must match trim inputs"
    );
    assert!(
        stage_port_matches(&trim.outputs, "fastq", &Cardinality::Many)
            && stage_port_matches(&filter.inputs, "fastq", &Cardinality::Many),
        "trim outputs must match filter inputs"
    );
    Ok(())
}
