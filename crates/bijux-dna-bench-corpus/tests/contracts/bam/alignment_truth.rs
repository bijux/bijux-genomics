//! Direct corpus owner admission of declared BAM alignment truth.
use bijux_dna_bench_corpus::bam::validate_bam_alignment_truth_manifest_path;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST: &str = "benchmarks/tests/fixtures/science/bam-alignment-truth/manifest.toml";
const EXPECTED: &str = "benchmarks/tests/fixtures/science/bam-alignment-truth/expected.json";

fn isolated_truth() -> (PathBuf, tempfile::TempDir, PathBuf, toml::Value, serde_json::Value) {
    let root = crate::support::repo_root();
    let scratch = tempfile::tempdir().expect("truth fixture scratch");
    let path = scratch.path().join("manifest.toml");
    let manifest = toml::from_str(&fs::read_to_string(root.join(MANIFEST)).expect("read manifest"))
        .expect("typed TOML fixture");
    let expected = serde_json::from_slice(&fs::read(root.join(EXPECTED)).expect("read bundle"))
        .expect("typed JSON fixture");
    (root, scratch, path, manifest, expected)
}

fn write_truth(path: &Path, manifest: &toml::Value, expected: &serde_json::Value) {
    fs::write(path, toml::to_string(manifest).expect("encode fixture manifest"))
        .expect("write fixture manifest");
    fs::write(
        path.with_file_name("expected.json"),
        serde_json::to_vec(expected).expect("encode bundle"),
    )
    .expect("write fixture bundle");
}

#[test]
fn governed_alignment_truth_retains_counts_record_order_and_report_fields() {
    let root = crate::support::repo_root();
    let report = validate_bam_alignment_truth_manifest_path(&root, &root.join(MANIFEST))
        .expect("admit real governed alignment records");
    assert_eq!((report.validated_sample_count, report.validated_record_count), (4, 14));
    assert!(report.valid);
    assert_eq!(
        report.checked_samples,
        [
            "human_like_validation",
            "human_like_duplicate_flagged_multicontig",
            "human_like_mapq_threshold_ladder",
            "human_like_mixed_filter_constraints"
        ]
    );
    let payload = serde_json::to_value(report).expect("encode same report");
    let fields = payload
        .as_object()
        .expect("report object")
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        fields,
        std::collections::BTreeSet::from([
            "schema_version",
            "fixture_id",
            "manifest_path",
            "expected_path",
            "validated_sample_count",
            "validated_record_count",
            "valid",
            "checked_samples"
        ])
    );
}

#[test]
fn manifest_identity_and_sources_reject_before_missing_expected_bundle() {
    let (root, _scratch, path, original, expected) = isolated_truth();
    let mut manifest = original.clone();
    let sample = manifest["sample_ids"].as_array().expect("sample IDs")[0].clone();
    manifest["sample_ids"] = toml::Value::Array(vec![sample.clone(), sample]);
    manifest["expected_path"] = "missing-expected.json".into();
    write_truth(&path, &manifest, &expected);
    let error =
        validate_bam_alignment_truth_manifest_path(&root, &path).expect_err("duplicate rejected");
    assert!(error.to_string().contains("repeats sample_id"));
    let mut manifest = original;
    manifest["source_paths"] = toml::Value::Array(vec!["missing-declared-alignment.sam".into()]);
    manifest["expected_path"] = "missing-expected.json".into();
    write_truth(&path, &manifest, &expected);
    let error = validate_bam_alignment_truth_manifest_path(&root, &path)
        .expect_err("missing source rejected");
    assert!(error.to_string().contains("source path is missing"));
}

#[test]
fn expected_sample_order_and_independent_alignment_aggregates_are_admitted() {
    let (root, _scratch, path, manifest, original) = isolated_truth();
    let mut reordered = original.clone();
    reordered["sample_truths"].as_array_mut().expect("truth rows").swap(0, 1);
    write_truth(&path, &manifest, &reordered);
    let error = validate_bam_alignment_truth_manifest_path(&root, &path)
        .expect_err("sample order rejected");
    assert!(error.to_string().contains("must contain samples"));
    let variants = [
        ("total_reads", serde_json::json!(3), "does not equal record count"),
        ("unmapped_reads", serde_json::json!(1), "does not equal mapped+unmapped"),
        ("mapped_contigs", serde_json::json!(["foreign-contig"]), "mapped_contigs drifted"),
        ("mapped_positions", serde_json::json!([]), "mapped_positions count"),
        ("mapped_mapq_classes", serde_json::json!([]), "mapped MAPQ class count"),
        ("cigar_classes", serde_json::json!([]), "cigar class count"),
    ];
    for (field, value, rejection) in variants {
        let mut expected = original.clone();
        expected["sample_truths"][0][field] = value;
        write_truth(&path, &manifest, &expected);
        let error = validate_bam_alignment_truth_manifest_path(&root, &path)
            .expect_err("aggregate rejected");
        assert!(error.to_string().contains(rejection), "{field}: {error:#}");
    }
}

#[test]
fn expected_record_substitution_cannot_pass_with_unchanged_aggregates() {
    let (root, _scratch, path, manifest, mut expected) = isolated_truth();
    expected["sample_truths"][0]["records"][0]["qname"] = "substituted-read".into();
    write_truth(&path, &manifest, &expected);
    let error = validate_bam_alignment_truth_manifest_path(&root, &path)
        .expect_err("native observed record identity rejects substitution");
    assert!(error.to_string().contains("truth drifted for sample `human_like_validation`"));
    assert_eq!(
        fs::read(&path).expect("read manifest"),
        toml::to_string(&manifest).expect("encode manifest").as_bytes()
    );
    assert_eq!(
        fs::read(path.with_file_name("expected.json")).expect("read expected"),
        serde_json::to_vec(&expected).expect("encode expected")
    );
}

#[test]
fn expected_alignment_count_overflow_refuses_without_unwinding() -> anyhow::Result<()> {
    use anyhow::Context as _;
    let (root, _scratch, path, manifest, original) = isolated_truth();
    write_truth(&path, &manifest, &original);
    let control = validate_bam_alignment_truth_manifest_path(&root, &path)?;
    assert!(control.valid);
    assert_eq!((control.validated_sample_count, control.validated_record_count), (4, 14));
    let mut observations = Vec::new();
    for field in ["mapped_and_unmapped", "mapped_positions", "mapped_mapq_classes", "cigar_classes"]
    {
        let mut expected = original.clone();
        let sample = &mut expected["sample_truths"][0];
        if field == "mapped_and_unmapped" {
            sample["mapped_reads"] = serde_json::json!(u64::MAX);
            sample["unmapped_reads"] = serde_json::json!(3_u64);
        } else {
            let first = sample[field]
                .as_array()
                .context("governed aggregate rows")?
                .first()
                .context("governed aggregate row")?
                .clone();
            let mut largest = first.clone();
            let mut remainder = first;
            largest["read_count"] = serde_json::json!(u64::MAX);
            remainder["read_count"] = serde_json::json!(3_u64);
            sample[field] = serde_json::json!([largest, remainder]);
        }
        write_truth(&path, &manifest, &expected);
        let observed =
            std::panic::catch_unwind(|| validate_bam_alignment_truth_manifest_path(&root, &path));
        observations.push((field, observed));
    }
    write_truth(&path, &manifest, &original);
    let restored = validate_bam_alignment_truth_manifest_path(&root, &path)?;
    assert!(restored.valid);
    for (field, observed) in observations {
        let error = observed
            .ok()
            .with_context(|| format!("{field} count overflow must return Err without unwinding"))?
            .err()
            .with_context(|| format!("{field} count overflow must not be accepted"))?;
        let message = format!("{error:#}");
        assert!(message.contains("overflow"), "{field}: {message}");
        assert!(message.contains("human_like_validation"), "{field}: {message}");
    }
    Ok(())
}
