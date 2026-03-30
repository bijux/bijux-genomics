use anyhow::{anyhow, Result};
use bijux_dna_core::contract::ToolRole;
use bijux_dna_core::ids::{StageId, ToolId};
use std::path::PathBuf;

fn workspace_root() -> Result<PathBuf> {
    crate::support::repo_root::resolve_repo_root()
}

pub fn workspace_domain_dir() -> Result<PathBuf> {
    Ok(workspace_root()?.join("domain"))
}

pub fn load_registry(
    source_path: &std::path::Path,
) -> Result<bijux_dna_core::contract::ToolRegistry> {
    let registry_path = if source_path.is_dir()
        && source_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "domain")
    {
        bijux_dna_infra::configs_file(
            source_path.parent().unwrap_or(source_path),
            "ci/registry/tool_registry.toml",
        )
    } else {
        source_path.to_path_buf()
    };
    bijux_dna_runtime::manifests::load_manifests(&registry_path)
        .map_err(|err| anyhow!("manifest validation failed: {err}"))
}

pub fn load_workspace_registry() -> Result<bijux_dna_core::contract::ToolRegistry> {
    load_registry(&workspace_domain_dir()?)
}

pub fn ensure_bench_runner(
    platform: &bijux_dna_environment::api::PlatformSpec,
    runner_override: Option<bijux_dna_environment::api::RuntimeKind>,
) -> Result<bijux_dna_environment::api::RuntimeKind> {
    let runner = runner_override.unwrap_or(platform.runner);
    if !matches!(
        runner,
        bijux_dna_environment::api::RuntimeKind::Docker
            | bijux_dna_environment::api::RuntimeKind::Apptainer
            | bijux_dna_environment::api::RuntimeKind::Singularity
    ) {
        return Err(anyhow!("benchmarking does not support runner {runner}"));
    }
    Ok(runner)
}

pub fn filter_tools_by_role(
    stage_id: &str,
    tools: &[String],
    registry: &bijux_dna_core::contract::ToolRegistry,
    strict: bool,
) -> Result<Vec<String>> {
    let allow_silver = std::env::var("BIJUX_ALLOW_SILVER").is_ok();
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let mut filtered = Vec::new();
    let stage_id = StageId::try_from(stage_id).map_err(|err| anyhow!("invalid stage id: {err}"))?;
    for tool in tools {
        let tool_id =
            ToolId::try_from(tool.as_str()).map_err(|err| anyhow!("invalid tool id: {err}"))?;
        let manifest = registry
            .tool_by_id(&stage_id, &tool_id)
            .ok_or_else(|| anyhow!("tool {tool} missing from manifests for stage {stage_id}"))?;
        let tier = match manifest.role {
            ToolRole::Authoritative => "gold",
            ToolRole::Diagnostic => "silver",
            ToolRole::Experimental => "experimental",
        };
        let allowed = match tier {
            "gold" => true,
            "silver" => allow_silver || allow_experimental,
            "experimental" => allow_experimental,
            _ => false,
        };
        if allowed {
            filtered.push(tool.clone());
        } else if strict {
            return Err(anyhow!(
                "tool {tool} is {tier}; enable --allow-silver or --allow-experimental"
            ));
        }
    }
    if filtered.is_empty() {
        return Err(anyhow!("no tools available after role filtering"));
    }
    Ok(filtered)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{filter_tools_by_role, load_workspace_registry, workspace_domain_dir};
    use bijux_dna_core::contract::{
        ExecutionContract, PortSpec, StageSpec, ToolConstraints, ToolManifest, ToolRegistry,
        ToolRole,
    };
    use bijux_dna_core::ids::{StageId, ToolId};

    fn test_registry(role: ToolRole) -> ToolRegistry {
        let stage_id = StageId::from_static("fastq.test_stage");
        let mut registry = ToolRegistry::default();
        registry.insert_stage(StageSpec {
            stage_id: stage_id.clone(),
            semantic_kind: bijux_dna_core::contract::StageSemanticKind::Transform,
            input_kind: bijux_dna_core::contract::ArtifactKind::Fastq,
            output_kind: bijux_dna_core::contract::ArtifactKind::Fastq,
            produced_artifacts: Vec::new(),
            stage_semver: "1.0.0".to_string(),
            runtime_scale: bijux_dna_core::contract::RuntimeScale::Small,
            inputs: Vec::new(),
            outputs: Vec::new(),
            parameters: Vec::new(),
            metrics: Vec::new(),
            description: None,
            behavior: bijux_dna_core::prelude::tooling::StageBehavior::default(),
            image_requirements: None,
            extends: None,
        });
        registry.insert_tool(ToolManifest {
            tool_id: ToolId::from_static("demo_tool"),
            stage_id,
            role,
            command_template: vec!["demo".to_string()],
            outputs: vec![PortSpec {
                name: "report_json".to_string(),
                data_type: "json".to_string(),
                cardinality: bijux_dna_core::contract::Cardinality::One,
            }],
            metrics_parser: None,
            constraints: ToolConstraints::default(),
            execution_contract: ExecutionContract::default(),
        });
        registry
    }

    #[test]
    fn workspace_domain_dir_resolves_repo_domain_tree() {
        let domain_dir = workspace_domain_dir().expect("workspace domain dir");
        assert!(
            domain_dir.join("fastq").is_dir(),
            "workspace domain dir must resolve the repo domain tree"
        );
        load_workspace_registry().unwrap_or_else(|err| {
            panic!("workspace registry should load from resolved domain dir: {err}")
        });
    }

    #[test]
    fn role_filtering_rejects_disallowed_toolsets_instead_of_reenabling_them() {
        let registry = test_registry(ToolRole::Experimental);
        let error = match filter_tools_by_role(
            "fastq.test_stage",
            &["demo_tool".to_string()],
            &registry,
            false,
        ) {
            Ok(value) => {
                panic!("disallowed toolsets must not silently bypass role filtering: {value:?}")
            }
            Err(err) => err,
        };

        assert!(error
            .to_string()
            .contains("no tools available after role filtering"));
    }

    #[test]
    fn role_filtering_keeps_authoritative_tools_available() {
        let registry = test_registry(ToolRole::Authoritative);
        let filtered = filter_tools_by_role(
            "fastq.test_stage",
            &["demo_tool".to_string()],
            &registry,
            false,
        )
        .unwrap_or_else(|err| panic!("authoritative tool should survive filtering: {err}"));

        assert_eq!(filtered, vec!["demo_tool".to_string()]);
    }
}
