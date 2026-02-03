use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_environment::{
    default_docker_tools, extract_version_from_dockerfile, load_platform, ImageRef, RunnerKind,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let platform = parse_arg_value(&args, "--platform");

    let platform_spec = load_platform(platform.as_deref())?;
    if platform_spec.runner != RunnerKind::Docker {
        return Err(format!(
            "platform runner must be docker, got {}",
            platform_spec.runner
        )
        .into());
    }

    let container_dir = PathBuf::from(&platform_spec.container_dir);
    let tools = default_docker_tools();

    for tool in tools {
        let dockerfile = container_dir.join(format!("{}.Dockerfile", tool.name));
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

        let inspect = run_command(&["docker", "images", "-q", &image_name])?;
        if output_stdout(&inspect).trim().is_empty() {
            return Err(format!("image not found: {image_name}").into());
        }

        if let Some(executable) = tool.executable.as_ref() {
            let which_output = run_command(&[
                "docker",
                "run",
                "--rm",
                "--entrypoint",
                "which",
                &image_name,
                executable,
            ])?;
            if output_stdout(&which_output).trim().is_empty() {
                return Err(format!("executable not found: {executable} in {image_name}").into());
            }
        }

        let version_output = run_command(&[
            "docker",
            "run",
            "--rm",
            "--entrypoint",
            "bash",
            &image_name,
            "-c",
            &tool.version_cmd,
        ])?;
        let combined = format!(
            "{}{}",
            output_stdout(&version_output),
            output_stderr(&version_output)
        );
        let expected = expected_version.trim_start_matches('v');
        if !combined.to_lowercase().contains(&expected.to_lowercase()) {
            return Err(format!(
                "version mismatch for {}: expected {}, got {}",
                tool.name,
                expected_version,
                combined.trim()
            )
            .into());
        }

        if let Some(help_cmd) = tool.help_cmd.as_ref() {
            let help_output = run_command(&[
                "docker",
                "run",
                "--rm",
                "--entrypoint",
                "bash",
                &image_name,
                "-c",
                help_cmd,
            ])?;
            if output_stdout(&help_output).trim().is_empty()
                && output_stderr(&help_output).trim().is_empty()
            {
                return Err(format!("help output empty for {}", tool.name).into());
            }
        }
    }

    Ok(())
}

fn run_command(args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let mut cmd = Command::new(args[0]);
    if args.len() > 1 {
        cmd.args(&args[1..]);
    }
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(format!("command failed: {}", args.join(" ")).into());
    }
    Ok(output)
}

fn output_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn output_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn parse_arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
}
