//! Genotype fixture admission compares complete evidence without altering declared files.
use std::fs;
use std::path::PathBuf;

use bijux_dna_bench_corpus::vcf::validate_vcf_genotype_truth_manifest_path;

const MANIFEST: &str = "benchmarks/tests/fixtures/science/vcf-genotype-truth/manifest.toml";
const EXPECTED: &str = "benchmarks/tests/fixtures/science/vcf-genotype-truth/expected.json";

struct Fixture {
    root: PathBuf,
    scratch: tempfile::TempDir,
    path: PathBuf,
    manifest: toml::Value,
    expected: serde_json::Value,
}

fn fixture() -> Fixture {
    let root = crate::support::repo_root();
    let scratch = tempfile::tempdir().expect("genotype truth scratch");
    let path = scratch.path().join("manifest.toml");
    let manifest =
        toml::from_str(&fs::read_to_string(root.join(MANIFEST)).expect("read declaration"))
            .expect("decode genotype truth declaration");
    let expected =
        serde_json::from_slice(&fs::read(root.join(EXPECTED)).expect("read expectation"))
            .expect("decode genotype truth expectation");
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
fn governed_truth_retains_declared_bytes_and_sorted_report_inventories() {
    let fixture = fixture();
    let paths = fixture.manifest["source_paths"]
        .as_array()
        .expect("declared sources")
        .iter()
        .map(|path| fixture.root.join(path.as_str().expect("source path")))
        .chain([fixture.root.join(MANIFEST), fixture.root.join(EXPECTED)])
        .collect::<Vec<_>>();
    let bytes =
        paths.iter().map(|path| fs::read(path).expect("capture evidence")).collect::<Vec<_>>();
    let report =
        validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.root.join(MANIFEST))
            .expect("canonical genotype and calling-boundary evidence is admitted");
    assert_eq!(report.schema_version, "bijux.bench.vcf_genotype_truth.validation.v1");
    assert_eq!(report.fixture_id, "vcf-genotype-truth");
    assert_eq!(report.manifest_path, MANIFEST);
    assert_eq!(report.expected_path, EXPECTED);
    assert_eq!(report.validated_case_count, 6);
    assert_eq!(
        report.validated_stage_ids,
        ["vcf.call_diploid", "vcf.call_gl", "vcf.call_pseudohaploid", "vcf.gl_propagation"]
    );
    assert_eq!(report.validated_tool_ids, ["angsd", "bcftools"]);
    assert!(report.valid);
    for (path, original) in paths.iter().zip(bytes) {
        assert_eq!(fs::read(path).expect("re-read evidence"), original, "{}", path.display());
    }
}

#[test]
fn declaration_reference_and_ploidy_errors_precede_expected_bundle_acquisition() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    fixture.manifest["source_paths"] = toml::Value::Array(vec!["missing-genotype-source".into()]);
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("source admission precedes bundle acquisition");
    assert!(error.to_string().contains("source path is missing"));
    fixture.manifest = original.clone();
    fixture.manifest["cases"][0]
        .as_table_mut()
        .expect("case table")
        .insert("command_path".into(), "missing-genotype-command".into());
    fixture.manifest["cases"][0]["declared_ploidy"] = "haploid".into();
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("command existence precedes ploidy and bundle admission");
    assert!(error.to_string().contains("command file is missing"));
    for (field, value, rejection) in [
        ("declared_ploidy", "haploid", "must declare `diploid` ploidy"),
        ("case_id", "", "contains an empty case_id"),
        ("tool_id", "", "non-empty stage_id and tool_id"),
    ] {
        fixture.manifest = original.clone();
        fixture.manifest["cases"][0][field] = value.into();
        fixture.manifest["expected_path"] = "missing-expected.json".into();
        write_fixture(&fixture);
        let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("case admission precedes bundle acquisition");
        assert!(error.to_string().contains(rejection), "{field}: {error:#}");
    }
    fixture.manifest = original;
    fixture.manifest["cases"][0].as_table_mut().expect("case table").remove("mean_coverage");
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("declared coverage is required before observation");
    assert!(error.to_string().contains("must declare mean coverage and minimum_mean_coverage"));
}

#[test]
fn duplicate_declarations_and_unknown_manifest_fields_reject_without_writes() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    let duplicate = fixture.manifest["cases"][0].clone();
    fixture.manifest["cases"].as_array_mut().expect("cases").push(duplicate);
    write_fixture(&fixture);
    let declaration = fs::read(&fixture.path).expect("capture declaration");
    let expectation =
        fs::read(fixture.scratch.path().join("expected.json")).expect("capture bundle");
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("declared duplicate identity rejects");
    assert!(error.to_string().contains("repeats case_id"));
    assert_eq!(fs::read(&fixture.path).expect("read declaration"), declaration);
    assert_eq!(
        fs::read(fixture.scratch.path().join("expected.json")).expect("read bundle"),
        expectation
    );
    fixture.manifest = original;
    fixture
        .manifest
        .as_table_mut()
        .expect("manifest table")
        .insert("unrecognized_manifest_authority".into(), true.into());
    write_fixture(&fixture);
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("closed declaration wire rejects unknown authority");
    assert!(format!("{error:#}").contains("unknown field `unrecognized_manifest_authority`"));
}

#[test]
fn expected_order_and_duplicates_retain_last_row_comparison_and_case_set_admission() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    fixture.expected["cases"].as_array_mut().expect("expected cases").reverse();
    let duplicate = fixture.expected["cases"][0].clone();
    fixture.expected["cases"].as_array_mut().expect("expected cases").push(duplicate);
    write_fixture(&fixture);
    assert_eq!(
        validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("reordered rows and identical duplicates retain set/map semantics")
            .validated_case_count,
        6
    );
    fixture.expected["cases"][0]["summary"]["called_calls"] = 999.into();
    write_fixture(&fixture);
    assert!(
        validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("a preceding duplicate remains superseded by the final correct row")
            .valid
    );
    fixture.expected["cases"][6]["summary"]["called_calls"] = 999.into();
    write_fixture(&fixture);
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("a changed winning duplicate rejects complete truth");
    assert!(error.to_string().contains("truth drifted for case `bcftools_gl_gp_pl_propagation`"));
    fixture.expected = original;
    fixture.expected["cases"][0]["case_id"] = "foreign-genotype-case".into();
    fixture.manifest["cases"][1]["command_path"] =
        fixture.scratch.path().join("bad-command.json").to_string_lossy().as_ref().into();
    fs::write(fixture.scratch.path().join("bad-command.json"), b"invalid JSON")
        .expect("write broken observation");
    write_fixture(&fixture);
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("bundle identity admission precedes command observation");
    assert!(error.to_string().contains("case ids do not match manifest"));
}

#[test]
fn genotype_missingness_and_boundary_drift_reject_with_unchanged_sample_variant_counts() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    for (case_index, field, value, case_id) in [
        (2, "missing_calls", serde_json::json!(0), "angsd_pseudohaploid_missingness"),
        (0, "mixed_allele_calls", serde_json::json!(0), "bcftools_diploid_calls"),
    ] {
        fixture.expected = original.clone();
        fixture.expected["cases"][case_index]["summary"][field] = value;
        write_fixture(&fixture);
        let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("equal aggregate sizes cannot hide genotype classification drift");
        assert!(error.to_string().contains(&format!("truth drifted for case `{case_id}`")));
    }
    fixture.expected = original;
    fixture.expected["cases"][3]["likelihood_boundary"]["prerequisites_passed"] = true.into();
    write_fixture(&fixture);
    let before = fs::read(fixture.scratch.path().join("expected.json")).expect("capture bundle");
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("likelihood refusal drift rejects independently of genotype counts");
    assert!(error.to_string().contains("truth drifted for case `bcftools_pl_likelihood_calls`"));
    assert_eq!(
        fs::read(fixture.scratch.path().join("expected.json")).expect("read bundle"),
        before
    );
}

#[test]
fn command_seed_metadata_is_observed_without_execution_and_unknown_fields_remain_accepted() {
    let mut fixture = fixture();
    let command_path = fixture.scratch.path().join("command.json");
    let declared = fixture.manifest["cases"][2]["command_path"].as_str().expect("command path");
    let mut command: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.root.join(declared)).expect("read command"))
            .expect("decode command");
    command["unknown_observational_metadata"] = true.into();
    command["argv"] =
        serde_json::json!(["not-an-executable", "-seed", "invalid", "--seed", "42", "-seed", "99"]);
    fixture.manifest["cases"][2]["command_path"] = command_path.to_string_lossy().as_ref().into();
    fixture.expected["cases"][2]["command"]["command_path"] = command_path
        .strip_prefix(&fixture.root)
        .unwrap_or(&command_path)
        .to_string_lossy()
        .replace('\\', "/")
        .into();
    fs::write(&command_path, serde_json::to_vec(&command).expect("encode command"))
        .expect("write command");
    write_fixture(&fixture);
    assert!(
        validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("metadata argv is not executed; the first parseable seed retains precedence")
            .valid
    );
    command["random_seed"] = 99.into();
    fs::write(&command_path, serde_json::to_vec(&command).expect("encode altered command"))
        .expect("write altered command");
    let before = fs::read(&command_path).expect("capture altered metadata");
    let error = validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("seed disagreement changes complete command truth");
    assert!(error.to_string().contains("truth drifted for case `angsd_pseudohaploid_missingness`"));
    assert_eq!(fs::read(&command_path).expect("read metadata"), before);
}

#[test]
fn absolute_declarations_and_backslash_report_paths_retain_supported_resolution() {
    let mut fixture = fixture();
    for source in fixture.manifest["source_paths"].as_array_mut().expect("declared sources") {
        *source = fixture
            .root
            .join(source.as_str().expect("source path"))
            .to_string_lossy()
            .as_ref()
            .into();
    }
    let input = fixture
        .root
        .join(fixture.manifest["cases"][0]["input_vcf_path"].as_str().expect("input path"));
    fixture.manifest["cases"][0]["input_vcf_path"] = input.to_string_lossy().as_ref().into();
    fixture.manifest["expected_path"] =
        fixture.scratch.path().join("expected.json").to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    assert!(
        validate_vcf_genotype_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("absolute paths preserve admitted local evidence")
            .valid
    );
    #[cfg(unix)]
    {
        let decorated = fixture.scratch.path().join("manifest\\genotype.toml");
        fs::write(&decorated, toml::to_string(&fixture.manifest).expect("encode declaration"))
            .expect("write filename containing a literal backslash");
        let report = validate_vcf_genotype_truth_manifest_path(&fixture.root, &decorated)
            .expect("backslash path normalization remains a report-only operation");
        assert_eq!(
            report.manifest_path,
            decorated
                .strip_prefix(&fixture.root)
                .unwrap_or(&decorated)
                .to_string_lossy()
                .replace('\\', "/")
        );
    }
}
