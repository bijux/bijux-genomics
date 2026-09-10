//! Pure projection from admitted binding declarations to the persisted pipeline contract.
use super::commands::{glob_manifest_path, split_catalog_command, wrap_real_command};
use crate::qualification::specification::QualificationSpecificationSourceV1;
use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    ContainerPipelineContract, ContainerPipelineV2, CONTAINER_PIPELINE_SCHEMA_V2,
};
use bijux_dna_stage_contract::{OutputDiscoveryV1, ToolBinding};
use serde_json::json;
use std::path::{Path, PathBuf};

pub(super) fn materialize_pipeline(
    source: &QualificationSpecificationSourceV1,
    catalog_tool: &bijux_dna_containers::api::ContainerCatalogTool,
    run: &bijux_dna_containers::api::QualificationRunDefinition,
    binding: &impl ToolBinding,
    image_path: &Path,
    image_sha256: &str,
    command: &[String],
) -> Result<ContainerPipelineV2> {
    let required_files = required_qualification_files(source, binding)?;
    let real = wrap_real_command(command, source, binding)?;
    let domain = run.stage_id.split_once('.').map_or(run.stage_id.as_str(), |value| value.0);
    let pipeline_value = json!({
        "schema_version": CONTAINER_PIPELINE_SCHEMA_V2,
        "run": {
            "id": source.run_id,
            "kind": "qualification",
            "campaign": source.campaign,
            "corpus": source.corpus,
            "sample": source.sample,
            "domain": domain,
            "stage": run.stage_id,
            "tool": source.tool_id,
            "operator": source.operator,
            "submitted_at_utc": source.submitted_at_utc,
        },
        "repository": {
            "path": source.repository_path,
            "commit": source.repository_commit,
            "clean": true,
        },
        "image": {"path": image_path, "sha256": image_sha256},
        "data": {"inputs": source.inputs.iter().map(|input| json!({
            "id": input.id,
            "kind": input.kind,
            "path": input.path,
            "mount_path": input.mount_path,
            "sha256": input.sha256,
        })).collect::<Vec<_>>()},
        "commands": {
            "version": split_catalog_command(&catalog_tool.version_command, "version")?,
            "help": split_catalog_command(&catalog_tool.help_command, "help")?,
            "real": real,
            "negative": source.negative_command,
        },
        "outputs": {
            "result_root": source.result_root,
            "mount_path": "/result",
            "required_files": required_files,
            "collection_rule": "required_files_and_metrics",
        },
        "resources": {
            "account": source.site.account,
            "partition": source.site.partition,
            "cpus": u64::from(binding.resources().threads),
            "memory_mb": u64::from(binding.resources().mem_gb) * 1024,
            "walltime": source.site.walltime,
            "scratch_mb": u64::from(binding.resources().tmp_gb) * 1024,
            "retry": {"max_attempts": 1, "backoff_seconds": 0, "retryable_exit_codes": []},
        },
        "execution": {
            "locale": "C",
            "timezone": "UTC",
            "runtime_network": false,
            "threads": u64::from(binding.resources().threads),
            "random_seed": null,
            "cache_policy": "cold",
            "determinism": "deterministic",
        },
        "dependencies": [],
        "acceptance": {
            "version_regex": catalog_tool.expected_version_regex,
            "negative_exit_nonzero": true,
            "metrics": [{
                "id": "tool_completed",
                "path": "qualification-metrics.json",
                "json_pointer": "/tool_completed",
                "operator": "eq",
                "expected": 1,
                "unit": "boolean",
            }],
        },
        "interface": {
            "schema_version": "bijux.benchmark.stage_interface_evidence.v1",
            "accepted_input_contracts": binding.inputs().iter()
                .map(|input| input.artifact_contract_id.clone()).collect::<Vec<_>>(),
            "output_contract": binding.normalized_output_contract_id(),
            "normalized_result_contract": "normalized-result.json",
            "adapter": binding.normalizer_id(),
            "downstream_probe": {
                "stage_id": binding.downstream_probe_id(),
                "input_contract": binding.normalized_output_contract_id(),
                "source": binding.normalizer_id(),
                "artifact": "downstream-probe.json",
            },
        },
    });
    let pipeline: ContainerPipelineV2 = serde_json::from_value(pipeline_value)
        .context("materialize compiled qualification pipeline")?;
    bijux_dna_containers::api::validate_container_pipeline(&ContainerPipelineContract::Current(
        Box::new(pipeline.clone()),
    ))?;
    Ok(pipeline)
}

fn required_qualification_files(
    source: &QualificationSpecificationSourceV1,
    binding: &impl ToolBinding,
) -> Result<Vec<PathBuf>> {
    Ok(source
        .outputs
        .iter()
        .map(|output| {
            let binding_output = binding
                .outputs()
                .iter()
                .find(|value| value.name == output.binding_output)
                .context("qualification output disappeared from its binding")?;
            Ok(match &binding_output.discovery {
                OutputDiscoveryV1::ExactPath | OutputDiscoveryV1::ExactPathWithSidecars { .. } => {
                    bijux_dna_stage_contract::required_binding_output_paths(
                        binding_output,
                        &output.path,
                    )?
                }
                OutputDiscoveryV1::Glob { .. } => vec![glob_manifest_path(&output.path)],
            })
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .chain([PathBuf::from("qualification-metrics.json")])
        .collect())
}
