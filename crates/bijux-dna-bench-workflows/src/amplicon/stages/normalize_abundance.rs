//! Owner: bijux-dna-bench-workflows
//! Amplicon stages normalize abundance.
use super::super::{
    anyhow, copy_absolute_file, copy_repo_relative_file, is_nonempty_file, json,
    load_amplicon_corpus_fixture_manifest_path, load_json, path_relative_to_repo, row, stage_root,
    write_json_file, BTreeMap, NormalizeAbundanceStageArtifacts, NormalizeAbundanceStageEvidence,
    Path, Result, DEFAULT_AMPLICON_NORMALIZED_ABUNDANCE_PATH,
    NORMALIZE_ABUNDANCE_STAGE_SCHEMA_VERSION, OTU_ABUNDANCE_TABLE_KIND,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_normalize_abundance_stage(
    repo_root: &Path,
    output_root: &Path,
    corpus_manifest_path: &Path,
) -> Result<NormalizeAbundanceStageArtifacts> {
    let source_report_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.normalize_abundance/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_normalize_abundance_stage(
            repo_root,
            output_root,
            corpus_manifest_path,
        );
    }
    let mut report: NormalizeAbundanceStageEvidence = load_json(&source_report_path)?;
    let manifest = load_amplicon_corpus_fixture_manifest_path(corpus_manifest_path)?;
    let abundance_table = manifest
        .abundance_tables
        .iter()
        .find(|table| table.table_kind == OTU_ABUNDANCE_TABLE_KIND)
        .ok_or_else(|| anyhow!("amplicon corpus fixture is missing otu abundance table"))?;
    let manifest_dir = corpus_manifest_path
        .parent()
        .ok_or_else(|| anyhow!("amplicon corpus manifest has no parent directory"))?;
    let otu_abundance_source = manifest_dir.join(&abundance_table.table_path);

    let stage_root = stage_root(output_root, stage::FASTQ_NORMALIZE_ABUNDANCE);
    let normalized_abundance_tsv = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.normalized_abundance_tsv,
        "normalized_abundance.tsv",
    )?;
    let case_report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.case_report_json,
        "corpus-03-otu-abundance-table/seqkit/normalize_abundance_report.json",
    )?;
    let otu_abundance_table_tsv =
        copy_absolute_file(repo_root, &stage_root, &otu_abundance_source, "otu_abundance.tsv")?;
    report.normalized_abundance_tsv = path_relative_to_repo(repo_root, &normalized_abundance_tsv);
    report.case_report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.otu_abundance_table_tsv = path_relative_to_repo(repo_root, &otu_abundance_table_tsv);
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        StageIdentity {
            stage_id: stage::FASTQ_NORMALIZE_ABUNDANCE,
            domain: "fastq",
            tool_id: &report.planned_tool_id,
            execution_mode: "local_smoke_copy",
        },
        "copied governed abundance-normalization smoke outputs into the amplicon micro pipeline",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        BTreeMap::from([(
            "otu_abundance_table".to_string(),
            path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
        )]),
        BTreeMap::from([
            (
                "otu_abundance_table".to_string(),
                path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
            ),
            ("normalized_abundance_tsv".to_string(), report.normalized_abundance_tsv.clone()),
            ("case_report_json".to_string(), report.case_report_json.clone()),
        ]),
        BTreeMap::from([
            ("table_rows".to_string(), json!(report.table_rows)),
            ("sample_count".to_string(), json!(report.sample_count)),
            ("feature_count".to_string(), json!(report.feature_count)),
            ("zero_fraction".to_string(), json!(report.zero_fraction)),
            ("numeric_output_valid".to_string(), json!(report.numeric_output_valid)),
        ]),
    );
    Ok(NormalizeAbundanceStageArtifacts { row, otu_abundance_table_tsv, normalized_abundance_tsv })
}

pub(crate) fn run_fixture_backed_normalize_abundance_stage(
    repo_root: &Path,
    output_root: &Path,
    corpus_manifest_path: &Path,
) -> Result<NormalizeAbundanceStageArtifacts> {
    let manifest = load_amplicon_corpus_fixture_manifest_path(corpus_manifest_path)?;
    let abundance_table = manifest
        .abundance_tables
        .iter()
        .find(|table| table.table_kind == OTU_ABUNDANCE_TABLE_KIND)
        .ok_or_else(|| anyhow!("amplicon corpus fixture is missing otu abundance table"))?;
    let manifest_dir = corpus_manifest_path
        .parent()
        .ok_or_else(|| anyhow!("amplicon corpus manifest has no parent directory"))?;
    let otu_abundance_source = manifest_dir.join(&abundance_table.table_path);

    let stage_root = stage_root(output_root, stage::FASTQ_NORMALIZE_ABUNDANCE);
    let otu_abundance_table_tsv =
        copy_absolute_file(repo_root, &stage_root, &otu_abundance_source, "otu_abundance.tsv")?;
    let normalized_abundance_tsv = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_NORMALIZED_ABUNDANCE_PATH),
        "normalized_abundance.tsv",
    )?;
    let case_report_json =
        stage_root.join("corpus-03-otu-abundance-table/seqkit/normalize_abundance_report.json");
    write_json_file(
        &case_report_json,
        &json!({
            "schema_version": NORMALIZE_ABUNDANCE_STAGE_SCHEMA_VERSION,
            "stage_id": stage::FASTQ_NORMALIZE_ABUNDANCE,
            "sample_id": "corpus-03-otu-abundance-table",
            "planned_tool_id": "seqkit",
            "table_rows": 4,
            "sample_count": 2,
            "feature_count": 3,
            "zero_fraction": 0.0,
        }),
    )?;
    let report = NormalizeAbundanceStageEvidence {
        schema_version: NORMALIZE_ABUNDANCE_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: stage::FASTQ_NORMALIZE_ABUNDANCE.to_string(),
        sample_id: "corpus-03-otu-abundance-table".to_string(),
        planned_tool_id: "seqkit".to_string(),
        report_tool_id: "seqkit".to_string(),
        method: "relative_abundance".to_string(),
        normalization_method: "relative_abundance".to_string(),
        table_rows: 4,
        sample_count: 2,
        feature_count: 3,
        zero_fraction: 0.0,
        normalized_abundance_tsv: path_relative_to_repo(repo_root, &normalized_abundance_tsv),
        sample_totals: vec![
            ("corpus-03-amplicon-se".to_string(), 1.0),
            ("corpus-03-otu-cluster-se".to_string(), 1.0),
        ],
        numeric_output_valid: true,
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        otu_abundance_table_tsv: path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(StageIdentity { stage_id: stage::FASTQ_NORMALIZE_ABUNDANCE, domain: "fastq", tool_id: "seqkit", execution_mode: "fixture_truth_copy" },
"materialized governed abundance-normalization evidence from tracked amplicon truth fixtures",
Some(path_relative_to_repo(repo_root, &evidence_path)),
Some(report.schema_version.clone()),
BTreeMap::from([(
            "otu_abundance_table".to_string(),
            path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
        )]),
BTreeMap::from([
            (
                "otu_abundance_table".to_string(),
                path_relative_to_repo(repo_root, &otu_abundance_table_tsv),
            ),
            ("normalized_abundance_tsv".to_string(), report.normalized_abundance_tsv.clone()),
            ("case_report_json".to_string(), report.case_report_json.clone()),
        ]),
BTreeMap::from([
            ("table_rows".to_string(), json!(report.table_rows)),
            ("sample_count".to_string(), json!(report.sample_count)),
            ("feature_count".to_string(), json!(report.feature_count)),
            ("zero_fraction".to_string(), json!(report.zero_fraction)),
            ("numeric_output_valid".to_string(), json!(report.numeric_output_valid)),
        ]));
    Ok(NormalizeAbundanceStageArtifacts { row, otu_abundance_table_tsv, normalized_abundance_tsv })
}
