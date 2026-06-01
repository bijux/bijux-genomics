use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_fastq::params::qc_post::{QcAggregationEngine, QcAggregationScope};
use bijux_dna_domain_fastq::{
    derived_governed_qc_lineage_hash, GovernedQcContributorV1, GovernedQcInputsManifestV1,
    GovernedQcManifestContributorV1, ReportQcReportV1, GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
    REPORT_QC_REPORT_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_STAGE_COMMAND_SCRIPT_SCHEMA_VERSION: &str = "bijux.bench.local_stage_commands.v1";
const DEFAULT_RENDERED_STAGE_COMMANDS_PATH: &str = "target/local-ready/rendered-stage-commands.sh";
const LOCAL_REPORT_QC_CONFIG_PATH: &str = "configs/bench/local/fastq-report-qc.toml";
const LOCAL_REPORT_QC_CONFIG_SCHEMA_VERSION: &str = "bijux.bench.fastq.local_report_qc.v1";
const DEFAULT_LOCAL_REPORT_QC_OUTPUT_DIR: &str = "target/local-smoke/fastq.report_qc";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageCommandEntry {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: LocalStageReadinessKind,
    pub(crate) command: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageCommandScript {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) command_count: usize,
    pub(crate) commands: Vec<BenchLocalStageCommandEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageMaterialization {
    pub(crate) stage_id: String,
    pub(crate) artifact_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalReportQcConfig {
    schema_version: String,
    tool_id: String,
    fixture_root: PathBuf,
    report_template: PathBuf,
    manifest_template: PathBuf,
    multiqc_report: PathBuf,
    multiqc_data_dir: PathBuf,
    aggregation_engine: Option<String>,
    aggregation_scope: Option<String>,
    output_dir: Option<PathBuf>,
}

pub(crate) fn run_materialize_stage(args: &parse::BenchLocalMaterializeStageArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let artifact_path = materialize_local_stage(&repo_root, &args.stage_id)?;
    if args.json {
        render::json::print_pretty(&BenchLocalStageMaterialization {
            stage_id: args.stage_id.clone(),
            artifact_path: path_relative_to_repo(&repo_root, &artifact_path),
        })?;
    } else {
        println!("{}", path_relative_to_repo(&repo_root, &artifact_path));
    }
    Ok(())
}

pub(crate) fn run_render_stage_commands(
    args: &parse::BenchLocalRenderStageCommandsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let script = render_local_stage_commands(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_RENDERED_STAGE_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&script)?;
    } else {
        println!("{}", script.output_path);
    }
    Ok(())
}

pub(crate) fn materialize_local_stage(repo_root: &Path, stage_id: &str) -> Result<PathBuf> {
    match stage_id {
        "fastq.cluster_otus" => {
            bijux_dna_api::v1::api::fastq::write_local_cluster_otus_smoke_report()
        }
        "fastq.correct_errors" => {
            bijux_dna_api::v1::api::fastq::write_local_correct_errors_smoke_plan()
        }
        "fastq.deplete_host" => bijux_dna_api::v1::api::fastq::write_local_deplete_host_plan(),
        "fastq.deplete_reference_contaminants" => {
            bijux_dna_api::v1::api::fastq::write_local_deplete_reference_contaminants_plan()
        }
        "fastq.deplete_rrna" => bijux_dna_api::v1::api::fastq::write_local_deplete_rrna_plan(),
        "fastq.detect_adapters" => {
            bijux_dna_api::v1::api::fastq::write_local_detect_adapters_smoke_report()
        }
        "fastq.detect_duplicates_premerge" => {
            bijux_dna_api::v1::api::fastq::write_local_detect_duplicates_premerge_smoke_report()
        }
        "fastq.estimate_library_complexity_prealign" => {
            bijux_dna_api::v1::api::fastq::write_local_estimate_library_complexity_prealign_smoke_report(
            )
        }
        "fastq.extract_umis" => {
            bijux_dna_api::v1::api::fastq::write_local_extract_umis_smoke_report()
        }
        "fastq.filter_low_complexity" => {
            bijux_dna_api::v1::api::fastq::write_local_filter_low_complexity_smoke_report()
        }
        "fastq.filter_reads" => {
            bijux_dna_api::v1::api::fastq::write_local_filter_reads_smoke_report()
        }
        "fastq.index_reference" => {
            bijux_dna_api::v1::api::fastq::write_local_index_reference_plan()
        }
        "fastq.infer_asvs" => {
            bijux_dna_api::v1::api::fastq::write_local_infer_asvs_smoke_report()
        }
        "fastq.merge_pairs" => {
            bijux_dna_api::v1::api::fastq::write_local_merge_pairs_smoke_report()
        }
        "fastq.normalize_abundance" => {
            bijux_dna_api::v1::api::fastq::write_local_normalize_abundance_smoke_report()
        }
        "fastq.normalize_primers" => {
            bijux_dna_api::v1::api::fastq::write_local_normalize_primers_smoke_report()
        }
        "fastq.profile_overrepresented_sequences" => bijux_dna_api::v1::api::fastq::write_local_profile_overrepresented_sequences_smoke_summary(),
        "fastq.profile_read_lengths" => {
            bijux_dna_api::v1::api::fastq::write_local_profile_read_lengths_smoke_summary()
        }
        "fastq.profile_reads" => {
            bijux_dna_api::v1::api::fastq::write_local_profile_reads_smoke_report()
        }
        "fastq.remove_chimeras" => {
            bijux_dna_api::v1::api::fastq::write_local_remove_chimeras_smoke_report()
        }
        "fastq.remove_duplicates" => {
            bijux_dna_api::v1::api::fastq::write_local_remove_duplicates_smoke_report()
        }
        "fastq.report_qc" => materialize_local_report_qc_smoke_report(repo_root),
        "fastq.screen_taxonomy" => {
            bijux_dna_api::v1::api::fastq::write_local_screen_taxonomy_plan()
        }
        "fastq.trim_polyg_tails" => {
            bijux_dna_api::v1::api::fastq::write_local_trim_polyg_tails_smoke_report()
        }
        "fastq.trim_reads" => {
            bijux_dna_api::v1::api::fastq::write_local_trim_reads_smoke_report()
        }
        "fastq.trim_terminal_damage" => {
            bijux_dna_api::v1::api::fastq::write_local_trim_terminal_damage_smoke_report()
        }
        "fastq.validate_reads" => {
            bijux_dna_api::v1::api::fastq::write_local_validate_reads_smoke_report()
        }
        "bam.align" => bijux_dna_api::v1::api::bam::write_local_align_plan(),
        "bam.authenticity" => {
            bijux_dna_api::v1::api::bam::write_local_authenticity_smoke_report()
        }
        "bam.complexity" => {
            bijux_dna_api::v1::api::bam::write_local_complexity_smoke_report()
        }
        "bam.contamination" => bijux_dna_api::v1::api::bam::write_local_contamination_plan(),
        "bam.coverage" => bijux_dna_api::v1::api::bam::write_local_coverage_smoke_summary(),
        "bam.damage" => bijux_dna_api::v1::api::bam::write_local_damage_smoke_report(),
        "bam.duplication_metrics" => {
            bijux_dna_api::v1::api::bam::write_local_duplication_metrics_smoke_report()
        }
        "bam.endogenous_content" => {
            bijux_dna_api::v1::api::bam::write_local_endogenous_content_smoke_report()
        }
        "bam.filter" => bijux_dna_api::v1::api::bam::write_local_filter_smoke_report(),
        "bam.gc_bias" => bijux_dna_api::v1::api::bam::write_local_gc_bias_smoke_summary(),
        "bam.insert_size" => {
            bijux_dna_api::v1::api::bam::write_local_insert_size_smoke_report()
        }
        "bam.length_filter" => {
            bijux_dna_api::v1::api::bam::write_local_length_filter_smoke_report()
        }
        "bam.mapping_summary" => {
            bijux_dna_api::v1::api::bam::write_local_mapping_summary_smoke_summary()
        }
        "bam.mapq_filter" => {
            bijux_dna_api::v1::api::bam::write_local_mapq_filter_smoke_report()
        }
        "bam.markdup" => bijux_dna_api::v1::api::bam::write_local_markdup_smoke_report(),
        "bam.overlap_correction" => {
            bijux_dna_api::v1::api::bam::write_local_overlap_correction_smoke_report()
        }
        "bam.qc_pre" => bijux_dna_api::v1::api::bam::write_local_qc_pre_smoke_report(),
        "bam.recalibration" => {
            bijux_dna_api::v1::api::bam::write_local_recalibration_smoke_report()
        }
        "bam.sex" => bijux_dna_api::v1::api::bam::write_local_sex_smoke_report(),
        "bam.validate" => bijux_dna_api::v1::api::bam::write_local_validate_smoke_report(),
        other => materialize_feature_gated_stage(other),
    }
}

#[cfg(feature = "bam_downstream")]
fn materialize_feature_gated_stage(stage_id: &str) -> Result<PathBuf> {
    match stage_id {
        "bam.bias_mitigation" => {
            bijux_dna_api::v1::api::bam::write_local_bias_mitigation_smoke_report()
        }
        "bam.genotyping" => bijux_dna_api::v1::api::bam::write_local_genotyping_plan(),
        "bam.haplogroups" => bijux_dna_api::v1::api::bam::write_local_haplogroups_plan(),
        "bam.kinship" => bijux_dna_api::v1::api::bam::write_local_kinship_smoke_report(),
        other => Err(anyhow!("unsupported local benchmark stage `{other}`")),
    }
}

#[cfg(not(feature = "bam_downstream"))]
fn materialize_feature_gated_stage(stage_id: &str) -> Result<PathBuf> {
    match stage_id {
        "bam.bias_mitigation" | "bam.genotyping" | "bam.haplogroups" | "bam.kinship" => Err(
            anyhow!(
                "stage `{stage_id}` requires the `bam_downstream` feature; rerun with `cargo run -p bijux-dna --features bam_downstream -- ...`"
            ),
        ),
        other => Err(anyhow!("unsupported local benchmark stage `{other}`")),
    }
}

pub(crate) fn render_local_stage_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BenchLocalStageCommandScript> {
    let fastq = load_local_stage_inventory(repo_root, BenchLocalDomain::Fastq)?;
    let bam = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?;
    let commands = fastq
        .stages
        .into_iter()
        .chain(bam.stages)
        .map(|stage| BenchLocalStageCommandEntry {
            command: format!(
                "cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id {}",
                stage.stage_id
            ),
            readiness_kind: stage.readiness_kind,
            stage_id: stage.stage_id,
        })
        .collect::<Vec<_>>();

    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut script = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    script.push_str("repo_root=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/../..\" && pwd)\"\n");
    script.push_str("cd \"$repo_root\"\n\n");
    for entry in &commands {
        script.push_str(&format!("# {}\n{}\n", entry.stage_id, entry.command));
    }
    fs::write(&absolute_output_path, script)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;

    Ok(BenchLocalStageCommandScript {
        schema_version: LOCAL_STAGE_COMMAND_SCRIPT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        command_count: commands.len(),
        commands,
    })
}

fn materialize_local_report_qc_smoke_report(repo_root: &Path) -> Result<PathBuf> {
    let config = load_local_report_qc_config(repo_root)?;
    if config.schema_version != LOCAL_REPORT_QC_CONFIG_SCHEMA_VERSION {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required",
            repo_root.join(LOCAL_REPORT_QC_CONFIG_PATH).display(),
            config.schema_version,
            LOCAL_REPORT_QC_CONFIG_SCHEMA_VERSION
        ));
    }
    if config.tool_id != "multiqc" {
        return Err(anyhow!(
            "local fastq.report_qc smoke requires tool_id `multiqc`, found `{}`",
            config.tool_id
        ));
    }

    let output_root = repo_root.join(
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_REPORT_QC_OUTPUT_DIR)),
    );
    let contributors_dir = output_root.join("contributors");
    let multiqc_data_dir = output_root.join("multiqc_data");
    fs::create_dir_all(&contributors_dir)
        .with_context(|| format!("create {}", contributors_dir.display()))?;
    fs::create_dir_all(&multiqc_data_dir)
        .with_context(|| format!("create {}", multiqc_data_dir.display()))?;

    let fixture_root = repo_root.join(&config.fixture_root);
    let manifest_template_path = repo_root.join(&config.manifest_template);
    let report_template_path = repo_root.join(&config.report_template);
    let multiqc_report_source = repo_root.join(&config.multiqc_report);
    let multiqc_data_source = repo_root.join(&config.multiqc_data_dir);

    if !fixture_root.is_dir() {
        return Err(anyhow!(
            "local fastq.report_qc fixture_root is missing: {}",
            fixture_root.display()
        ));
    }

    let raw_manifest = fs::read_to_string(&manifest_template_path)
        .with_context(|| format!("read {}", manifest_template_path.display()))?;
    let mut manifest: GovernedQcInputsManifestV1 = serde_json::from_str(&raw_manifest)
        .with_context(|| format!("parse {}", manifest_template_path.display()))?;
    if manifest.schema_version != GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "{} declares unsupported schema `{}`",
            manifest_template_path.display(),
            manifest.schema_version
        ));
    }

    for qc_input in &mut manifest.qc_inputs {
        let source = fixture_root.join(&qc_input.path);
        let file_name = source
            .file_name()
            .ok_or_else(|| anyhow!("fixture path has no file name: {}", source.display()))?;
        let destination = contributors_dir.join(file_name);
        copy_file(&source, &destination)?;
        qc_input.path = repo_relative_pathbuf(repo_root, &destination);
    }
    for contributor in &mut manifest.contributors {
        let source = fixture_root.join(&contributor.path);
        let file_name = source
            .file_name()
            .ok_or_else(|| anyhow!("fixture path has no file name: {}", source.display()))?;
        let destination = contributors_dir.join(file_name);
        copy_file(&source, &destination)?;
        contributor.path = repo_relative_pathbuf(repo_root, &destination);
    }
    manifest.lineage_hash = derived_governed_qc_lineage_hash(&manifest.contributors);

    let manifest_output_path = output_root.join("governed_qc_inputs_manifest.json");
    bijux_dna_infra::atomic_write_json(&manifest_output_path, &manifest)?;

    let multiqc_report_path = output_root.join("multiqc_report.html");
    copy_file(&multiqc_report_source, &multiqc_report_path)?;
    copy_dir_contents(&multiqc_data_source, &multiqc_data_dir)?;

    let raw_report = fs::read_to_string(&report_template_path)
        .with_context(|| format!("read {}", report_template_path.display()))?;
    let mut report: ReportQcReportV1 = serde_json::from_str(&raw_report)
        .with_context(|| format!("parse {}", report_template_path.display()))?;
    if report.schema_version != REPORT_QC_REPORT_SCHEMA_VERSION {
        return Err(anyhow!(
            "{} declares unsupported schema `{}`",
            report_template_path.display(),
            report.schema_version
        ));
    }
    report.tool_id = config.tool_id;
    report.aggregation_engine =
        parse_aggregation_engine(config.aggregation_engine.as_deref().unwrap_or("multiqc"))?;
    report.aggregation_scope = parse_aggregation_scope(
        config.aggregation_scope.as_deref().unwrap_or("governed_qc_artifacts"),
    )?;
    report.multiqc_report = Some(path_relative_to_repo(repo_root, &multiqc_report_path));
    report.multiqc_data = Some(path_relative_to_repo(repo_root, &multiqc_data_dir));
    report.governed_qc_inputs_manifest =
        Some(path_relative_to_repo(repo_root, &manifest_output_path));
    report.governed_qc_input_count = manifest.qc_inputs.len() as u64;
    report.governed_qc_contributor_stage_ids = contributor_stage_ids(&manifest.contributors);
    report.governed_qc_contributor_tool_ids = contributor_tool_ids(&manifest.contributors);
    report.governed_qc_contributors =
        manifest.contributors.iter().map(report_contributor_from_manifest).collect();
    report.governed_qc_lineage_hash = manifest.lineage_hash;

    let report_output_path = output_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&report_output_path, &report)?;
    Ok(report_output_path)
}

fn load_local_report_qc_config(repo_root: &Path) -> Result<LocalReportQcConfig> {
    let config_path = repo_root.join(LOCAL_REPORT_QC_CONFIG_PATH);
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

fn parse_aggregation_engine(value: &str) -> Result<QcAggregationEngine> {
    match value {
        "auto" | "multiqc" => Ok(QcAggregationEngine::Multiqc),
        other => Err(anyhow!("unsupported local fastq.report_qc aggregation_engine `{other}`")),
    }
}

fn parse_aggregation_scope(value: &str) -> Result<QcAggregationScope> {
    match value {
        "governed_qc_artifacts" => Ok(QcAggregationScope::GovernedQcArtifacts),
        other => Err(anyhow!("unsupported local fastq.report_qc aggregation_scope `{other}`")),
    }
}

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_file() {
        return Err(anyhow!("missing fixture file {}", source.display()));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::copy(source, destination).with_context(|| {
        format!("copy fixture {} -> {}", source.display(), destination.display())
    })?;
    Ok(())
}

fn copy_dir_contents(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_dir() {
        return Err(anyhow!("missing fixture directory {}", source.display()));
    }
    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry.with_context(|| format!("read entry in {}", source.display()))?;
        let entry_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().with_context(|| format!("stat {}", entry_path.display()))?.is_dir() {
            fs::create_dir_all(&destination_path)
                .with_context(|| format!("create {}", destination_path.display()))?;
            copy_dir_contents(&entry_path, &destination_path)?;
        } else {
            copy_file(&entry_path, &destination_path)?;
        }
    }
    Ok(())
}

fn contributor_stage_ids(contributors: &[GovernedQcManifestContributorV1]) -> Vec<String> {
    let mut stage_ids =
        contributors.iter().map(|contributor| contributor.stage_id.clone()).collect::<Vec<_>>();
    stage_ids.sort();
    stage_ids.dedup();
    stage_ids
}

fn contributor_tool_ids(contributors: &[GovernedQcManifestContributorV1]) -> Vec<String> {
    let mut tool_ids =
        contributors.iter().map(|contributor| contributor.tool_id.clone()).collect::<Vec<_>>();
    tool_ids.sort();
    tool_ids.dedup();
    tool_ids
}

fn report_contributor_from_manifest(
    contributor: &GovernedQcManifestContributorV1,
) -> GovernedQcContributorV1 {
    GovernedQcContributorV1 {
        contributor_id: contributor.contributor_id.clone(),
        stage_id: contributor.stage_id.clone(),
        tool_id: contributor.tool_id.clone(),
        artifact_id: contributor.artifact_id.clone(),
        artifact_role: contributor.artifact_role.as_str().to_string(),
        path: contributor.path.display().to_string(),
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn repo_relative_pathbuf(repo_root: &Path, path: &Path) -> PathBuf {
    PathBuf::from(path_relative_to_repo(repo_root, path))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        materialize_local_report_qc_smoke_report, render_local_stage_commands, BenchLocalDomain,
        DEFAULT_RENDERED_STAGE_COMMANDS_PATH,
    };
    use crate::commands::benchmark::local_stage_inventory::load_local_stage_inventory;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn rendered_local_stage_commands_cover_governed_51_stage_slice() {
        let root = repo_root();
        let rendered =
            render_local_stage_commands(&root, PathBuf::from(DEFAULT_RENDERED_STAGE_COMMANDS_PATH))
                .expect("render local stage commands");

        let fastq = load_local_stage_inventory(&root, BenchLocalDomain::Fastq)
            .expect("load FASTQ local stage inventory");
        let bam = load_local_stage_inventory(&root, BenchLocalDomain::Bam)
            .expect("load BAM local stage inventory");

        assert_eq!(rendered.command_count, fastq.stage_count + bam.stage_count);
        assert_eq!(rendered.commands.len(), 51);
        assert!(rendered.commands.iter().any(|entry| entry.stage_id == "fastq.report_qc"));
        assert!(rendered.commands.iter().all(|entry| {
            entry.command.contains("bench local materialize-stage")
                && entry.command.contains(&entry.stage_id)
        }));
    }

    #[test]
    fn local_report_qc_smoke_materialization_writes_governed_bundle() {
        let root = repo_root();
        let report_path = materialize_local_report_qc_smoke_report(&root)
            .expect("materialize local report_qc smoke report");

        assert_eq!(
            report_path.strip_prefix(&root).expect("relative report path").display().to_string(),
            "target/local-smoke/fastq.report_qc/report.json"
        );
        assert!(root
            .join("target/local-smoke/fastq.report_qc/governed_qc_inputs_manifest.json")
            .is_file());
        assert!(root.join("target/local-smoke/fastq.report_qc/multiqc_report.html").is_file());
        assert!(root
            .join("target/local-smoke/fastq.report_qc/multiqc_data/multiqc_data.json")
            .is_file());
    }
}
