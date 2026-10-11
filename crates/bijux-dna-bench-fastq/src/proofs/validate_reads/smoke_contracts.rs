#![cfg(test)]
use super::load_fastq_validate_reads_smoke_evidence;
use crate::{
    proofs::inputs::{
        observations::{
            CommandArgvObservation, ComparableMetricObservation, ExpectedResultObservation,
        },
        ports::{DeclarationEvidence, ValidationEvidence},
        ProofPaths, ValidationContext,
    },
    readiness::{
        active::FastqActiveStageToolMatrixRow, output_contract::FastqAdapterOutputContractRow,
        parsers::FastqParserCoverageRow, report_map::FastqReportMapRow,
    },
};
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::path::Path;
struct RetainedOnly;
impl DeclarationEvidence for RetainedOnly {
    fn active(&self, _: &Path) -> Result<Vec<FastqActiveStageToolMatrixRow>> {
        bail!("unexpected declaration acquisition")
    }
    fn outputs(&self, _: &Path) -> Result<Vec<FastqAdapterOutputContractRow>> {
        bail!("unexpected declaration acquisition")
    }
    fn parsers(&self, _: &Path) -> Result<Vec<FastqParserCoverageRow>> {
        bail!("unexpected declaration acquisition")
    }
    fn expected(&self, _: &Path) -> Result<Vec<ExpectedResultObservation>> {
        bail!("unexpected declaration acquisition")
    }
    fn report_map(&self, _: &Path) -> Result<Vec<FastqReportMapRow>> {
        bail!("unexpected declaration acquisition")
    }
}
impl ValidationEvidence for RetainedOnly {
    fn validate_command(&self, _: &Path, _: &CommandArgvObservation) -> Result<()> {
        bail!("unexpected command admission")
    }
    fn metrics(&self, _: &Path) -> Result<Vec<ComparableMetricObservation>> {
        bail!("unexpected metric acquisition")
    }
    fn argv(&self, _: &Path) -> Result<Vec<CommandArgvObservation>> {
        bail!("unexpected command acquisition")
    }
    fn materialize(&self, _: &Path, _: &str) -> Result<()> {
        bail!("retained proof must not execute a backend")
    }
}
fn retained(root: &Path) -> Result<(Value, Value)> {
    let stage = bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS;
    let report = json!({"schema_version":"bijux.fastq.validate.report.v1","stage":stage,"stage_id":stage,"tool_id":"bijux","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":"reads.fastq","input_r2":null,"validation_log_r1":"reads.log","validation_log_r2":null,"validated_inputs":1,"validated_reads_r1":2,"validated_reads_r2":null,"validated_pairs":null,"status_r1":0,"status_r2":0,"pair_sync_checked":false,"pair_sync_pass":null,"pair_count_match":null,"failure_class":"none","strict_pass":true,"exit_code":0});
    let manifest = json!({"schema_version":"bijux.fastq.validate.lineage.v1","stage_id":stage,"tool_id":"bijux","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":"reads.fastq","input_r2":null,"validation_report":root.join("validation.json"),"paired_mode":"single_end","validated_stream_ids":["reads_r1"],"pair_sync_checked":false,"pair_sync_pass":null,"validated_pairs":null});
    let smoke = root.join("artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json");
    std::fs::create_dir_all(
        smoke.parent().ok_or_else(|| anyhow::anyhow!("missing smoke parent"))?,
    )?;
    // Two retained cases satisfy the existing cardinality, independently of required cohort labels.
    let case = json!({"sample_id":"toy-se","layout":"single_end","input_read_count_total":2,"input_pair_count":null,"validation_status":"pass","validation_report":"validation.json","validated_reads_manifest":"lineage.json"});
    std::fs::write(smoke, serde_json::to_vec(&json!({"cases":[case.clone(),case]}))?)?;
    Ok((report, manifest))
}
fn observe(
    root: &Path,
    report: &Value,
    manifest: &Value,
) -> Result<super::super::models::FastqValidateReadsSmokeEvidence> {
    std::fs::write(root.join("validation.json"), serde_json::to_vec(report)?)?;
    std::fs::write(root.join("lineage.json"), serde_json::to_vec(manifest)?)?;
    let ports = RetainedOnly;
    load_fastq_validate_reads_smoke_evidence(&ValidationContext {
        repo_root: root,
        declarations: &ports,
        validation: &ports,
        paths: ProofPaths {
            expected_results: "expected.tsv",
            command_argv: "commands.jsonl",
            comparable_metrics: "metrics.tsv",
        },
    })
}
#[test]
fn retained_failure_class_cannot_normalize_a_claimed_success() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (mut report, manifest) = retained(root.path())?;
    let valid = observe(root.path(), &report, &manifest)?;
    assert!(valid.parseable && valid.failure_reason_normalized);
    report["failure_class"] = json!("malformed_record");
    let corrupt = observe(root.path(), &report, &manifest)?;
    assert!(!corrupt.parseable && !corrupt.failure_reason_normalized);
    Ok(())
}
#[test]
fn retained_lineage_must_identify_the_actual_report_and_inputs() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (report, mut manifest) = retained(root.path())?;
    assert!(observe(root.path(), &report, &manifest)?.parseable);
    manifest["validation_report"] = json!("foreign.json");
    assert!(!observe(root.path(), &report, &manifest)?.parseable);
    manifest["validation_report"] = json!("validation.json");
    assert!(observe(root.path(), &report, &manifest)?.parseable);
    manifest["input_r1"] = json!("foreign.fastq");
    assert!(!observe(root.path(), &report, &manifest)?.parseable);
    Ok(())
}

#[test]
fn convincing_command_text_requires_canonical_projection_admission() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ports = RetainedOnly;
    let context = ValidationContext {
        repo_root: root.path(),
        declarations: &ports,
        validation: &ports,
        paths: ProofPaths {
            expected_results: "expected.tsv",
            command_argv: "commands.jsonl",
            comparable_metrics: "metrics.tsv",
        },
    };
    let (report, manifest) = retained(root.path())?;
    let command = CommandArgvObservation {
        stage_id: bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS.to_string(),
        tool_id: "bijux".into(),
        readiness_kind: "smoke".into(),
        argv: vec![
            "echo".into(),
            format!("{report} {manifest} validation.json validated_reads_manifest.json"),
        ],
    };
    assert!(!super::super::contracts::command_is_admitted(&context, &command, "bijux"));
    Ok(())
}
