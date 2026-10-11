use anyhow::Result;
use bijux_dna_bench_vcf_evidence::commands::projection::{
    render_shell_command, render_vcf_commands_shell_script,
    render_vcf_commands_shell_script_at_repository,
};
use std::fmt::Write;
#[test]
fn command_scripts_keep_repository_identity_outside_the_repository() -> Result<()> {
    let sandbox = tempfile::tempdir()?;
    let repo = sandbox.path().join("repository with spaces");
    std::fs::create_dir_all(&repo)?;
    let output = sandbox.path().join("external").join("scripts").join("commands.sh");
    std::fs::create_dir_all(output.parent().ok_or_else(|| anyhow::anyhow!("script parent"))?)?;
    let observed = sandbox.path().join("observed-root.txt");
    let mut script = match super::paths::script_repository_root(&repo, &output)? {
        super::paths::ScriptRepositoryRoot::Relative(path) => {
            render_vcf_commands_shell_script(&[], &path)
        }
        super::paths::ScriptRepositoryRoot::Absolute(path) => {
            render_vcf_commands_shell_script_at_repository(&[], &path)
        }
    };
    let _ = writeln!(
        script,
        "printf '%s' \"$PWD\" > {}",
        render_shell_command(&[observed.to_string_lossy().into_owned()])
    );
    std::fs::write(&output, script)?;
    let status =
        std::process::Command::new("bash").arg(&output).current_dir(sandbox.path()).status()?;
    assert!(status.success());
    assert_eq!(std::fs::read_to_string(observed)?, repo.canonicalize()?.to_string_lossy());
    Ok(())
}

#[test]
fn script_repository_paths_are_literal_and_owned_scripts_remain_relative() -> Result<()> {
    let sandbox = tempfile::tempdir()?;
    let name = "repø $(touch injected) `touch injected` \"quotes\" 'apostrophe'";
    let repo = sandbox.path().join(name);
    std::fs::create_dir_all(&repo)?;
    let output = repo.join("artifacts/benchmark-smoke/readiness/vcf/commands.sh");
    match super::paths::script_repository_root(&repo, &output)? {
        super::paths::ScriptRepositoryRoot::Relative(path) => assert_eq!(path, "../../../.."),
        super::paths::ScriptRepositoryRoot::Absolute(_) => {
            anyhow::bail!("owned scripts must remain relocatable")
        }
    }
    let observation = sandbox.path().join("observed-root.txt");
    for (label, mut script) in [
        ("relative", render_vcf_commands_shell_script(&[], name)),
        (
            "absolute",
            render_vcf_commands_shell_script_at_repository(
                &[],
                repo.to_str().ok_or_else(|| anyhow::anyhow!("UTF-8 test path"))?,
            ),
        ),
    ] {
        let _ = writeln!(
            script,
            "printf '%s' \"$PWD\" > {}",
            render_shell_command(&[observation.to_string_lossy().into_owned()])
        );
        let path = sandbox.path().join(format!("{label}.sh"));
        std::fs::write(&path, script)?;
        assert!(std::process::Command::new("bash")
            .arg(&path)
            .current_dir(sandbox.path())
            .status()?
            .success());
        assert_eq!(std::fs::read_to_string(&observation)?, repo.canonicalize()?.to_string_lossy());
        assert!(!sandbox.path().join("injected").exists());
        assert!(!repo.join("injected").exists());
    }
    match super::paths::script_repository_root(&repo, &repo.join("../outside.sh"))? {
        super::paths::ScriptRepositoryRoot::Absolute(path) => {
            assert_eq!(path, repo.canonicalize()?.to_string_lossy());
        }
        super::paths::ScriptRepositoryRoot::Relative(_) => {
            anyhow::bail!("parent traversal cannot establish repository ownership")
        }
    }
    Ok(())
}
