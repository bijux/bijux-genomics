use std::io::Read;
use std::path::{Path, PathBuf};

use bijux_dna::commands::run_with_args;
use gag::BufferRedirect;
use serde_json::Value;

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
fastp = { version = "0.0.0" }
seqkit = { version = "0.0.0" }
"#,
        );
    }

    fn setup_configs_with_images(&self, images: &str) {
        let configs_dir = self.path().join("configs");
        std::fs::create_dir_all(&configs_dir).expect("create configs");
        std::fs::write(
            configs_dir.join("profile.local.toml"),
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
            configs_dir.join("platforms.toml"),
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
        std::fs::write(configs_dir.join("images.toml"), images).expect("write images");
    }

    #[cfg(unix)]
    fn link_repo_dir(&self, name: &str) {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let source = repo_root.join(name);
        let target = self.path().join(name);
        if target.exists() {
            return;
        }
        std::os::unix::fs::symlink(&source, &target).expect("symlink repo dir");
    }

    #[cfg(not(unix))]
    fn link_repo_dir(&self, _name: &str) {}
}

fn run_cli_capture(workspace: &CliWorkspace, args: &[&str]) -> Result<String, String> {
    let mut buffer = BufferRedirect::stdout().expect("capture stdout");
    std::env::set_var("HOME", &workspace.home);
    std::env::set_var("BIJUX_SKIP_QA", "1");
    std::env::set_var("BIJUX_ALLOW_SILVER", "1");
    std::env::set_var("BIJUX_SKIP_IMAGE_CHECK", "1");
    let result = run_with_args(args, workspace.path());
    let mut output = String::new();
    buffer
        .read_to_string(&mut output)
        .expect("read stdout");
    result.map(|_| output).map_err(|err| err.to_string())
}

fn scrub_paths(value: &mut Value, root: &str) {
    match value {
        Value::String(s) => {
            if s.contains(root) {
                *s = s.replace(root, "<temp>");
            }
        }
        Value::Array(items) => {
            for item in items {
                scrub_paths(item, root);
            }
        }
        Value::Object(map) => {
            for value in map.values_mut() {
                scrub_paths(value, root);
            }
        }
        _ => {}
    }
}

fn prepare_fastq_preprocess(workspace: &CliWorkspace, out_dir: &Path) -> PathBuf {
    let input = workspace.path().join("reads.fastq");
    std::fs::write(&input, "@r1\nACGT\n+\n####\n").expect("write fastq");

    let defaults_dir = out_dir
        .join("bench")
        .join("preprocess")
        .join("sample")
        .join("tools");
    std::fs::create_dir_all(&defaults_dir).expect("create defaults dir");
    let defaults = serde_json::json!({
        "pipeline_id": "fastq-to-fastq__default__v1",
        "tools": {},
        "params": {},
        "thresholds": {},
        "tool_provenance": {},
        "param_provenance": {},
        "assumptions": [],
        "citations": {},
    });
    std::fs::write(
        defaults_dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults).expect("serialize defaults"),
    )
    .expect("write defaults ledger");
    std::fs::write(
        out_dir.join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults).expect("serialize defaults"),
    )
    .expect("write root defaults ledger");

    workspace.link_repo_dir("domain");
    workspace.link_repo_dir("assets");

    input
}

#[test]
fn cli_env_info_is_deterministic() {
    let workspace = CliWorkspace::new();
    workspace.setup_configs();

    let stdout = run_cli_capture(&workspace, &["--platform", "test", "dna", "env", "info"])
        .expect("cli ok");
    assert!(stdout.contains("platform: test"));
    assert!(stdout.contains("runner: docker"));
    assert!(stdout.contains("image count: 2"));
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
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("fastp:"));
    assert!(lines[1].starts_with("seqkit:"));
}

#[test]
fn cli_env_images_are_deterministic_across_input_order() {
    let workspace_a = CliWorkspace::new();
    workspace_a.setup_configs_with_images(
        r#"
fastp = { version = "0.0.0" }
seqkit = { version = "0.0.0" }
"#,
    );
    let workspace_b = CliWorkspace::new();
    workspace_b.setup_configs_with_images(
        r#"
seqkit = { version = "0.0.0" }
fastp = { version = "0.0.0" }
"#,
    );

    let stdout_a = run_cli_capture(&workspace_a, &["--platform", "test", "dna", "env", "images"])
        .expect("cli ok");
    let stdout_b = run_cli_capture(&workspace_b, &["--platform", "test", "dna", "env", "images"])
        .expect("cli ok");

    assert_eq!(stdout_a, stdout_b);
}

#[test]
fn cli_pipelines_list_includes_default_fastq() {
    let workspace = CliWorkspace::new();
    let stdout = run_cli_capture(&workspace, &["dna", "pipelines", "list"]).expect("cli ok");
    assert!(stdout.contains("fastq-to-fastq__default__v1"));
}

#[test]
fn cli_pipelines_list_can_filter_domain() {
    let workspace = CliWorkspace::new();
    let stdout = run_cli_capture(
        &workspace,
        &["dna", "pipelines", "list", "--domain", "fastq"],
    )
    .expect("cli ok");
    assert!(stdout.contains("fastq-to-fastq__default__v1"));
    assert!(!stdout.contains("bam-to-bam__default__v1"));
}

#[test]
fn cli_pipelines_explain_returns_profile_payload() {
    let workspace = CliWorkspace::new();
    let stdout = run_cli_capture(
        &workspace,
        &[
            "dna",
            "pipelines",
            "explain",
            "fastq-to-fastq__default__v1",
        ],
    )
    .expect("cli ok");
    let payload: Value = serde_json::from_str(&stdout).expect("parse explain json");
    let profile_id = payload
        .get("profile")
        .and_then(|p| p.get("id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert_eq!(profile_id, "fastq-to-fastq__default__v1");
    assert!(payload.get("defaults_ledger").is_some());
}

#[test]
fn cli_pipelines_explain_unknown_pipeline_fails() {
    let workspace = CliWorkspace::new();
    let err = run_cli_capture(&workspace, &["dna", "pipelines", "explain", "nope"])
        .expect_err("cli should fail");
    assert!(err.contains("unknown pipeline profile"));
}

#[test]
fn cli_fastq_preprocess_dry_run_writes_artifacts() {
    let workspace = CliWorkspace::new();
    workspace.setup_configs();
    let out_dir = workspace.path().join("out");
    let input = prepare_fastq_preprocess(&workspace, &out_dir);

    run_cli_capture(
        &workspace,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--dry-run",
            "--r1",
            input.to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--sample-id",
            "sample",
        ],
    )
    .expect("cli ok");

    let manifest = out_dir.join("run_manifest.json");
    assert!(manifest.exists());
    let graph = out_dir
        .join("bench")
        .join("preprocess")
        .join("sample")
        .join("run_artifacts")
        .join("graph.json");
    assert!(graph.exists());
}

#[test]
fn cli_fastq_preprocess_dry_run_reports_manifests() {
    let workspace = CliWorkspace::new();
    workspace.setup_configs();
    let out_dir = workspace.path().join("out");
    let input = prepare_fastq_preprocess(&workspace, &out_dir);

    let stdout = run_cli_capture(
        &workspace,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--dry-run",
            "--r1",
            input.to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--sample-id",
            "sample",
        ],
    )
    .expect("cli ok");
    assert!(stdout.contains("manifests:"));
    assert!(stdout.contains("\"stage\""));
    assert!(stdout.contains("\"tool\""));
}

#[test]
fn cli_fastq_preprocess_plan_falls_back_to_dry_run() {
    let workspace = CliWorkspace::new();
    workspace.setup_configs();
    let out_dir = workspace.path().join("out");
    let input = prepare_fastq_preprocess(&workspace, &out_dir);

    run_cli_capture(
        &workspace,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--r1",
            input.to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--sample-id",
            "sample",
        ],
    )
    .expect("cli ok");
    assert!(out_dir.join("run_manifest.json").exists());
}

#[test]
fn cli_dry_run_manifest_is_deterministic_after_path_scrub() {
    let workspace_a = CliWorkspace::new();
    let workspace_b = CliWorkspace::new();
    workspace_a.setup_configs();
    workspace_b.setup_configs();
    let out_a = workspace_a.path().join("out");
    let out_b = workspace_b.path().join("out");
    let input_a = prepare_fastq_preprocess(&workspace_a, &out_a);
    let input_b = prepare_fastq_preprocess(&workspace_b, &out_b);

    run_cli_capture(
        &workspace_a,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--dry-run",
            "--r1",
            input_a.to_str().unwrap(),
            "--out",
            out_a.to_str().unwrap(),
            "--sample-id",
            "sample",
        ],
    )
    .expect("cli ok");

    run_cli_capture(
        &workspace_b,
        &[
            "--platform",
            "test",
            "dna",
            "fastq",
            "preprocess",
            "--dry-run",
            "--r1",
            input_b.to_str().unwrap(),
            "--out",
            out_b.to_str().unwrap(),
            "--sample-id",
            "sample",
        ],
    )
    .expect("cli ok");

    let raw_a = std::fs::read_to_string(out_a.join("run_manifest.json")).expect("read manifest");
    let raw_b = std::fs::read_to_string(out_b.join("run_manifest.json")).expect("read manifest");
    let mut manifest_a: Value = serde_json::from_str(&raw_a).expect("parse manifest");
    let mut manifest_b: Value = serde_json::from_str(&raw_b).expect("parse manifest");
    scrub_paths(
        &mut manifest_a,
        workspace_a.path().to_str().unwrap_or_default(),
    );
    scrub_paths(
        &mut manifest_b,
        workspace_b.path().to_str().unwrap_or_default(),
    );

    let canonical_a =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest_a)
            .expect("canonical");
    let canonical_b =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest_b)
            .expect("canonical");
    assert_eq!(canonical_a, canonical_b);
}
