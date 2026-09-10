//! Owner: bijux-dna-bench-workflows
//! Amplicon stages remove chimeras.
use super::super::{
    build_three_column_table, copy_absolute_file, copy_repo_relative_file, is_nonempty_file, json,
    load_json, path_relative_to_repo, row, stage_root, write_json_file, write_text_file,
    AmpliconTruthBundle, BTreeMap, Path, RemoveChimerasStageArtifacts, RemoveChimerasStageEvidence,
    Result, DEFAULT_AMPLICON_CHIMERA_CONTROL_FASTQ_PATH,
    DEFAULT_AMPLICON_NON_CHIMERIC_REPRESENTATIVES_PATH, DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH,
    REMOVE_CHIMERAS_STAGE_SCHEMA_VERSION,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;
use std::fmt::Write as _;

pub(crate) fn run_remove_chimeras_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<RemoveChimerasStageArtifacts> {
    let source_report_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.remove_chimeras/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_remove_chimeras_stage(repo_root, output_root);
    }
    let mut report: RemoveChimerasStageEvidence = load_json(&source_report_path)?;
    let stage_root = stage_root(output_root, stage::FASTQ_REMOVE_CHIMERAS);
    let non_chimeric_fasta = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.non_chimeric_fasta,
        "non_chimeric.fasta",
    )?;
    let chimeras_tsv =
        copy_repo_relative_file(repo_root, &stage_root, &report.chimeras_tsv, "chimeras.tsv")?;
    let case_report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.case_report_json,
        "chimera-control-se/vsearch/remove_chimeras_report.json",
    )?;
    let chimera_metrics_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.chimera_metrics_json,
        "chimera-control-se/vsearch/chimera_metrics.json",
    )?;
    let chimeras_fasta = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.chimeras_fasta,
        "chimera-control-se/vsearch/chimeras.fasta",
    )?;
    let raw_backend_report = report
        .raw_backend_report
        .as_deref()
        .and_then(|path| repo_root.join(path).is_file().then_some(path))
        .map(|path| {
            copy_repo_relative_file(
                repo_root,
                &stage_root,
                path,
                "chimera-control-se/vsearch/uchime.tsv",
            )
        })
        .transpose()?;
    report.filtered_representative_sequences =
        path_relative_to_repo(repo_root, &non_chimeric_fasta);
    report.non_chimeric_fasta = path_relative_to_repo(repo_root, &non_chimeric_fasta);
    report.chimeras_tsv = path_relative_to_repo(repo_root, &chimeras_tsv);
    report.case_report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.chimera_metrics_json = path_relative_to_repo(repo_root, &chimera_metrics_json);
    report.chimeras_fasta = path_relative_to_repo(repo_root, &chimeras_fasta);
    report.raw_backend_report =
        raw_backend_report.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let mut row = row(StageIdentity { stage_id: stage::FASTQ_REMOVE_CHIMERAS, domain: "fastq", tool_id: &report.planned_tool_id, execution_mode: "local_smoke_copy" },
"copied governed chimera-removal smoke outputs into the amplicon micro pipeline",
Some(path_relative_to_repo(repo_root, &evidence_path)),
Some(report.schema_version.clone()),
BTreeMap::from([
            (
                "asv_representatives".to_string(),
                "artifacts/benchmark-smoke/micro/pipelines/amplicon/artifacts/fastq.infer_asvs/representatives.fasta"
                    .to_string(),
            ),
            (
                "chimera_control_contract".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv"
                    .to_string(),
            ),
        ]),
stage_outputs(&report),
stage_metrics(&report));
    if let Some(path) = &report.raw_backend_report {
        row.outputs.insert("raw_backend_report".to_string(), path.clone());
    }
    Ok(RemoveChimerasStageArtifacts { row, chimeras_tsv, non_chimeric_fasta })
}

pub(crate) fn run_fixture_backed_remove_chimeras_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<RemoveChimerasStageArtifacts> {
    let truth_bundle: AmpliconTruthBundle =
        load_json(&repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH))?;
    let stage_root = stage_root(output_root, stage::FASTQ_REMOVE_CHIMERAS);
    let non_chimeric_fasta = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_NON_CHIMERIC_REPRESENTATIVES_PATH),
        "non_chimeric.fasta",
    )?;
    let chimeras_tsv = stage_root.join("chimeras.tsv");
    let case_report_json =
        stage_root.join("chimera-control-se/vsearch/remove_chimeras_report.json");
    let chimera_metrics_json = stage_root.join("chimera-control-se/vsearch/chimera_metrics.json");
    let chimeras_fasta = stage_root.join("chimera-control-se/vsearch/chimeras.fasta");
    write_text_file(
        &chimeras_tsv,
        &build_three_column_table(
            "chimera_id\tsequence\tsample_id\texpected_presence\n",
            truth_bundle
                .chimera_truths
                .iter()
                .filter(|row| row.expected_presence == "present")
                .map(|row| {
                    format!(
                        "{}\t{}\t{}\t{}\n",
                        row.chimera_id, row.sequence, row.sample_id, row.expected_presence
                    )
                }),
        ),
    )?;
    write_json_file(
        &case_report_json,
        &json!({
            "schema_version": REMOVE_CHIMERAS_STAGE_SCHEMA_VERSION,
            "stage_id": stage::FASTQ_REMOVE_CHIMERAS,
            "sample_id": "chimera-control-se",
            "planned_tool_id": "vsearch",
            "chimera_count": 1,
            "non_chimera_count": 2,
        }),
    )?;
    write_json_file(
        &chimera_metrics_json,
        &json!({
            "checked_sequence_count": 3,
            "chimera_count": 1,
            "non_chimera_count": 2,
            "input_fastq": DEFAULT_AMPLICON_CHIMERA_CONTROL_FASTQ_PATH,
        }),
    )?;
    write_text_file(
        &chimeras_fasta,
        &truth_bundle.chimera_truths.iter().filter(|row| row.expected_presence == "present").fold(
            String::new(),
            |mut fasta, row| {
                let _ = writeln!(fasta, ">{}", row.chimera_id);
                let _ = writeln!(fasta, "{}", row.sequence);
                fasta
            },
        ),
    )?;
    let report = RemoveChimerasStageEvidence {
        schema_version: REMOVE_CHIMERAS_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: stage::FASTQ_REMOVE_CHIMERAS.to_string(),
        sample_id: "chimera-control-se".to_string(),
        planned_tool_id: "vsearch".to_string(),
        report_tool_id: "vsearch".to_string(),
        checked_sequence_count: 3,
        chimera_count: 1,
        non_chimera_count: 2,
        filtered_representative_sequences: path_relative_to_repo(repo_root, &non_chimeric_fasta),
        non_chimeric_fasta: path_relative_to_repo(repo_root, &non_chimeric_fasta),
        chimeras_tsv: path_relative_to_repo(repo_root, &chimeras_tsv),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        chimera_metrics_json: path_relative_to_repo(repo_root, &chimera_metrics_json),
        chimeras_fasta: path_relative_to_repo(repo_root, &chimeras_fasta),
        raw_backend_report: None,
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(StageIdentity { stage_id: stage::FASTQ_REMOVE_CHIMERAS, domain: "fastq", tool_id: "vsearch", execution_mode: "fixture_truth_copy" },
"materialized governed chimera-removal evidence from tracked amplicon truth fixtures",
Some(path_relative_to_repo(repo_root, &evidence_path)),
Some(report.schema_version.clone()),
BTreeMap::from([
            (
                "asv_representatives".to_string(),
                "artifacts/benchmark-smoke/micro/pipelines/amplicon/artifacts/fastq.infer_asvs/representatives.fasta"
                    .to_string(),
            ),
            (
                "chimera_control_contract".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv"
                    .to_string(),
            ),
        ]),
stage_outputs(&report),
stage_metrics(&report));
    Ok(RemoveChimerasStageArtifacts { row, chimeras_tsv, non_chimeric_fasta })
}

fn stage_outputs(report: &RemoveChimerasStageEvidence) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("non_chimeric_fasta".to_string(), report.non_chimeric_fasta.clone()),
        ("chimeras_tsv".to_string(), report.chimeras_tsv.clone()),
        ("case_report_json".to_string(), report.case_report_json.clone()),
        ("chimera_metrics_json".to_string(), report.chimera_metrics_json.clone()),
        ("chimeras_fasta".to_string(), report.chimeras_fasta.clone()),
    ])
}

fn stage_metrics(report: &RemoveChimerasStageEvidence) -> BTreeMap<String, serde_json::Value> {
    BTreeMap::from([
        ("checked_sequence_count".to_string(), json!(report.checked_sequence_count)),
        ("chimera_count".to_string(), json!(report.chimera_count)),
        ("non_chimera_count".to_string(), json!(report.non_chimera_count)),
    ])
}
