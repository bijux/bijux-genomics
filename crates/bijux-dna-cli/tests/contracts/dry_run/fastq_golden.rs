use bijux_dna::commands::run_with_args;
use serde_json::Value;

fn scrub_paths(value: &mut Value, root: &str) {
    match value {
        Value::String(s) => {
            if s.contains(root) {
                *s = s.replace(root, "<temp>");
            }
            if let Some(idx) = s.find("bench/") {
                *s = s[idx..].to_string();
            } else if let Some(idx) = s.find("run_artifacts/") {
                *s = s[idx..].to_string();
            }
            if s.contains("artifacts/isolates/") && s.contains('/') {
                let trimmed = s.trim_end_matches('/');
                if let Some(name) = trimmed.rsplit('/').next() {
                    *s = name.to_string();
                }
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

fn normalize_json(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items.iter_mut() {
                normalize_json(item);
            }
            items.sort_by(|a, b| {
                serde_json::to_string(a)
                    .expect("serialize")
                    .cmp(&serde_json::to_string(b).expect("serialize"))
            });
            items.dedup_by(|a, b| {
                serde_json::to_string(a).expect("serialize")
                    == serde_json::to_string(b).expect("serialize")
            });
        }
        Value::Object(map) => {
            for value in map.values_mut() {
                normalize_json(value);
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
    let workspace_images = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs")
        .join("images.toml");
    std::fs::copy(workspace_images, configs_dir.join("images.toml")).expect("write images");
    let workspace_tool_registry = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs")
        .join("tool_registry.toml");
    std::fs::copy(
        workspace_tool_registry,
        configs_dir.join("tool_registry.toml"),
    )
    .expect("write tool registry");
    let workspace_stages = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs")
        .join("stages.toml");
    std::fs::copy(workspace_stages, configs_dir.join("stages.toml")).expect("write stages");

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
    normalize_json(&mut graph);
    normalize_json(&mut manifest);
    let payload = serde_json::json!({
        "graph": graph,
        "manifest": manifest,
    });
    bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload).expect("canonical")
}

#[test]
fn cli_dry_run_output_is_deterministic() {
    let temp = tempfile::tempdir().expect("tempdir");
    let base_a = temp.path().join("a");
    let base_b = temp.path().join("b");
    let out_a = base_a.join("out");
    let out_b = base_b.join("out");
    std::fs::create_dir_all(&base_a).expect("create base a");
    std::fs::create_dir_all(&base_b).expect("create base b");

    let payload_a = run_dry_run(&base_a, &out_a);
    let payload_b = run_dry_run(&base_b, &out_b);

    let json_a: Value = serde_json::from_slice(&payload_a).expect("parse payload a");
    let json_b: Value = serde_json::from_slice(&payload_b).expect("parse payload b");

    let pipeline_a = json_a["graph"]["pipeline_id"].as_str().unwrap_or_default();
    let pipeline_b = json_b["graph"]["pipeline_id"].as_str().unwrap_or_default();
    assert_eq!(pipeline_a, pipeline_b);

    let mut steps_a: Vec<String> = json_a["graph"]["steps"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item["step_id"].as_str().map(ToOwned::to_owned))
        .collect();
    let mut steps_b: Vec<String> = json_b["graph"]["steps"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item["step_id"].as_str().map(ToOwned::to_owned))
        .collect();
    steps_a.sort();
    steps_b.sort();
    assert_eq!(steps_a, steps_b);
}
