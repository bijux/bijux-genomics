#![cfg(test)]
//! Retained admixture parser and service assertions.
use crate::admixture::observations::records::parse_admixture_rows;
use crate::admixture::workflow::run_local_vcf_admixture_smoke;
use bijux_dna_bench_corpus::vcf::SampleMetadataRow;
use std::collections::BTreeMap;

#[test]
fn admixture_rows_reject_duplicate_samples() {
    let temp = tempfile::tempdir().expect("tempdir");
    let table = temp.path().join("admixture.tsv");
    bijux_dna_infra::write_payload(
        &table,
        "sample\tcluster_1\tcluster_2\nsample_a\t0.4\t0.6\nsample_a\t0.3\t0.7\n",
    )
    .expect("write admixture table");
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "sample_a".to_string(),
        SampleMetadataRow {
            sample_id: "sample_a".to_string(),
            population_id: "cohort_alpha".to_string(),
            sex: "female".to_string(),
            role: "cohort".to_string(),
            description: "sample a".to_string(),
        },
    );
    let mut populations = BTreeMap::new();
    populations.insert("cohort_alpha".to_string(), "Cohort Alpha".to_string());
    let err = parse_admixture_rows(
        &table,
        &metadata,
        &populations,
        &["cluster_1".to_string(), "cluster_2".to_string()],
        2,
        "complete",
    )
    .expect_err("duplicate samples");
    assert!(err.to_string().contains("duplicated sample"));
}

#[test]
fn admixture_rows_require_ancestry_sum_to_one() {
    let temp = tempfile::tempdir().expect("tempdir");
    let table = temp.path().join("admixture.tsv");
    bijux_dna_infra::write_payload(&table, "sample\tcluster_1\tcluster_2\nsample_a\t0.4\t0.5\n")
        .expect("write admixture table");
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "sample_a".to_string(),
        SampleMetadataRow {
            sample_id: "sample_a".to_string(),
            population_id: "cohort_alpha".to_string(),
            sex: "female".to_string(),
            role: "cohort".to_string(),
            description: "sample a".to_string(),
        },
    );
    let mut populations = BTreeMap::new();
    populations.insert("cohort_alpha".to_string(), "Cohort Alpha".to_string());
    let err = parse_admixture_rows(
        &table,
        &metadata,
        &populations,
        &["cluster_1".to_string(), "cluster_2".to_string()],
        2,
        "complete",
    )
    .expect_err("ancestry sum mismatch");
    assert!(err.to_string().contains("expected 1.0"));
}

#[test]
fn governed_vcf_admixture_smoke_reports_complete_cohort_rows() {
    let fixture = crate::test_support::fixture();
    let repo_root = fixture.path().to_path_buf();
    let report =
        run_local_vcf_admixture_smoke(&crate::test_support::inputs(), &repo_root, "plink2")
            .expect("run local admixture smoke");
    assert_eq!(report.stage_id, "vcf.admixture");
    assert_eq!(report.tool_id, "plink2");
    assert_eq!(report.corpus_id, "vcf_production_regression");
    assert_eq!(report.input_fixture_id, "vcf_mini_multisample_cohort");
    assert_eq!(report.selected_k, 2);
    assert_eq!(report.status, "complete");
    assert_eq!(report.sample_count, 4);
    assert_eq!(report.population_count, 2);
    assert_eq!(report.cluster_headers, vec!["cluster_1", "cluster_2"]);
    assert_eq!(
        report.rows.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
        vec!["sample_a", "sample_b", "sample_c", "sample_d"]
    );
    assert!(
        report.rows.iter().all(|row| {
            row.clusters.values().copied().sum::<f64>() >= 1.0 - 1e-6
                && row.clusters.values().copied().sum::<f64>() <= 1.0 + 1e-6
        }),
        "expected ancestry rows to stay normalized"
    );
}

#[test]
fn admixture_rows_require_finite_bounded_ancestry_components() -> anyhow::Result<()> {
    use anyhow::Context as _;
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .context("population crate must be in the workspace")?;
    let output_owner = workspace.join("artifacts/tests/ancestry-component-admission");
    bijux_dna_infra::ensure_dir(&output_owner)?;
    let fixture = bijux_dna_infra::temp_dir_in(&output_owner, "ancestry-observation-")?;
    let table = fixture.path().join("admixture.tsv");
    let metadata = BTreeMap::from([(
        "sample_a".to_string(),
        SampleMetadataRow {
            sample_id: "sample_a".into(),
            population_id: "cohort_alpha".into(),
            sex: "female".into(),
            role: "cohort".into(),
            description: "sample a".into(),
        },
    )]);
    let populations = BTreeMap::from([("cohort_alpha".to_string(), "Cohort Alpha".to_string())]);
    let headers = ["cluster_1".to_string(), "cluster_2".to_string()];
    for (first, second, diagnostic) in [
        ("NaN", "0.0", "must be finite"),
        ("inf", "-inf", "must be finite"),
        ("1.2", "-0.2", "must be within [0, 1]"),
        ("-0.2", "1.2", "must be within [0, 1]"),
    ] {
        let payload = format!("sample\tcluster_1\tcluster_2\nsample_a\t{first}\t{second}\n");
        std::fs::write(&table, &payload)?;
        let error = parse_admixture_rows(&table, &metadata, &populations, &headers, 2, "complete")
            .err()
            .context("invalid ancestry components were admitted")?;
        assert!(format!("{error:#}").contains(diagnostic), "{error:#}");
        assert_eq!(std::fs::read_to_string(&table)?, payload);
    }
    for (first, second) in [("0.4", "0.6"), ("0.0", "1.0"), ("1.0", "0.0")] {
        std::fs::write(
            &table,
            format!("sample\tcluster_1\tcluster_2\nsample_a\t{first}\t{second}\n"),
        )?;
        let rows = parse_admixture_rows(&table, &metadata, &populations, &headers, 2, "complete")?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].clusters["cluster_1"].to_bits(), first.parse::<f64>()?.to_bits());
        assert_eq!(rows[0].clusters["cluster_2"].to_bits(), second.parse::<f64>()?.to_bits());
        assert_eq!(rows[0].population_label, "Cohort Alpha");
        assert_eq!(rows[0].status, "complete");
    }
    Ok(())
}
