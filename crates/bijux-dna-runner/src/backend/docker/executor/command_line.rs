use std::path::Path;
use std::process::Command;

use bijux_dna_environment::api::ResolvedImage;

use super::StageExecutionPlan;

pub(super) fn build_docker_run_command(
    plan: &StageExecutionPlan,
    image: &ResolvedImage,
    input_mount: &Path,
    output_mount: &Path,
    container_name: &str,
) -> (Command, Vec<String>) {
    let input_mount = format!("{}:/data/input:ro", input_mount.display());
    let output_mount = format!("{}:/data/output", output_mount.display());

    let mut cmd = Command::new("docker");
    let mut args: Vec<String> = Vec::new();
    push_arg(&mut cmd, &mut args, "run");
    push_arg(&mut cmd, &mut args, "-d");
    push_arg(&mut cmd, &mut args, "--rm=false");
    push_arg(&mut cmd, &mut args, "--name");
    push_arg(&mut cmd, &mut args, container_name);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, input_mount);
    push_arg(&mut cmd, &mut args, "-v");
    push_arg(&mut cmd, &mut args, output_mount);
    for (key, value) in &plan.env {
        push_arg(&mut cmd, &mut args, "-e");
        push_arg(&mut cmd, &mut args, format!("{key}={value}"));
    }
    push_arg(&mut cmd, &mut args, image.full_name.clone());
    for arg in &plan.container_args {
        push_arg(&mut cmd, &mut args, arg.clone());
    }
    (cmd, args)
}

pub(super) fn push_arg(cmd: &mut Command, args: &mut Vec<String>, value: impl Into<String>) {
    let value = value.into();
    cmd.arg(&value);
    args.push(value);
}

pub(super) fn command_string(args: &[String]) -> String {
    std::iter::once("docker")
        .chain(args.iter().map(String::as_str))
        .map(shell_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_token(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(ch, '.' | '_' | '/' | '-' | ':' | '=' | '@' | '+' | ',' | '%')
    }) {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::command_string;

    #[test]
    fn docker_command_string_quotes_args_with_spaces() {
        let args = vec![
            "run".to_string(),
            "-v".to_string(),
            "/tmp/input reads:/data/input:ro".to_string(),
        ];

        assert_eq!(command_string(&args), "docker run -v '/tmp/input reads:/data/input:ro'");
    }
}
