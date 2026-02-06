use anyhow::{anyhow, Result};
use bijux_core::contract::ToolRole;
use bijux_core::ids::{StageId, ToolId};

pub fn load_registry(domain_root: &std::path::Path) -> Result<bijux_core::contract::ToolRegistry> {
    bijux_runtime::manifests::load_manifests(domain_root)
        .map_err(|err| anyhow!("manifest validation failed: {err}"))
}

pub fn ensure_bench_runner(
    platform: &bijux_environment::api::PlatformSpec,
    runner_override: Option<bijux_environment::api::RunnerKind>,
) -> Result<bijux_environment::api::RunnerKind> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != bijux_environment::api::RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    Ok(runner)
}

pub fn filter_tools_by_role(
    stage_id: &str,
    tools: &[String],
    registry: &bijux_core::contract::ToolRegistry,
    strict: bool,
) -> Result<Vec<String>> {
    let allow_silver = std::env::var("BIJUX_ALLOW_SILVER").is_ok();
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let mut filtered = Vec::new();
    let stage_id = StageId::try_from(stage_id)
        .map_err(|err| anyhow!("invalid stage id: {err}"))?;
    for tool in tools {
        let tool_id =
            ToolId::try_from(tool.as_str()).map_err(|err| anyhow!("invalid tool id: {err}"))?;
        let manifest = registry
            .tool_by_id(&stage_id, &tool_id)
            .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
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
        if !strict {
            return Ok(tools.to_vec());
        }
        return Err(anyhow!("no tools available after role filtering"));
    }
    Ok(filtered)
}
