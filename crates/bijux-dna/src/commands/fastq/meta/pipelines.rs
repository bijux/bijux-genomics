use crate::commands::fastq::api_bridge::resolve_profile_alias;
use crate::commands::support::prelude::{
    anyhow, cli, load_manifests, render, PipelinesCommand, Result,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const LOCAL_PIPELINE_DAG_VALIDATION_SET_SCHEMA_VERSION: &str =
    "bijux.bench.local_pipeline_dag_validation_set.v1";

#[derive(Debug, Clone, Serialize)]
struct LocalPipelineDagValidationSetReport {
    schema_version: &'static str,
    benchmark_root: String,
    config_root: String,
    output_path: String,
    pipeline_count: usize,
    valid_pipeline_count: usize,
    all_valid: bool,
    pipelines:
        Vec<crate::commands::benchmark::local_pipeline_dag::LocalPipelineDagValidationReport>,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn handle_pipelines_command(
    args: &crate::cli::PipelinesRootArgs,
    registry_path: &std::path::Path,
) -> Result<bool> {
    match &args.command {
        PipelinesCommand::List { domain, show_experimental } => {
            let profiles = bijux_dna_api::v1::api::plan::select_pipelines(
                domain.map(cli::parse::PipelineDomainArg::as_domain),
                *show_experimental,
            );
            for profile in profiles {
                println!(
                    "{}\t{}\t{}",
                    profile.id.as_str(),
                    profile.stability.as_str(),
                    profile.description
                );
            }
            Ok(true)
        }
        PipelinesCommand::Explain { id, explain_io } => {
            let profile = bijux_dna_api::v1::api::plan::select_pipelines(None, true)
                .into_iter()
                .find(|profile| profile.id.as_str() == id)
                .ok_or_else(|| anyhow!("unknown pipeline profile: {id}"))?;
            let io_graph = if *explain_io {
                let registry = load_manifests(registry_path)?;
                let nodes = profile
                    .capabilities
                    .required_stages
                    .iter()
                    .filter_map(|stage_id| {
                        let key = bijux_dna_api::v1::api::run::StageId::new((*stage_id).clone());
                        registry.stages().get(&key).map(|stage| {
                            serde_json::json!({
                                "stage_id": stage_id,
                                "semantic_kind": stage.semantic_kind,
                                "input_kind": stage.input_kind,
                                "output_kind": stage.output_kind,
                                "produced_artifacts": stage.produced_artifacts,
                                "consumes": stage.inputs.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                                "produces": stage.outputs.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                            })
                        })
                    })
                    .collect::<Vec<_>>();
                serde_json::Value::Array(nodes)
            } else {
                serde_json::Value::Null
            };
            let payload = serde_json::json!({
                "profile": profile,
                "defaults_ledger": profile.defaults_ledger(),
                "promised_outputs": profile.capabilities.produces_outputs,
                "report_sections": profile.capabilities.report_sections,
                "artifact_graph": io_graph,
            });
            render::json::print_pretty(&payload)?;
            Ok(true)
        }
        PipelinesCommand::ExplainProfile { id } => {
            let resolved_id = resolve_profile_alias(id);
            render::json::print_pretty(&bijux_dna_api::v1::api::plan::explain_pipeline_profile(
                resolved_id,
            )?)?;
            Ok(true)
        }
        PipelinesCommand::ValidateProfile { id } => {
            let resolved_id = resolve_profile_alias(id);
            render::json::print_pretty(&bijux_dna_api::v1::api::plan::validate_pipeline_profile(
                resolved_id,
            )?)?;
            Ok(true)
        }
        PipelinesCommand::Validate { id, all, benchmark_root, strict, output, json } => {
            let repo_root = std::env::current_dir()?;
            let benchmark_root = resolve_benchmark_root(&repo_root, benchmark_root.as_deref());
            if *all {
                let report = validate_all_local_pipelines(
                    &repo_root,
                    &benchmark_root,
                    *strict,
                    output.as_deref(),
                )?;
                if *json {
                    render::json::print_pretty(&report)?;
                } else {
                    println!("{}", report.output_path);
                }
            } else {
                let pipeline_id = id
                    .as_deref()
                    .ok_or_else(|| anyhow!("pipeline id is required unless `--all` is passed"))?;
                let config_path = local_pipeline_config_path(&benchmark_root, pipeline_id)?;
                let output_path = output.clone().unwrap_or_else(|| {
                    PathBuf::from("target/local-ready/pipeline-dag")
                        .join(format!("{pipeline_id}.json"))
                });
                let report =
                    crate::commands::benchmark::local_pipeline_dag::validate_pipeline_dag_path(
                        &repo_root,
                        &config_path,
                        &absolute_or_repo_relative(&repo_root, &output_path),
                    )?;

                if *strict {
                    validate_local_pipeline_identity(
                        &repo_root,
                        &report,
                        &config_path,
                        pipeline_id,
                    )?;
                }

                if *json {
                    render::json::print_pretty(&report)?;
                } else {
                    println!("{}", report.output_path);
                }
            }
            Ok(true)
        }
        PipelinesCommand::ProfileDiff { left, right } => {
            let left_id = resolve_profile_alias(left);
            let right_id = resolve_profile_alias(right);
            let profiles = bijux_dna_api::v1::api::plan::select_pipelines(None, true);
            let left_profile = profiles
                .iter()
                .find(|profile| profile.id.as_str() == left_id)
                .ok_or_else(|| anyhow!("unknown pipeline profile: {left}"))?;
            let right_profile = profiles
                .iter()
                .find(|profile| profile.id.as_str() == right_id)
                .ok_or_else(|| anyhow!("unknown pipeline profile: {right}"))?;
            let left_has_fastq = left_profile
                .capabilities
                .required_stages
                .iter()
                .any(|stage| stage.starts_with("fastq."));
            let left_has_vcf = left_profile
                .capabilities
                .required_stages
                .iter()
                .any(|stage| stage.starts_with("vcf."));
            let right_has_fastq = right_profile
                .capabilities
                .required_stages
                .iter()
                .any(|stage| stage.starts_with("fastq."));
            let right_has_vcf = right_profile
                .capabilities
                .required_stages
                .iter()
                .any(|stage| stage.starts_with("vcf."));
            let payload = serde_json::json!({
                "left": left_profile.id,
                "right": right_profile.id,
                "tools_left": left_profile.defaults.tools,
                "tools_right": right_profile.defaults.tools,
                "params_left": left_profile.defaults.params,
                "params_right": right_profile.defaults.params,
                "invariants_left": if left_has_fastq {
                    serde_json::to_value(bijux_dna_api::v1::api::plan::validate_fastq_profile(left_profile))?
                } else if left_has_vcf {
                    serde_json::to_value(bijux_dna_api::v1::api::plan::validate_vcf_profile(left_profile))?
                } else {
                    serde_json::to_value(bijux_dna_api::v1::api::plan::validate_bam_profile(left_profile))?
                },
                "invariants_right": if right_has_fastq {
                    serde_json::to_value(bijux_dna_api::v1::api::plan::validate_fastq_profile(right_profile))?
                } else if right_has_vcf {
                    serde_json::to_value(bijux_dna_api::v1::api::plan::validate_vcf_profile(right_profile))?
                } else {
                    serde_json::to_value(bijux_dna_api::v1::api::plan::validate_bam_profile(right_profile))?
                },
            });
            render::json::print_pretty(&payload)?;
            Ok(true)
        }
        PipelinesCommand::Audit { domain, show_experimental } => {
            let profiles = bijux_dna_api::v1::api::plan::select_pipelines(
                domain.map(cli::parse::PipelineDomainArg::as_domain),
                *show_experimental,
            );
            for profile in profiles {
                println!(
                    "{}\t{}\t{}",
                    profile.id.as_str(),
                    profile.stability.as_str(),
                    profile.description
                );
                let id_catalog = match profile.id.as_str() {
                    "fastq-to-fastq__default__v1" | "fastq-to-fastq__minimal__v1" => {
                        bijux_dna_api::v1::api::plan::fastq_pipeline_id_catalog(profile.id.as_str())
                    }
                    "fastq-to-bam__default__v1" | "fastq-to-bam__adna_shotgun__v1" => {
                        bijux_dna_api::v1::api::plan::cross_fastq_to_bam_id_catalog(
                            profile.id.as_str(),
                        )
                    }
                    "bam-to-vcf__default__v1" | "fastq-to-vcf__minimal__v1" => {
                        profile.capabilities.required_stages.clone()
                    }
                    "bam-to-bam__default__v1"
                    | "bam-to-bam__adna_shotgun__v1"
                    | "bam-to-bam__adna_capture__v1" => {
                        bijux_dna_api::v1::api::plan::bam_pipeline_id_catalog(profile.id.as_str())
                    }
                    _ => Vec::new(),
                };
                for stage_id in id_catalog {
                    if stage_id.starts_with("bam.") {
                        let stage =
                            bijux_dna_api::v1::api::bench::BamStage::try_from(stage_id.as_str())
                                .map_err(|_| anyhow!("unknown BAM stage {stage_id}"))?;
                        let completeness =
                            bijux_dna_api::v1::api::bench::bam_stage_completeness(stage);
                        println!(
                            "  {stage_id}\tcomplete={}\targs={}\tartifacts={}\tparsers={}\tinvariants={}",
                            completeness.is_complete(),
                            completeness.has_args_builder,
                            completeness.has_artifact_contract,
                            completeness.has_parser_fixtures,
                            completeness.has_invariants
                        );
                    } else {
                        println!("  {stage_id}\tcomplete=unknown");
                    }
                }
            }
            Ok(true)
        }
    }
}

fn absolute_or_repo_relative(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn resolve_benchmark_root(repo_root: &Path, benchmark_root: Option<&Path>) -> PathBuf {
    match benchmark_root {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => repo_root.join(path),
        None => repo_root.join("benchmarks"),
    }
}

fn benchmark_pipeline_config_root(benchmark_root: &Path) -> PathBuf {
    benchmark_root.join("configs/pipelines/local")
}

fn local_pipeline_config_path(benchmark_root: &Path, pipeline_id: &str) -> Result<PathBuf> {
    if pipeline_id.trim().is_empty() {
        return Err(anyhow!("pipeline id must be non-empty"));
    }
    if pipeline_id.contains('/') || pipeline_id.contains('\\') {
        return Err(anyhow!(
            "pipeline id `{pipeline_id}` must be a governed local pipeline id, not a path"
        ));
    }
    Ok(benchmark_pipeline_config_root(benchmark_root).join(format!("{pipeline_id}.toml")))
}

fn validate_all_local_pipelines(
    repo_root: &Path,
    benchmark_root: &Path,
    strict: bool,
    output: Option<&Path>,
) -> Result<LocalPipelineDagValidationSetReport> {
    let config_root = benchmark_pipeline_config_root(benchmark_root);
    let config_paths = discover_local_pipeline_config_paths(&config_root)?;
    if config_paths.is_empty() {
        return Err(anyhow!(
            "benchmark pipeline config root `{}` does not contain any governed local pipeline configs",
            config_root.display()
        ));
    }

    let output_path = absolute_or_repo_relative(
        repo_root,
        output.unwrap_or_else(|| {
            Path::new("target/local-ready/pipeline-dag/all-benchmark-pipelines.json")
        }),
    );
    let mut pipelines = Vec::with_capacity(config_paths.len());
    for config_path in config_paths {
        let pipeline_id =
            config_path.file_stem().and_then(|value| value.to_str()).ok_or_else(|| {
                anyhow!("pipeline config `{}` must have a utf-8 file stem", config_path.display())
            })?;
        let pipeline_output_path =
            repo_root.join("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json"));
        let report = crate::commands::benchmark::local_pipeline_dag::validate_pipeline_dag_path(
            repo_root,
            &config_path,
            &pipeline_output_path,
        )?;
        if strict {
            validate_local_pipeline_identity(repo_root, &report, &config_path, pipeline_id)?;
        }
        pipelines.push(report);
    }

    let valid_pipeline_count = pipelines.iter().filter(|report| report.valid).count();
    let report = LocalPipelineDagValidationSetReport {
        schema_version: LOCAL_PIPELINE_DAG_VALIDATION_SET_SCHEMA_VERSION,
        benchmark_root: path_relative_to_repo(repo_root, benchmark_root),
        config_root: path_relative_to_repo(repo_root, &config_root),
        output_path: path_relative_to_repo(repo_root, &output_path),
        pipeline_count: pipelines.len(),
        valid_pipeline_count,
        all_valid: valid_pipeline_count == pipelines.len(),
        pipelines,
    };
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn discover_local_pipeline_config_paths(config_root: &Path) -> Result<Vec<PathBuf>> {
    let mut config_paths = fs::read_dir(config_root)
        .map_err(|error| anyhow!("read {}: {error}", config_root.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_file() && path.extension().and_then(|value| value.to_str()) == Some("toml")
        })
        .collect::<Vec<_>>();
    config_paths.sort();
    Ok(config_paths)
}

fn validate_local_pipeline_identity(
    repo_root: &Path,
    report: &crate::commands::benchmark::local_pipeline_dag::LocalPipelineDagValidationReport,
    config_path: &Path,
    expected_pipeline_id: &str,
) -> Result<()> {
    if report.pipeline_id != expected_pipeline_id {
        return Err(anyhow!(
            "governed local pipeline `{expected_pipeline_id}` resolved config `{}` with pipeline_id `{}`",
            config_path.display(),
            report.pipeline_id
        ));
    }
    let expected_config_path = path_relative_to_repo(repo_root, config_path);
    if report.config_path != expected_config_path {
        return Err(anyhow!(
            "strict local pipeline validation expected config `{expected_config_path}` but validator resolved `{}`",
            report.config_path
        ));
    }
    Ok(())
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
