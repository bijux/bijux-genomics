mod tools;

pub use tools::{
    normalize_correct_tool_list, normalize_filter_tool_list, normalize_merge_tool_list,
    normalize_qc2_tool_list, normalize_screen_tool_list, normalize_stats_tool_list,
    normalize_trim_tool_list, normalize_umi_tool_list, normalize_validate_tool_list,
};

use anyhow::{anyhow, Result};
use bijux_core::{load_manifests, ToolRegistry};

use crate::core::types::Policy;
use crate::core::types::{ExecutionContext, RunPlan, ToolInvocation};

pub fn load_registry(domain_root: &std::path::Path) -> Result<ToolRegistry> {
    load_manifests(domain_root).map_err(|err| anyhow!("manifest validation failed: {err}"))
}

pub fn plan_tool(
    context: &ExecutionContext,
    invocation: ToolInvocation,
    image_digest: String,
) -> Result<RunPlan> {
    if let Some(requirements) = &invocation.requirements {
        check_capabilities(context, requirements)?;
    }
    let runner = context.runner_override.unwrap_or(context.platform.runner);
    if crate::core::types::trace_enabled() {
        println!(
            "[engine][composer] stage={} tool={} runner={}",
            invocation.stage_id, invocation.tool_id, runner
        );
    }
    Ok(RunPlan {
        invocation,
        image_digest,
        runner,
    })
}

fn check_capabilities(
    context: &ExecutionContext,
    requirements: &crate::core::types::StageRequirement,
) -> Result<()> {
    for capability in &requirements.capabilities {
        if !context.capabilities.contains(capability) {
            return Err(anyhow!("missing required capability: {capability:?}"));
        }
    }
    Ok(())
}

pub fn apply_policy(tools: &[String], policy: Policy) -> Vec<String> {
    if crate::core::types::trace_enabled() {
        println!("[engine][composer] policy={policy:?} tools={}", tools.len());
    }
    match policy {
        Policy::PreferAccuracy | Policy::PreferSpeed | Policy::PreferMemory => tools.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_environment::api::RunnerKind;
    use std::collections::BTreeMap;

    fn context_with_runner(runner: RunnerKind) -> ExecutionContext {
        ExecutionContext {
            platform: bijux_environment::api::PlatformSpec {
                name: "local".to_string(),
                runner,
                container_dir: std::path::PathBuf::from("/tmp"),
                image_prefix: "bijux".to_string(),
                arch: "arm64".to_string(),
            },
            runner_override: None,
            env: BTreeMap::new(),
            capabilities: vec![crate::core::types::Capability::Fastq],
        }
    }

    #[test]
    fn plan_uses_platform_runner_by_default() {
        let context = context_with_runner(RunnerKind::Docker);
        let invocation = ToolInvocation {
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            inputs: Vec::new(),
            params: serde_json::json!({}),
            requirements: None,
        };
        match plan_tool(&context, invocation, "sha256:abc".to_string()) {
            Ok(plan) => assert_eq!(plan.runner, RunnerKind::Docker),
            Err(err) => panic!("plan failed: {err}"),
        }
    }

    #[test]
    fn plan_uses_override_runner() {
        let mut context = context_with_runner(RunnerKind::Docker);
        context.runner_override = Some(RunnerKind::Apptainer);
        let invocation = ToolInvocation {
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            inputs: Vec::new(),
            params: serde_json::json!({}),
            requirements: None,
        };
        match plan_tool(&context, invocation, "sha256:abc".to_string()) {
            Ok(plan) => assert_eq!(plan.runner, RunnerKind::Apptainer),
            Err(err) => panic!("plan failed: {err}"),
        }
    }

    #[test]
    fn plan_rejects_missing_capability() {
        let mut context = context_with_runner(RunnerKind::Docker);
        context.capabilities = vec![crate::core::types::Capability::Fastq];
        let invocation = ToolInvocation {
            stage_id: "fastq.trim".to_string(),
            tool_id: "fastp".to_string(),
            inputs: Vec::new(),
            params: serde_json::json!({}),
            requirements: Some(crate::core::types::StageRequirement {
                capabilities: vec![crate::core::types::Capability::Bam],
            }),
        };
        match plan_tool(&context, invocation, "sha256:abc".to_string()) {
            Ok(_) => panic!("expected capability error"),
            Err(err) => assert!(err.to_string().contains("missing required capability")),
        }
    }
}
