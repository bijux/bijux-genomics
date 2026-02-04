use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

use bijux_environment::build::{
    default_docker_tools, extract_version_from_dockerfile, DockerToolSpec,
};
use bijux_environment::resolve::{load_platform, ImageRef, RunnerKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let platform = parse_arg_value(&args, "--platform");
    let only_tools = parse_list_arg(&args, "--only");
    let continue_on_error = has_flag(&args, "--continue");
    let skip_existing = has_flag(&args, "--skip-existing");

    let platform_spec = load_platform(platform.as_deref())?;
    if platform_spec.runner != RunnerKind::Docker {
        return Err(format!(
            "platform runner must be docker, got {}",
            platform_spec.runner
        )
        .into());
    }

    let container_dir = PathBuf::from(&platform_spec.container_dir);
    let mut tools = default_docker_tools();
    tools = reorder_tools(tools);
    if let Some(list) = only_tools.as_ref() {
        tools.retain(|tool| list.contains(&tool.name));
    }
    if tools.is_empty() {
        return Err("no tools selected for build".into());
    }

    let mut failures = Vec::new();

    for tool in tools {
        let dockerfile = container_dir.join(format!("{}.Dockerfile", tool.name));
        if !dockerfile.exists() {
            let err = format!("Dockerfile not found: {}", dockerfile.display());
            if continue_on_error {
                eprintln!("{err}");
                failures.push(err);
                continue;
            }
            return Err(err.into());
        }
        let version = extract_version_from_dockerfile(&dockerfile, &tool.name)?;
        let image = ImageRef {
            tool: tool.name.clone(),
            version,
            arch: platform_spec.arch.clone(),
        };
        let image_name = image.to_full_name(&platform_spec.image_prefix);

        if skip_existing && image_exists(&image_name)? {
            println!("skip existing: {image_name}");
            continue;
        }

        let status = Command::new("docker")
            .arg("build")
            .arg("-t")
            .arg(&image_name)
            .arg(&container_dir)
            .arg("-f")
            .arg(&dockerfile)
            .status()?;
        if !status.success() {
            let err = format!("build failed for {image_name}");
            if continue_on_error {
                eprintln!("{err}");
                failures.push(err);
                continue;
            }
            return Err(err.into());
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("build completed with {} failures", failures.len()).into())
    }
}

fn parse_arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn parse_list_arg(args: &[String], name: &str) -> Option<HashSet<String>> {
    parse_arg_value(args, name).map(|value| {
        value
            .split(',')
            .map(|item| item.trim().to_lowercase())
            .filter(|item| !item.is_empty())
            .collect()
    })
}

fn reorder_tools(mut tools: Vec<DockerToolSpec>) -> Vec<DockerToolSpec> {
    let priority = [
        "fastp",
        "cutadapt",
        "bbduk",
        "adapterremoval",
        "trimmomatic",
        "trim_galore",
    ];
    let mut ordered = Vec::with_capacity(tools.len());
    for name in priority {
        if let Some(pos) = tools.iter().position(|tool| tool.name == name) {
            ordered.push(tools.remove(pos));
        }
    }
    ordered.extend(tools);
    ordered
}

fn image_exists(image_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("docker")
        .arg("images")
        .arg("-q")
        .arg(image_name)
        .output()?;
    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}
