//! Corpus admission compares complete phasing/imputation records without rewriting evidence.
use std::fs;
use std::path::PathBuf;

use bijux_dna_bench_corpus::vcf::validate_phasing_imputation_truth_manifest_path;

const MANIFEST: &str = "benchmarks/tests/fixtures/science/phasing-imputation-truth/manifest.toml";
const EXPECTED: &str = "benchmarks/tests/fixtures/science/phasing-imputation-truth/expected.json";

struct Fixture {
    root: PathBuf,
    scratch: tempfile::TempDir,
    path: PathBuf,
    manifest: toml::Value,
    expected: serde_json::Value,
}

fn fixture() -> Fixture {
    let root = crate::support::repo_root();
    let scratch = tempfile::tempdir().expect("phasing truth scratch");
    let path = scratch.path().join("manifest.toml");
    let manifest =
        toml::from_str(&fs::read_to_string(root.join(MANIFEST)).expect("read declaration"))
            .expect("decode phasing/imputation declaration");
    let expected = serde_json::from_slice(&fs::read(root.join(EXPECTED)).expect("read expected"))
        .expect("decode phasing/imputation expectation");
    Fixture { root, scratch, path, manifest, expected }
}

fn write_fixture(fixture: &Fixture) {
    fs::write(&fixture.path, toml::to_string(&fixture.manifest).expect("encode declaration"))
        .expect("write isolated declaration");
    fs::write(
        fixture.scratch.path().join("expected.json"),
        serde_json::to_vec(&fixture.expected).expect("encode expectation"),
    )
    .expect("write isolated expectation");
}

#[test]
fn governed_cases_preserve_complete_report_inventory_and_declared_source_bytes() {
    let fixture = fixture();
    let paths = fixture.manifest["source_paths"]
        .as_array()
        .expect("declared sources")
        .iter()
        .map(|value| fixture.root.join(value.as_str().expect("source path")))
        .chain([fixture.root.join(MANIFEST), fixture.root.join(EXPECTED)])
        .collect::<Vec<_>>();
    let before =
        paths.iter().map(|path| fs::read(path).expect("capture evidence")).collect::<Vec<_>>();
    let report = validate_phasing_imputation_truth_manifest_path(
        &fixture.root,
        &fixture.root.join(MANIFEST),
    )
    .expect("admit all three metrics families and their canonical output truth");
    assert_eq!(report.schema_version, "bijux.bench.phasing_imputation_truth.validation.v1");
    assert_eq!(report.fixture_id, "phasing-imputation-truth");
    assert_eq!(report.manifest_path, MANIFEST);
    assert_eq!(report.expected_path, EXPECTED);
    assert_eq!(report.validated_case_count, 11);
    assert_eq!(report.validated_stage_ids, ["vcf.imputation_metrics", "vcf.impute", "vcf.phasing"]);
    assert_eq!(
        report.validated_tool_ids,
        ["beagle", "eagle", "glimpse", "impute5", "minimac4", "shapeit5"]
    );
    assert!(report.valid);
    for (path, bytes) in paths.iter().zip(before) {
        assert_eq!(fs::read(path).expect("re-read evidence"), bytes, "{}", path.display());
    }
}

#[test]
fn source_artifact_and_output_failures_precede_support_and_expected_acquisition() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    fixture.manifest["source_paths"] = toml::Value::Array(vec!["missing-phasing-source".into()]);
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("source reference precedes expected bundle");
    assert!(error.to_string().contains("source path is missing"));
    for (field, path, rejection) in [
        ("artifact_root_path", "missing-artifact-root", "artifact root is missing"),
        ("output_vcf_path", "missing-output.vcf", "output VCF is missing"),
    ] {
        fixture.manifest = original.clone();
        fixture.manifest["cases"][0][field] = path.into();
        fixture.manifest["cases"][0]["stage_id"] = "unsupported-stage".into();
        fixture.manifest["expected_path"] = "missing-expected.json".into();
        write_fixture(&fixture);
        let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("local evidence reference precedes support and bundle acquisition");
        assert!(error.to_string().contains(rejection), "{field}: {error:#}");
    }
}

#[test]
fn supported_combinations_and_truth_path_requirements_reject_before_bundle_reads() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    for (field, value, rejection) in [
        ("tool_id", "bcftools", "unsupported tool/stage/truth combination"),
        ("truth_kind", "imputation_output", "unsupported tool/stage/truth combination"),
        ("stage_id", "vcf.filter", "unsupported phasing/imputation truth stage"),
        ("case_id", "", "empty identity fields"),
    ] {
        fixture.manifest = original.clone();
        fixture.manifest["cases"][0][field] = value.into();
        fixture.manifest["expected_path"] = "missing-expected.json".into();
        write_fixture(&fixture);
        let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("support admission precedes missing bundle");
        assert!(error.to_string().contains(rejection), "{field}: {error:#}");
    }
    fixture.manifest = original.clone();
    fixture.manifest["cases"][0]
        .as_table_mut()
        .expect("phasing case")
        .insert("truth_vcf_path".into(), "missing-truth.vcf".into());
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("phasing must reject a declared truth path before acquiring it");
    assert!(error.to_string().contains("must not declare truth_vcf_path for phasing output"));
    fixture.manifest = original.clone();
    fixture.manifest["cases"][3].as_table_mut().expect("imputation case").remove("truth_vcf_path");
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("imputation requires truth declaration before acquiring expected bundle");
    assert!(error.to_string().contains("must declare truth_vcf_path for imputation output"));
    fixture.manifest = original;
    fixture.manifest["cases"][3]["truth_vcf_path"] = "missing-truth.vcf".into();
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("missing truth evidence precedes expected acquisition");
    assert!(error.to_string().contains("truth VCF is missing"));
}

#[test]
fn duplicate_manifest_and_closed_wire_fields_reject_without_rewriting_inputs() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    let duplicate = fixture.manifest["cases"][0].clone();
    fixture.manifest["cases"].as_array_mut().expect("cases").push(duplicate);
    write_fixture(&fixture);
    let declaration = fs::read(&fixture.path).expect("capture declaration");
    let expected = fs::read(fixture.scratch.path().join("expected.json")).expect("capture bundle");
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("declared identity must be unique");
    assert!(error.to_string().contains("repeats case_id"));
    assert_eq!(fs::read(&fixture.path).expect("read declaration"), declaration);
    assert_eq!(
        fs::read(fixture.scratch.path().join("expected.json")).expect("read bundle"),
        expected
    );
    fixture.manifest = original.clone();
    fixture
        .manifest
        .as_table_mut()
        .expect("manifest table")
        .insert("unrecognized_authority".into(), true.into());
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("manifest wire is closed");
    assert!(format!("{error:#}").contains("unknown field `unrecognized_authority`"));
    fixture.manifest = original;
    fixture.expected["unrecognized_authority"] = true.into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("expected wire is closed independently of manifest");
    assert!(format!("{error:#}").contains("unknown field `unrecognized_authority`"));
}

#[test]
fn expected_order_and_last_duplicate_semantics_keep_full_comparison_and_identity_admission() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    fixture.expected["cases"].as_array_mut().expect("expected cases").reverse();
    let duplicate = fixture.expected["cases"][0].clone();
    fixture.expected["cases"].as_array_mut().expect("expected cases").push(duplicate);
    write_fixture(&fixture);
    assert_eq!(
        validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("expected order and identical duplicate collapse stay admitted")
            .validated_case_count,
        11
    );
    fixture.expected["cases"][0]["imputation_output_truth"]["genotype_concordance"] = 0.0.into();
    write_fixture(&fixture);
    assert!(
        validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("a final correct duplicate supersedes earlier altered truth")
            .valid
    );
    fixture.expected["cases"][11]["imputation_output_truth"]["genotype_concordance"] = 0.0.into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("a changed winning duplicate rejects full truth");
    assert!(error
        .to_string()
        .contains("truth drifted for case `minimac4_imputation_metrics_output`"));
    fixture.expected = original;
    fixture.expected["cases"][0]["case_id"] = "foreign-phasing-case".into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("foreign expected identity rejects before native observations");
    assert!(error.to_string().contains("case ids do not match manifest"));
}

#[test]
fn phase_set_and_imputation_concordance_drift_reject_with_unchanged_aggregate_sizes() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    fixture.expected["cases"][0]["phasing_output_truth"]["phase_sets_by_sample"]["cohort_alpha"]
        [0] = "999".into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("per-sample phase identity cannot drift behind unchanged counts");
    assert!(error.to_string().contains("truth drifted for case `shapeit5_phasing_output`"));
    fixture.expected = original;
    fixture.expected["cases"][3]["imputation_output_truth"]["genotype_concordance"] = 0.0.into();
    write_fixture(&fixture);
    let before = fs::read(fixture.scratch.path().join("expected.json")).expect("capture truth");
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("imputed genotype truth cannot drift behind unchanged sample/variant counts");
    assert!(error.to_string().contains("truth drifted for case `beagle_impute_output`"));
    assert_eq!(fs::read(fixture.scratch.path().join("expected.json")).expect("read truth"), before);
}

#[test]
fn native_metrics_acquisition_precedes_output_truth_and_keeps_evidence_bytes() {
    let mut fixture = fixture();
    let artifact_root = fixture.scratch.path().join("phasing-artifacts");
    fs::create_dir(&artifact_root).expect("create isolated native artifacts");
    let source_root = fixture
        .root
        .join(fixture.manifest["cases"][0]["artifact_root_path"].as_str().expect("artifact root"));
    for entry in fs::read_dir(&source_root).expect("read canonical artifact directory") {
        let entry = entry.expect("native artifact entry");
        if entry.path().is_file() {
            fs::copy(entry.path(), artifact_root.join(entry.file_name()))
                .expect("copy native artifact");
        }
    }
    let command = artifact_root.join("raw.command.json");
    let original_command = fs::read(&command).expect("capture canonical command");
    fs::write(&command, b"invalid JSON").expect("write broken parser input");
    let output = fixture.scratch.path().join("invalid-output.vcf");
    fs::write(&output, b"invalid VCF").expect("write broken truth input");
    fixture.manifest["cases"][0]["artifact_root_path"] =
        artifact_root.to_string_lossy().as_ref().into();
    fixture.manifest["cases"][0]["output_vcf_path"] = output.to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("backend metrics parser is observed before output truth");
    assert!(format!("{error:#}").contains("raw.command.json"));
    assert_eq!(fs::read(&command).expect("read broken command"), b"invalid JSON");
    fs::write(&command, original_command).expect("repair canonical command");
    let error = validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("repaired metrics permit output truth parsing, which still refuses broken VCF");
    assert!(!format!("{error:#}").contains("raw.command.json"));
    assert_eq!(error.to_string(), "malformed VCF record at line 1: expected at least 8 fields");
    assert_eq!(fs::read(&output).expect("read broken VCF"), b"invalid VCF");
}

#[test]
fn absolute_source_and_evidence_paths_retain_local_resolution_and_report_normalization() {
    let mut fixture = fixture();
    for path in fixture.manifest["source_paths"].as_array_mut().expect("sources") {
        *path = fixture
            .root
            .join(path.as_str().expect("source path"))
            .to_string_lossy()
            .as_ref()
            .into();
    }
    for field in ["artifact_root_path", "output_vcf_path"] {
        let path =
            fixture.root.join(fixture.manifest["cases"][0][field].as_str().expect("evidence path"));
        fixture.manifest["cases"][0][field] = path.to_string_lossy().as_ref().into();
    }
    let truth = fixture
        .root
        .join(fixture.manifest["cases"][3]["truth_vcf_path"].as_str().expect("truth path"));
    fixture.manifest["cases"][3]["truth_vcf_path"] = truth.to_string_lossy().as_ref().into();
    fixture.manifest["expected_path"] =
        fixture.scratch.path().join("expected.json").to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    assert!(
        validate_phasing_imputation_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("absolute declarations preserve the original admitted local evidence")
            .valid
    );
    #[cfg(unix)]
    {
        let decorated = fixture.scratch.path().join("manifest\\phasing.toml");
        fs::write(&decorated, toml::to_string(&fixture.manifest).expect("encode declaration"))
            .expect("write filename with literal backslash");
        let report = validate_phasing_imputation_truth_manifest_path(&fixture.root, &decorated)
            .expect("path normalization remains a display operation");
        assert_eq!(report.manifest_path, decorated.to_string_lossy().replace('\\', "/"));
    }
}
