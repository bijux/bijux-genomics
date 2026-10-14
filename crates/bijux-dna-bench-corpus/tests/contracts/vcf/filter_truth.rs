//! Corpus-owned filter truth rejects invalid declarations without publishing or altering evidence.
use std::fs;
use std::path::PathBuf;

use bijux_dna_bench_corpus::vcf::validate_vcf_filter_truth_manifest_path;

const MANIFEST: &str = "benchmarks/tests/fixtures/science/vcf-filter-truth/manifest.toml";
const EXPECTED: &str = "benchmarks/tests/fixtures/science/vcf-filter-truth/expected.json";

struct Fixture {
    root: PathBuf,
    scratch: tempfile::TempDir,
    path: PathBuf,
    manifest: toml::Value,
    expected: serde_json::Value,
}

fn fixture() -> Fixture {
    let root = crate::support::repo_root();
    let scratch = tempfile::tempdir().expect("filter truth scratch");
    let path = scratch.path().join("manifest.toml");
    let manifest = toml::from_str(&fs::read_to_string(root.join(MANIFEST)).expect("read manifest"))
        .expect("decode declared truth fixture");
    let expected = serde_json::from_slice(&fs::read(root.join(EXPECTED)).expect("read expected"))
        .expect("decode expected truth fixture");
    Fixture { root, scratch, path, manifest, expected }
}

fn write_fixture(fixture: &Fixture) {
    fs::write(&fixture.path, toml::to_string(&fixture.manifest).expect("encode manifest"))
        .expect("write isolated declaration");
    fs::write(
        fixture.scratch.path().join("expected.json"),
        serde_json::to_vec(&fixture.expected).expect("encode expected"),
    )
    .expect("write isolated expected observation");
}

#[test]
fn declared_truth_validation_retains_inputs_and_sorted_stage_inventory() {
    let root = crate::support::repo_root();
    let manifest: toml::Value =
        toml::from_str(&fs::read_to_string(root.join(MANIFEST)).expect("read declaration"))
            .expect("decode declaration");
    let paths = manifest["source_paths"]
        .as_array()
        .expect("source paths")
        .iter()
        .map(|path| root.join(path.as_str().expect("declared path")))
        .chain([root.join(MANIFEST), root.join(EXPECTED)])
        .collect::<Vec<_>>();
    let before =
        paths.iter().map(|path| fs::read(path).expect("capture source bytes")).collect::<Vec<_>>();
    let report = validate_vcf_filter_truth_manifest_path(&root, &root.join(MANIFEST))
        .expect("admit canonical parser and native filter observations");
    assert_eq!(report.validated_case_count, 2);
    assert_eq!(report.validated_stage_ids, ["vcf.damage_filter", "vcf.filter"]);
    assert_eq!(report.validated_tool_ids, ["bcftools"]);
    assert!(report.valid);
    for (path, bytes) in paths.iter().zip(before) {
        assert_eq!(fs::read(path).expect("re-read declared source"), bytes, "{}", path.display());
    }
}

#[test]
fn manifest_reference_failures_precede_bundle_and_support_failures() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    fixture.manifest["source_paths"] =
        toml::Value::Array(vec!["missing-declared-filter-source".into()]);
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("declared source admission precedes expected-bundle acquisition");
    assert!(error.to_string().contains("source path is missing"));
    fixture.manifest = original;
    fixture.manifest["cases"][0]["artifact_root_path"] = "missing-filter-artifact-root".into();
    fixture.manifest["cases"][0]["stage_id"] = "unsupported-stage".into();
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("artifact admission precedes stage support and expected acquisition");
    assert!(error.to_string().contains("artifact root is missing"));
}

#[test]
fn manifest_case_identity_and_supported_combinations_reject_independently() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    let duplicate = fixture.manifest["cases"][0].clone();
    fixture.manifest["cases"].as_array_mut().expect("declared cases").push(duplicate);
    write_fixture(&fixture);
    let error = validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("duplicate declared case is rejected");
    assert!(error.to_string().contains("repeats case_id"));
    for (field, value, rejection) in [
        ("truth_kind", "damage_removal", "unsupported tool/stage/truth combination"),
        ("tool_id", "angsd", "unsupported tool/stage/truth combination"),
        ("case_id", "", "empty identity fields"),
    ] {
        fixture.manifest = original.clone();
        fixture.manifest["cases"][0][field] = value.into();
        write_fixture(&fixture);
        let error = validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("case support invariant rejects declaration");
        assert!(error.to_string().contains(rejection), "{field}: {error:#}");
    }
}

#[test]
fn expected_case_sets_preserve_order_and_duplicate_collapse_without_foreign_identity() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    fixture.expected["cases"].as_array_mut().expect("expected cases").reverse();
    write_fixture(&fixture);
    let report = validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
        .expect("existing set-based expected identity accepts reordered rows");
    assert_eq!(report.validated_case_count, 2);
    let duplicate = fixture.expected["cases"][0].clone();
    fixture.expected["cases"].as_array_mut().expect("expected cases").push(duplicate);
    write_fixture(&fixture);
    assert_eq!(
        validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("existing map projection collapses an identical duplicate")
            .validated_case_count,
        2
    );
    fixture.expected = original;
    fixture.expected["cases"][0]["case_id"] = "foreign-filter-case".into();
    write_fixture(&fixture);
    let error = validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("a foreign identity cannot be compared as a declared case");
    assert!(error.to_string().contains("case ids do not match manifest"));
}

#[test]
fn altered_threshold_and_native_site_truth_reject_with_unchanged_aggregate_counts() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    fixture.expected["cases"][0]["filter_metrics"]["quality_threshold"] = 35.0.into();
    write_fixture(&fixture);
    let before_manifest = fs::read(&fixture.path).expect("capture isolated declaration");
    let before_expected = fs::read(fixture.scratch.path().join("expected.json"))
        .expect("capture isolated expectation");
    let error = validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("threshold drift rejects canonical parsed metrics");
    assert!(error.to_string().contains("truth drifted for case `bcftools_filter_labels`"));
    assert_eq!(fs::read(&fixture.path).expect("read declaration"), before_manifest);
    assert_eq!(
        fs::read(fixture.scratch.path().join("expected.json")).expect("read expectation"),
        before_expected
    );
    fixture.expected = original;
    fixture.expected["cases"][0]["output_truth"]["per_filter_sites"]["LOWQUAL"][0] =
        "chr1:999".into();
    write_fixture(&fixture);
    let error = validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("site identity drift rejects even when variant counts remain equal");
    assert!(error.to_string().contains("truth drifted for case `bcftools_filter_labels`"));
}

#[test]
fn optional_native_output_and_backslash_display_keep_declared_evidence_semantics() {
    let mut fixture = fixture();
    fixture.manifest["cases"][0].as_table_mut().expect("case table").remove("output_vcf_path");
    fixture.expected["cases"][0].as_object_mut().expect("expected case").remove("output_truth");
    write_fixture(&fixture);
    assert!(
        validate_vcf_filter_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("metrics-only declared filter truth stays supported")
            .valid
    );
    #[cfg(unix)]
    {
        let decorated = fixture.scratch.path().join("manifest\\filter.toml");
        fs::write(&decorated, toml::to_string(&fixture.manifest).expect("encode declaration"))
            .expect("write Unix filename containing backslash");
        let report = validate_vcf_filter_truth_manifest_path(&fixture.root, &decorated)
            .expect("declare an existing path with compatibility report normalization");
        assert_eq!(report.manifest_path, decorated.to_string_lossy().replace('\\', "/"));
    }
}
