use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[path = "../workspace_paths.rs"]
mod workspace_paths;

fn fixture_command(repo_root: &Path, owner: &Path) -> Command {
    let mut command = Command::new(
        bijux_dna_testkit::workspace_support::workspace_binary_path(
            "bijux-dna",
            std::path::Path::new(env!("CARGO_BIN_EXE_bijux-dna")),
        )
        .expect("resolve CLI executable"),
    );
    command
        .env_clear()
        .current_dir(repo_root)
        .env("BIJUX_REPO_ROOT", repo_root)
        .env("BIJUX_BENCHMARK_ROOT", repo_root.join("benchmarks"))
        .env("HOME", owner.join("home"))
        .env("TMPDIR", owner.join("tmp"))
        .env("TMP", owner.join("tmp"))
        .env("TEMP", owner.join("tmp"))
        .env("ARTIFACT_ROOT", owner.join("artifacts"))
        .env("ISO_ROOT", owner.join("artifacts"))
        .env("ARTIFACT_DIR", owner.join("artifacts"))
        .env("PATH", owner.join("bin"))
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("TZ", "UTC")
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1");
    command
}

fn with_output<T>(args: &[&str], consume: impl FnOnce(Output) -> T) -> T {
    let root = workspace_paths::repo_root()
        .unwrap_or_else(|error| panic!("resolve fixture repository: {error}"));
    let repo_root = if root.is_absolute() {
        root
    } else {
        std::env::current_dir()
            .unwrap_or_else(|error| panic!("resolve fixture current directory: {error}"))
            .join(root)
    };
    let parent = repo_root.join("artifacts/tests/fixture-commands");
    fs::create_dir_all(&parent).unwrap_or_else(|error| {
        panic!("create fixture command parent {}: {error}", parent.display())
    });
    let owner = tempfile::Builder::new()
        .prefix("fixture-command-")
        .tempdir_in(&parent)
        .unwrap_or_else(|error| panic!("create fixture command directory: {error}"));
    for name in ["home", "tmp", "artifacts", "bin"] {
        let path = owner.path().join(name);
        fs::create_dir(&path).unwrap_or_else(|error| {
            panic!("create fixture command directory {}: {error}", path.display())
        });
    }
    let output = fixture_command(&repo_root, owner.path())
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("run fixture command: {error}"));
    consume(output)
}

pub(crate) fn run_cli(args: &[&str]) -> Output {
    with_output(args, |output| output)
}

pub(crate) fn run_cli_json(args: &[&str]) -> serde_json::Value {
    with_output(args, |output| {
        assert!(
            output.status.success(),
            "command failed: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("parse stdout as json: {error}"))
    })
}
