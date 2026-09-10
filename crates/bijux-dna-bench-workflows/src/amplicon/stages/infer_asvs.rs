//! Owner: bijux-dna-bench-workflows
//! Amplicon stages infer asvs.
use super::super::{
    build_three_column_table, copy_absolute_file, copy_repo_relative_file, is_nonempty_file, json,
    load_json, path_relative_to_repo, row, stage_root, write_fastq_records, write_json_file,
    write_text_file, AmpliconTruthBundle, BTreeMap, InferAsvsStageArtifacts,
    InferAsvsStageEvidence, Path, Result, DEFAULT_AMPLICON_ASV_REPRESENTATIVES_PATH,
    DEFAULT_AMPLICON_INFER_ASVS_FASTQ_PATH, DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH,
    INFER_ASVS_STAGE_SCHEMA_VERSION,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_infer_asvs_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<InferAsvsStageArtifacts> {
    let source_report_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.infer_asvs/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_infer_asvs_stage(repo_root, output_root);
    }
    let mut report: InferAsvsStageEvidence = load_json(&source_report_path)?;
    let stage_root = stage_root(output_root, stage::FASTQ_INFER_ASVS);
    let asv_table_tsv = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.asv_table_tsv,
        "corpus-03-amplicon-se/dada2/asv_table.tsv",
    )?;
    let representatives_fasta = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.representatives_fasta,
        "representatives.fasta",
    )?;
    let case_report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.case_report_json,
        "corpus-03-amplicon-se/dada2/infer_asvs_report.json",
    )?;
    let taxonomy_ready_sequences_path = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.taxonomy_ready_fasta,
        "corpus-03-amplicon-se/dada2/taxonomy_ready.fasta",
    )?;
    let taxonomy_ready_reads_path = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.taxonomy_ready_fastq,
        "corpus-03-amplicon-se/dada2/taxonomy_ready.fastq",
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
                "corpus-03-amplicon-se/dada2/infer_asvs_backend_report.json",
            )
        })
        .transpose()?;
    report.asv_table_tsv = path_relative_to_repo(repo_root, &asv_table_tsv);
    report.representatives_fasta = path_relative_to_repo(repo_root, &representatives_fasta);
    report.case_report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.taxonomy_ready_fasta = path_relative_to_repo(repo_root, &taxonomy_ready_sequences_path);
    report.taxonomy_ready_fastq = path_relative_to_repo(repo_root, &taxonomy_ready_reads_path);
    report.raw_backend_report =
        raw_backend_report.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let mut row = row(StageIdentity { stage_id: stage::FASTQ_INFER_ASVS, domain: "fastq", tool_id: &report.planned_tool_id, execution_mode: "local_smoke_copy" },
"copied governed ASV inference smoke outputs into the amplicon micro pipeline",
Some(path_relative_to_repo(repo_root, &evidence_path)),
Some(report.schema_version.clone()),
BTreeMap::from([(
            "normalized_amplicon_reads".to_string(),
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/corpus-03-amplicon-se.fastq.gz"
                .to_string(),
        )]),
stage_outputs(&report),
stage_metrics(&report));
    if let Some(path) = &report.raw_backend_report {
        row.outputs.insert("raw_backend_report".to_string(), path.clone());
    }
    Ok(InferAsvsStageArtifacts { row, asv_table_tsv, representatives_fasta })
}

pub(crate) fn run_fixture_backed_infer_asvs_stage(
    repo_root: &Path,
    output_root: &Path,
) -> Result<InferAsvsStageArtifacts> {
    let truth_bundle: AmpliconTruthBundle =
        load_json(&repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH))?;
    let stage_root = stage_root(output_root, stage::FASTQ_INFER_ASVS);
    let representatives_fasta = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_ASV_REPRESENTATIVES_PATH),
        "representatives.fasta",
    )?;
    let asv_table_tsv = stage_root.join("corpus-03-amplicon-se/dada2/asv_table.tsv");
    let case_report_json = stage_root.join("corpus-03-amplicon-se/dada2/infer_asvs_report.json");
    let taxonomy_ready_fasta = stage_root.join("corpus-03-amplicon-se/dada2/taxonomy_ready.fasta");
    let taxonomy_ready_reads_fastq =
        stage_root.join("corpus-03-amplicon-se/dada2/taxonomy_ready.fastq");
    write_text_file(
        &asv_table_tsv,
        &build_three_column_table(
            "sample_id\tasv_id\tabundance\n",
            truth_bundle
                .asv_representatives
                .iter()
                .map(|row| format!("corpus-03-amplicon-se\t{}\t1\n", row.id)),
        ),
    )?;
    write_json_file(
        &case_report_json,
        &json!({
            "schema_version": INFER_ASVS_STAGE_SCHEMA_VERSION,
            "stage_id": stage::FASTQ_INFER_ASVS,
            "sample_id": "corpus-03-amplicon-se",
            "planned_tool_id": "dada2",
            "asv_count": truth_bundle.asv_representatives.len(),
        }),
    )?;
    copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_ASV_REPRESENTATIVES_PATH),
        "corpus-03-amplicon-se/dada2/taxonomy_ready.fasta",
    )?;
    write_fastq_records(
        &taxonomy_ready_reads_fastq,
        &truth_bundle
            .asv_representatives
            .iter()
            .map(|row| (row.id.as_str(), row.sequence.as_str()))
            .collect::<Vec<_>>(),
    )?;
    let report = InferAsvsStageEvidence {
        schema_version: INFER_ASVS_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: stage::FASTQ_INFER_ASVS.to_string(),
        sample_id: "corpus-03-amplicon-se".to_string(),
        planned_tool_id: "dada2".to_string(),
        report_tool_id: "dada2".to_string(),
        asv_count: truth_bundle.asv_representatives.len() as u64,
        sample_count: 1,
        representative_sequence_count: truth_bundle.asv_representatives.len() as u64,
        asv_table_tsv: path_relative_to_repo(repo_root, &asv_table_tsv),
        representatives_fasta: path_relative_to_repo(repo_root, &representatives_fasta),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        taxonomy_ready_fasta: path_relative_to_repo(repo_root, &taxonomy_ready_fasta),
        taxonomy_ready_fastq: path_relative_to_repo(repo_root, &taxonomy_ready_reads_fastq),
        raw_backend_report: None,
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        StageIdentity {
            stage_id: stage::FASTQ_INFER_ASVS,
            domain: "fastq",
            tool_id: "dada2",
            execution_mode: "fixture_truth_copy",
        },
        "materialized governed ASV inference evidence from tracked amplicon truth fixtures",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([(
            "normalized_amplicon_reads".to_string(),
            DEFAULT_AMPLICON_INFER_ASVS_FASTQ_PATH.to_string(),
        )]),
        stage_outputs(&report),
        stage_metrics(&report),
    );
    Ok(InferAsvsStageArtifacts { row, asv_table_tsv, representatives_fasta })
}

fn stage_outputs(report: &InferAsvsStageEvidence) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("asv_table_tsv".to_string(), report.asv_table_tsv.clone()),
        ("representatives_fasta".to_string(), report.representatives_fasta.clone()),
        ("case_report_json".to_string(), report.case_report_json.clone()),
        ("taxonomy_ready_fasta".to_string(), report.taxonomy_ready_fasta.clone()),
        ("taxonomy_ready_fastq".to_string(), report.taxonomy_ready_fastq.clone()),
    ])
}

fn stage_metrics(report: &InferAsvsStageEvidence) -> BTreeMap<String, serde_json::Value> {
    BTreeMap::from([
        ("asv_count".to_string(), json!(report.asv_count)),
        ("sample_count".to_string(), json!(report.sample_count)),
        ("representative_sequence_count".to_string(), json!(report.representative_sequence_count)),
    ])
}
