#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_dna_api::v1::api::plan::{
    explain_pipeline_profile, select_pipelines, validate_pipeline_profile,
};

struct CliWorkspace {
    root: tempfile::TempDir,
    home: PathBuf,
}

impl CliWorkspace {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("tempdir");
        let home = root.path().join("home");
        std::fs::create_dir_all(&home).expect("create home");
        Self { root, home }
    }

    fn path(&self) -> &Path {
        self.root.path()
    }

    fn setup_configs(&self) {
        let configs_dir = self.path().join("configs");
        let runtime_dir = configs_dir.join("runtime");
        let ci_dir = configs_dir.join("ci");
        let ci_registry_dir = ci_dir.join("registry");
        let ci_stages_dir = ci_dir.join("stages");
        let ci_tools_dir = ci_dir.join("tools");

        std::fs::create_dir_all(runtime_dir.join("profiles")).expect("create runtime profiles");
        std::fs::create_dir_all(&ci_registry_dir).expect("create ci registry");
        std::fs::create_dir_all(&ci_stages_dir).expect("create ci stages");
        std::fs::create_dir_all(&ci_tools_dir).expect("create ci tools");

        std::fs::write(
            runtime_dir.join("profiles").join("local.toml"),
            r#"
container_runtime = "docker"
default_threads = 1
default_mem_gb = 1
default_time_minutes = 1
run_base_dir = "runs"
image_pull_policy = "if_not_present"
"#,
        )
        .expect("write profile");
        std::fs::write(
            runtime_dir.join("platforms.toml"),
            r#"
default = "test"
[platforms.test]
runner = "docker"
container_dir = "containers"
image_prefix = "local"
arch = "x86_64"
"#,
        )
        .expect("write platforms");

        let repo_root = crate::support::repo_root().expect("repo root");
        std::fs::copy(
            repo_root.join("configs/ci/registry/tool_registry.toml"),
            ci_registry_dir.join("tool_registry.toml"),
        )
        .expect("copy tool registry");
        std::fs::copy(
            repo_root.join("configs/ci/stages/stages.toml"),
            ci_stages_dir.join("stages.toml"),
        )
        .expect("copy stages");
        std::fs::copy(
            repo_root.join("configs/ci/tools/images.toml"),
            ci_tools_dir.join("images.toml"),
        )
        .expect("copy images");
    }
}

fn snapshot_path() -> PathBuf {
    crate::support::crate_root("bijux-dna")
        .expect("crate root")
        .join("tests")
        .join("snapshots")
        .join("bijux-dna__contracts__cli_library_parity.json")
}

fn run_cli_json(workspace: &CliWorkspace, args: &[&str]) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(workspace.path())
        .env("HOME", &workspace.home)
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn cli_planning_outputs_match_library_api_for_representative_domains() {
    let _cwd_guard = crate::support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = crate::support::EnvGuard::new().expect("capture env");

    let workspace = CliWorkspace::new();
    workspace.setup_configs();

    let fastq_profile = "fastq-to-fastq__default__v1";
    let bam_profile = "bam-to-bam__adna_capture__v1";
    let cross_explain_profile = "fastq-to-vcf__minimal__v1";
    let cross_validate_profile = "bam-to-vcf__default__v1";
    let vcf_profile = "vcf-to-vcf__minimal__v1";

    assert!(
        select_pipelines(None, true)
            .into_iter()
            .any(|profile| profile.id.as_str() == fastq_profile),
        "expected {fastq_profile} to remain governed"
    );
    assert!(
        select_pipelines(None, true).into_iter().any(|profile| profile.id.as_str() == bam_profile),
        "expected {bam_profile} to remain governed"
    );

    let fastq_cli = run_cli_json(&workspace, &["plan", "explain-profile", fastq_profile]);
    let fastq_api = explain_pipeline_profile(fastq_profile).expect("fastq explain profile");
    assert_eq!(fastq_cli, fastq_api, "fastq explain-profile parity drift");

    let bam_cli = run_cli_json(&workspace, &["plan", "validate-profile", bam_profile]);
    let bam_api = validate_pipeline_profile(bam_profile).expect("bam validate profile");
    assert_eq!(bam_cli, bam_api, "bam validate-profile parity drift");

    let cross_explain_cli =
        run_cli_json(&workspace, &["plan", "explain-profile", cross_explain_profile]);
    let cross_explain_api =
        explain_pipeline_profile(cross_explain_profile).expect("cross explain profile");
    assert_eq!(cross_explain_cli, cross_explain_api, "cross explain-profile parity drift");

    let cross_validate_cli =
        run_cli_json(&workspace, &["plan", "validate-profile", cross_validate_profile]);
    let cross_validate_api =
        validate_pipeline_profile(cross_validate_profile).expect("cross validate profile");
    assert_eq!(cross_validate_cli, cross_validate_api, "cross validate-profile parity drift");

    let vcf_cli = run_cli_json(&workspace, &["vcf", "plan", "--profile", vcf_profile]);
    let vcf_api = bijux_dna_api::v1::api::vcf::plan(vcf_profile);
    assert_eq!(vcf_cli, vcf_api, "vcf plan parity drift");

    let rendered = serde_json::to_string_pretty(&serde_json::json!({
        "fastq_explain_profile": fastq_cli,
        "bam_validate_profile": bam_cli,
        "cross_explain_profile": cross_explain_cli,
        "cross_validate_profile": cross_validate_cli,
        "vcf_plan": vcf_cli,
    }))
    .expect("serialize parity snapshot");
    let snapshot =
        std::fs::read_to_string(snapshot_path()).expect("read cli library parity snapshot");
    assert_eq!(rendered.trim(), snapshot.trim());
}
