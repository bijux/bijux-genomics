use std::path::Path;
use std::collections::BTreeSet;

use bijux_dna_environment::build::{
    default_docker_tools, extract_version_from_dockerfile, DockerToolSpec,
};
use bijux_dna_environment::resolve::{load_platform, ImageRef, PlatformSpec, RuntimeKind};

use super::models::ImagePlan;

pub(super) fn load_platform_spec(
    platform: Option<&str>,
) -> Result<PlatformSpec, Box<dyn std::error::Error>> {
    let platform_spec = load_platform(platform)?;
    if platform_spec.runner != RuntimeKind::Docker {
        return Err(format!("platform runner must be docker, got {}", platform_spec.runner).into());
    }
    Ok(platform_spec)
}

pub(super) fn filter_tools(
    tools_filter: Option<String>,
) -> Result<Vec<DockerToolSpec>, Box<dyn std::error::Error>> {
    let mut tools = default_docker_tools();
    if let Some(filter) = tools_filter {
        let wanted: BTreeSet<String> = filter
            .split(',')
            .map(|item| item.trim().to_lowercase())
            .filter(|item| !item.is_empty())
            .collect();
        if wanted.is_empty() {
            return Err("empty --tools filter".into());
        }
        let available = tools.iter().map(|tool| tool.name.to_lowercase()).collect::<BTreeSet<_>>();
        let unknown = wanted.difference(&available).cloned().collect::<Vec<_>>();
        if !unknown.is_empty() {
            return Err(format!("unknown tools in --tools filter: {}", unknown.join(", ")).into());
        }
        tools.retain(|tool| wanted.contains(&tool.name.to_lowercase()));
        if tools.is_empty() {
            return Err("no matching tools for --tools filter".into());
        }
    }
    Ok(tools)
}

pub(super) fn build_image_plans(
    platform_spec: &PlatformSpec,
    tools: &[DockerToolSpec],
) -> Result<Vec<ImagePlan>, Box<dyn std::error::Error>> {
    let container_dir = Path::new(&platform_spec.container_dir);
    let mut plans = Vec::new();
    for tool in tools {
        let dockerfile = container_dir.join(format!("Dockerfile.{}", tool.name));
        if !dockerfile.exists() {
            return Err(format!("Dockerfile not found: {}", dockerfile.display()).into());
        }
        let expected_version = extract_version_from_dockerfile(&dockerfile, &tool.name)?;
        let image = ImageRef {
            tool: tool.name.clone(),
            version: expected_version.clone(),
            arch: platform_spec.arch.clone(),
        };
        let image_name = image.to_full_name(&platform_spec.image_prefix);
        plans.push(ImagePlan {
            image_name,
            expected_version,
            probe_cmd: tool.probe_cmd.clone(),
            probe_expected_exit: tool.probe_expected_exit.clone(),
            executable: tool.executable.clone(),
        });
    }
    if plans.is_empty() {
        return Err("no images discovered".into());
    }
    Ok(plans)
}
