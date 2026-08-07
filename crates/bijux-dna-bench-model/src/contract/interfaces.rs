//! Owner: bijux-dna-bench-model
//! Semantic validation for benchmark stage-interface contracts.

use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_stage_contract::canonical_artifact_contract_catalog;

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkParameterValue, BenchmarkPipelineParameterConfig, BenchmarkStageCampaignRegistry,
    BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceSpec,
};

use super::{BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1};

/// # Errors
/// Returns an error if the interface catalog shape or invariants are invalid.
pub fn validate_stage_interface_catalog(
    catalog: &BenchmarkStageInterfaceCatalog,
) -> Result<(), BenchError> {
    if catalog.schema_version != BENCHMARK_STAGE_INTERFACES_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "stage interface schema mismatch: {}",
            catalog.schema_version
        )));
    }
    if catalog.interfaces.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "stage interface catalog contains no interfaces".to_string(),
        ));
    }

    let artifact_catalog = canonical_artifact_contract_catalog();
    artifact_catalog.validate().map_err(|error| {
        BenchError::InvalidPolicy(format!(
            "canonical artifact contract catalog is invalid: {error}"
        ))
    })?;

    let mut seen_stages = BTreeSet::new();
    for interface in &catalog.interfaces {
        validate_stage_interface(interface)?;
        for contract_id in &interface.accepted_input_contracts {
            if artifact_catalog.contract(contract_id).is_none() {
                return Err(BenchError::InvalidPolicy(format!(
                    "stage interface {} references unknown canonical artifact contract: {contract_id}",
                    interface.stage_id
                )));
            }
        }
        if !seen_stages.insert(interface.stage_id.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "stage interface stage id is duplicated: {}",
                interface.stage_id
            )));
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if a pipeline parameter configuration is invalid.
pub fn validate_pipeline_parameter_config(
    config: &BenchmarkPipelineParameterConfig,
) -> Result<(), BenchError> {
    if config.schema_version != BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "pipeline parameter schema mismatch: {}",
            config.schema_version
        )));
    }
    require_text(&config.profile, "pipeline parameter profile")?;
    if config.stages.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "pipeline parameter configuration contains no stages".to_string(),
        ));
    }
    for (stage_id, stage) in &config.stages {
        require_text(stage_id, "pipeline parameter stage id")?;
        require_text(&stage.parameter_schema_version, "stage parameter schema version")?;
        require_text(&stage.defaults_provider, "stage defaults provider")?;
        for (key, value) in &stage.overrides {
            require_text(key, "stage parameter override key")?;
            if key == "threads"
                && !matches!(value, BenchmarkParameterValue::Integer(value) if *value > 0)
            {
                return Err(BenchError::InvalidPolicy(format!(
                    "pipeline parameter threads override must be a positive integer: {stage_id}"
                )));
            }
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if interface tool ownership drifts from a defined campaign.
pub fn validate_defined_interface_campaign_alignment(
    catalog: &BenchmarkStageInterfaceCatalog,
    registry: &BenchmarkStageCampaignRegistry,
) -> Result<(), BenchError> {
    let campaigns = registry
        .campaigns
        .iter()
        .map(|campaign| (campaign.stage_id.as_str(), campaign))
        .collect::<BTreeMap<_, _>>();
    for interface in &catalog.interfaces {
        let campaign = campaigns.get(interface.stage_id.as_str()).ok_or_else(|| {
            BenchError::InvalidPolicy(format!(
                "stage interface has no defined campaign: {}",
                interface.stage_id
            ))
        })?;
        if interface.tool_ids != campaign.tool_ids {
            return Err(BenchError::InvalidPolicy(format!(
                "stage interface tool drift for {}: interface={:?}, campaign={:?}",
                interface.stage_id, interface.tool_ids, campaign.tool_ids
            )));
        }
        let declared_metrics = campaign
            .performance_metrics
            .iter()
            .map(String::as_str)
            .chain([
                "cpu_utilization_percent",
                "throughput_input_bytes_per_second",
                "throughput_reads_per_second",
            ])
            .collect::<BTreeSet<_>>();
        if let Some(metric) = interface
            .report_presentation
            .performance_columns
            .iter()
            .map(|column| column.metric.as_str())
            .find(|metric| !declared_metrics.contains(metric))
        {
            return Err(BenchError::InvalidPolicy(format!(
                "stage report presentation references undeclared performance metric for {}: {metric}",
                interface.stage_id
            )));
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if an interface and its parameter configuration disagree.
pub fn validate_interface_parameter_alignment(
    interface: &BenchmarkStageInterfaceSpec,
    config: &BenchmarkPipelineParameterConfig,
) -> Result<(), BenchError> {
    let stage = config.stages.get(&interface.stage_id).ok_or_else(|| {
        BenchError::InvalidPolicy(format!(
            "stage parameter configuration is absent: {}",
            interface.stage_id
        ))
    })?;
    if stage.parameter_schema_version != interface.parameter_schema_version {
        return Err(BenchError::InvalidPolicy(format!(
            "stage parameter schema drift for {}",
            interface.stage_id
        )));
    }
    if stage.defaults_provider != interface.default_provider {
        return Err(BenchError::InvalidPolicy(format!(
            "stage defaults provider drift for {}",
            interface.stage_id
        )));
    }
    let allowed =
        interface.operator_override_keys.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let unsupported = stage
        .overrides
        .keys()
        .map(String::as_str)
        .filter(|key| !allowed.contains(key))
        .collect::<Vec<_>>();
    if !unsupported.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "stage parameter configuration contains unsupported overrides for {}: {:?}",
            interface.stage_id, unsupported
        )));
    }
    Ok(())
}

fn validate_stage_interface(interface: &BenchmarkStageInterfaceSpec) -> Result<(), BenchError> {
    validate_stage_interface_shape(interface)?;
    validate_stage_interface_paths(interface)?;
    validate_stage_interface_tools(interface)?;
    validate_stage_interface_parameter_ownership(interface)?;
    validate_stage_interface_downstream_probe(interface)?;
    validate_report_presentation(interface)
}

fn validate_stage_interface_shape(
    interface: &BenchmarkStageInterfaceSpec,
) -> Result<(), BenchError> {
    require_text(&interface.stage_id, "stage interface stage_id")?;
    require_text(&interface.output_contract, "stage interface output_contract")?;
    require_text(
        &interface.normalized_result_contract,
        "stage interface normalized_result_contract",
    )?;
    require_text(&interface.parameter_schema_version, "stage interface parameter_schema_version")?;
    require_text(&interface.default_provider, "stage interface default_provider")?;
    require_text(&interface.parameter_config, "stage interface parameter_config")?;
    require_text(
        &interface.effective_parameters_artifact,
        "stage interface effective_parameters_artifact",
    )?;
    validate_text_list(
        &interface.accepted_input_contracts,
        "stage interface accepted_input_contracts",
        false,
    )?;
    validate_text_list(&interface.tool_ids, "stage interface tool_ids", false)?;
    validate_text_list(&interface.provenance_sources, "stage interface provenance_sources", false)?;
    validate_text_list(
        &interface.operator_override_keys,
        "stage interface operator_override_keys",
        true,
    )?;
    validate_text_list(
        &interface.input_derived_parameters,
        "stage interface input_derived_parameters",
        true,
    )?;
    validate_text_list(
        &interface.immutable_parameters,
        "stage interface immutable_parameters",
        false,
    )?;
    Ok(())
}

fn validate_stage_interface_paths(
    interface: &BenchmarkStageInterfaceSpec,
) -> Result<(), BenchError> {
    for (field, path) in [
        ("normalized_result_contract", interface.normalized_result_contract.as_str()),
        ("parameter_config", interface.parameter_config.as_str()),
        ("downstream_probe.source", interface.downstream_probe.source.as_str()),
    ] {
        validate_relative_path(path, field)?;
    }
    if let Some(path) = &interface.planner_contract {
        validate_relative_path(path, "planner_contract")?;
    }
    for path in &interface.provenance_sources {
        validate_relative_path(path, "provenance_source")?;
    }
    if !is_safe_filename(&interface.effective_parameters_artifact) {
        return Err(BenchError::InvalidPolicy(format!(
            "stage interface effective parameters artifact must be a safe filename: {}",
            interface.stage_id
        )));
    }
    Ok(())
}

fn validate_stage_interface_tools(
    interface: &BenchmarkStageInterfaceSpec,
) -> Result<(), BenchError> {
    let tool_ids = interface.tool_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let adapter_ids = interface.adapters.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if adapter_ids != tool_ids {
        return Err(BenchError::InvalidPolicy(format!(
            "stage interface adapters must cover the exact tool set: {}",
            interface.stage_id
        )));
    }
    for adapter in interface.adapters.values() {
        require_text(adapter, "stage interface adapter")?;
    }
    for (tool_id, projections) in &interface.parameter_projections {
        if !tool_ids.contains(tool_id.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "stage interface parameter projection references unknown tool: {tool_id}"
            )));
        }
        for (parameter, description) in projections {
            if !interface.operator_override_keys.contains(parameter) {
                return Err(BenchError::InvalidPolicy(format!(
                    "stage interface parameter projection references unowned parameter: {parameter}"
                )));
            }
            require_text(description, "stage interface parameter projection")?;
        }
    }
    Ok(())
}

fn validate_stage_interface_parameter_ownership(
    interface: &BenchmarkStageInterfaceSpec,
) -> Result<(), BenchError> {
    let override_keys =
        interface.operator_override_keys.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let input_derived =
        interface.input_derived_parameters.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let immutable =
        interface.immutable_parameters.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let overlap = override_keys
        .intersection(&input_derived)
        .chain(override_keys.intersection(&immutable))
        .chain(input_derived.intersection(&immutable))
        .copied()
        .collect::<BTreeSet<_>>();
    if !overlap.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "stage interface parameter ownership overlaps for {}: {:?}",
            interface.stage_id, overlap
        )));
    }
    Ok(())
}

fn validate_stage_interface_downstream_probe(
    interface: &BenchmarkStageInterfaceSpec,
) -> Result<(), BenchError> {
    require_text(
        &interface.downstream_probe.consumer_stage_id,
        "stage interface downstream consumer_stage_id",
    )?;
    require_text(
        &interface.downstream_probe.input_contract,
        "stage interface downstream input_contract",
    )?;
    validate_text_list(
        &interface.downstream_probe.assertions,
        "stage interface downstream assertions",
        false,
    )?;
    if interface.downstream_probe.input_contract != interface.output_contract {
        return Err(BenchError::InvalidPolicy(format!(
            "stage interface downstream input differs from canonical output: {}",
            interface.stage_id
        )));
    }
    Ok(())
}

fn validate_report_presentation(interface: &BenchmarkStageInterfaceSpec) -> Result<(), BenchError> {
    let presentation = &interface.report_presentation;
    let report_extension = std::path::Path::new(&presentation.report_file_stem)
        .extension()
        .and_then(std::ffi::OsStr::to_str);
    if !is_safe_filename(&presentation.report_file_stem)
        || report_extension.is_some_and(|extension| {
            extension.eq_ignore_ascii_case("json") || extension.eq_ignore_ascii_case("md")
        })
    {
        return Err(BenchError::InvalidPolicy(format!(
            "stage report presentation has an unsafe file stem: {}",
            interface.stage_id
        )));
    }
    require_text(&presentation.generated_source, "stage report generated source")?;
    if !presentation
        .generated_source
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':'))
    {
        return Err(BenchError::InvalidPolicy(format!(
            "stage report presentation contains unsafe Markdown text: {}",
            interface.stage_id
        )));
    }
    if presentation.performance_columns.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "stage report presentation has no performance columns: {}",
            interface.stage_id
        )));
    }
    let mut metrics = BTreeSet::new();
    for column in &presentation.performance_columns {
        require_text(&column.metric, "stage report performance metric")?;
        require_text(&column.heading, "stage report performance heading")?;
        if column.heading.contains(['\n', '|']) || !metrics.insert(column.metric.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "stage report presentation has unsafe or duplicate columns: {}",
                interface.stage_id
            )));
        }
    }
    Ok(())
}

fn validate_text_list(values: &[String], field: &str, allow_empty: bool) -> Result<(), BenchError> {
    if values.is_empty() && !allow_empty {
        return Err(BenchError::InvalidPolicy(format!("{field} cannot be empty")));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !seen.insert(value.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "{field} contains duplicate value: {value}"
            )));
        }
    }
    Ok(())
}

fn validate_relative_path(value: &str, field: &str) -> Result<(), BenchError> {
    let path = std::path::Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(BenchError::InvalidPolicy(format!(
            "stage interface {field} must be repository-relative: {value}"
        )));
    }
    Ok(())
}

fn is_safe_filename(value: &str) -> bool {
    let path = std::path::Path::new(value);
    !value.trim().is_empty()
        && path.file_name().and_then(|name| name.to_str()) == Some(value)
        && value != "."
        && value != ".."
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return Err(BenchError::InvalidPolicy(format!("{field} cannot be empty")));
    }
    Ok(())
}
