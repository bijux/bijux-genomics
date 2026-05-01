use std::path::Path;

use anyhow::{anyhow, Result};

use bijux_dna_core::contract::{
    ArtifactKind, ArtifactRole, Cardinality, PortSpec, RuntimeScale, StageFamily,
    StageSemanticKind, ToolRole,
};
use bijux_dna_core::prelude::tooling::{ReadCountChangePolicy, StageBehavior};
use serde::Deserialize;

pub fn declared_file_name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| anyhow!("path has no declared file name: {}", path.display()))
}

pub fn to_cardinality(raw: &str) -> Cardinality {
    if raw.eq_ignore_ascii_case("one") {
        Cardinality::One
    } else {
        Cardinality::Many
    }
}

pub fn to_ports(ports: Vec<DomainPortYaml>) -> Vec<PortSpec> {
    ports
        .into_iter()
        .map(|port| PortSpec {
            artifact_role: artifact_role_from_port(&port),
            name: port.name,
            data_type: port.data_type,
            cardinality: to_cardinality(&port.cardinality),
        })
        .collect()
}

pub fn parse_tool_role(raw: Option<&str>) -> ToolRole {
    match raw {
        Some("diagnostic") => ToolRole::Diagnostic,
        Some("experimental") => ToolRole::Experimental,
        _ => ToolRole::Authoritative,
    }
}

pub fn list_strings(table: &toml::Value, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|arr| {
            arr.iter().filter_map(toml::Value::as_str).map(str::to_string).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn has_index_suffix(stage_id: &str) -> bool {
    Path::new(stage_id)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("index"))
}

pub fn stage_semantic_from_id(stage_id: &str) -> StageSemanticKind {
    if has_index_suffix(stage_id) || stage_id.contains("prepare_reference") {
        StageSemanticKind::Index
    } else if stage_id.contains("qc") || stage_id.contains("stats") || stage_id.contains("summary")
    {
        StageSemanticKind::Qc
    } else if stage_id.contains("report") {
        StageSemanticKind::Report
    } else if stage_id.contains("filter") || stage_id.contains("trim") {
        StageSemanticKind::Filter
    } else if stage_id.contains("annot") || stage_id.contains("haplogroup") {
        StageSemanticKind::Annotate
    } else {
        StageSemanticKind::Transform
    }
}

pub fn stage_family_from_id(stage_id: &str) -> StageFamily {
    if stage_id.starts_with("fastq.") {
        StageFamily::Fastq
    } else if stage_id.starts_with("bam.") {
        StageFamily::Bam
    } else if stage_id.starts_with("vcf.") {
        StageFamily::Vcf
    } else {
        StageFamily::Cross
    }
}

pub fn artifact_kind_from_stage(stage_id: &str) -> ArtifactKind {
    if stage_id.starts_with("fastq.") {
        ArtifactKind::Fastq
    } else if stage_id.starts_with("bam.") {
        ArtifactKind::Bam
    } else if stage_id.starts_with("vcf.") {
        ArtifactKind::Vcf
    } else {
        ArtifactKind::Unknown
    }
}

pub fn output_artifact_kind_from_stage(stage_id: &str) -> ArtifactKind {
    if stage_id.contains("qc") || stage_id.contains("stats") || stage_id.contains("summary") {
        ArtifactKind::Metrics
    } else if has_index_suffix(stage_id) {
        ArtifactKind::Index
    } else {
        artifact_kind_from_stage(stage_id)
    }
}

pub fn stage_scale_from_row(stage: &toml::Value) -> RuntimeScale {
    let mem = stage.get("resource_memory_gb").and_then(toml::Value::as_integer).unwrap_or(4);
    let mins = stage.get("resource_time_minutes").and_then(toml::Value::as_integer).unwrap_or(30);
    if mem >= 24 || mins >= 180 {
        RuntimeScale::Large
    } else if mem >= 12 || mins >= 90 {
        RuntimeScale::Medium
    } else if mem >= 4 || mins >= 30 {
        RuntimeScale::Small
    } else {
        RuntimeScale::Tiny
    }
}

pub fn parse_stage_semver(stage: &toml::Value) -> String {
    stage.get("stage_semver").and_then(toml::Value::as_str).unwrap_or("1.0.0").to_string()
}

pub fn stable_produced_artifacts(stage_id: &str, output_kind: ArtifactKind) -> Vec<String> {
    let base = stage_id.replace('.', "_");
    match output_kind {
        ArtifactKind::Fastq => vec![format!("{base}_fastq_out")],
        ArtifactKind::Bam => vec![format!("{base}_bam_out")],
        ArtifactKind::Vcf => vec![format!("{base}_vcf_out")],
        ArtifactKind::Index => vec![format!("{base}_index_out")],
        ArtifactKind::Metrics => vec![format!("{base}_metrics_out")],
        ArtifactKind::Report => vec![format!("{base}_report_out")],
        ArtifactKind::Unknown => vec![format!("{base}_out")],
    }
}

pub fn artifact_role_from_port(port: &DomainPortYaml) -> ArtifactRole {
    ArtifactRole::from_port_name(&port.name).unwrap_or_else(|| match port.data_type.as_str() {
        "fastq" => ArtifactRole::Reads,
        "bam" => ArtifactRole::Bam,
        "vcf" => ArtifactRole::Variant,
        "json" => ArtifactRole::ReportJson,
        "html" => ArtifactRole::ReportHtml,
        "tsv" => ArtifactRole::SummaryTsv,
        "index" => ArtifactRole::Index,
        "metrics" => ArtifactRole::MetricsJson,
        _ => ArtifactRole::Unknown,
    })
}

#[allow(dead_code)]
pub fn default_behavior(read_count_changes: bool) -> StageBehavior {
    StageBehavior {
        idempotent: true,
        mutates_fastq: false,
        report_only: false,
        read_count_change: ReadCountChangePolicy::from_bool(read_count_changes),
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DomainPortYaml {
    pub name: String,
    pub data_type: String,
    pub cardinality: String,
}
