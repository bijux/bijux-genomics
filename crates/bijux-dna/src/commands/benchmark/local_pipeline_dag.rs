pub(crate) use bijux_dna_api::v1::api::bench::benchmark_local_pipeline_config_path;
use bijux_dna_api::v1::api::bench::load_benchmark_local_pipeline_dag_config as load_local_pipeline_dag_config;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
#[cfg(test)]
use bijux_dna_api::v1::api::bench::BenchmarkLocalPipelineProjectSourceConfig as LocalPipelineProjectSourceConfig;
use bijux_dna_api::v1::api::bench::{
    load_benchmark_pipeline_inventory_index as load_pipeline_inventory_index,
    load_benchmark_pipeline_vcf_stage_matrix_index as load_local_vcf_stage_matrix_index,
    validate_benchmark_local_pipeline_dag_config as validate_pipeline_contract,
    validate_benchmark_local_pipeline_dag_declarations,
    validate_benchmark_local_pipeline_dag_validation,
    validate_benchmark_pipeline_execution_context, validate_benchmark_pipeline_profiles,
    validate_benchmark_pipeline_reference_context, BenchmarkLocalCorpusStageCompatibilityReport,
    PipelineProfileEvidence, VcfStageMatrixRow, BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    BenchmarkLocalPipelineAncientDnaPolicy as LocalPipelineAncientDnaPolicy,
    BenchmarkLocalPipelineDagConfig as LocalPipelineDagConfig,
    BenchmarkLocalPipelineDagValidationNode as LocalPipelineDagValidationNodeReport,
    BenchmarkLocalPipelineDagValidationReport as LocalPipelineDagValidationReport,
    BenchmarkLocalPipelineExecutionContext as LocalPipelineExecutionContext,
    BenchmarkLocalPipelineProjectSource as LocalPipelineProjectSource,
    BenchmarkLocalPipelineReferenceAssets as LocalPipelineReferenceAssets,
    BenchmarkLocalPipelineReferenceContext as LocalPipelineReferenceContext,
    BenchmarkLocalPipelineSnpEffAssets as LocalPipelineSnpEffAssets,
    BenchmarkLocalPipelineVariantAssets as LocalPipelineVariantAssets,
};
use bijux_dna_api::v1::api::plan::{validate_artifact_flow, ArtifactFlowNode};

use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH: &str =
    "benchmarks/configs/pipelines/local/fastq-core-preprocess.toml";

pub(crate) fn run_validate_pipeline_dag(
    args: &parse::BenchLocalValidatePipelineDagArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let config_path = match &args.config {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH),
    };
    let config = load_local_pipeline_dag_config(&config_path)?;
    let output_path = match &args.output {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(default_pipeline_report_relative_path(&config.pipeline_id)),
    };

    let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn validate_pipeline_dag_path(
    repo_root: &Path,
    config_path: &Path,
    output_path: &Path,
) -> Result<LocalPipelineDagValidationReport> {
    let report = load_validated_local_pipeline_dag_report(repo_root, config_path)?;
    validate_benchmark_local_pipeline_dag_validation(&report)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

pub(crate) fn load_validated_local_pipeline_dag_report(
    repo_root: &Path,
    config_path: &Path,
) -> Result<LocalPipelineDagValidationReport> {
    let config = load_local_pipeline_dag_config(config_path)?;
    let output_path = repo_root.join(default_pipeline_report_relative_path(&config.pipeline_id));
    validate_pipeline_dag(repo_root, config_path, &output_path, &config)
}

fn validate_pipeline_dag(
    repo_root: &Path,
    config_path: &Path,
    output_path: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationReport> {
    validate_pipeline_contract(config)?;
    validate_pipeline_metadata(repo_root, config_path, config)?;

    let inventory_index = load_pipeline_inventory_index(repo_root, config.domain)?;

    let mut seen_stage_ids = BTreeSet::new();
    for node in &config.nodes {
        require_non_empty(
            &node.stage_id,
            "local pipeline DAG nodes must declare a non-empty `stage_id`",
        )?;
        require_non_empty(
            &node.summary,
            "local pipeline DAG nodes must declare a non-empty `summary`",
        )?;
        if !seen_stage_ids.insert(node.stage_id.clone()) {
            return Err(anyhow!("local pipeline DAG repeats stage_id `{}`", node.stage_id));
        }
        let Some(expected_readiness_kind) = inventory_index.get(&node.stage_id) else {
            return Err(anyhow!(
                "local pipeline DAG stage `{}` is not present in the governed `{}` local stage inventory",
                node.stage_id,
                config.domain.as_str()
            ));
        };
        if node.readiness_kind != *expected_readiness_kind {
            return Err(anyhow!(
                "local pipeline DAG stage `{}` declares readiness `{}` but governed `{}` local inventory requires `{}`",
                node.stage_id,
                node.readiness_kind.as_str(),
                config.domain.as_str(),
                expected_readiness_kind.as_str()
            ));
        }
    }

    let artifact_nodes = config
        .nodes
        .iter()
        .map(|node| ArtifactFlowNode {
            node_id: &node.node_id,
            depends_on: &node.depends_on,
            external_inputs: &node.external_inputs,
            upstream_inputs: &node.upstream_inputs,
            outputs: &node.outputs,
        })
        .collect::<Vec<_>>();
    let topological_order = validate_artifact_flow(&config.pipeline_id, &artifact_nodes)?;

    let validation_profiles =
        validate_benchmark_pipeline_profiles(config, &LocalPipelineEvidence { repo_root })?;

    let nodes = config
        .nodes
        .iter()
        .map(|node| LocalPipelineDagValidationNodeReport {
            node_id: node.node_id.clone(),
            stage_id: node.stage_id.clone(),
            readiness_kind: node.readiness_kind.as_str().to_string(),
            dependency_count: node.depends_on.len(),
            external_input_count: node.external_inputs.len(),
            upstream_input_count: node.upstream_inputs.len(),
            output_count: node.outputs.len(),
            depends_on: node.depends_on.clone(),
            external_inputs: node.external_inputs.clone(),
            upstream_inputs: node.upstream_inputs.clone(),
            outputs: node.outputs.clone(),
        })
        .collect::<Vec<_>>();

    let report = LocalPipelineDagValidationReport {
        schema_version: BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1.to_string(),
        config_path: path_relative_to_repo(repo_root, config_path),
        output_path: path_relative_to_repo(repo_root, output_path),
        pipeline_id: config.pipeline_id.clone(),
        domain: config.domain.as_str().to_string(),
        default_corpus_id: config.default_corpus_id.clone(),
        summary: config.summary.clone(),
        reference_context: config.reference_context.clone(),
        execution_context: config.execution_context.clone(),
        project_source_count: config.project_sources.len(),
        project_sources: config
            .project_sources
            .iter()
            .map(|source| LocalPipelineProjectSource {
                project_id: source.project_id.clone(),
                metadata_url: source.metadata_url.clone(),
                retries: source.retries,
            })
            .collect(),
        ancient_dna_policy: config.ancient_dna_policy.clone(),
        reference_assets: config.reference_assets.clone(),
        variant_assets: config.variant_assets.clone(),
        node_count: nodes.len(),
        edge_count: config.nodes.iter().map(|node| node.depends_on.len()).sum(),
        acyclic: true,
        valid: true,
        topological_order,
        validation_profiles,
        nodes,
    };
    validate_benchmark_local_pipeline_dag_validation(&report)?;
    Ok(report)
}

struct LocalPipelineEvidence<'a> {
    repo_root: &'a Path,
}

impl PipelineProfileEvidence for LocalPipelineEvidence<'_> {
    fn corpus_stage_compatibility(&self) -> Result<BenchmarkLocalCorpusStageCompatibilityReport> {
        validate_corpus_stage_compatibility_path(
            self.repo_root,
            &self.repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
        )
    }

    fn vcf_stage_matrix(&self) -> Result<BTreeMap<String, VcfStageMatrixRow>> {
        load_local_vcf_stage_matrix_index(self.repo_root)
    }
}

fn validate_pipeline_metadata(
    repo_root: &Path,
    config_path: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<()> {
    if let Some(reference_context) = &config.reference_context {
        validate_benchmark_pipeline_reference_context(repo_root, reference_context)?;
    }
    if let Some(execution_context) = &config.execution_context {
        validate_benchmark_pipeline_execution_context(repo_root, execution_context)?;
    }
    validate_benchmark_local_pipeline_dag_declarations(config_path, config)
}

fn require_non_empty(value: &str, message: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!(message.to_string()));
    }
    Ok(())
}

fn default_pipeline_report_relative_path(pipeline_id: &str) -> PathBuf {
    PathBuf::from("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag")
        .join(format!("{pipeline_id}.json"))
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        validate_pipeline_dag_path, LocalPipelineProjectSourceConfig,
        DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH,
    };
    use std::fs;

    fn repo_root() -> std::path::PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn project_source_declaration_preserves_default_retry_policy() {
        let source: LocalPipelineProjectSourceConfig = serde_yaml::from_str(
            "project_id: ena-study\nmetadata_url: https://example.invalid/study\n",
        )
        .expect("parse project source declaration");

        assert_eq!(source.retries, 3);
    }

    #[test]
    fn cross_pipeline_dag_can_mix_fastq_and_bam_inventory_nodes() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_path = tempdir.path().join("fastq-to-bam-cross.toml");
        fs::write(
            &config_path,
            r#"
schema_version = "bijux.bench.local_pipeline_dag.v1"
pipeline_id = "fastq-to-bam-cross"
domain = "cross"
summary = "Cross-domain proof that local pipeline validation can mix governed FASTQ and BAM stages."
default_corpus_id = "corpus-01-mini"

[[nodes]]
node_id = "fastq.validate_reads"
stage_id = "fastq.validate_reads"
readiness_kind = "smoke"
summary = "Validate governed FASTQ inputs before alignment."
depends_on = []
external_inputs = ["corpus.raw_fastq_reads"]
upstream_inputs = []
outputs = ["validated_reads_r1_path", "validated_reads_r2_path", "validation_report"]

[[nodes]]
node_id = "bam.align"
stage_id = "bam.align"
readiness_kind = "dry_or_smoke"
summary = "Align validated FASTQ paths against the governed BAM alignment contracts."
depends_on = ["fastq.validate_reads"]
external_inputs = [
  "alignment_reference_fasta_contract",
  "alignment_reference_index_contract",
  "alignment_read_group_contract",
]
upstream_inputs = ["validated_reads_r1_path", "validated_reads_r2_path"]
outputs = ["align_bam", "align_bai", "align_metrics"]
"#,
        )
        .expect("write cross config");

        let output_path = tempdir.path().join("fastq-to-bam-cross.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate cross local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-to-bam-cross");
        assert_eq!(report.domain, "cross");
        assert_eq!(report.node_count, 2);
        assert_eq!(report.edge_count, 1);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.align"
                    && node.upstream_inputs
                        == vec!["validated_reads_r1_path", "validated_reads_r2_path"]
            }),
            "cross DAG validation must admit BAM alignment consumers of governed FASTQ path outputs"
        );
    }

    #[test]
    fn yaml_pipeline_bundle_can_mix_fastq_and_bam_inventory_nodes() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_dir = tempdir.path().join("fastq-to-bam-cross");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(
            config_dir.join("config.yaml"),
            r"
schema_version: bijux.bench.local_pipeline_dag.v1
pipeline_id: fastq-to-bam-cross
domain: cross
summary: Cross-domain proof that YAML local pipeline validation can mix governed FASTQ and BAM stages.
default_corpus_id: corpus-01-mini
reference_context:
  species_id: Homo sapiens
  build_id: GRCh38
  reference_bundle_id: hsapiens_grch38_primary
execution_context:
  site_profile_id: lunarc
  compute_surfaces:
    - fastq-to-bam
",
        )
        .expect("write yaml metadata");
        fs::write(
            config_dir.join("stages.yaml"),
            r"
nodes:
  - node_id: fastq.validate_reads
    stage_id: fastq.validate_reads
    readiness_kind: smoke
    summary: Validate governed FASTQ inputs before alignment.
    depends_on: []
    external_inputs:
      - corpus.raw_fastq_reads
    upstream_inputs: []
    outputs:
      - validated_reads_r1_path
      - validated_reads_r2_path
      - validation_report
  - node_id: bam.align
    stage_id: bam.align
    readiness_kind: dry_or_smoke
    summary: Align validated FASTQ paths against the governed BAM alignment contracts.
    depends_on:
      - fastq.validate_reads
    external_inputs:
      - alignment_reference_fasta_contract
      - alignment_reference_index_contract
      - alignment_read_group_contract
    upstream_inputs:
      - validated_reads_r1_path
      - validated_reads_r2_path
    outputs:
      - align_bam
      - align_bai
      - align_metrics
",
        )
        .expect("write yaml stages");

        let output_path = tempdir.path().join("fastq-to-bam-cross.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_dir, &output_path)
            .expect("validate yaml local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-to-bam-cross");
        assert_eq!(report.domain, "cross");
        assert_eq!(report.node_count, 2);
        assert_eq!(report.config_path, super::path_relative_to_repo(&repo_root, &config_dir));
        assert_eq!(report.project_source_count, 0);
        assert_eq!(
            report.reference_context.as_ref().map(|context| context.reference_bundle_id.as_str()),
            Some("hsapiens_grch38_primary")
        );
        assert_eq!(
            report.execution_context.as_ref().map(|context| context.site_profile_id.as_str()),
            Some("lunarc")
        );
    }

    #[test]
    fn cross_pipeline_dag_can_mix_fastq_bam_and_vcf_inventory_nodes() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_path = tempdir.path().join("fastq-bam-vcf-cross.toml");
        fs::write(
            &config_path,
            r#"
schema_version = "bijux.bench.local_pipeline_dag.v1"
pipeline_id = "fastq-bam-vcf-cross"
domain = "cross"
summary = "Cross-domain proof that local pipeline validation can mix governed FASTQ, BAM, and VCF stages."
default_corpus_id = "corpus-01-mini"

[[nodes]]
node_id = "fastq.validate_reads"
stage_id = "fastq.validate_reads"
readiness_kind = "smoke"
summary = "Validate governed FASTQ inputs before downstream alignment."
depends_on = []
external_inputs = ["corpus.raw_fastq_reads"]
upstream_inputs = []
outputs = ["validated_reads_r1_path", "validated_reads_r2_path", "validation_report"]

[[nodes]]
node_id = "bam.align"
stage_id = "bam.align"
readiness_kind = "dry_or_smoke"
summary = "Align governed FASTQ path outputs into BAM."
depends_on = ["fastq.validate_reads"]
external_inputs = [
  "alignment_reference_fasta_contract",
  "alignment_reference_index_contract",
  "alignment_read_group_contract",
]
upstream_inputs = ["validated_reads_r1_path", "validated_reads_r2_path"]
outputs = ["aligned_bam", "aligned_bai", "align_metrics"]

[[nodes]]
node_id = "vcf.call"
stage_id = "vcf.call"
readiness_kind = "smoke"
summary = "Call variants from the governed aligned BAM handoff."
depends_on = ["bam.align"]
external_inputs = ["reference_fasta_contract", "reference_fai_contract"]
upstream_inputs = ["aligned_bam", "aligned_bai"]
outputs = ["called_vcf", "called_vcf_tbi", "call_stage_metrics"]
"#,
        )
        .expect("write cross VCF config");

        let output_path = tempdir.path().join("fastq-bam-vcf-cross.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate cross local pipeline dag with VCF");

        assert_eq!(report.pipeline_id, "fastq-bam-vcf-cross");
        assert_eq!(report.domain, "cross");
        assert_eq!(report.node_count, 3);
        assert_eq!(report.edge_count, 2);
        assert!(report.acyclic);
        assert_eq!(report.topological_order, vec!["fastq.validate_reads", "bam.align", "vcf.call"]);
    }

    #[test]
    fn fastq_core_preprocess_pipeline_dag_is_acyclic_and_inventory_aligned() {
        let repo_root = repo_root();
        let config_path = repo_root.join(DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH);
        let output_path = repo_root
            .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-core-preprocess.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-core-preprocess");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert_eq!(
            report.topological_order.first().map(String::as_str),
            Some("fastq.validate_reads")
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.report_qc"
                    && node.upstream_inputs
                        == vec![
                            "validation_report",
                            "read_length_metrics",
                            "adapter_report",
                            "trim_metrics",
                            "filter_metrics",
                            "filtered_profile",
                        ]
            }),
            "report_qc must consume the governed upstream preprocessing metrics"
        );
    }

    #[test]
    fn fastq_paired_merge_pipeline_dag_tracks_merged_and_unmerged_handoffs() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/fastq-paired-merge.toml");
        let output_path = repo_root.join(
            "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-paired-merge.json",
        );
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate paired merge local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-paired-merge");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.merge_pairs"
                    && node.outputs
                        == vec![
                            "merged_reads",
                            "unmerged_r1_reads",
                            "unmerged_r2_reads",
                            "merge_metrics",
                        ]
            }),
            "merge_pairs must declare merged and unmerged outputs explicitly"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.filter_reads"
                    && node.upstream_inputs
                        == vec!["merged_reads", "unmerged_r1_reads", "unmerged_r2_reads"]
            }),
            "filter_reads must consume the merged and unmerged merge-pairs outputs"
        );
    }

    #[test]
    fn fastq_edna_taxonomy_pipeline_dag_tracks_taxonomy_assets_and_outputs() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/fastq-edna-taxonomy.toml");
        let output_path = repo_root.join(
            "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-edna-taxonomy.json",
        );
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate edna taxonomy local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-edna-taxonomy");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-02-edna-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.screen_taxonomy"
                    && node.external_inputs
                        == vec!["taxonomy_database.root", "taxonomy_expected_truth_table"]
                    && node.outputs
                        == vec!["taxonomy_classification", "unclassified_reads", "taxonomy_summary"]
            }),
            "screen_taxonomy must carry governed taxonomy assets and explicit classified plus unclassified outputs"
        );
    }

    #[test]
    fn edna_taxonomy_no_vcf_pipeline_dag_keeps_taxonomy_reporting_local_to_fastq() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/edna-taxonomy-no-vcf.toml");
        let output_path = repo_root
            .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/edna-taxonomy-no-vcf.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate edna taxonomy no-vcf local pipeline dag");

        assert_eq!(report.pipeline_id, "edna-taxonomy-no-vcf");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-02-edna-mini");
        assert_eq!(report.node_count, 6);
        assert_eq!(report.edge_count, 10);
        assert!(report.acyclic);
        assert!(
            report
                .validation_profiles
                .iter()
                .any(|profile| profile.profile_id == "edna_taxonomy_no_vcf"
                    && profile.check_count == 8),
            "edna taxonomy no-vcf pipeline must surface the governed separation profile"
        );
        assert!(
            report.nodes.iter().all(|node| node.stage_id.starts_with("fastq.")),
            "taxonomy-only pipeline must keep every declared stage in the fastq domain"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.screen_taxonomy"
                    && node.external_inputs
                        == vec!["taxonomy_database.root", "taxonomy_expected_truth_table"]
                    && node.outputs
                        == vec!["taxonomy_classification", "unclassified_reads", "taxonomy_summary"]
            }),
            "screen_taxonomy must keep the governed taxonomy assets and outputs explicit"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.report_qc"
                    && node.depends_on
                        == vec![
                            "fastq.validate_reads",
                            "fastq.detect_adapters",
                            "fastq.trim_reads",
                            "fastq.filter_reads",
                            "fastq.screen_taxonomy",
                        ]
                    && node.upstream_inputs
                        == vec![
                            "validation_report",
                            "adapter_report",
                            "trim_metrics",
                            "filter_metrics",
                            "taxonomy_summary",
                        ]
            }),
            "report_qc must stay downstream of taxonomy screening without any bam or vcf handoffs"
        );
    }

    #[test]
    fn edna_taxonomy_no_vcf_profile_rejects_vcf_bridge_from_taxonomy_outputs() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_path = tempdir.path().join("edna-taxonomy-no-vcf.toml");
        fs::write(
            &config_path,
            r#"
schema_version = "bijux.bench.local_pipeline_dag.v1"
pipeline_id = "edna-taxonomy-no-vcf"
domain = "cross"
summary = "Invalid proof that taxonomy outputs must not bridge into VCF stages."
default_corpus_id = "corpus-02-edna-mini"

[[nodes]]
node_id = "fastq.validate_reads"
stage_id = "fastq.validate_reads"
readiness_kind = "smoke"
summary = "Validate governed eDNA FASTQ inputs."
depends_on = []
external_inputs = ["corpus.raw_fastq_reads"]
upstream_inputs = []
outputs = ["validated_reads", "validation_report"]

[[nodes]]
node_id = "fastq.filter_reads"
stage_id = "fastq.filter_reads"
readiness_kind = "smoke"
summary = "Filter validated reads."
depends_on = ["fastq.validate_reads"]
external_inputs = []
upstream_inputs = ["validated_reads"]
outputs = ["filtered_reads", "filter_metrics"]

[[nodes]]
node_id = "fastq.screen_taxonomy"
stage_id = "fastq.screen_taxonomy"
readiness_kind = "dry_or_smoke"
summary = "Screen taxonomy from filtered reads."
depends_on = ["fastq.filter_reads"]
external_inputs = ["taxonomy_database.root", "taxonomy_expected_truth_table"]
upstream_inputs = ["filtered_reads"]
outputs = ["taxonomy_classification", "unclassified_reads", "taxonomy_summary"]

[[nodes]]
node_id = "vcf.call"
stage_id = "vcf.call"
readiness_kind = "smoke"
summary = "Invalid germline bridge from taxonomy-only outputs."
depends_on = ["fastq.screen_taxonomy"]
external_inputs = ["reference_fasta_contract", "reference_fai_contract"]
upstream_inputs = ["taxonomy_summary"]
outputs = ["called_vcf", "called_vcf_tbi", "call_stage_metrics"]
"#,
        )
        .expect("write invalid edna taxonomy config");

        let output_path = tempdir.path().join("edna-taxonomy-no-vcf.json");
        let error = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect_err("taxonomy-only profile must reject a vcf bridge");
        assert!(
            error
                .to_string()
                .contains("taxonomy-only pipeline must not admit BAM or VCF germline stages"),
            "profile must fail closed when taxonomy outputs bridge into germline stages: {error:#}"
        );
    }

    #[test]
    fn fastq_amplicon_pipeline_dag_tracks_amplicon_handoffs() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/fastq-amplicon.toml");
        let output_path = repo_root.join(
            "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-amplicon.json",
        );
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate amplicon local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-amplicon");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-03-amplicon-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.cluster_otus"
                    && node.upstream_inputs
                        == vec!["normalized_amplicon_reads", "non_chimeric_representatives"]
                    && node.outputs == vec!["otu_table", "otu_representatives"]
            }),
            "cluster_otus must consume normalized reads plus non-chimeric representatives"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.normalize_abundance"
                    && node.upstream_inputs == vec!["otu_table"]
                    && node.outputs == vec!["normalized_abundance_table"]
            }),
            "normalize_abundance must consume the clustered OTU table"
        );
    }

    #[test]
    fn amplicon_asv_otu_no_vcf_pipeline_dag_keeps_amplicon_reporting_local_to_fastq() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/amplicon-asv-otu-no-vcf.toml");
        let output_path = repo_root
            .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/amplicon-asv-otu-no-vcf.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate amplicon asv otu no-vcf local pipeline dag");

        assert_eq!(report.pipeline_id, "amplicon-asv-otu-no-vcf");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-03-amplicon-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert!(
            report
                .validation_profiles
                .iter()
                .any(|profile| profile.profile_id == "amplicon_asv_otu_no_vcf"
                    && profile.check_count == 8),
            "amplicon asv otu no-vcf pipeline must surface the governed separation profile"
        );
        assert!(
            report.nodes.iter().all(|node| node.stage_id.starts_with("fastq.")),
            "amplicon-only pipeline must keep every declared stage in the fastq domain"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.cluster_otus"
                    && node.upstream_inputs
                        == vec!["normalized_amplicon_reads", "non_chimeric_representatives"]
                    && node.outputs == vec!["otu_table", "otu_representatives"]
            }),
            "cluster_otus must keep the governed normalized-read and non-chimeric representative handoffs explicit"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.report_qc"
                    && node.upstream_inputs
                        == vec![
                            "validation_report",
                            "primer_normalization_metrics",
                            "asv_metrics",
                            "chimera_report",
                            "otu_table",
                            "normalized_abundance_table",
                        ]
            }),
            "report_qc must stay downstream of asv, otu, and abundance evidence without bam or vcf handoffs"
        );
    }

    #[test]
    fn amplicon_asv_otu_no_vcf_profile_rejects_vcf_bridge_from_asv_or_otu_outputs() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_path = tempdir.path().join("amplicon-asv-otu-no-vcf.toml");
        fs::write(
            &config_path,
            r#"
schema_version = "bijux.bench.local_pipeline_dag.v1"
pipeline_id = "amplicon-asv-otu-no-vcf"
domain = "cross"
summary = "Invalid proof that amplicon outputs must not bridge into VCF stages."
default_corpus_id = "corpus-03-amplicon-mini"

[[nodes]]
node_id = "fastq.validate_reads"
stage_id = "fastq.validate_reads"
readiness_kind = "smoke"
summary = "Validate governed amplicon FASTQ inputs."
depends_on = []
external_inputs = ["corpus.raw_amplicon_fastq_reads"]
upstream_inputs = []
outputs = ["validated_amplicon_reads", "validation_report"]

[[nodes]]
node_id = "fastq.normalize_primers"
stage_id = "fastq.normalize_primers"
readiness_kind = "smoke"
summary = "Normalize primers before ASV inference."
depends_on = ["fastq.validate_reads"]
external_inputs = ["amplicon_primer_contract"]
upstream_inputs = ["validated_amplicon_reads"]
outputs = ["normalized_amplicon_reads", "primer_normalization_metrics"]

[[nodes]]
node_id = "fastq.infer_asvs"
stage_id = "fastq.infer_asvs"
readiness_kind = "smoke"
summary = "Infer ASVs from normalized reads."
depends_on = ["fastq.normalize_primers"]
external_inputs = []
upstream_inputs = ["normalized_amplicon_reads"]
outputs = ["asv_table", "asv_representatives", "asv_metrics"]

[[nodes]]
node_id = "vcf.call"
stage_id = "vcf.call"
readiness_kind = "smoke"
summary = "Invalid germline bridge from amplicon-only outputs."
depends_on = ["fastq.infer_asvs"]
external_inputs = ["reference_fasta_contract", "reference_fai_contract"]
upstream_inputs = ["asv_table"]
outputs = ["called_vcf", "called_vcf_tbi", "call_stage_metrics"]
"#,
        )
        .expect("write invalid amplicon config");

        let output_path = tempdir.path().join("amplicon-asv-otu-no-vcf.json");
        let error = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect_err("amplicon-only profile must reject a vcf bridge");
        assert!(
            error
                .to_string()
                .contains("amplicon-only pipeline must not admit BAM or VCF germline stages"),
            "profile must fail closed when asv or otu outputs bridge into germline stages: {error:#}"
        );
    }

    #[test]
    fn fastq_umi_pipeline_dag_tracks_downstream_umi_consumers() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/fastq-umi.toml");
        let output_path = repo_root
            .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-umi.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate umi local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-umi");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert_eq!(report.node_count, 8);
        assert_eq!(report.edge_count, 13);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.extract_umis"
                    && node.outputs
                        == vec![
                            "umi_tagged_reads_r1",
                            "umi_tagged_reads_r2",
                            "umi_extraction_report",
                        ]
            }),
            "extract_umis must emit UMI-tagged reads and the governed extraction report"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.remove_duplicates"
                    && node.upstream_inputs
                        == vec!["filtered_umi_reads_r1", "filtered_umi_reads_r2"]
                    && node.outputs
                        == vec![
                            "deduplicated_umi_reads_r1",
                            "deduplicated_umi_reads_r2",
                            "duplicate_classes_tsv",
                            "duplicate_provenance_json",
                            "deduplication_report",
                        ]
            }),
            "remove_duplicates must consume filtered UMI-tagged reads and expose duplicate-aware outputs"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.report_qc"
                    && node.upstream_inputs
                        == vec![
                            "validation_report",
                            "umi_extraction_report",
                            "duplicate_signal_report",
                            "trim_metrics",
                            "filter_metrics",
                            "deduplication_report",
                            "deduplicated_profile",
                        ]
            }),
            "report_qc must collate UMI extraction and duplicate-aware preprocessing metrics"
        );
    }

    #[test]
    fn bam_core_qc_pipeline_dag_tracks_qc_summary_filter_and_coverage_handoffs() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/bam-core-qc.toml");
        let output_path = repo_root
            .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-core-qc.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate bam core qc local pipeline dag");

        assert_eq!(report.pipeline_id, "bam-core-qc");
        assert_eq!(report.domain, "bam");
        assert_eq!(report.default_corpus_id, "corpus-01-bam-mini");
        assert_eq!(report.node_count, 5);
        assert_eq!(report.edge_count, 5);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.qc_pre"
                    && node.upstream_inputs == vec!["validation_report"]
                    && node.outputs
                        == vec![
                            "qc_pre_flagstat",
                            "qc_pre_idxstats",
                            "qc_pre_stats",
                            "qc_pre_stage_metrics",
                        ]
            }),
            "bam.qc_pre must consume validation output and expose governed pre-QC metric artifacts"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.filter"
                    && node.readiness_kind == "dry_or_smoke"
                    && node.upstream_inputs
                        == vec!["qc_pre_flagstat", "qc_pre_idxstats", "qc_pre_stats"]
                    && node.outputs
                        == vec![
                            "filtered_bam",
                            "filtered_bai",
                            "filter_report_json",
                            "filter_stage_metrics",
                        ]
            }),
            "bam.filter must consume pre-QC outputs and expose the filtered BAM handoff"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.coverage"
                    && node.upstream_inputs
                        == vec!["mapping_summary_report_json", "filtered_bam", "filtered_bai"]
                    && node.external_inputs == vec!["coverage_region_contract"]
            }),
            "bam.coverage must consume both mapping summary context and filtered BAM outputs"
        );
    }

    #[test]
    fn bam_authenticity_pipeline_dag_tracks_only_required_authenticity_evidence() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/bam-authenticity.toml");
        let output_path = repo_root.join(
            "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-authenticity.json",
        );
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate bam authenticity local pipeline dag");

        assert_eq!(report.pipeline_id, "bam-authenticity");
        assert_eq!(report.domain, "bam");
        assert_eq!(report.default_corpus_id, "corpus-01-adna-damage-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 7);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.sex"
                    && node.upstream_inputs == vec!["coverage_report_json"]
                    && node.external_inputs == vec!["corpus.aligned_bam", "sex_reference_contract"]
            }),
            "bam.sex must stay explicit as a coverage-driven diagnostic branch"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.authenticity"
                    && node.upstream_inputs
                        == vec![
                            "mapping_summary_report_json",
                            "coverage_report_json",
                            "damage_report_json",
                            "contamination_report_json",
                            "complexity_report_json",
                        ]
                    && node.depends_on
                        == vec![
                            "bam.mapping_summary",
                            "bam.coverage",
                            "bam.damage",
                            "bam.contamination",
                            "bam.complexity",
                        ]
            }),
            "bam.authenticity must depend only on the governed metrics it currently consumes"
        );
        assert!(
            report.nodes.iter().all(|node| {
                node.stage_id != "bam.authenticity"
                    || !node.upstream_inputs.iter().any(|input| input == "sex_report_json")
            }),
            "bam.authenticity must not claim bam.sex as a required upstream metric"
        );
    }

    #[test]
    fn bam_genotyping_pipeline_dag_tracks_recalibration_skip_and_run_handoffs() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/bam-genotyping.toml");
        let output_path = repo_root.join(
            "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-genotyping.json",
        );
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate bam genotyping local pipeline dag");

        assert_eq!(report.pipeline_id, "bam-genotyping");
        assert_eq!(report.domain, "bam");
        assert_eq!(report.default_corpus_id, "corpus-01-bam-mini");
        assert_eq!(report.node_count, 4);
        assert_eq!(report.edge_count, 5);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.recalibration"
                    && node.upstream_inputs == vec!["filtered_bam", "coverage_report_json"]
                    && node.outputs
                        == vec![
                            "recalibrated_bam",
                            "recalibrated_bai",
                            "recalibration_report_text",
                            "recalibration_summary_json",
                            "recalibration_stage_metrics",
                        ]
            }),
            "bam.recalibration must expose the coverage-gated decision summary plus recalibrated BAM outputs"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.genotyping"
                    && node.readiness_kind == "dry_or_smoke"
                    && node.depends_on == vec!["bam.filter", "bam.recalibration"]
                    && node.upstream_inputs
                        == vec![
                            "filtered_bam",
                            "filtered_bai",
                            "recalibrated_bam",
                            "recalibrated_bai",
                            "recalibration_summary_json",
                        ]
                    && node.external_inputs
                        == vec![
                            "genotyping_reference_contract",
                            "candidate_sites_vcf_contract",
                            "target_regions_contract",
                        ]
            }),
            "bam.genotyping must expose both the filtered-BAM skip path and recalibrated-BAM run path"
        );
    }

    #[test]
    fn bam_kinship_pipeline_dag_keeps_overlap_insufficiency_local_to_kinship() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/bam-kinship.toml");
        let output_path = repo_root
            .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-kinship.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate bam kinship local pipeline dag");

        assert_eq!(report.pipeline_id, "bam-kinship");
        assert_eq!(report.domain, "bam");
        assert_eq!(report.default_corpus_id, "corpus-01-bam-mini");
        assert_eq!(report.node_count, 4);
        assert_eq!(report.edge_count, 4);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.overlap_correction"
                    && node.readiness_kind == "dry_or_smoke"
                    && node.upstream_inputs == vec!["filtered_bam", "filtered_bai"]
                    && node.outputs
                        == vec![
                            "overlap_corrected_bam",
                            "overlap_corrected_bai",
                            "overlap_correction_summary_json",
                            "overlap_correction_stage_metrics",
                        ]
            }),
            "bam.overlap_correction must expose the overlap-sufficiency handoff needed only by kinship"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.kinship"
                    && node.depends_on == vec!["bam.overlap_correction", "bam.genotyping"]
                    && node.upstream_inputs
                        == vec![
                            "overlap_corrected_bam",
                            "overlap_corrected_bai",
                            "overlap_correction_summary_json",
                            "genotyping_report_json",
                        ]
                    && node.external_inputs
                        == vec![
                            "kinship_reference_panel_contract",
                            "kinship_population_scope_contract",
                            "kinship_pairing_contract",
                        ]
            }),
            "bam.kinship must consume overlap and genotype-readiness handoffs before pairwise inference"
        );
        assert!(
            report.nodes.iter().all(|node| {
                node.stage_id == "bam.kinship"
                    || !node
                        .upstream_inputs
                        .iter()
                        .any(|input| input == "overlap_correction_summary_json")
            }),
            "overlap insufficiency must stay local to bam.kinship instead of blocking unrelated BAM stages"
        );
    }

    #[test]
    fn fastq_to_bam_pipeline_dag_maps_trimmed_fastq_paths_into_bam_alignment() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/fastq-to-bam.toml");
        let output_path = repo_root
            .join("artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-to-bam.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate fastq-to-bam local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-to-bam");
        assert_eq!(report.domain, "cross");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert_eq!(report.node_count, 6);
        assert_eq!(report.edge_count, 7);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.trim_reads"
                    && node.outputs
                        == vec!["trimmed_reads_r1_path", "trimmed_reads_r2_path", "trim_metrics"]
            }),
            "fastq.trim_reads must emit explicit R1 and R2 path outputs for cross-domain alignment"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.align"
                    && node.readiness_kind == "dry_or_smoke"
                    && node.depends_on == vec!["fastq.trim_reads"]
                    && node.upstream_inputs
                        == vec!["trimmed_reads_r1_path", "trimmed_reads_r2_path"]
                    && node.external_inputs
                        == vec![
                            "alignment_reference_fasta_contract",
                            "alignment_reference_index_contract",
                            "alignment_read_group_contract",
                        ]
            }),
            "bam.align must consume the governed trimmed FASTQ path handoff and alignment contracts"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.mapping_summary"
                    && node.depends_on == vec!["bam.align", "bam.qc_pre"]
                    && node.upstream_inputs
                        == vec!["align_bam", "qc_pre_flagstat", "qc_pre_idxstats", "qc_pre_stats"]
            }),
            "bam.mapping_summary must consume both alignment output and BAM pre-QC context"
        );
    }

    #[test]
    fn adna_equus_caballus_pipeline_bundle_tracks_short_read_and_equcab3_handoffs() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/adna-equus-caballus-fastq-bam-vcf");
        let output_path = repo_root.join(
            "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/adna-equus-caballus-fastq-bam-vcf.json",
        );
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate horse aDNA local pipeline dag");

        assert_eq!(report.pipeline_id, "adna-equus-caballus-fastq-bam-vcf");
        assert_eq!(report.domain, "cross");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert!(report.acyclic);
        assert_eq!(
            report.reference_context.as_ref().map(|context| {
                (
                    context.species_id.as_str(),
                    context.build_id.as_str(),
                    context.reference_bundle_id.as_str(),
                )
            }),
            Some(("Equus caballus", "EquCab3", "ecaballus_equcab3_primary"))
        );
        assert_eq!(
            report.execution_context.as_ref().map(|context| {
                (
                    context.site_profile_id.as_str(),
                    context.default_campaign_config.as_deref(),
                    context.frontend_surfaces.clone(),
                    context.compute_surfaces.clone(),
                )
            }),
            Some((
                "lunarc",
                Some("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-vcf-local-ready.toml"),
                vec!["downloads".to_string(), "ref-prep".to_string(), "snp-prep".to_string(),],
                vec![
                    "fastq-qc".to_string(),
                    "fastq-to-bam".to_string(),
                    "bam-qc".to_string(),
                    "bam-to-vcf".to_string(),
                ],
            ))
        );
        assert_eq!(report.project_source_count, 5);
        assert_eq!(
            report.ancient_dna_policy.as_ref().map(|policy| {
                (
                    policy.aligner_mode.as_str(),
                    policy.auto_bwa_aln_p75_threshold_bp,
                    policy.duplicate_stage_id.as_str(),
                    policy.duplicate_tool.as_str(),
                )
            }),
            Some(("auto", 35, "bam.markdup", "samtools"))
        );
        assert_eq!(
            report.reference_assets.as_ref().map(|assets| {
                (
                    assets.directory_name.as_str(),
                    assets.genome_filename.as_str(),
                    assets.assembly_report_filename.as_str(),
                )
            }),
            Some((
                "ref_EquCab3",
                "GCF_002863925.1_EquCab3.0_genomic.fna.gz",
                "GCF_002863925.1_EquCab3.0_assembly_report.txt",
            ))
        );
        assert_eq!(
            report.variant_assets.as_ref().map(|assets| {
                (
                    assets.directory_name.as_str(),
                    assets.source_vcf_filename.as_str(),
                    assets.refseq_vcf_filename.as_str(),
                    assets.annotated_vcf_filename.as_str(),
                    assets.snpeff.genome_id.as_str(),
                )
            }),
            Some((
                "snp_EquCab3",
                "equus_caballus.vcf.gz",
                "equus_caballus_EquCab3_refseq.vcf.gz",
                "equus_caballus_EquCab3_annotated.vcf.gz",
                "EquCab3.0.99",
            ))
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.align"
                    && node.depends_on == vec!["fastq.profile_read_lengths", "fastq.filter_reads"]
                    && node.upstream_inputs
                        == vec![
                            "filtered_merged_reads",
                            "filtered_unmerged_r1_reads",
                            "filtered_unmerged_r2_reads",
                            "read_length_metrics",
                        ]
                    && node.external_inputs
                        == vec![
                            "equcab3_reference_fasta_contract",
                            "equcab3_reference_index_contract",
                            "adna_short_read_alignment_policy_contract",
                            "alignment_read_group_contract",
                        ]
            }),
            "bam.align must consume filtered merge-path reads plus the pre-trim read-length profile"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.markdup"
                    && node.depends_on == vec!["bam.align"]
                    && node.upstream_inputs == vec!["aligned_bam", "aligned_bai", "align_metrics"]
            }),
            "bam.markdup must stay downstream of alignment so duplicate handling remains post-alignment"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "vcf.call"
                    && node.depends_on
                        == vec!["bam.markdup", "bam.coverage", "bam.damage", "bam.duplication_metrics"]
                    && node.external_inputs
                        == vec![
                            "equcab3_reference_fasta_contract",
                            "equcab3_reference_fai_contract",
                            "equcab3_candidate_sites_contract",
                        ]
                    && node.upstream_inputs
                        == vec![
                            "markdup_bam",
                            "markdup_bai",
                            "coverage_report_json",
                            "damage_report_json",
                            "duplication_report_json",
                        ]
            }),
            "vcf.call must stay tied to the EquCab3 reference contracts, post-alignment deduplication, and BAM damage evidence"
        );
    }
}
