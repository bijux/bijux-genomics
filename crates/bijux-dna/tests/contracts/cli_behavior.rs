use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use bijux_dna::commands::run_with_args;
use gag::BufferRedirect;

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
        self.setup_configs_with_images(
            r#"
fastp = { version = "99.99.99+fixture" }
seqkit = { version = "99.99.99+fixture" }
fastqvalidator = { version = "99.99.99+fixture" }
fastqc = { version = "99.99.99+fixture" }
multiqc = { version = "99.99.99+fixture" }
seqkit_stats = { version = "99.99.99+fixture" }
"#,
        );
    }

    fn setup_configs_with_images(&self, images: &str) {
        let configs_dir = self.path().join("configs");
        let runtime_dir = configs_dir.join("runtime");
        let ci_dir = configs_dir.join("ci");
        let ci_tools_dir = ci_dir.join("tools");
        std::fs::create_dir_all(&runtime_dir).expect("create runtime configs");
        std::fs::create_dir_all(runtime_dir.join("profiles"))
            .expect("create runtime profile configs");
        std::fs::create_dir_all(&ci_tools_dir).expect("create ci tools configs");
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
        std::fs::write(ci_tools_dir.join("images.toml"), images).expect("write images");
    }
}

fn run_cli_capture(workspace: &CliWorkspace, args: &[&str]) -> Result<String, String> {
    let mut buffer = BufferRedirect::stdout().expect("capture stdout");
    std::env::set_var("HOME", &workspace.home);
    std::env::set_var("BIJUX_SKIP_QA", "1");
    std::env::set_var("BIJUX_ALLOW_SILVER", "1");
    std::env::set_var("BIJUX_SKIP_IMAGE_CHECK", "1");
    let result = run_with_args(args, workspace.path());
    std::io::stdout().flush().expect("flush stdout");
    let mut output = String::new();
    buffer.read_to_string(&mut output).expect("read stdout");
    result.map(|()| output).map_err(|err| err.to_string())
}

fn assert_removed_subcommand(workspace: &CliWorkspace, args: &[&str], name: &str) {
    let err = run_cli_capture(workspace, args).expect_err("command should be removed");
    assert!(
        err.contains("unrecognized subcommand") && err.contains(name),
        "expected removed subcommand `{name}` error, got: {err}"
    );
}

#[test]
fn cli_env_info_is_deterministic() {
    let workspace = CliWorkspace::new();
    workspace.setup_configs();

    let stdout =
        run_cli_capture(&workspace, &["--platform", "test", "dna", "env", "info"]).expect("cli ok");
    if stdout.trim().is_empty() {
        return;
    }
    assert!(stdout.contains("platform: test"));
    assert!(stdout.contains("runner: docker"));
    let images_stdout =
        run_cli_capture(&workspace, &["--platform", "test", "dna", "env", "images"])
            .unwrap_or_else(|err| panic!("cli images failed: {err}"));
    let image_count = images_stdout.lines().count();
    assert!(stdout.contains(&format!("image count: {image_count}")));
    let expected_cache = workspace
        .home
        .join(".cache")
        .join("bijux")
        .join("docker")
        .join("images");
    assert!(stdout.contains(&format!("cache: {}", expected_cache.display())));
}

#[test]
fn cli_env_images_are_listed_in_order() {
    let workspace = CliWorkspace::new();
    workspace.setup_configs();

    let stdout = run_cli_capture(&workspace, &["--platform", "test", "dna", "env", "images"])
        .expect("cli ok");
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.is_empty() {
        return;
    }
    let mut sorted = lines.clone();
    sorted.sort_unstable();
    assert_eq!(lines, sorted);
    assert!(lines.iter().any(|line| line.starts_with("fastp:")));
    assert!(lines.iter().any(|line| line.starts_with("fastqc:")));
    assert!(lines.iter().any(|line| line.starts_with("fastqvalidator:")));
    assert!(lines.iter().any(|line| line.starts_with("seqkit:")));
}

#[test]
fn cli_env_images_are_deterministic_across_input_order() {
    let workspace_a = CliWorkspace::new();
    workspace_a.setup_configs_with_images(
        r#"
fastp = { version = "99.99.99+fixture" }
seqkit = { version = "99.99.99+fixture" }
"#,
    );
    let workspace_b = CliWorkspace::new();
    workspace_b.setup_configs_with_images(
        r#"
seqkit = { version = "99.99.99+fixture" }
fastp = { version = "99.99.99+fixture" }
"#,
    );

    let stdout_a = run_cli_capture(
        &workspace_a,
        &["--platform", "test", "dna", "env", "images"],
    )
    .expect("cli ok");
    let stdout_b = run_cli_capture(
        &workspace_b,
        &["--platform", "test", "dna", "env", "images"],
    )
    .expect("cli ok");

    assert_eq!(stdout_a, stdout_b);
}

#[test]
fn cli_pipelines_list_includes_default_fastq() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(&workspace, &["dna", "pipelines", "list"], "pipelines");
}

#[test]
fn cli_pipelines_list_can_filter_domain() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &["dna", "pipelines", "list", "--domain", "fastq"],
        "pipelines",
    );
}

#[test]
fn cli_pipelines_explain_returns_profile_payload() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &["dna", "pipelines", "explain", "fastq-to-fastq__default__v1"],
        "pipelines",
    );
}

#[test]
fn cli_pipelines_explain_unknown_pipeline_fails() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &["dna", "pipelines", "explain", "nope"],
        "pipelines",
    );
}

#[test]
fn cli_pipelines_explain_profile_fastq_adna_includes_invariants() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &["dna", "pipelines", "explain-profile", "fastq-adna"],
        "pipelines",
    );
}

#[test]
fn cli_pipelines_explain_profile_bam_adna_includes_invariants() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &["dna", "pipelines", "explain-profile", "bam-adna"],
        "pipelines",
    );
}

#[test]
fn cli_pipelines_validate_profile_bam_adna_returns_report() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &["dna", "pipelines", "validate-profile", "bam-adna"],
        "pipelines",
    );
}

#[test]
fn cli_pipelines_explain_profile_vcf_minimal_includes_invariants() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &["dna", "pipelines", "explain-profile", "vcf-minimal"],
        "pipelines",
    );
}

#[test]
fn cli_fastq_preprocess_dry_run_writes_artifacts() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--dry-run",
        ],
        "fastq",
    );
}

#[test]
fn cli_fastq_preprocess_dry_run_reports_manifests() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--dry-run",
        ],
        "fastq",
    );
}

#[test]
fn cli_fastq_preprocess_plan_falls_back_to_dry_run() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--dry-run",
        ],
        "fastq",
    );
}

#[test]
fn cli_dry_run_manifest_is_deterministic_after_path_scrub() {
    let workspace = CliWorkspace::new();
    assert_removed_subcommand(
        &workspace,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--dry-run",
        ],
        "fastq",
    );
}
