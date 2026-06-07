#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_normalized_metrics_schema_writes_governed_schema_files() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-vcf-normalized-metrics-schema"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/schemas/vcf-normalized-metrics.v1.json");

    let schema_path = repo_root.join(rendered_path.trim());
    let schema = std::fs::read_to_string(&schema_path).expect("read shared schema");
    let parsed: serde_json::Value = serde_json::from_str(&schema).expect("parse shared schema");

    assert_eq!(
        parsed.get("$schema").and_then(serde_json::Value::as_str),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        parsed.get("$id").and_then(serde_json::Value::as_str),
        Some("bijux.schemas.bench.vcf-normalized-metrics.v1")
    );
    assert_eq!(parsed.get("oneOf").and_then(serde_json::Value::as_array).map(Vec::len), Some(20));

    let stage_defs = parsed
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .expect("stage defs");
    assert_eq!(stage_defs.len(), 20);

    let pca = stage_defs
        .get("vcf.pca")
        .and_then(|value| value.get("x-bijux-extension-id"))
        .and_then(serde_json::Value::as_str);
    assert_eq!(pca, Some("vcf_pca_normalized_v1"));

    let stage_dir = repo_root.join("benchmarks/schemas/vcf-normalized-metrics");
    let pca_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(stage_dir.join("pca.v1.json")).expect("read pca schema"),
    )
    .expect("parse pca schema");
    assert_eq!(
        pca_schema.get("$id").and_then(serde_json::Value::as_str),
        Some("bijux.schemas.bench.vcf-normalized-metrics.pca.v1")
    );
    assert_eq!(
        pca_schema
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.get(1))
            .and_then(|value| value.get("properties"))
            .and_then(|value| value.get("schema_version"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_str),
        Some("bijux.vcf.pca.v1")
    );

    let admixture_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(stage_dir.join("admixture.v1.json"))
            .expect("read admixture schema"),
    )
    .expect("parse admixture schema");
    assert_eq!(
        admixture_schema
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.get(1))
            .and_then(|value| value.get("properties"))
            .and_then(|value| value.get("schema_version"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_str),
        Some("bijux.vcf.admixture.v1")
    );
}
