#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
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
fn bench_readiness_all_domain_expected_benchmark_results_tracks_governed_rows() -> anyhow::Result<()>
{
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-expected-benchmark-results",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_expected_benchmark_results.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/expected-benchmark-results-all-domains.tsv")
    );
    let rows = support::json_array(&payload, "rows")?;
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    assert_eq!(u64::try_from(rows.len()).expect("row count fits u64"), row_count);
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
    assert_eq!(payload.get("corpus_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("asset_profile_count").and_then(serde_json::Value::as_u64), Some(15));

    let domain_counts = support::json_object(&payload, "domain_counts")?;
    for domain in ["fastq", "bam", "vcf"] {
        let expected_count = u64::try_from(
            rows.iter()
                .filter(|row| row.get("domain").and_then(serde_json::Value::as_str) == Some(domain))
                .count(),
        )
        .expect("domain count fits u64");
        assert!(expected_count > 0, "{domain} rows must be present");
        assert_eq!(
            domain_counts.get(domain).and_then(serde_json::Value::as_u64),
            Some(expected_count)
        );
    }
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let section_counts = support::json_object(&payload, "report_section_counts")?;
    assert_eq!(support::object_u64_sum(section_counts), row_count);
    assert_eq!(section_counts.get("variant_calling").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(section_counts.get("imputation").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        section_counts.get("population_structure").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        section_counts.get("runs_of_homozygosity").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    let result_ids = rows
        .iter()
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(u64::try_from(result_ids.len()).expect("result count fits u64"), row_count);

    let taxonomy_rows = rows
        .iter()
        .filter(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
        })
        .collect::<Vec<_>>();
    assert_eq!(taxonomy_rows.len(), 4);
    for (tool_id, expected_outputs) in [
        ("centrifuge", vec!["classification_report_json", "screen_report_tsv"]),
        ("kaiju", vec!["classification_report_json", "screen_report_tsv"]),
        (
            "kraken2",
            vec![
                "classification_report_json",
                "screen_report_tsv",
                "unclassified_reads_r1",
                "unclassified_reads_r2",
            ],
        ),
        ("krakenuniq", vec!["classification_report_json", "screen_report_tsv"]),
    ] {
        let taxonomy = taxonomy_rows
            .iter()
            .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id))
            .unwrap_or_else(|| panic!("taxonomy result row missing for {tool_id}"));
        assert_eq!(
            taxonomy.get("asset_profile_id").and_then(serde_json::Value::as_str),
            Some("database_artifact_id+taxonomy_database_root")
        );
        assert_eq!(
            taxonomy.get("report_section").and_then(serde_json::Value::as_str),
            Some("contamination_screening")
        );
        let outputs = taxonomy
            .get("expected_outputs")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("expected_outputs missing for {tool_id}"));
        for output_id in expected_outputs {
            assert!(
                outputs.iter().any(|value| value.as_str() == Some(output_id)),
                "taxonomy result row for {tool_id} must include expected output `{output_id}`"
            );
        }
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
                "taxonomy result row for {tool_id} must include expected metric `{metric_id}`"
            );
        }
    }

    let kinship = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
        })
        .expect("kinship result row");
    assert_eq!(
        kinship.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("reference_fasta+reference_panel")
    );
    assert_eq!(
        kinship.get("report_section").and_then(serde_json::Value::as_str),
        Some("sample_identity")
    );
    assert!(kinship.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
        |outputs| outputs.iter().any(|value| value.as_str() == Some("kinship_report"))
    ));
    assert!(kinship.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
        |metrics| metrics.iter().any(|value| value.as_str() == Some("observed_max_overlap_snps"))
    ));

    let vcf_call = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
        })
        .expect("VCF call result row");
    assert_eq!(
        vcf_call.get("report_section").and_then(serde_json::Value::as_str),
        Some("variant_calling")
    );
    assert!(vcf_call
        .get("expected_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs.iter().any(|value| value.as_str() == Some("called_vcf"))));
    assert!(vcf_call.get("expected_metrics").and_then(serde_json::Value::as_array).is_some_and(
        |metrics| metrics.iter().any(|value| value.as_str() == Some("variant_count"))
    ));

    let vcf_postprocess = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools")
        })
        .expect("VCF postprocess result row");
    assert_eq!(
        vcf_postprocess.get("report_section").and_then(serde_json::Value::as_str),
        Some("normalization")
    );
    assert!(vcf_postprocess
        .get("expected_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| outputs
            .iter()
            .any(|value| value.as_str() == Some("postprocess_vcf"))));
    assert!(vcf_postprocess
        .get("expected_metrics")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|metrics| metrics.iter().any(|value| value.as_str() == Some("readable_vcf"))));

    let imputation_metrics = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some(
                    "vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle",
                )
        })
        .expect("VCF imputation metrics row");
    assert_eq!(
        imputation_metrics.get("report_section").and_then(serde_json::Value::as_str),
        Some("imputation")
    );
    assert!(imputation_metrics
        .get("expected_metrics")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|metrics| metrics
            .iter()
            .any(|value| value.as_str() == Some("mean_info_score"))));

    let vcf_pca = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.pca:vcf_cohort:eigensoft")
        })
        .expect("VCF PCA result row");
    assert_eq!(
        vcf_pca.get("report_section").and_then(serde_json::Value::as_str),
        Some("population_structure")
    );

    let vcf_roh = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.roh:vcf_cohort:plink2")
        })
        .expect("VCF roh result row");
    assert_eq!(
        vcf_roh.get("report_section").and_then(serde_json::Value::as_str),
        Some("runs_of_homozygosity")
    );
    let population_structure = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.population_structure:vcf_cohort:plink2")
        })
        .expect("VCF population-structure row");
    assert_eq!(
        population_structure.get("report_section").and_then(serde_json::Value::as_str),
        Some("population_structure")
    );
    assert!(population_structure
        .get("expected_outputs")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|outputs| {
            outputs.iter().any(|value| value.as_str() == Some("population_structure_report"))
        }));
    Ok(())
}
