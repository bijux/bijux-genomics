#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_phasing_family_adapter_writes_governed_json_files() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    for command_name in
        ["render-vcf-shapeit5-adapter", "render-vcf-eagle-adapter", "render-vcf-beagle-adapter"]
    {
        let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
            .current_dir(&repo_root)
            .env("HOME", home.path())
            .env("BIJUX_SKIP_QA", "1")
            .env("BIJUX_ALLOW_SILVER", "1")
            .env("BIJUX_SKIP_IMAGE_CHECK", "1")
            .args(["bench", "readiness", command_name])
            .output()
            .expect("run cli");

        assert!(
            output.status.success(),
            "command failed: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    for (tool_id, schema_version, benchmark_ready_count) in [
        ("shapeit5", "bijux.bench.readiness.vcf_shapeit5_adapter.v1", 1_u64),
        ("eagle", "bijux.bench.readiness.vcf_eagle_adapter.v1", 0_u64),
        ("beagle", "bijux.bench.readiness.vcf_beagle_adapter.v1", 0_u64),
    ] {
        let report_path =
            repo_root.join(format!("benchmarks/readiness/adapters/{tool_id}.vcf.json"));
        assert!(report_path.is_file(), "{tool_id} VCF adapter JSON must exist");

        let payload = serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(&report_path).expect("read phasing adapter JSON"),
        )
        .expect("parse phasing adapter JSON");

        assert_eq!(
            payload.get("schema_version").and_then(serde_json::Value::as_str),
            Some(schema_version)
        );
        assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(1));
        assert_eq!(
            payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
            Some(benchmark_ready_count)
        );
        let row = payload
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .and_then(|rows| rows.first())
            .expect("adapter row");
        assert_eq!(row.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.phasing"));

        let declared_output_ids = row
            .get("declared_outputs")
            .and_then(serde_json::Value::as_array)
            .expect("declared outputs")
            .iter()
            .filter_map(|artifact| artifact.get("artifact_id"))
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        for artifact_id in [
            "phased_vcf",
            "phased_vcf_tbi",
            "phase_block_stats",
            "switch_error_proxy",
            "phasing_qc",
            "phasing_manifest",
            "phasing_log",
        ] {
            assert!(
                declared_output_ids.iter().any(|candidate| candidate == &artifact_id),
                "{tool_id} row must declare `{artifact_id}`"
            );
        }

        let panel_vcf_path = repo_root.join(
            row.get("panel_vcf_path").and_then(serde_json::Value::as_str).expect("panel_vcf_path"),
        );
        let genetic_map_path = repo_root.join(
            row.get("genetic_map_path")
                .and_then(serde_json::Value::as_str)
                .expect("genetic_map_path"),
        );
        assert!(panel_vcf_path.is_file(), "{tool_id} row must materialize the governed panel");
        assert!(genetic_map_path.is_file(), "{tool_id} row must materialize the governed map");
    }
}
