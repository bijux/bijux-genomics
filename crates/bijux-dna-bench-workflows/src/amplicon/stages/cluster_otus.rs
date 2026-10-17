//! Owner: bijux-dna-bench-workflows
//! Amplicon stages cluster otus.
use super::super::{
    build_three_column_table, copy_absolute_file, copy_repo_relative_file, is_nonempty_file, json,
    load_json, path_relative_to_repo, row, stage_root, write_fastq_records, write_json_file,
    write_text_file, AmpliconTruthBundle, BTreeMap, ClusterOtusStageArtifacts,
    ClusterOtusStageEvidence, Path, RemoveChimerasStageArtifacts, Result,
    CLUSTER_OTUS_STAGE_SCHEMA_VERSION, DEFAULT_AMPLICON_CLUSTER_OTUS_FASTQ_PATH,
    DEFAULT_AMPLICON_OTU_REPRESENTATIVES_PATH, DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_cluster_otus_stage(
    repo_root: &Path,
    output_root: &Path,
    chimera: &RemoveChimerasStageArtifacts,
) -> Result<ClusterOtusStageArtifacts> {
    let source_report_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.cluster_otus/report.json");
    if !is_nonempty_file(&source_report_path) {
        return run_fixture_backed_cluster_otus_stage(repo_root, output_root, chimera);
    }
    let mut report: ClusterOtusStageEvidence = load_json(&source_report_path)?;
    let stage_root = stage_root(output_root, stage::FASTQ_CLUSTER_OTUS);
    let otu_table_tsv =
        copy_repo_relative_file(repo_root, &stage_root, &report.otu_table_tsv, "otu_table.tsv")?;
    let otu_representatives_fasta = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.otu_representatives_fasta,
        "otu_representatives.fasta",
    )?;
    let case_report_json = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.case_report_json,
        "corpus-03-otu-cluster-se/vsearch/cluster_otus_report.json",
    )?;
    let taxonomy_ready_sequences_path = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.taxonomy_ready_fasta,
        "corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fasta",
    )?;
    let taxonomy_ready_reads_path = copy_repo_relative_file(
        repo_root,
        &stage_root,
        &report.taxonomy_ready_fastq,
        "corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fastq",
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
                "corpus-03-otu-cluster-se/vsearch/otu_clusters.uc",
            )
        })
        .transpose()?;
    report.otu_table_tsv = path_relative_to_repo(repo_root, &otu_table_tsv);
    report.representative_sequences_fasta =
        path_relative_to_repo(repo_root, &otu_representatives_fasta);
    report.otu_representatives_fasta = path_relative_to_repo(repo_root, &otu_representatives_fasta);
    report.case_report_json = path_relative_to_repo(repo_root, &case_report_json);
    report.taxonomy_ready_fasta = path_relative_to_repo(repo_root, &taxonomy_ready_sequences_path);
    report.taxonomy_ready_fastq = path_relative_to_repo(repo_root, &taxonomy_ready_reads_path);
    report.raw_backend_report =
        raw_backend_report.as_ref().map(|path| path_relative_to_repo(repo_root, path));
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let mut row = row(
        StageIdentity {
            stage_id: stage::FASTQ_CLUSTER_OTUS,
            domain: "fastq",
            tool_id: &report.planned_tool_id,
            execution_mode: "local_smoke_copy",
        },
        "copied governed OTU-clustering smoke outputs into the amplicon micro pipeline",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        stage_inputs(repo_root, chimera),
        stage_outputs(&report),
        stage_metrics(&report),
    );
    if let Some(path) = &report.raw_backend_report {
        row.outputs.insert("raw_backend_report".to_string(), path.clone());
    }
    Ok(ClusterOtusStageArtifacts { row, otu_table_tsv, otu_representatives_fasta })
}

pub(crate) fn run_fixture_backed_cluster_otus_stage(
    repo_root: &Path,
    output_root: &Path,
    chimera: &RemoveChimerasStageArtifacts,
) -> Result<ClusterOtusStageArtifacts> {
    let truth_bundle: AmpliconTruthBundle =
        load_json(&repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH))?;
    let stage_root = stage_root(output_root, stage::FASTQ_CLUSTER_OTUS);
    let otu_representatives_fasta = copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_OTU_REPRESENTATIVES_PATH),
        "otu_representatives.fasta",
    )?;
    let otu_table_tsv = stage_root.join("otu_table.tsv");
    let case_report_json =
        stage_root.join("corpus-03-otu-cluster-se/vsearch/cluster_otus_report.json");
    let taxonomy_ready_fasta =
        stage_root.join("corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fasta");
    let taxonomy_ready_reads_fastq =
        stage_root.join("corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fastq");
    let representative_path = path_relative_to_repo(repo_root, &otu_representatives_fasta);
    write_text_file(
        &otu_table_tsv,
        &build_three_column_table(
            "sample_id\totu_id\tabundance\trepresentative_id\trepresentative_fasta\n",
            truth_bundle.otu_representatives.iter().map(|row| {
                format!(
                    "corpus-03-otu-cluster-se\t{}\t1\t{}\t{}\n",
                    row.id, row.id, representative_path
                )
            }),
        ),
    )?;
    write_json_file(
        &case_report_json,
        &json!({
            "schema_version": CLUSTER_OTUS_STAGE_SCHEMA_VERSION,
            "stage_id": stage::FASTQ_CLUSTER_OTUS,
            "sample_id": "corpus-03-otu-cluster-se",
            "planned_tool_id": "vsearch",
            "otu_count": truth_bundle.otu_representatives.len(),
            "clustering_threshold": 0.97,
        }),
    )?;
    copy_absolute_file(
        repo_root,
        &stage_root,
        &repo_root.join(DEFAULT_AMPLICON_OTU_REPRESENTATIVES_PATH),
        "corpus-03-otu-cluster-se/vsearch/taxonomy_ready.fasta",
    )?;
    write_fastq_records(
        &taxonomy_ready_reads_fastq,
        &truth_bundle
            .otu_representatives
            .iter()
            .map(|row| (row.id.as_str(), row.sequence.as_str()))
            .collect::<Vec<_>>(),
    )?;
    let report = ClusterOtusStageEvidence {
        schema_version: CLUSTER_OTUS_STAGE_SCHEMA_VERSION.to_string(),
        stage_id: stage::FASTQ_CLUSTER_OTUS.to_string(),
        sample_id: "corpus-03-otu-cluster-se".to_string(),
        planned_tool_id: "vsearch".to_string(),
        report_tool_id: "vsearch".to_string(),
        clustering_threshold: 0.97,
        otu_count: truth_bundle.otu_representatives.len() as u64,
        sample_count: 1,
        representative_sequence_count: truth_bundle.otu_representatives.len() as u64,
        otu_table_tsv: path_relative_to_repo(repo_root, &otu_table_tsv),
        representative_sequences_fasta: path_relative_to_repo(
            repo_root,
            &otu_representatives_fasta,
        ),
        otu_representatives_fasta: path_relative_to_repo(repo_root, &otu_representatives_fasta),
        case_report_json: path_relative_to_repo(repo_root, &case_report_json),
        taxonomy_ready_fasta: path_relative_to_repo(repo_root, &taxonomy_ready_fasta),
        taxonomy_ready_fastq: path_relative_to_repo(repo_root, &taxonomy_ready_reads_fastq),
        raw_backend_report: None,
    };
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        StageIdentity {
            stage_id: stage::FASTQ_CLUSTER_OTUS,
            domain: "fastq",
            tool_id: "vsearch",
            execution_mode: "fixture_truth_copy",
        },
        "materialized governed OTU-clustering evidence from tracked amplicon truth fixtures",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.clone()),
        stage_inputs(repo_root, chimera),
        stage_outputs(&report),
        stage_metrics(&report),
    );
    Ok(ClusterOtusStageArtifacts { row, otu_table_tsv, otu_representatives_fasta })
}

fn stage_inputs(
    repo_root: &Path,
    chimera: &RemoveChimerasStageArtifacts,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "normalized_amplicon_reads".to_string(),
            DEFAULT_AMPLICON_CLUSTER_OTUS_FASTQ_PATH.to_string(),
        ),
        (
            "non_chimeric_representatives".to_string(),
            path_relative_to_repo(repo_root, &chimera.non_chimeric_fasta),
        ),
    ])
}

fn stage_outputs(report: &ClusterOtusStageEvidence) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("otu_table_tsv".to_string(), report.otu_table_tsv.clone()),
        ("otu_representatives_fasta".to_string(), report.otu_representatives_fasta.clone()),
        ("case_report_json".to_string(), report.case_report_json.clone()),
        ("taxonomy_ready_fasta".to_string(), report.taxonomy_ready_fasta.clone()),
        ("taxonomy_ready_fastq".to_string(), report.taxonomy_ready_fastq.clone()),
    ])
}

fn stage_metrics(report: &ClusterOtusStageEvidence) -> BTreeMap<String, serde_json::Value> {
    BTreeMap::from([
        ("otu_count".to_string(), json!(report.otu_count)),
        ("sample_count".to_string(), json!(report.sample_count)),
        ("representative_sequence_count".to_string(), json!(report.representative_sequence_count)),
        ("clustering_threshold".to_string(), json!(report.clustering_threshold)),
    ])
}
