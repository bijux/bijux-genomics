//! Owner: bijux-dna-bench-workflows
//! Amplicon stages normalize primers.
use super::super::{
    anyhow, bail, copy_absolute_file, copy_repo_relative_file,
    copy_repo_relative_file_with_absolute_fallback, is_nonempty_file, json, load_json,
    path_relative_to_repo, row, stage_root, write_json_file, write_text_file, BTreeMap,
    NormalizePrimersEvidence, NormalizePrimersLocalSmokeReport, NormalizePrimersStageArtifacts,
    Path, Result, DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH,
    DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH, NORMALIZE_PRIMERS_STAGE_SCHEMA_VERSION,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_normalize_primers_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<NormalizePrimersStageArtifacts> {
    let source_report_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.normalize_primers/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_normalize_primers_stage(repo_root, output_root);
    }
    let source_report: NormalizePrimersLocalSmokeReport = load_json(&source_report_path)?;
    if source_report.stage_id != stage::FASTQ_NORMALIZE_PRIMERS {
        bail!("normalize primers source report drifted stage id");
    }
    let source_case =
        source_report
            .cases
            .iter()
            .find(|case| case.sample_id == "amplicon-16s-se")
            .ok_or_else(|| anyhow!("normalize primers source report is missing amplicon-16s-se"))?;

    if source_case.layout != "single_end" || source_case.normalized_reads_r2.is_some() {
        bail!("amplicon-16s-se primer normalization requires single-ended smoke evidence");
    }
    let stage_root = stage_root(output_root, stage::FASTQ_NORMALIZE_PRIMERS);
    let normalized_reads_r1 = copy_repo_relative_file_with_absolute_fallback(
        repo_root,
        &stage_root,
        &source_case.normalized_reads_r1,
        &repo_root.join(DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH),
        "amplicon-16s-se/cutadapt/primer_normalized.fastq.gz",
    )?;
    let report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &source_case.report_json,
        "amplicon-16s-se/cutadapt/normalize_primers_report.json",
    )?;
    let primer_orientation_report = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &source_case.primer_orientation_report,
        "amplicon-16s-se/cutadapt/primer_orientation.tsv",
    )?;
    let primer_stats_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &source_case.primer_stats_json,
        "amplicon-16s-se/cutadapt/primer_stats.json",
    )?;
    let evidence_path = stage_root.join("report.json");
    let evidence = NormalizePrimersEvidence {
        schema_version: source_report.schema_version.clone(),
        stage_id: source_report.stage_id.clone(),
        sample_id: source_case.sample_id.clone(),
        tool_id: source_case.tool_id.clone(),
        layout: source_case.layout.clone(),
        primer_set_id: source_case.primer_set_id.clone(),
        marker_id: source_case.marker_id.clone(),
        orientation_policy: source_case.orientation_policy.clone(),
        input_reads: source_case.input_reads,
        matched_reads: source_case.matched_reads,
        unmatched_reads: source_case.unmatched_reads,
        output_reads: source_case.output_reads,
        normalized_reads_r1: path_relative_to_repo(repo_root, &normalized_reads_r1),
        normalized_reads_r2: None,
        report_json: path_relative_to_repo(repo_root, &report_json),
        primer_orientation_report: path_relative_to_repo(repo_root, &primer_orientation_report),
        primer_stats_json: path_relative_to_repo(repo_root, &primer_stats_json),
        used_fallback: source_case.used_fallback,
        source_report_path: path_relative_to_repo(repo_root, &source_report_path),
    };
    bijux_dna_infra::atomic_write_json(&evidence_path, &evidence)?;

    let row = row(StageIdentity { stage_id: stage::FASTQ_NORMALIZE_PRIMERS, domain: "fastq", tool_id: &source_case.tool_id, execution_mode: "local_smoke_copy" },
"copied governed primer-normalization smoke outputs into the amplicon micro pipeline",
Some(path_relative_to_repo(repo_root, &evidence_path)),
Some(source_report.schema_version.clone()),
BTreeMap::from([
            (
                "primer_contract".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/primers.tsv".to_string(),
            ),
            (
                "sample_fastq".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/amplicon-16s-se.fastq.gz"
                    .to_string(),
            ),
        ]),
stage_outputs(&evidence),
BTreeMap::from([
            ("case_count".to_string(), json!(source_report.case_count)),
            ("input_reads".to_string(), json!(source_case.input_reads)),
            ("matched_reads".to_string(), json!(source_case.matched_reads)),
            ("unmatched_reads".to_string(), json!(source_case.unmatched_reads)),
            ("output_reads".to_string(), json!(source_case.output_reads)),
            ("used_fallback".to_string(), json!(source_case.used_fallback)),
        ]));
    Ok(NormalizePrimersStageArtifacts { row, normalized_reads_r1 })
}

pub(crate) fn run_fixture_backed_normalize_primers_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<NormalizePrimersStageArtifacts> {
    let stage_root = stage_root(output_root, stage::FASTQ_NORMALIZE_PRIMERS);
    let normalized_reads_r1 = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH),
        "amplicon-16s-se/cutadapt/primer_normalized.fastq.gz",
    )?;
    let report_json = stage_root.join("amplicon-16s-se/cutadapt/normalize_primers_report.json");
    let primer_orientation_report =
        stage_root.join("amplicon-16s-se/cutadapt/primer_orientation.tsv");
    let primer_stats_json = stage_root.join("amplicon-16s-se/cutadapt/primer_stats.json");
    write_json_file(
        &report_json,
        &json!({
            "schema_version": NORMALIZE_PRIMERS_STAGE_SCHEMA_VERSION,
            "stage_id": stage::FASTQ_NORMALIZE_PRIMERS,
            "sample_id": "amplicon-16s-se",
            "tool_id": "cutadapt",
            "input_reads": 3,
            "matched_reads": 2,
            "unmatched_reads": 1,
            "output_reads": 3,
        }),
    )?;
    write_text_file(
        &primer_orientation_report,
        "sample_id\tprimer_set_id\torientation_policy\namplicon-16s-se\t16S_universal_v1\tnormalize_to_forward_primer\n",
    )?;
    write_json_file(
        &primer_stats_json,
        &json!({
            "sample_id": "amplicon-16s-se",
            "matched_reads": 2,
            "unmatched_reads": 1,
            "used_fixture_contract": true,
        }),
    )?;
    let evidence_path = stage_root.join("report.json");
    let evidence = NormalizePrimersEvidence {
        schema_version: NORMALIZE_PRIMERS_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: stage::FASTQ_NORMALIZE_PRIMERS.to_string(),
        sample_id: "amplicon-16s-se".to_string(),
        tool_id: "cutadapt".to_string(),
        layout: "single_end".to_string(),
        primer_set_id: "16S_universal_v1".to_string(),
        marker_id: "16S".to_string(),
        orientation_policy: "normalize_to_forward_primer".to_string(),
        input_reads: 3,
        matched_reads: 2,
        unmatched_reads: 1,
        output_reads: 3,
        normalized_reads_r1: path_relative_to_repo(repo_root, &normalized_reads_r1),
        normalized_reads_r2: None,
        report_json: path_relative_to_repo(repo_root, &report_json),
        primer_orientation_report: path_relative_to_repo(repo_root, &primer_orientation_report),
        primer_stats_json: path_relative_to_repo(repo_root, &primer_stats_json),
        used_fallback: true,
        source_report_path: DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH.to_string(),
    };
    bijux_dna_infra::atomic_write_json(&evidence_path, &evidence)?;

    let row = row(
        StageIdentity {
            stage_id: stage::FASTQ_NORMALIZE_PRIMERS,
            domain: "fastq",
            tool_id: "cutadapt",
            execution_mode: "fixture_truth_copy",
        },
        "materialized governed primer-normalization evidence from tracked amplicon fixtures",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(evidence.schema_version.clone()),
        BTreeMap::from([
            (
                "primer_contract".to_string(),
                "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/primers.tsv".to_string(),
            ),
            (
                "sample_fastq".to_string(),
                DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH.to_string(),
            ),
        ]),
        stage_outputs(&evidence),
        BTreeMap::from([
            ("case_count".to_string(), json!(1)),
            ("input_reads".to_string(), json!(3)),
            ("matched_reads".to_string(), json!(2)),
            ("unmatched_reads".to_string(), json!(1)),
            ("output_reads".to_string(), json!(3)),
            ("used_fallback".to_string(), json!(true)),
        ]),
    );
    Ok(NormalizePrimersStageArtifacts { row, normalized_reads_r1 })
}

fn stage_outputs(evidence: &NormalizePrimersEvidence) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("normalized_reads_r1".to_string(), evidence.normalized_reads_r1.clone()),
        ("report_json".to_string(), evidence.report_json.clone()),
        ("primer_orientation_report".to_string(), evidence.primer_orientation_report.clone()),
        ("primer_stats_json".to_string(), evidence.primer_stats_json.clone()),
    ])
}

#[cfg(test)]
#[path = "../../../tests/contracts/primer_input.rs"]
mod input_tests;
