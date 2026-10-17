//! Population fixture admission retains its complete native observations and source bytes.
use bijux_dna_bench_corpus::vcf::validate_population_structure_truth_manifest_path;

#[test]
fn governed_population_cases_retain_directory_sources_and_typed_inventory() {
    let root = crate::support::repo_root();
    let manifest_path =
        root.join("benchmarks/tests/fixtures/science/population-structure-truth/manifest.toml");
    let expected_path =
        root.join("benchmarks/tests/fixtures/science/population-structure-truth/expected.json");
    let manifest_bytes = std::fs::read(&manifest_path).expect("read population declaration");
    let expected_bytes = std::fs::read(&expected_path).expect("read population expectation");
    let manifest: toml::Value =
        toml::from_str(std::str::from_utf8(&manifest_bytes).expect("declaration UTF8"))
            .expect("decode population declaration");
    assert!(
        manifest["source_paths"]
            .as_array()
            .expect("source paths")
            .iter()
            .any(|path| root.join(path.as_str().expect("source path")).is_dir()),
        "declared directories are intentionally admissible source references"
    );
    let evidence = evidence_bytes(&manifest, &root);
    let report = validate_population_structure_truth_manifest_path(&root, &manifest_path)
        .expect("admit complete population fixture truth");
    assert_eq!(report.schema_version, "bijux.bench.population_structure_truth.validation.v1");
    assert_eq!(report.fixture_id, "population-structure-truth");
    assert_eq!(
        report.manifest_path,
        "benchmarks/tests/fixtures/science/population-structure-truth/manifest.toml"
    );
    assert_eq!(
        report.expected_path,
        "benchmarks/tests/fixtures/science/population-structure-truth/expected.json"
    );
    assert_eq!(report.validated_case_count, 5);
    assert_eq!(
        report.validated_stage_ids,
        ["vcf.admixture", "vcf.pca", "vcf.population_structure"]
    );
    assert_eq!(report.validated_tool_ids, ["eigensoft", "plink2"]);
    assert!(report.valid);
    assert_eq!(std::fs::read(&manifest_path).expect("re-read declaration"), manifest_bytes);
    assert_eq!(std::fs::read(&expected_path).expect("re-read expectation"), expected_bytes);
    for (path, original) in evidence {
        assert_eq!(
            std::fs::read(&path).expect("re-read native evidence"),
            original,
            "{}",
            path.display()
        );
    }
}

const MANIFEST: &str = "benchmarks/tests/fixtures/science/population-structure-truth/manifest.toml";
const EXPECTED: &str = "benchmarks/tests/fixtures/science/population-structure-truth/expected.json";

struct Fixture {
    root: std::path::PathBuf,
    scratch: tempfile::TempDir,
    path: std::path::PathBuf,
    manifest: toml::Value,
    expected: serde_json::Value,
}

fn fixture() -> Fixture {
    let root = crate::support::repo_root();
    let scratch = tempfile::tempdir().expect("isolated population declarations");
    let path = scratch.path().join("manifest.toml");
    let manifest =
        toml::from_str(&std::fs::read_to_string(root.join(MANIFEST)).expect("read declaration"))
            .expect("decode population declaration");
    let expected =
        serde_json::from_slice(&std::fs::read(root.join(EXPECTED)).expect("read expectation"))
            .expect("decode population expectation");
    Fixture { root, scratch, path, manifest, expected }
}

fn write_fixture(fixture: &Fixture) {
    std::fs::write(&fixture.path, toml::to_string(&fixture.manifest).expect("encode declaration"))
        .expect("write isolated declaration");
    std::fs::write(
        fixture.scratch.path().join("expected.json"),
        serde_json::to_vec(&fixture.expected).expect("encode expectation"),
    )
    .expect("write isolated expectation");
}

fn evidence_bytes(
    manifest: &toml::Value,
    root: &std::path::Path,
) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut pending = manifest["source_paths"]
        .as_array()
        .expect("source paths")
        .iter()
        .map(|path| root.join(path.as_str().expect("source path")))
        .collect::<Vec<_>>();
    let mut files = std::collections::BTreeSet::new();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(
                std::fs::read_dir(&path)
                    .expect("read evidence directory")
                    .map(|entry| entry.expect("evidence entry").path()),
            );
        } else {
            files.insert(path);
        }
    }
    files
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).expect("read declared evidence");
            (path, bytes)
        })
        .collect()
}

#[test]
fn reference_and_support_admission_precede_expected_bundle_acquisition() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    fixture.manifest["source_paths"] = toml::Value::Array(vec!["missing-population-source".into()]);
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("source existence precedes bundle acquisition");
    assert!(error.to_string().contains("source path is missing"));
    for (field, value, rejection) in [
        ("artifact_root_path", "missing-population-artifacts", "artifact_root_path is missing"),
        ("sample_metadata_path", "missing-population-metadata", "sample_metadata_path is missing"),
    ] {
        fixture.manifest = original.clone();
        fixture.manifest["cases"][0][field] = value.into();
        fixture.manifest["cases"][0]["stage_id"] = "unsupported-stage".into();
        fixture.manifest["expected_path"] = "missing-expected.json".into();
        write_fixture(&fixture);
        let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("local evidence admission precedes support and expected reads");
        assert!(error.to_string().contains(rejection), "{field}: {error:#}");
    }
    fixture.manifest = original.clone();
    fixture.manifest["cases"][0]["stage_id"] = "vcf.foreign".into();
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("unsupported stage precedes bundle acquisition");
    assert!(error.to_string().contains("found `vcf.foreign`"));
    fixture.manifest = original;
    fixture.manifest["cases"][2]["tool_id"] = "eigensoft".into();
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("eigensoft admixture is outside the existing support contract");
    assert!(error
        .to_string()
        .contains("does not support tool `eigensoft` on stage `vcf.admixture`"));
}

#[test]
fn duplicate_declarations_reject_after_per_case_support_without_altering_inputs() {
    let mut fixture = fixture();
    let mut duplicate = fixture.manifest["cases"][0].clone();
    duplicate["stage_id"] = "vcf.foreign".into();
    fixture.manifest["cases"].as_array_mut().expect("cases").push(duplicate);
    write_fixture(&fixture);
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("invalid duplicate must first satisfy per-case admission");
    assert!(error.to_string().contains("found `vcf.foreign`"));
    fixture.manifest["cases"][5]["stage_id"] = "vcf.pca".into();
    write_fixture(&fixture);
    let before = std::fs::read(&fixture.path).expect("capture declaration");
    let expected =
        std::fs::read(fixture.scratch.path().join("expected.json")).expect("capture expectation");
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("admitted duplicate identity rejects");
    assert!(error.to_string().contains("repeats case_id `plink2_pca_output`"));
    assert_eq!(std::fs::read(&fixture.path).expect("read declaration"), before);
    assert_eq!(
        std::fs::read(fixture.scratch.path().join("expected.json")).expect("read expectation"),
        expected
    );
}

#[test]
fn wire_authority_is_closed_while_dynamic_metrics_reach_complete_comparison() {
    let mut fixture = fixture();
    let original_manifest = fixture.manifest.clone();
    let original_expected = fixture.expected.clone();
    for case_field in [false, true] {
        fixture.manifest = original_manifest.clone();
        let record =
            if case_field { &mut fixture.manifest["cases"][0] } else { &mut fixture.manifest };
        record.as_table_mut().expect("wire table").insert("foreign_authority".into(), true.into());
        write_fixture(&fixture);
        let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("manifest and case authority remain closed");
        assert!(format!("{error:#}").contains("unknown field `foreign_authority`"));
    }
    fixture.manifest = original_manifest;
    for case_field in [false, true] {
        fixture.expected = original_expected.clone();
        let record =
            if case_field { &mut fixture.expected["cases"][0] } else { &mut fixture.expected };
        record["foreign_authority"] = true.into();
        write_fixture(&fixture);
        let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("expected bundle and case authority remain closed");
        assert!(format!("{error:#}").contains("unknown field `foreign_authority`"));
    }
    fixture.expected = original_expected;
    fixture.expected["cases"][0]["pca_metrics"]["additional_native_observation"] =
        serde_json::json!({"nested": [1, null, {"flag": true}]});
    write_fixture(&fixture);
    let before = std::fs::read(fixture.scratch.path().join("expected.json"))
        .expect("capture dynamic metrics");
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("arbitrary nested metrics are decoded and compared without narrowing");
    assert!(error.to_string().contains("truth drifted for case `plink2_pca_output`"));
    assert!(!format!("{error:#}").contains("unknown field"));
    assert_eq!(
        std::fs::read(fixture.scratch.path().join("expected.json")).expect("read expectation"),
        before
    );
}

#[test]
fn expected_order_and_last_duplicate_preserve_case_identity_admission() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    fixture.expected["cases"].as_array_mut().expect("expected cases").reverse();
    let duplicate = fixture.expected["cases"][0].clone();
    fixture.expected["cases"].as_array_mut().expect("expected cases").push(duplicate);
    write_fixture(&fixture);
    assert_eq!(
        validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("order and identical duplicates retain set/map semantics")
            .validated_case_count,
        5
    );
    fixture.expected["cases"][0]["population_structure_metrics"]["variant_count"] = 999.into();
    write_fixture(&fixture);
    assert!(
        validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("final correct duplicate supersedes earlier changed row")
            .valid
    );
    fixture.expected["cases"][5]["population_structure_metrics"]["variant_count"] = 999.into();
    write_fixture(&fixture);
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("changed winning duplicate changes complete truth");
    assert!(error
        .to_string()
        .contains("truth drifted for case `eigensoft_population_structure_output`"));
    fixture.expected = original;
    fixture.expected["cases"][0]["case_id"] = "foreign-population-case".into();
    fixture.manifest["cases"][0]["artifact_root_path"] =
        fixture.scratch.path().to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("expected identity admission precedes missing raw observations");
    assert!(error.to_string().contains("case ids do not match manifest"));
}

#[test]
fn geometry_and_ancestry_drift_reject_despite_unchanged_aggregate_sizes() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    for (index, family, section, field, value, case_id) in [
        (
            0,
            "pca_output_truth",
            "pairwise_distances",
            "distance",
            serde_json::json!(0.5),
            "plink2_pca_output",
        ),
        (
            2,
            "admixture_output_truth",
            "rows",
            "dominant_cluster",
            serde_json::json!("cluster_2"),
            "plink2_admixture_output",
        ),
        (
            4,
            "population_structure_output_truth",
            "sample_groups",
            "population_id",
            serde_json::json!("cohort_beta"),
            "eigensoft_population_structure_output",
        ),
    ] {
        fixture.expected = original.clone();
        fixture.expected["cases"][index][family][section][0][field] = value;
        assert_eq!(
            fixture.expected["cases"][index][family]["sample_count"],
            original["cases"][index][family]["sample_count"]
        );
        assert_eq!(
            fixture.expected["cases"][index][family][section]
                .as_array()
                .expect("changed rows")
                .len(),
            original["cases"][index][family][section].as_array().expect("original rows").len()
        );
        write_fixture(&fixture);
        let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("complete scientific records cannot be replaced by aggregate count checks");
        assert!(error.to_string().contains(&format!("truth drifted for case `{case_id}`")));
    }
}

#[test]
fn raw_parser_failure_precedes_metadata_join_and_preserves_observed_bytes() {
    let mut fixture = fixture();
    let raw = fixture
        .root
        .join(fixture.manifest["cases"][0]["artifact_root_path"].as_str().expect("artifact path"));
    let isolated = fixture.scratch.path().join("native-pca");
    std::fs::create_dir(&isolated).expect("create isolated evidence");
    for entry in std::fs::read_dir(raw).expect("read native evidence") {
        let entry = entry.expect("native evidence entry");
        std::fs::copy(entry.path(), isolated.join(entry.file_name()))
            .expect("copy native evidence");
    }
    let native_manifest = isolated.join("raw.pca_manifest.json");
    let original = std::fs::read(&native_manifest).expect("read native manifest");
    std::fs::write(&native_manifest, b"invalid JSON").expect("write broken native observation");
    let metadata = fixture.scratch.path().join("population-metadata.tsv");
    std::fs::write(&metadata, b"foreign_header\n").expect("write malformed existing metadata");
    fixture.manifest["cases"][0]["artifact_root_path"] = isolated.to_string_lossy().as_ref().into();
    fixture.manifest["cases"][0]["sample_metadata_path"] =
        metadata.to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("native parser reads precede metadata summary");
    assert!(format!("{error:#}").contains("raw.pca_manifest.json"));
    assert!(!format!("{error:#}").contains("missing required column"));
    assert_eq!(std::fs::read(&native_manifest).expect("read failed evidence"), b"invalid JSON");
    assert_eq!(std::fs::read(&metadata).expect("read failed metadata"), b"foreign_header\n");
    std::fs::write(&native_manifest, original).expect("repair only isolated native manifest");
    let error = validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("repaired native observation exposes metadata admission");
    assert!(error.to_string().contains("missing required column `sample_id`"));
    assert_eq!(std::fs::read(&metadata).expect("read failed metadata"), b"foreign_header\n");
}

#[test]
fn absolute_references_and_backslash_display_paths_retain_local_resolution() {
    let mut fixture = fixture();
    for source in fixture.manifest["source_paths"].as_array_mut().expect("source paths") {
        *source = fixture
            .root
            .join(source.as_str().expect("source path"))
            .to_string_lossy()
            .as_ref()
            .into();
    }
    for case in fixture.manifest["cases"].as_array_mut().expect("cases") {
        for key in ["artifact_root_path", "sample_metadata_path"] {
            case[key] = fixture
                .root
                .join(case[key].as_str().expect("case path"))
                .to_string_lossy()
                .as_ref()
                .into();
        }
    }
    fixture.manifest["expected_path"] =
        fixture.scratch.path().join("expected.json").to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    assert!(
        validate_population_structure_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("absolute evidence references remain supported")
            .valid
    );
    #[cfg(unix)]
    {
        let decorated = fixture.scratch.path().join("manifest\\population.toml");
        std::fs::write(&decorated, toml::to_string(&fixture.manifest).expect("encode declaration"))
            .expect("write literal backslash filename");
        let report = validate_population_structure_truth_manifest_path(&fixture.root, &decorated)
            .expect("display normalization does not change acquisition path");
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
