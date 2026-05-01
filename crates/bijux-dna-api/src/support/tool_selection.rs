use anyhow::{anyhow, Result};
use bijux_dna_core::contract::ToolRole;
use bijux_dna_core::ids::{StageId, ToolId};

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
    use super::filter_tools_by_role;
    use bijux_dna_core::contract::{
        ArtifactRole, BackendVersionPolicy, ExecutionContract, PortSpec, StageCapabilitySpec,
        StageEnvironmentRequirements, StageOperatingMode, StageSpec, ToolConstraints, ToolManifest,
        ToolRegistry, ToolRole,
    };
    use bijux_dna_core::ids::{StageId, ToolId};

    fn test_registry(role: ToolRole) -> ToolRegistry {
        let stage_id = StageId::from_static("fastq.test_stage");
        let mut registry = ToolRegistry::default();
        registry.insert_stage(StageSpec {
            stage_id: stage_id.clone(),
            stage_family: bijux_dna_core::contract::StageFamily::Fastq,
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
            environment_requirements: StageEnvironmentRequirements::default(),
            report_contracts: Vec::new(),
            capability_contract: StageCapabilitySpec::default(),
            refusal_codes: Vec::new(),
            operating_mode: StageOperatingMode::Enforced,
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
                artifact_role: ArtifactRole::ReportJson,
            }],
            metrics_parser: None,
            constraints: ToolConstraints::default(),
            execution_contract: ExecutionContract::default(),
            supported_modes: vec![StageOperatingMode::Enforced],
            backend_version_policy: BackendVersionPolicy::Pinned,
            capability_contract: StageCapabilitySpec::default(),
        });
        registry
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

        assert!(error.to_string().contains("no tools available after role filtering"));
    }

    #[test]
    fn role_filtering_keeps_authoritative_tools_available() {
        let registry = test_registry(ToolRole::Authoritative);
        let filtered =
            filter_tools_by_role("fastq.test_stage", &["demo_tool".to_string()], &registry, false)
                .unwrap_or_else(|err| panic!("authoritative tool should survive filtering: {err}"));

        assert_eq!(filtered, vec!["demo_tool".to_string()]);
    }
}
