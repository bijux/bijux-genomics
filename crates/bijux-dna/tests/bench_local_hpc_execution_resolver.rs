#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_manifest_path(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = support::hpc_dry_run_test_directory(repo_root, label)
        .expect("temporary HPC dry-run directory");
    let manifest_path = temp_dir.path().join("execution-resolver.tsv");
    (temp_dir, manifest_path)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let runtime_root = tempfile::tempdir().expect("runtime tempdir");

    let mut command = Command::new(support::cli_binary_path());
    support::configure_isolated_runtime_resolution(&mut command, runtime_root.path());
    command
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn load_rows(path: &Path) -> Vec<BTreeMap<String, String>> {
    let body = fs::read_to_string(path).expect("read execution resolver");
    let mut lines = body.lines();
    let header = lines.next().expect("header row");
    let columns = header.split('\t').map(str::to_string).collect::<Vec<_>>();
    lines
        .map(|line| {
            let values = line.split('\t').map(str::to_string).collect::<Vec<_>>();
            columns.iter().cloned().zip(values).collect::<BTreeMap<String, String>>()
        })
        .collect()
}

#[test]
fn bench_local_render_hpc_execution_resolver_reports_selected_tool_runtime_surface() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, manifest_path) =
        render_manifest_path(&repo_root, "render-hpc-execution-resolver-");
    let manifest_arg = manifest_path.to_string_lossy().into_owned();

    let output =
        run_cli(&["bench", "local", "render-hpc-execution-resolver", "--output", &manifest_arg]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let printed_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        printed_path.trim(),
        manifest_path
            .strip_prefix(&repo_root)
            .expect("manifest path relative to repo root")
            .to_string_lossy()
    );

    let rows = load_rows(&manifest_path);
    assert!(rows.len() >= 70, "execution resolver must cover the governed selected HPC tool scope");

    let bijux_runtime = rows
        .iter()
        .find(|row| row.get("tool_id").map(String::as_str) == Some("bijux_dna"))
        .expect("bijux_dna selected job row");
    assert_eq!(bijux_runtime.get("lookup_tool_id").map(String::as_str), Some("bijux_dna"));
    assert_eq!(bijux_runtime.get("execution_mode").map(String::as_str), Some("internal"));
    assert_eq!(bijux_runtime.get("resolution_kind").map(String::as_str), Some("host_binary"));
    assert_eq!(bijux_runtime.get("resolution_target").map(String::as_str), Some("bijux-dna"));

    let contammix = rows
        .iter()
        .find(|row| row.get("tool_id").map(String::as_str) == Some("contammix"))
        .expect("contammix selected benchmark row");
    assert_eq!(contammix.get("resolution_kind").map(String::as_str), Some("apptainer_image"));
    assert!(
        contammix
            .get("resolution_target")
            .is_some_and(|value| value
                .starts_with("${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/contammix/")),
        "container-backed selected tools must resolve to the governed Apptainer cache path"
    );

    let germline = rows
        .iter()
        .find(|row| row.get("tool_id").map(String::as_str) == Some("germline"))
        .expect("germline selected benchmark row");
    assert_eq!(germline.get("execution_mode").map(String::as_str), Some("containerized"));
    assert_eq!(germline.get("resolution_kind").map(String::as_str), Some("apptainer_image"));
    assert!(
        germline
            .get("resolution_target")
            .is_some_and(|value| value.starts_with("${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/germline/")),
        "classified VCF tools with admitted images must resolve through the governed Apptainer cache"
    );

    let plink = rows
        .iter()
        .find(|row| row.get("tool_id").map(String::as_str) == Some("plink"))
        .expect("plink selected benchmark row");
    assert_eq!(plink.get("execution_mode").map(String::as_str), Some("containerized"));
    assert_eq!(plink.get("resolution_kind").map(String::as_str), Some("apptainer_image"));
    assert!(
        plink
            .get("resolution_target")
            .is_some_and(|value| value.starts_with("containers/apptainer/plink.def@sha256:")),
        "repository-distributable PLINK must resolve to its governed recipe identity"
    );
    assert_eq!(plink.get("unavailable_reason").map(String::as_str), Some(""));

    for tool_id in ["angsd", "fastuniq", "impute5", "ngsbriggs"] {
        let row = rows
            .iter()
            .find(|row| row.get("tool_id").map(String::as_str) == Some(tool_id))
            .expect("licensed external selected benchmark row");
        assert_eq!(row.get("execution_mode").map(String::as_str), Some("external"));
        assert_eq!(row.get("resolution_kind").map(String::as_str), Some("unavailable_with_reason"));
        assert!(
            row.get("unavailable_reason")
                .is_some_and(|value| value.contains("external artifact admission")),
            "licensed external product support must remain classified independently from artifact admission"
        );
    }
}
