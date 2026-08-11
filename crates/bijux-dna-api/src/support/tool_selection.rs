use anyhow::{anyhow, Result};
use bijux_dna_core::contract::ToolRole;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_policies::{
    select_eligible_tools, ToolEligibilityCandidate, ToolEligibilityPolicy, ToolEligibilityTier,
};

pub fn filter_tools_by_role(
    stage_id: &str,
    tools: &[String],
    registry: &bijux_dna_core::contract::ToolRegistry,
    strict: bool,
) -> Result<Vec<String>> {
    let allow_silver = std::env::var("BIJUX_ALLOW_SILVER").is_ok();
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let stage_id = StageId::try_from(stage_id).map_err(|err| anyhow!("invalid stage id: {err}"))?;
    let candidates = tools
        .iter()
        .map(|tool| {
            let tool_id =
                ToolId::try_from(tool.as_str()).map_err(|err| anyhow!("invalid tool id: {err}"))?;
            let manifest = registry.tool_by_id(&stage_id, &tool_id).ok_or_else(|| {
                anyhow!("tool {tool} missing from manifests for stage {stage_id}")
            })?;
            let tier = match manifest.role {
                ToolRole::Authoritative => ToolEligibilityTier::Authoritative,
                ToolRole::Diagnostic => ToolEligibilityTier::Diagnostic,
                ToolRole::Experimental => ToolEligibilityTier::Experimental,
            };
            Ok(ToolEligibilityCandidate { tool_id: tool.clone(), tier })
        })
        .collect::<Result<Vec<_>>>()?;
    select_eligible_tools(
        &candidates,
        ToolEligibilityPolicy {
            allow_diagnostic: allow_silver,
            allow_experimental,
            reject_ineligible: strict,
        },
    )
}
