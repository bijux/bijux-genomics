#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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
fn bench_readiness_all_domain_active_stage_catalog_reports_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-active-stage-catalog", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_active_stage_catalog.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/active-stage-catalog.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(66));
    assert_eq!(
        payload.get("stages_with_benchmark_ready_tools").and_then(serde_json::Value::as_u64),
        Some(66)
    );
    assert_eq!(
        payload.get("not_benchmark_ready_only_stage_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("stages_with_parser_rows").and_then(serde_json::Value::as_u64),
        Some(66)
    );
    assert_eq!(payload.get("stages_with_schema").and_then(serde_json::Value::as_u64), Some(66));
    assert_eq!(
        payload.get("stages_with_report_rows").and_then(serde_json::Value::as_u64),
        Some(66)
    );

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(17));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 66);

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
            && row.get("readiness_kind").and_then(serde_json::Value::as_str) == Some("smoke")
            && row.get("active_tool_count").and_then(serde_json::Value::as_u64) == Some(6)
            && row.get("benchmark_ready_tool_count").and_then(serde_json::Value::as_u64) == Some(6)
            && row.get("parser_row_count").and_then(serde_json::Value::as_u64) == Some(6)
            && row.get("parser_covered_row_count").and_then(serde_json::Value::as_u64) == Some(6)
            && row.get("schema_present").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
            && row.get("report_section_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("ancient_signal".to_string())])
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.trim_reads")
            && row.get("readiness_kind").and_then(serde_json::Value::as_str) == Some("smoke")
            && row.get("active_tool_count").and_then(serde_json::Value::as_u64) == Some(13)
            && row.get("benchmark_ready_tool_count").and_then(serde_json::Value::as_u64) == Some(13)
            && row.get("parser_row_count").and_then(serde_json::Value::as_u64) == Some(13)
            && row.get("parser_covered_row_count").and_then(serde_json::Value::as_u64) == Some(13)
            && row.get("schema_present").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
            && row.get("benchmark_ready_tool_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![
                    serde_json::Value::String("adapterremoval".to_string()),
                    serde_json::Value::String("alientrimmer".to_string()),
                    serde_json::Value::String("atropos".to_string()),
                    serde_json::Value::String("bbduk".to_string()),
                    serde_json::Value::String("cutadapt".to_string()),
                    serde_json::Value::String("fastp".to_string()),
                    serde_json::Value::String("fastx_clipper".to_string()),
                    serde_json::Value::String("leehom".to_string()),
                    serde_json::Value::String("prinseq".to_string()),
                    serde_json::Value::String("seqkit".to_string()),
                    serde_json::Value::String("skewer".to_string()),
                    serde_json::Value::String("trim_galore".to_string()),
                    serde_json::Value::String("trimmomatic".to_string()),
                ])
            && row.get("report_section_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("read_cleanup".to_string())])
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
            && row.get("readiness_kind").and_then(serde_json::Value::as_str) == Some("smoke")
            && row.get("active_tool_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_ready_tool_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("parser_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("parser_covered_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("schema_present").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
            && row.get("active_tool_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("bcftools".to_string())])
            && row.get("benchmark_ready_tool_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("bcftools".to_string())])
            && row.get("report_section_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("quality_control".to_string())])
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("readiness_kind").and_then(serde_json::Value::as_str) == Some("smoke")
            && row.get("active_tool_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_ready_tool_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("parser_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("parser_covered_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("schema_present").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_statuses").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("benchmark_ready".to_string())])
            && row.get("active_tool_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("bcftools".to_string())])
            && row.get("benchmark_ready_tool_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("bcftools".to_string())])
            && row.get("report_section_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("normalization".to_string())])
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
            && row.get("readiness_kind").and_then(serde_json::Value::as_str) == Some("smoke")
            && row.get("active_tool_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_ready_tool_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("parser_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("parser_covered_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("schema_present").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_row_count").and_then(serde_json::Value::as_u64) == Some(1)
            && row.get("benchmark_ready_tool_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("plink2".to_string())])
            && row.get("report_section_ids").and_then(serde_json::Value::as_array)
                == Some(&vec![serde_json::Value::String("population_structure".to_string())])
    }));
    assert!(
        rows.iter().all(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) != Some("fastq.index_reference")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    != Some("fastq.report_qc")
        }),
        "active stage catalog must exclude not-benchmark-ready-only stages"
    );
}
