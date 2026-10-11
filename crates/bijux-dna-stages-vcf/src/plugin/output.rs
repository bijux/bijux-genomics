use std::collections::BTreeMap;
use std::io::Read;

use anyhow::{Context, Result};
use bijux_dna_core::contract::canonical::parameters_json_canonicalization;
use bijux_dna_core::contract::{
    ContractVersion, ReportSeverity, StageReportContract, StageReportKind,
};
use bijux_dna_core::metrics::MetricsEnvelope;
use bijux_dna_core::prelude::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_domain_vcf::VcfDomainStage;
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1, StagePluginOutputV1, StageReportPartV1};

use crate::stage_specs::vcf_domain_stage_parser_id;

pub(super) fn parse_stage_outputs(
    plan: &StagePlanV1,
    outputs: &[ArtifactRef],
) -> Result<StagePluginOutputV1> {
    let stage = VcfDomainStage::try_from(plan.stage_id.as_str())?;
    let parser_id =
        vcf_domain_stage_parser_id(stage).context("VCF stage has no declared parser")?;
    let artifacts = if outputs.is_empty() { plan.io.outputs.clone() } else { outputs.to_vec() };
    let exact_contract = plan.io.outputs.len() == artifacts.len()
        && artifact_contracts(&plan.io.outputs) == artifact_contracts(&artifacts);
    let materialized_count = artifacts.iter().filter(|artifact| artifact.path.exists()).count();
    let all_materialized = !artifacts.is_empty() && materialized_count == artifacts.len();
    let artifact_facts = artifact_facts(&artifacts);
    let semantic = semantic_observation(parser_id, &artifacts);
    let metrics = build_metrics_envelope(
        plan,
        parser_id,
        materialized_count,
        &artifact_facts,
        &semantic.metrics,
    )?;
    let invariants = vec![
        invariant(
            "vcf_stage_output_contract_exact",
            if exact_contract { InvariantStatusV1::Pass } else { InvariantStatusV1::Fail },
            if exact_contract {
                "observed VCF outputs exactly match declared names and artifact contracts"
                    .to_string()
            } else {
                "observed VCF outputs differ from declared names or artifact contracts".to_string()
            },
        ),
        invariant(
            "vcf_stage_outputs_materialized",
            if all_materialized { InvariantStatusV1::Pass } else { InvariantStatusV1::Fail },
            format!("materialized {materialized_count} of {} VCF stage outputs", artifacts.len()),
        ),
        invariant(
            "vcf_stage_parser_declared",
            InvariantStatusV1::Pass,
            format!("stage output normalizer uses declared parser {parser_id}"),
        ),
        invariant(
            "vcf_stage_output_semantics",
            if semantic.passed { InvariantStatusV1::Pass } else { InvariantStatusV1::Fail },
            semantic.detail,
        ),
    ];
    let verdict = stage_verdict(plan, &invariants, parser_id, materialized_count, artifacts.len());
    let warnings = if all_materialized {
        Vec::new()
    } else {
        vec!["one or more declared VCF stage outputs are not materialized".to_string()]
    };
    let report_parts = vec![StageReportPartV1 {
        name: "vcf_stage_outputs".to_string(),
        file_name: "vcf-stage-outputs.json".to_string(),
        contract: StageReportContract {
            report_id: "vcf.stage_outputs".to_string(),
            kind: StageReportKind::Qc,
            schema_version: "bijux.vcf.stage_outputs.report.v1".to_string(),
            required_fields: vec![
                "stage_id".to_string(),
                "tool_id".to_string(),
                "parser_id".to_string(),
                "artifacts".to_string(),
            ],
            advisory_fields: Vec::new(),
            severity: ReportSeverity::Warning,
        },
        payload: serde_json::json!({
            "stage_id": plan.stage_id,
            "tool_id": plan.tool_id,
            "parser_id": parser_id,
            "artifacts": artifact_facts,
            "semantic_metrics": semantic.metrics,
        }),
    }];

    Ok(StagePluginOutputV1 {
        metrics,
        artifacts,
        operating_mode: plan.operating_mode,
        report_parts,
        warnings,
        invariants,
        verdict: Some(verdict),
        event_hints: Vec::new(),
    })
}

fn artifact_contracts(artifacts: &[ArtifactRef]) -> BTreeMap<String, Option<String>> {
    artifacts
        .iter()
        .map(|artifact| (artifact.name.as_str().to_string(), artifact.artifact_contract_id.clone()))
        .collect()
}

fn artifact_facts(artifacts: &[ArtifactRef]) -> Vec<serde_json::Value> {
    let mut facts = artifacts
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "artifact_id": artifact.name,
                "artifact_contract_id": artifact.artifact_contract_id,
                "materialized": artifact.path.exists(),
                "bytes": artifact.path.metadata().ok().filter(|metadata| metadata.is_file()).map(|metadata| metadata.len()),
                "sha256": artifact_sha256(&artifact.path),
            })
        })
        .collect::<Vec<_>>();
    facts.sort_by(|left, right| left["artifact_id"].as_str().cmp(&right["artifact_id"].as_str()));
    facts
}

fn artifact_sha256(path: &std::path::Path) -> Option<String> {
    if path.is_file() {
        bijux_dna_infra::hash_file_sha256(path).ok()
    } else {
        None
    }
}

fn build_metrics_envelope(
    plan: &StagePlanV1,
    parser_id: &str,
    materialized_count: usize,
    artifact_facts: &[serde_json::Value],
    semantic_metrics: &serde_json::Value,
) -> Result<MetricsEnvelope<serde_json::Value>> {
    let mut input_hashes = plan
        .io
        .inputs
        .iter()
        .filter(|input| input.path.is_file())
        .filter_map(|input| bijux_dna_infra::hash_file_sha256(&input.path).ok())
        .collect::<Vec<_>>();
    input_hashes.sort();
    let parameters_fingerprint = parameters_fingerprint(&plan.params)?;
    Ok(MetricsEnvelope {
        schema_version: "bijux.metrics_envelope.v2".to_string(),
        contract_version: ContractVersion::v1(),
        stage_id: plan.stage_id.as_str().to_string(),
        stage_version: plan.stage_version.0,
        tool_id: plan.tool_id.as_str().to_string(),
        tool_version: plan.tool_version.trim().to_string(),
        image_digest: plan.image.digest.clone().unwrap_or_else(|| plan.image.image.clone()),
        parameters_fingerprint,
        input_fingerprint: input_fingerprint(&input_hashes),
        parameters_json_normalized: parameters_json_canonicalization(&plan.params),
        input_hashes,
        metric_provenance: None,
        metrics: serde_json::json!({
            "parser_id": parser_id,
            "artifact_count": artifact_facts.len(),
            "materialized_artifact_count": materialized_count,
            "artifacts": artifact_facts,
            "semantic_metrics": semantic_metrics,
        }),
    })
}

struct SemanticObservation {
    passed: bool,
    detail: String,
    metrics: serde_json::Value,
}

fn semantic_observation(parser_id: &str, artifacts: &[ArtifactRef]) -> SemanticObservation {
    let result = match parser_id {
        "vcf.parser.call_summary" => first_materialized_file(artifacts)
            .and_then(read_vcf_payload)
            .and_then(|text| summarize_call_payload(&text)),
        "vcf.parser.vcf_output" => first_materialized_file(artifacts)
            .and_then(|path| read_vcf_payload(path).and_then(|text| summarize_vcf_payload(&text))),
        "vcf.parser.stats_report" => first_materialized_file(artifacts).and_then(|path| {
            crate::metrics::parse_vcf_stats(path)
                .map(|metrics| crate::metrics::summarize_vcf_metrics(&metrics))
        }),
        "vcf.parser.qc_report" | "vcf.parser.report_json" => first_materialized_file(artifacts)
            .and_then(|path| {
                std::fs::read(path).map_err(anyhow::Error::from).and_then(|bytes| {
                    serde_json::from_slice::<serde_json::Value>(&bytes).map_err(anyhow::Error::from)
                })
            }),
        other => Err(anyhow::anyhow!("unsupported VCF parser identity {other}")),
    };
    match result {
        Ok(metrics) => SemanticObservation {
            passed: true,
            detail: format!("declared parser {parser_id} accepted the materialized stage output"),
            metrics,
        },
        Err(error) => SemanticObservation {
            passed: false,
            detail: format!("declared parser {parser_id} rejected the stage output: {error:#}"),
            metrics: serde_json::json!({"parser_error": error.to_string()}),
        },
    }
}

fn read_vcf_payload(path: &std::path::Path) -> Result<String> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("gz") => {
            let file = std::fs::File::open(path)?;
            let mut reader = noodles_bgzf::io::Reader::new(file);
            let mut text = String::new();
            reader.read_to_string(&mut text)?;
            Ok(text)
        }
        Some("bcf") => anyhow::bail!("BCF semantic normalization requires a declared BCF parser"),
        _ => std::fs::read_to_string(path).map_err(anyhow::Error::from),
    }
}

fn first_materialized_file(artifacts: &[ArtifactRef]) -> Result<&std::path::Path> {
    artifacts
        .iter()
        .map(|artifact| artifact.path.as_path())
        .find(|path| path.is_file())
        .context("no materialized file is available to the declared VCF parser")
}

fn summarize_vcf_payload(text: &str) -> Result<serde_json::Value> {
    let has_fileformat = text.lines().any(|line| line.starts_with("##fileformat=VCF"));
    let has_columns = text.lines().any(|line| line.starts_with("#CHROM\tPOS\tID\tREF\tALT"));
    if !has_fileformat || !has_columns {
        anyhow::bail!("VCF payload lacks canonical fileformat or column headers");
    }
    let record_count = text
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| line.split('\t').count() >= 8)
        .count();
    Ok(serde_json::json!({
        "canonical_headers": true,
        "record_count": record_count,
    }))
}

fn summarize_call_payload(text: &str) -> Result<serde_json::Value> {
    let summary = summarize_vcf_payload(text)?;
    let mut snps = 0usize;
    let mut indels = 0usize;
    for fields in text
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .filter(|fields| fields.len() >= 8)
    {
        let reference = fields[3];
        let alternate = fields[4];
        if reference.len() == 1 && alternate.len() == 1 && !alternate.contains(',') {
            snps += 1;
        } else {
            indels += 1;
        }
    }
    Ok(serde_json::json!({
        "record_count": summary["record_count"],
        "snps": snps,
        "indels": indels,
    }))
}

fn invariant(id: &str, status: InvariantStatusV1, message: String) -> InvariantResultV1 {
    InvariantResultV1 { id: id.to_string(), status, message, remediation: None }
}

fn stage_verdict(
    plan: &StagePlanV1,
    invariants: &[InvariantResultV1],
    parser_id: &str,
    materialized_count: usize,
    artifact_count: usize,
) -> StageVerdictV1 {
    invariants.iter().fold(
        StageVerdictV1 {
            stage_id: plan.stage_id.as_str().to_string(),
            verdict: InvariantStatusV1::Pass,
            reasons: Vec::new(),
            key_metrics: serde_json::json!({
                "parser_id": parser_id,
                "artifact_count": artifact_count,
                "materialized_artifact_count": materialized_count,
            }),
        },
        |mut verdict, invariant| {
            verdict.verdict = std::cmp::max(verdict.verdict, invariant.status.clone());
            verdict.reasons.push(invariant.message.clone());
            verdict
        },
    )
}
