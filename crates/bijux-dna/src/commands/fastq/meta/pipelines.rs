use crate::commands::fastq::api_bridge::resolve_profile_alias;
use crate::commands::support::prelude::{
    anyhow, cli, load_manifests, render, PipelinesCommand, Result,
};

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
            render::json::print_pretty(
                &bijux_dna_api::v1::api::plan::explain_pipeline_profile(&resolved_id)?,
            )?;
            Ok(true)
        }
        PipelinesCommand::ValidateProfile { id } => {
            let resolved_id = resolve_profile_alias(id);
            render::json::print_pretty(
                &bijux_dna_api::v1::api::plan::validate_pipeline_profile(&resolved_id)?,
            )?;
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
