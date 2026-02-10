use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

use bijux_dna_environment::build::{
    default_docker_tools, extract_version_from_dockerfile, DockerToolSpec,
};
use bijux_dna_environment::resolve::{load_platform, ImageRef, RuntimeKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let platform = parse_arg_value(&args, "--platform");
    let only_tools = parse_list_arg(&args, "--only");
    let continue_on_error = has_flag(&args, "--continue");
    let skip_existing = has_flag(&args, "--skip-existing");

    let platform_spec = load_platform(platform.as_deref())?;
    if platform_spec.runner != RuntimeKind::Docker {
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
    let oci_revision = git_head_revision().unwrap_or_else(|| "unknown".to_string());
    let oci_created = utc_created_timestamp().unwrap_or_else(|| "unknown".to_string());

    for tool in tools {
        let dockerfile = container_dir.join(format!("Dockerfile.{}", tool.name));
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
            .arg("--build-arg")
            .arg(format!("OCI_REVISION={oci_revision}"))
            .arg("--build-arg")
            .arg(format!("OCI_CREATED={oci_created}"))
            .arg("--build-arg")
            .arg(format!("TOOL_VERSION={}", image.version))
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

        let smoke_status = Command::new("docker")
            .arg("run")
            .arg("--rm")
            .arg("--entrypoint")
            .arg("/bin/sh")
            .arg(&image_name)
            .arg("-lc")
            .arg(&tool.version_cmd)
            .status()?;
        if !smoke_status.success() {
            let err = format!(
                "version smoke failed for {image_name}: {}",
                tool.version_cmd
            );
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

fn git_head_revision() -> Option<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if revision.is_empty() {
        None
    } else {
        Some(revision)
    }
}

fn utc_created_timestamp() -> Option<String> {
    let output = Command::new("date")
        .arg("-u")
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let created = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if created.is_empty() {
        None
    } else {
        Some(created)
    }
}
