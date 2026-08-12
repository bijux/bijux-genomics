//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted local benchmark pipeline DAG validation evidence.

use std::collections::{BTreeMap, BTreeSet};

use crate::{model::BenchmarkLocalPipelineDagValidationReport, BenchError};

use super::BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1;

/// Validate one persisted local pipeline DAG validation report.
///
/// # Errors
/// Returns an error when identity, topology, or aggregate evidence drifts.
pub fn validate_benchmark_local_pipeline_dag_validation(
    report: &BenchmarkLocalPipelineDagValidationReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1 {
        return invalid("unsupported local pipeline DAG validation schema");
    }
    for (field, value) in [
        ("config_path", report.config_path.as_str()),
        ("output_path", report.output_path.as_str()),
        ("pipeline_id", report.pipeline_id.as_str()),
        ("domain", report.domain.as_str()),
        ("default_corpus_id", report.default_corpus_id.as_str()),
        ("summary", report.summary.as_str()),
    ] {
        require_text(value, field)?;
    }
    if !matches!(report.domain.as_str(), "fastq" | "bam" | "vcf" | "cross") {
        return invalid("local pipeline DAG validation domain is unsupported");
    }
    if !report.acyclic || !report.valid {
        return invalid("admitted local pipeline DAG validation must be valid and acyclic");
    }
    if report.project_source_count != report.project_sources.len()
        || report.node_count != report.nodes.len()
        || report.edge_count != report.nodes.iter().map(|node| node.depends_on.len()).sum::<usize>()
    {
        return invalid("local pipeline DAG validation aggregates do not close");
    }
    validate_metadata(report)?;
    validate_topology(report)?;
    Ok(())
}

fn validate_topology(report: &BenchmarkLocalPipelineDagValidationReport) -> Result<(), BenchError> {
    let mut node_ids = BTreeSet::new();
    let mut stage_ids = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    let mut output_producers = BTreeMap::new();
    let mut profile_ids = BTreeSet::new();
    let mut node_positions = BTreeMap::new();
    for (position, node_id) in report.topological_order.iter().enumerate() {
        require_text(node_id, "topological_order[]")?;
        if node_positions.insert(node_id.as_str(), position).is_some() {
            return invalid("local pipeline DAG topological order repeats a node");
        }
    }
    if report.topological_order.len() != report.nodes.len() {
        return invalid("local pipeline DAG topological order does not cover every node");
    }

    for node in &report.nodes {
        require_text(&node.node_id, "nodes[].node_id")?;
        require_text(&node.stage_id, "nodes[].stage_id")?;
        require_text(&node.readiness_kind, "nodes[].readiness_kind")?;
        if !matches!(node.readiness_kind.as_str(), "dry_run" | "smoke" | "dry_or_smoke") {
            return invalid("local pipeline DAG node readiness kind is unsupported");
        }
        if !node_ids.insert(node.node_id.as_str()) || !stage_ids.insert(node.stage_id.as_str()) {
            return invalid("local pipeline DAG repeats a node or stage identity");
        }
        if node.dependency_count != node.depends_on.len()
            || node.external_input_count != node.external_inputs.len()
            || node.upstream_input_count != node.upstream_inputs.len()
            || node.output_count != node.outputs.len()
        {
            return invalid("local pipeline DAG node aggregates do not close");
        }
        validate_unique_texts(&node.depends_on, "nodes[].depends_on")?;
        validate_unique_texts(&node.external_inputs, "nodes[].external_inputs")?;
        validate_unique_texts(&node.upstream_inputs, "nodes[].upstream_inputs")?;
        validate_unique_texts(&node.outputs, "nodes[].outputs")?;
        if node.external_inputs.is_empty() && node.upstream_inputs.is_empty() {
            return invalid("local pipeline DAG node has no declared input");
        }
        if node.outputs.is_empty() {
            return invalid("local pipeline DAG node has no declared output");
        }
        for output in &node.outputs {
            if !outputs.insert(output.as_str()) {
                return invalid("local pipeline DAG repeats an output identity");
            }
            output_producers.insert(output.as_str(), node.node_id.as_str());
        }
    }
    if node_ids != node_positions.keys().copied().collect() {
        return invalid("local pipeline DAG node roster and topological order differ");
    }
    for node in &report.nodes {
        let Some(&node_position) = node_positions.get(node.node_id.as_str()) else {
            return invalid("local pipeline DAG topological order omits a node");
        };
        let mut required_dependencies = BTreeSet::new();
        for upstream_input in &node.upstream_inputs {
            let Some(producer) = output_producers.get(upstream_input.as_str()) else {
                return invalid("local pipeline DAG consumes an output with no producer");
            };
            if *producer == node.node_id.as_str() {
                return invalid("local pipeline DAG node consumes its own output");
            }
            required_dependencies.insert(*producer);
        }
        let declared_dependencies =
            node.depends_on.iter().map(String::as_str).collect::<BTreeSet<_>>();
        if declared_dependencies != required_dependencies {
            return invalid("local pipeline DAG dependencies do not match upstream producers");
        }
        for dependency in &node.depends_on {
            let Some(dependency_position) = node_positions.get(dependency.as_str()) else {
                return invalid("local pipeline DAG node depends on an unknown node");
            };
            if dependency_position >= &node_position {
                return invalid("local pipeline DAG dependency violates topological order");
            }
        }
    }
    for profile in &report.validation_profiles {
        require_text(&profile.profile_id, "validation_profiles[].profile_id")?;
        if !profile_ids.insert(profile.profile_id.as_str()) {
            return invalid("local pipeline DAG repeats a validation profile identity");
        }
        if profile.check_count != profile.checks.len() {
            return invalid("local pipeline DAG validation profile aggregate drifted");
        }
        validate_unique_texts(&profile.checks, "validation_profiles[].checks")?;
    }
    Ok(())
}

fn validate_metadata(report: &BenchmarkLocalPipelineDagValidationReport) -> Result<(), BenchError> {
    if let Some(context) = &report.reference_context {
        require_text(&context.species_id, "reference_context.species_id")?;
        require_text(&context.build_id, "reference_context.build_id")?;
        require_text(&context.reference_bundle_id, "reference_context.reference_bundle_id")?;
    }
    if let Some(context) = &report.execution_context {
        require_text(&context.site_profile_id, "execution_context.site_profile_id")?;
        if let Some(config) = &context.default_campaign_config {
            require_text(config, "execution_context.default_campaign_config")?;
        }
        validate_unique_texts(&context.frontend_surfaces, "execution_context.frontend_surfaces")?;
        validate_unique_texts(&context.compute_surfaces, "execution_context.compute_surfaces")?;
    }
    let mut project_ids = BTreeSet::new();
    for source in &report.project_sources {
        require_text(&source.project_id, "project_sources[].project_id")?;
        require_text(&source.metadata_url, "project_sources[].metadata_url")?;
        if source.retries == 0 || !project_ids.insert(source.project_id.as_str()) {
            return invalid("local pipeline DAG project source identity or retry policy drifted");
        }
    }
    if let Some(policy) = &report.ancient_dna_policy {
        for (field, value) in [
            ("ancient_dna_policy.aligner_mode", policy.aligner_mode.as_str()),
            ("ancient_dna_policy.adapter_cleanup_tool", policy.adapter_cleanup_tool.as_str()),
            ("ancient_dna_policy.duplicate_stage_id", policy.duplicate_stage_id.as_str()),
            ("ancient_dna_policy.duplicate_tool", policy.duplicate_tool.as_str()),
            ("ancient_dna_policy.caller_tool", policy.caller_tool.as_str()),
        ] {
            require_text(value, field)?;
        }
    }
    if let Some(assets) = &report.reference_assets {
        for (field, value) in [
            ("reference_assets.directory_name", assets.directory_name.as_str()),
            ("reference_assets.genome_url", assets.genome_url.as_str()),
            ("reference_assets.genome_filename", assets.genome_filename.as_str()),
            ("reference_assets.assembly_report_url", assets.assembly_report_url.as_str()),
            ("reference_assets.assembly_report_filename", assets.assembly_report_filename.as_str()),
        ] {
            require_text(value, field)?;
        }
    }
    if let Some(assets) = &report.variant_assets {
        for (field, value) in [
            ("variant_assets.directory_name", assets.directory_name.as_str()),
            ("variant_assets.source_vcf_url", assets.source_vcf_url.as_str()),
            ("variant_assets.source_vcf_filename", assets.source_vcf_filename.as_str()),
            ("variant_assets.refseq_vcf_filename", assets.refseq_vcf_filename.as_str()),
            ("variant_assets.annotated_vcf_filename", assets.annotated_vcf_filename.as_str()),
            ("variant_assets.snpeff.genome_id", assets.snpeff.genome_id.as_str()),
            ("variant_assets.snpeff.database_url", assets.snpeff.database_url.as_str()),
            ("variant_assets.snpeff.database_filename", assets.snpeff.database_filename.as_str()),
            ("variant_assets.snpeff.core_url", assets.snpeff.core_url.as_str()),
            ("variant_assets.snpeff.core_filename", assets.snpeff.core_filename.as_str()),
        ] {
            require_text(value, field)?;
        }
    }
    Ok(())
}

fn validate_unique_texts(values: &[String], field: &str) -> Result<(), BenchError> {
    let mut seen = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !seen.insert(value.as_str()) {
            return invalid(format!("local pipeline DAG repeats {field}"));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local pipeline DAG validation requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
