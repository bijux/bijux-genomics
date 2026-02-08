use bijux_dna_cli::commands::run_with_args;
use serde_json::Value;

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

#[allow(clippy::too_many_lines)]
fn run_dry_run(base: &std::path::Path, out_dir: &std::path::Path) -> Vec<u8> {
    let input = base.join("reads.fastq");
    std::fs::write(&input, "@r1\nACGT\n+\n####\n").expect("write fastq");

    let configs_dir = base.join("configs");
    let profiles_dir = configs_dir.join("profiles");
    std::fs::create_dir_all(&profiles_dir).expect("create profiles");
    std::fs::write(
        profiles_dir.join("local.toml"),
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
    std::fs::write(
        configs_dir.join("images.toml"),
        r#"
fastqvalidator_official = { version = "0.0.0" }
fastqc = { version = "0.0.0" }
fastp = { version = "0.0.0" }
seqkit = { version = "0.0.0" }
seqkit_stats = { version = "0.0.0" }
multiqc = { version = "0.0.0" }
"#,
    )
    .expect("write images");

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("domain"),
            base.join("domain"),
        )
        .expect("symlink domain");
        std::os::unix::fs::symlink(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("assets"),
            base.join("assets"),
        )
        .expect("symlink assets");
    }

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

    std::env::set_var("BIJUX_SKIP_QA", "1");
    std::env::set_var("BIJUX_ALLOW_SILVER", "1");
    std::env::set_var("BIJUX_SKIP_IMAGE_CHECK", "1");
    let args = [
        "bijux",
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
    ];
    run_with_args(&args, base).expect("run cli");

    let artifacts_root = out_dir
        .join("bench")
        .join("preprocess")
        .join("sample")
        .join("run_artifacts");
    let graph_raw = std::fs::read_to_string(artifacts_root.join("graph.json")).expect("read graph");
    let manifest_raw =
        std::fs::read_to_string(out_dir.join("run_manifest.json")).expect("read manifest");
    let mut graph: Value = serde_json::from_str(&graph_raw).expect("parse graph");
    let mut manifest: Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    let root_str = base.to_str().unwrap_or_default();
    scrub_paths(&mut graph, root_str);
    scrub_paths(&mut manifest, root_str);
    let payload = serde_json::json!({
        "graph": graph,
        "manifest": manifest,
    });
    bijux_core::contract::canonical::to_canonical_json_bytes(&payload).expect("canonical")
}

#[test]
fn cli_dry_run_output_is_deterministic() {
    let temp_a = tempfile::tempdir().expect("tempdir");
    let temp_b = tempfile::tempdir().expect("tempdir");

    let out_a = temp_a.path().join("out");
    let out_b = temp_b.path().join("out");

    let payload_a = run_dry_run(temp_a.path(), &out_a);
    let payload_b = run_dry_run(temp_b.path(), &out_b);

    assert_eq!(payload_a, payload_b);
}
