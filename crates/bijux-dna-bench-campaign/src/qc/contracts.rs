#![allow(clippy::expect_used, clippy::too_many_lines)]
use super::prepare_report_qc_sample;
use crate::planning::collect_stage_manifest_fields;
use crate::ports::{CampaignCommand, CampaignExecution, CampaignPaths};
use bijux_dna_bench_corpus::normalized::NormalizedFastqSample as CorpusNormalizedSample;
use std::{
    fs,
    path::{Path, PathBuf},
};
struct TestPaths {
    results_root: PathBuf,
}
impl CampaignPaths for TestPaths {
    fn stage_out_root(&self, corpus_id: &str, stage_id: &str) -> anyhow::Result<PathBuf> {
        Ok(self.results_root.join(corpus_id).join(stage_id).join("cluster-apptainer"))
    }
    fn apptainer_bind_root(&self) -> Option<PathBuf> {
        None
    }
}
struct RejectExecution;
impl CampaignExecution for RejectExecution {
    fn execute(&self, _command: CampaignCommand<'_>) -> anyhow::Result<i32> {
        anyhow::bail!("dry-run must never invoke a process")
    }
}
#[test]
fn report_qc_preparation_writes_governed_manifest_during_dry_run() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let out_root = temp.path().join("results").join("corpus-01").join("fastq.report_qc");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&out_root).expect("out root");
    let workspace = TestPaths { results_root: temp.path().join("remote-results") };
    let corpus_root = temp.path().join("corpus");
    let sample = CorpusNormalizedSample {
        sample_id: "sample_0001".to_string(),
        r1: corpus_root.join("sample_0001_R1.fastq.gz"),
        r2: Some(corpus_root.join("sample_0001_R2.fastq.gz")),
        layout: "pe".to_string(),
    };

    let prepared = prepare_report_qc_sample(
        &super::PreparationContext {
            program: Path::new("/bin/true"),
            repo_root: &repo_root,
            workspace_config: &workspace,
            corpus_id: "corpus-01",
            platform: "cluster-apptainer",
            dry_run: true,
            executor: &RejectExecution,
        },
        &out_root,
        &sample,
    )
    .expect("prepare report qc sample");

    assert_eq!(
        prepared.extra_stage_args,
        vec![
            "--aggregation-engine".to_string(),
            "multiqc".to_string(),
            "--aggregation-scope".to_string(),
            "governed_qc_artifacts".to_string(),
            "--governed-qc-manifest".to_string(),
            out_root
                .join("bench")
                .join("report_qc")
                .join("sample_0001")
                .join("governed_qc_inputs_manifest.json")
                .display()
                .to_string(),
        ]
    );
    assert_eq!(
        prepared.run_extra_fields.get("governed_qc_input_count"),
        Some(&serde_json::Value::Number(serde_json::Number::from(6_u64)))
    );
    let governed_manifest = out_root
        .join("bench")
        .join("report_qc")
        .join("sample_0001")
        .join("governed_qc_inputs_manifest.json");
    let expected_raw_fastqc_dir = temp
        .path()
        .join("remote-results")
        .join("corpus-01")
        .join("fastq.detect_adapters")
        .join("cluster-apptainer")
        .join("bench")
        .join("detect_adapters")
        .join("sample_0001")
        .join("tools")
        .join("fastqc")
        .join("fastqc")
        .display()
        .to_string();
    let payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&governed_manifest).expect("read governed manifest"),
    )
    .expect("parse governed manifest");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.fastq.report_qc.inputs.v1")
    );
    assert_eq!(
        payload.get("qc_inputs").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(6)
    );
    assert_eq!(
        payload.get("raw_fastqc_dir").and_then(serde_json::Value::as_str),
        Some(expected_raw_fastqc_dir.as_str())
    );
}

#[test]
fn report_qc_preparation_reuses_existing_validation_backend_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let out_root = temp.path().join("results").join("corpus-01").join("fastq.report_qc");
    let remote_results = temp.path().join("remote-results");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&out_root).expect("out root");

    let workspace = TestPaths { results_root: temp.path().join("remote-results") };
    let corpus_root = temp.path().join("corpus");
    let sample = CorpusNormalizedSample {
        sample_id: "sample_0001".to_string(),
        r1: corpus_root.join("sample_0001_R1.fastq.gz"),
        r2: Some(corpus_root.join("sample_0001_R2.fastq.gz")),
        layout: "pe".to_string(),
    };

    let fastqc_root = remote_results
        .join("corpus-01")
        .join("fastq.validate_reads")
        .join("cluster-apptainer")
        .join("bench")
        .join("validate_reads")
        .join("sample_0001")
        .join("tools")
        .join("fastqc");
    fs::create_dir_all(&fastqc_root).expect("create fastqc validation root");
    fs::write(fastqc_root.join("validation.json"), "{\n  \"tool_id\": \"fastqc\"\n}\n")
        .expect("write fastqc validation report");
    fs::write(
        fastqc_root.join("validated_reads_manifest.json"),
        "{\n  \"tool_id\": \"fastqc\"\n}\n",
    )
    .expect("write fastqc validated reads manifest");

    let prepared = prepare_report_qc_sample(
        &super::PreparationContext {
            program: Path::new("/bin/true"),
            repo_root: &repo_root,
            workspace_config: &workspace,
            corpus_id: "corpus-01",
            platform: "cluster-apptainer",
            dry_run: true,
            executor: &RejectExecution,
        },
        &out_root,
        &sample,
    )
    .expect("prepare report qc sample");
    assert_eq!(
        prepared.run_extra_fields.get("governed_qc_input_count"),
        Some(&serde_json::Value::Number(serde_json::Number::from(6_u64)))
    );

    let governed_manifest = out_root
        .join("bench")
        .join("report_qc")
        .join("sample_0001")
        .join("governed_qc_inputs_manifest.json");
    let payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&governed_manifest).expect("read governed manifest"),
    )
    .expect("parse governed manifest");
    let qc_inputs =
        payload.get("qc_inputs").and_then(serde_json::Value::as_array).expect("qc inputs array");
    assert!(
            qc_inputs.iter().any(|entry| {
                entry.get("name").and_then(serde_json::Value::as_str)
                    == Some("fastq.validate_reads.tool.fastqc.validation_report")
                    && entry.get("path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                        path.ends_with(
                            "/fastq.validate_reads/cluster-apptainer/bench/validate_reads/sample_0001/tools/fastqc/validation.json",
                        )
                    })
            }),
            "report_qc must reuse an existing fastqc validation report when it is already present"
        );
    assert!(
            qc_inputs.iter().any(|entry| {
                entry.get("name").and_then(serde_json::Value::as_str)
                    == Some("fastq.validate_reads.tool.fastqc.validated_reads_manifest")
                    && entry.get("path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                        path.ends_with(
                            "/fastq.validate_reads/cluster-apptainer/bench/validate_reads/sample_0001/tools/fastqc/validated_reads_manifest.json",
                        )
                    })
            }),
            "report_qc must reuse an existing fastqc lineage manifest when it is already present"
        );
}

#[test]
fn report_qc_preparation_reuses_existing_profile_reads_backend_artifacts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    let out_root = temp.path().join("results").join("corpus-01").join("fastq.report_qc");
    let remote_results = temp.path().join("remote-results");
    fs::create_dir_all(&repo_root).expect("repo root");
    fs::create_dir_all(&out_root).expect("out root");

    let workspace = TestPaths { results_root: temp.path().join("remote-results") };
    let corpus_root = temp.path().join("corpus");
    let sample = CorpusNormalizedSample {
        sample_id: "sample_0001".to_string(),
        r1: corpus_root.join("sample_0001_R1.fastq.gz"),
        r2: Some(corpus_root.join("sample_0001_R2.fastq.gz")),
        layout: "pe".to_string(),
    };

    let seqfu_root = remote_results
        .join("corpus-01")
        .join("fastq.profile_reads")
        .join("cluster-apptainer")
        .join("bench")
        .join("profile_reads")
        .join("sample_0001")
        .join("tools")
        .join("seqfu");
    fs::create_dir_all(&seqfu_root).expect("create seqfu profile root");
    fs::write(seqfu_root.join("qc.json"), "{\n  \"tool_id\": \"seqfu\"\n}\n")
        .expect("write seqfu qc json");

    let prepared = prepare_report_qc_sample(
        &super::PreparationContext {
            program: Path::new("/bin/true"),
            repo_root: &repo_root,
            workspace_config: &workspace,
            corpus_id: "corpus-01",
            platform: "cluster-apptainer",
            dry_run: true,
            executor: &RejectExecution,
        },
        &out_root,
        &sample,
    )
    .expect("prepare report qc sample");
    assert_eq!(
        prepared.run_extra_fields.get("governed_qc_input_count"),
        Some(&serde_json::Value::Number(serde_json::Number::from(6_u64)))
    );

    let governed_manifest = out_root
        .join("bench")
        .join("report_qc")
        .join("sample_0001")
        .join("governed_qc_inputs_manifest.json");
    let payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&governed_manifest).expect("read governed manifest"),
    )
    .expect("parse governed manifest");
    let qc_inputs =
        payload.get("qc_inputs").and_then(serde_json::Value::as_array).expect("qc inputs array");
    assert!(
            qc_inputs.iter().any(|entry| {
                entry.get("name").and_then(serde_json::Value::as_str)
                    == Some("fastq.profile_reads.tool.seqfu.qc_json")
                    && entry.get("path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                        path.ends_with(
                            "/fastq.profile_reads/cluster-apptainer/bench/profile_reads/sample_0001/tools/seqfu/qc.json",
                        )
                    })
            }),
            "report_qc must reuse an existing seqfu profile report when it is already present"
        );
}

#[test]
fn report_qc_manifest_fields_advertise_all_validation_backends() {
    let fields =
        collect_stage_manifest_fields("fastq.report_qc", &[], &[]).expect("report_qc fields");
    let tool_ids = fields
        .get("governed_contributor_tool_ids")
        .and_then(serde_json::Value::as_array)
        .expect("governed contributor tool ids");
    for tool_id in [
        "fastq_scan",
        "fastqc",
        "fastqvalidator",
        "fqtools",
        "seqfu",
        "seqkit",
        "seqkit_stats",
        "seqtk",
    ] {
        assert!(
            tool_ids.iter().any(|entry| entry.as_str() == Some(tool_id)),
            "report_qc contributor metadata must advertise {tool_id}"
        );
    }
}
