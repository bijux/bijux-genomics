use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::commands::benchmark::local_pipeline_dag::{
    load_validated_local_pipeline_dag_report, LocalPipelineAncientDnaPolicy,
    LocalPipelineExecutionContext, LocalPipelineProjectSource, LocalPipelineReferenceAssets,
    LocalPipelineReferenceContext, LocalPipelineVariantAssets,
};

const PIPELINE_OPERATIONS_REPORT_SCHEMA_VERSION: &str = "bijux.hpc.pipeline_operations.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PipelineOperationsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) pipeline_id: String,
    pub(crate) summary: String,
    pub(crate) pipeline_config_path: String,
    pub(crate) operations_root: String,
    pub(crate) campaign_config_path: String,
    pub(crate) reference_context: Option<LocalPipelineReferenceContext>,
    pub(crate) execution_context: Option<LocalPipelineExecutionContext>,
    pub(crate) project_sources: Vec<LocalPipelineProjectSource>,
    pub(crate) ancient_dna_policy: Option<LocalPipelineAncientDnaPolicy>,
    pub(crate) reference_assets: Option<LocalPipelineReferenceAssets>,
    pub(crate) variant_assets: Option<LocalPipelineVariantAssets>,
    pub(crate) project_downloads: Vec<ProjectDownloadPlan>,
    pub(crate) reference_materialization: Option<AssetMaterializationPlan>,
    pub(crate) variant_materialization: Option<AssetMaterializationPlan>,
    pub(crate) commands: Vec<PipelineOperationCommand>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectDownloadPlan {
    pub(crate) project_id: String,
    pub(crate) metadata_url: String,
    pub(crate) snapshot_path: String,
    pub(crate) raw_dir: String,
    pub(crate) select_command: String,
    pub(crate) fetch_command: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AssetMaterializationPlan {
    pub(crate) surface: String,
    pub(crate) root_path: String,
    pub(crate) files: Vec<AssetFilePlan>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AssetFilePlan {
    pub(crate) role: String,
    pub(crate) target_path: String,
    pub(crate) source_url: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PipelineOperationCommand {
    pub(crate) operation_id: String,
    pub(crate) surface: String,
    pub(crate) description: String,
    pub(crate) command: String,
}

pub(crate) fn pipeline_operations_report(
    repo_root: &Path,
    args: &crate::commands::cli::PipelineOperationsReportArgs,
) -> Result<PipelineOperationsReport> {
    render_pipeline_operations_report(
        repo_root,
        &resolve_candidate(repo_root, &args.config),
        args.operations_root.as_deref(),
        args.campaign_config.as_deref(),
    )
}

pub(crate) fn render_pipeline_operations_report(
    repo_root: &Path,
    config_path: &Path,
    operations_root: Option<&Path>,
    campaign_config_override: Option<&Path>,
) -> Result<PipelineOperationsReport> {
    let pipeline = load_validated_local_pipeline_dag_report(repo_root, config_path)?;
    let operations_root =
        operations_root.map(|path| resolve_candidate(repo_root, path)).unwrap_or_else(|| {
            repo_root.join("artifacts/pipeline-operations").join(&pipeline.pipeline_id)
        });

    let campaign_config_path = resolve_campaign_config_path(
        repo_root,
        pipeline.execution_context.as_ref(),
        campaign_config_override,
    )?;

    let project_downloads = build_project_downloads(
        pipeline.reference_context.as_ref(),
        &pipeline.project_sources,
        &operations_root,
    )?;
    let reference_materialization = pipeline
        .reference_assets
        .as_ref()
        .map(|assets| build_reference_materialization(&operations_root, assets));
    let variant_materialization = pipeline
        .variant_assets
        .as_ref()
        .map(|assets| build_variant_materialization(&operations_root, assets));
    let commands = build_commands(
        config_path,
        &operations_root,
        &project_downloads,
        reference_materialization.as_ref(),
        variant_materialization.as_ref(),
        &campaign_config_path,
    );

    Ok(PipelineOperationsReport {
        schema_version: PIPELINE_OPERATIONS_REPORT_SCHEMA_VERSION,
        pipeline_id: pipeline.pipeline_id,
        summary: pipeline.summary,
        pipeline_config_path: config_path.display().to_string(),
        operations_root: operations_root.display().to_string(),
        campaign_config_path: campaign_config_path.display().to_string(),
        reference_context: pipeline.reference_context,
        execution_context: pipeline.execution_context,
        project_sources: pipeline.project_sources,
        ancient_dna_policy: pipeline.ancient_dna_policy,
        reference_assets: pipeline.reference_assets,
        variant_assets: pipeline.variant_assets,
        project_downloads,
        reference_materialization,
        variant_materialization,
        commands,
    })
}

fn build_project_downloads(
    reference_context: Option<&LocalPipelineReferenceContext>,
    project_sources: &[LocalPipelineProjectSource],
    operations_root: &Path,
) -> Result<Vec<ProjectDownloadPlan>> {
    if project_sources.is_empty() {
        return Ok(Vec::new());
    }
    let species = reference_context
        .map(|context| context.species_id.as_str())
        .ok_or_else(|| anyhow!("pipeline operations report requires reference_context when project_sources are present"))?;
    Ok(project_sources
        .iter()
        .map(|source| {
            let download_root = operations_root.join("downloads").join(&source.project_id);
            let snapshot_path = download_root.join("ENA_METADATA.snapshot.json");
            let raw_dir = download_root.join("raw");
            ProjectDownloadPlan {
                project_id: source.project_id.clone(),
                metadata_url: source.metadata_url.clone(),
                snapshot_path: snapshot_path.display().to_string(),
                raw_dir: raw_dir.display().to_string(),
                select_command: format!(
                    "bijux-dna ena select --project {} --species {} --corpus-id {} --out {}",
                    shell_quote(&source.project_id),
                    shell_quote(species),
                    shell_quote(&source.project_id),
                    shell_quote_path(&snapshot_path),
                ),
                fetch_command: format!(
                    "bijux-dna ena fetch --species {} --snapshot {} --out {}",
                    shell_quote(species),
                    shell_quote_path(&snapshot_path),
                    shell_quote_path(&raw_dir),
                ),
            }
        })
        .collect())
}

fn build_reference_materialization(
    operations_root: &Path,
    assets: &LocalPipelineReferenceAssets,
) -> AssetMaterializationPlan {
    let root = operations_root.join("reference").join(&assets.directory_name);
    let genome_path = root.join(&assets.genome_filename);
    let assembly_report_path = root.join(&assets.assembly_report_filename);
    AssetMaterializationPlan {
        surface: "ref-prep".to_string(),
        root_path: root.display().to_string(),
        files: vec![
            remote_asset_file("reference_fasta_gz", &genome_path, &assets.genome_url),
            remote_asset_file(
                "assembly_report",
                &assembly_report_path,
                &assets.assembly_report_url,
            ),
        ],
    }
}

fn build_variant_materialization(
    operations_root: &Path,
    assets: &LocalPipelineVariantAssets,
) -> AssetMaterializationPlan {
    let root = operations_root.join("variants").join(&assets.directory_name);
    let source_vcf_path = root.join(&assets.source_vcf_filename);
    let refseq_vcf_path = root.join(&assets.refseq_vcf_filename);
    let annotated_vcf_path = root.join(&assets.annotated_vcf_filename);
    let snpeff_db_path = root.join(&assets.snpeff.database_filename);
    let snpeff_core_path = root.join(&assets.snpeff.core_filename);
    AssetMaterializationPlan {
        surface: "snp-prep".to_string(),
        root_path: root.display().to_string(),
        files: vec![
            remote_asset_file("source_vcf_gz", &source_vcf_path, &assets.source_vcf_url),
            remote_asset_file(
                "snpeff_database_zip",
                &snpeff_db_path,
                &assets.snpeff.database_url,
            ),
            remote_asset_file("snpeff_core_zip", &snpeff_core_path, &assets.snpeff.core_url),
            derived_asset_file(
                "refseq_normalized_vcf_gz",
                &refseq_vcf_path,
                "derive this file from source_vcf_gz after EquCab3 contig remapping and normalization",
            ),
            derived_asset_file(
                "annotated_vcf_gz",
                &annotated_vcf_path,
                "derive this file after snpEff annotation using the downloaded source VCF and snpEff assets",
            ),
        ],
    }
}

fn build_commands(
    config_path: &Path,
    operations_root: &Path,
    project_downloads: &[ProjectDownloadPlan],
    reference_materialization: Option<&AssetMaterializationPlan>,
    variant_materialization: Option<&AssetMaterializationPlan>,
    campaign_config_path: &Path,
) -> Vec<PipelineOperationCommand> {
    let mut commands = Vec::new();
    for project in project_downloads {
        commands.push(PipelineOperationCommand {
            operation_id: format!("downloads:{}:select", project.project_id),
            surface: "downloads".to_string(),
            description: format!("select governed ENA runs for {}", project.project_id),
            command: project.select_command.clone(),
        });
        commands.push(PipelineOperationCommand {
            operation_id: format!("downloads:{}:fetch", project.project_id),
            surface: "downloads".to_string(),
            description: format!("fetch governed FASTQ files for {}", project.project_id),
            command: project.fetch_command.clone(),
        });
    }
    extend_asset_commands(&mut commands, config_path, operations_root, reference_materialization);
    extend_asset_commands(&mut commands, config_path, operations_root, variant_materialization);
    let campaign_config = shell_quote_path(campaign_config_path);
    commands.push(PipelineOperationCommand {
        operation_id: "campaign:preparation-graph".to_string(),
        surface: "campaign".to_string(),
        description: "inspect campaign foundation dependencies".to_string(),
        command: format!("bijux-dna config preparation-graph --config {campaign_config} --json"),
    });
    commands.push(PipelineOperationCommand {
        operation_id: "campaign:prepare-foundation".to_string(),
        surface: "campaign".to_string(),
        description: "prepare governed campaign roots before submission".to_string(),
        command: format!(
            "bijux-dna config prepare-foundation --config {campaign_config} --dry-run --json"
        ),
    });
    commands.push(PipelineOperationCommand {
        operation_id: "campaign:dry-run".to_string(),
        surface: "campaign".to_string(),
        description: "inspect planned Lunarc jobs before submission".to_string(),
        command: format!("bijux-dna config campaign-dry-run --config {campaign_config} --json"),
    });
    commands.push(PipelineOperationCommand {
        operation_id: "campaign:submit".to_string(),
        surface: "campaign".to_string(),
        description: "render mock submission metadata for the full campaign".to_string(),
        command: format!(
            "bijux-dna slurm submit-campaign --config {campaign_config} --mock-submit --json"
        ),
    });
    commands
}

fn extend_asset_commands(
    commands: &mut Vec<PipelineOperationCommand>,
    config_path: &Path,
    operations_root: &Path,
    plan: Option<&AssetMaterializationPlan>,
) {
    let Some(plan) = plan else {
        return;
    };
    commands.push(PipelineOperationCommand {
        operation_id: format!("{}:materialize", plan.surface),
        surface: plan.surface.clone(),
        description: format!("materialize {} assets", plan.surface),
        command: format!(
            "bijux-dna config materialize-pipeline-assets --config {} --operations-root {} --surface {} --json",
            shell_quote_path(config_path),
            shell_quote_path(operations_root),
            shell_quote(&plan.surface),
        ),
    });
}

fn remote_asset_file(role: &str, target_path: &Path, source_url: &str) -> AssetFilePlan {
    AssetFilePlan {
        role: role.to_string(),
        target_path: target_path.display().to_string(),
        source_url: Some(source_url.to_string()),
        command: Some(format!(
            "mkdir -p {} && curl -L --fail -o {} {}",
            shell_quote_path(
                target_path
                    .parent()
                    .expect("remote asset target path must have a parent directory")
            ),
            shell_quote_path(target_path),
            shell_quote(source_url),
        )),
        note: None,
    }
}

fn derived_asset_file(role: &str, target_path: &Path, note: &str) -> AssetFilePlan {
    AssetFilePlan {
        role: role.to_string(),
        target_path: target_path.display().to_string(),
        source_url: None,
        command: None,
        note: Some(note.to_string()),
    }
}

fn resolve_campaign_config_path(
    repo_root: &Path,
    execution_context: Option<&LocalPipelineExecutionContext>,
    campaign_config_override: Option<&Path>,
) -> Result<PathBuf> {
    let candidate = if let Some(path) = campaign_config_override {
        resolve_candidate(repo_root, path)
    } else {
        let default_campaign_config = execution_context
            .and_then(|context| context.default_campaign_config.as_deref())
            .ok_or_else(|| anyhow!("pipeline operations report requires --campaign-config or execution_context.default_campaign_config"))?;
        resolve_candidate(repo_root, Path::new(default_campaign_config))
    };
    if !candidate.is_file() {
        return Err(anyhow!(
            "pipeline operations report campaign config `{}` is missing",
            candidate.display()
        ));
    }
    Ok(candidate)
}

fn resolve_candidate(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn shell_quote(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', "'\"'\"'"))
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote(&path.display().to_string())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::render_pipeline_operations_report;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates root")
            .parent()
            .expect("repo root")
            .to_path_buf()
    }

    #[test]
    fn horse_pipeline_operations_report_tracks_equcab3_assets_and_lunarc_commands() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/adna-equus-caballus-fastq-bam-vcf");
        let operations_root = repo_root.join("artifacts/tests/pipeline-operations");
        let report = render_pipeline_operations_report(
            &repo_root,
            &config_path,
            Some(&operations_root),
            None,
        )
        .expect("render horse pipeline operations report");

        assert_eq!(report.pipeline_id, "adna-equus-caballus-fastq-bam-vcf");
        assert_eq!(
            report.campaign_config_path,
            repo_root
                .join("benchmarks/configs/hpc/campaign/lunarc-fastq-bam-vcf-local-ready.toml")
                .display()
                .to_string()
        );
        assert_eq!(report.project_downloads.len(), 5);
        assert!(report.project_downloads.iter().any(|project| {
            project.project_id == "PRJEB22390"
                && project.select_command.contains("bijux-dna ena select --project 'PRJEB22390'")
                && project.fetch_command.contains("bijux-dna ena fetch")
        }));
        let reference_materialization =
            report.reference_materialization.as_ref().expect("reference materialization");
        assert_eq!(reference_materialization.surface, "ref-prep");
        assert!(reference_materialization.files.iter().any(|file| file.role
            == "reference_fasta_gz"
            && file.target_path.ends_with("ref_EquCab3/GCF_002863925.1_EquCab3.0_genomic.fna.gz")));
        let variant_materialization =
            report.variant_materialization.as_ref().expect("variant materialization");
        assert!(variant_materialization.files.iter().any(|file| {
            file.role == "snpeff_database_zip"
                && file
                    .source_url
                    .as_deref()
                    == Some(
                        "https://snpeff-public.s3.amazonaws.com/databases/v5_0/snpEff_v5_0_EquCab3.0.99.zip"
                    )
        }));
        assert!(report.commands.iter().any(|command| {
            command.operation_id == "ref-prep:materialize"
                && command.command.contains("bijux-dna config materialize-pipeline-assets --config")
                && command.command.contains("--surface 'ref-prep' --json")
        }));
        assert!(report.commands.iter().any(|command| {
            command.operation_id == "campaign:submit"
                && command.command.contains("bijux-dna slurm submit-campaign --config")
                && command.command.contains("--mock-submit --json")
        }));
    }
}
