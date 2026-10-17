#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;

use crate::support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = support::bijux_dna_command(&repo_root)
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
fn bench_readiness_all_domain_expected_result_coverage_reports_complete_active_rows(
) -> anyhow::Result<()> {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-expected-result-coverage",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_expected_result_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/all-domains/expected-result-coverage.tsv")
    );
    let rows = support::json_array(&payload, "rows")?;
    let row_count = u64::try_from(rows.len()).expect("row count fits u64");
    assert_eq!(support::json_u64(&payload, "row_count"), Some(row_count));
    let stage_count = u64::try_from(
        rows.iter()
            .filter_map(|row| row.get("stage_id").and_then(serde_json::Value::as_str))
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .expect("stage count fits u64");
    let tool_count = u64::try_from(
        rows.iter()
            .filter_map(|row| row.get("tool_id").and_then(serde_json::Value::as_str))
            .collect::<BTreeSet<_>>()
            .len(),
    )
    .expect("tool count fits u64");
    assert_eq!(support::json_u64(&payload, "result_id_count"), Some(row_count));
    assert_eq!(support::json_u64(&payload, "stage_count"), Some(stage_count));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(tool_count));
    assert_eq!(
        payload.get("expected_result_binding_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(
        payload.get("covered_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(payload.get("missing_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("coverage_percent").and_then(serde_json::Value::as_f64), Some(100.0));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let domain_counts = support::json_object(&payload, "domain_counts")?;
    for domain in ["fastq", "bam", "vcf"] {
        let expected_count = u64::try_from(
            rows.iter()
                .filter(|row| row.get("domain").and_then(serde_json::Value::as_str) == Some(domain))
                .count(),
        )
        .expect("domain count fits u64");
        assert_eq!(
            domain_counts.get(domain).and_then(serde_json::Value::as_u64),
            Some(expected_count)
        );
    }
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let report_section_counts = payload
        .get("report_section_counts")
        .and_then(serde_json::Value::as_object)
        .expect("report section counts");
    assert_eq!(
        report_section_counts
            .values()
            .map(|count| count.as_u64().expect("section count"))
            .sum::<u64>(),
        row_count
    );
    assert_eq!(
        report_section_counts.get("variant_calling").and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        report_section_counts.get("normalization").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        report_section_counts.get("runs_of_homozygosity").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let coverage_status_counts = support::json_object(&payload, "coverage_status_counts")?;
    assert_eq!(
        coverage_status_counts.get("covered").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert!(rows.iter().all(|row| {
        row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
    }));

    let variant_calling_bindings = rows
        .iter()
        .filter(|row| {
            row.get("report_section").and_then(serde_json::Value::as_str) == Some("variant_calling")
        })
        .map(|row| {
            (
                row.get("stage_id")
                    .and_then(serde_json::Value::as_str)
                    .expect("variant-calling stage id"),
                row.get("tool_id")
                    .and_then(serde_json::Value::as_str)
                    .expect("variant-calling tool id"),
            )
        })
        .collect::<BTreeSet<_>>();
    assert!(variant_calling_bindings.contains(&("vcf.call_gl", "angsd")));
    assert!(variant_calling_bindings.contains(&("vcf.call_pseudohaploid", "angsd")));

    let taxonomy_rows = rows
        .iter()
        .filter(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
        })
        .collect::<Vec<_>>();
    assert_eq!(taxonomy_rows.len(), 4);
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        let taxonomy = taxonomy_rows
            .iter()
            .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id))
            .unwrap_or_else(|| panic!("taxonomy coverage row missing for {tool_id}"));
        assert_eq!(
            taxonomy.get("adapter_id").and_then(serde_json::Value::as_str),
            Some("fastq.adapter.screen_taxonomy")
        );
        assert_eq!(
            taxonomy.get("parser_id").and_then(serde_json::Value::as_str),
            Some("fastq.parser.screen_taxonomy")
        );
        assert_eq!(
            taxonomy.get("schema_id").and_then(serde_json::Value::as_str),
            Some("fastq_screen_taxonomy_v1")
        );
        assert_eq!(
            taxonomy.get("report_section").and_then(serde_json::Value::as_str),
            Some("contamination_screening")
        );
        let metrics = taxonomy
            .get("expected_metrics")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("expected_metrics missing for {tool_id}"));
        for metric_id in [
            "classification_report_json",
            "taxonomy_database_id",
            "classified_reads",
            "unclassified_reads",
            "classified_fraction",
            "unclassified_fraction",
            "top_taxa",
        ] {
            assert!(
                metrics.iter().any(|value| value.as_str() == Some(metric_id)),
                "taxonomy coverage row for {tool_id} must include expected metric `{metric_id}`"
            );
        }
    }
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("bam.adapter.kinship")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("bam.parser.kinship")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bam_kinship_normalized_v1")
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("sample_identity")
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("vcf.adapter.transform")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("vcf.parser.vcf_output")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.schemas.bench.vcf-normalized-metrics.postprocess.v1")
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("normalization")
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some(
                "vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle",
            )
            && row.get("report_section").and_then(serde_json::Value::as_str) == Some("imputation")
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("vcf:vcf_production_regression:vcf.roh:vcf_cohort:plink2")
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("runs_of_homozygosity")
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
    }));

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(
        violations.is_empty(),
        "all active rows must retain complete governed expected-result coverage"
    );
    Ok(())
}
