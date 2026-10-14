//! Segment fixture admission retains complete native truth and declared source bytes.
use bijux_dna_bench_corpus::vcf::validate_segments_demography_truth_manifest_path;

const MANIFEST: &str = "benchmarks/tests/fixtures/science/segments-demography-truth/manifest.toml";
const EXPECTED: &str = "benchmarks/tests/fixtures/science/segments-demography-truth/expected.json";

struct Fixture {
    root: std::path::PathBuf,
    scratch: tempfile::TempDir,
    path: std::path::PathBuf,
    manifest: toml::Value,
    expected: serde_json::Value,
}

fn fixture() -> Fixture {
    let root = crate::support::repo_root();
    let scratch = tempfile::tempdir().expect("isolated segment declarations");
    let path = scratch.path().join("manifest.toml");
    let manifest =
        toml::from_str(&std::fs::read_to_string(root.join(MANIFEST)).expect("read declaration"))
            .expect("decode segment declaration");
    let expected =
        serde_json::from_slice(&std::fs::read(root.join(EXPECTED)).expect("read expectation"))
            .expect("decode segment expectation");
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
fn governed_segment_cases_retain_complete_and_insufficient_truth_without_writes() {
    let fixture = fixture();
    let evidence = evidence_bytes(&fixture.manifest, &fixture.root);
    assert!(
        fixture.manifest["source_paths"]
            .as_array()
            .expect("sources")
            .iter()
            .any(|path| fixture.root.join(path.as_str().expect("source path")).is_dir()),
        "directory source references remain intentionally admitted"
    );
    let manifest = std::fs::read(fixture.root.join(MANIFEST)).expect("capture declaration");
    let expected = std::fs::read(fixture.root.join(EXPECTED)).expect("capture expectation");
    let report = validate_segments_demography_truth_manifest_path(
        &fixture.root,
        &fixture.root.join(MANIFEST),
    )
    .expect("admit complete governed interval and inference truth");
    assert_eq!(report.schema_version, "bijux.bench.segments_demography_truth.validation.v1");
    assert_eq!(report.fixture_id, "segments-demography-truth");
    assert_eq!(report.manifest_path, MANIFEST);
    assert_eq!(report.expected_path, EXPECTED);
    assert_eq!(report.validated_case_count, 9);
    assert_eq!(report.validated_stage_ids, ["vcf.demography", "vcf.ibd", "vcf.roh"]);
    assert_eq!(report.validated_tool_ids, ["germline", "ibdhap", "ibdne", "ibdseq", "plink2"]);
    assert!(report.valid);
    assert_eq!(std::fs::read(fixture.root.join(MANIFEST)).expect("read declaration"), manifest);
    assert_eq!(std::fs::read(fixture.root.join(EXPECTED)).expect("read expectation"), expected);
    for (path, original) in evidence {
        assert_eq!(
            std::fs::read(&path).expect("read native evidence"),
            original,
            "{}",
            path.display()
        );
    }
}

#[test]
fn reference_and_support_admission_precede_expected_bundle_acquisition() {
    let mut fixture = fixture();
    let original = fixture.manifest.clone();
    fixture.manifest["source_paths"] = toml::Value::Array(vec!["missing-segment-source".into()]);
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("source existence precedes expected acquisition");
    assert!(error.to_string().contains("source path is missing"));
    fixture.manifest = original.clone();
    fixture.manifest["cases"][0]["artifact_root_path"] = "missing-segment-artifacts".into();
    fixture.manifest["cases"][0]["stage_id"] = "vcf.foreign".into();
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("artifact existence precedes support and expected reads");
    assert!(error.to_string().contains("artifact_root_path is missing"));
    fixture.manifest = original.clone();
    fixture.manifest["cases"][0]["stage_id"] = "vcf.foreign".into();
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("unsupported stage precedes expected acquisition");
    assert!(error.to_string().contains("found `vcf.foreign`"));
    fixture.manifest = original;
    fixture.manifest["cases"][1]["tool_id"] = "plink2".into();
    fixture.manifest["expected_path"] = "missing-expected.json".into();
    write_fixture(&fixture);
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("plink2 IBD is outside the retained support contract");
    assert!(error.to_string().contains("does not support tool `plink2` on stage `vcf.ibd`"));
}

#[test]
fn duplicate_declarations_reject_after_per_case_support_without_altering_inputs() {
    let mut fixture = fixture();
    let mut duplicate = fixture.manifest["cases"][0].clone();
    duplicate["stage_id"] = "vcf.foreign".into();
    fixture.manifest["cases"].as_array_mut().expect("cases").push(duplicate);
    write_fixture(&fixture);
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("invalid duplicate first receives per-case admission");
    assert!(error.to_string().contains("found `vcf.foreign`"));
    fixture.manifest["cases"][9]["stage_id"] = "vcf.roh".into();
    write_fixture(&fixture);
    let before = std::fs::read(&fixture.path).expect("capture declaration");
    let expected =
        std::fs::read(fixture.scratch.path().join("expected.json")).expect("capture expectation");
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("admitted duplicate identity rejects");
    assert!(error.to_string().contains("repeats case_id `plink2_roh_output`"));
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
        let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
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
        let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err("expected bundle and case authority remain closed");
        assert!(format!("{error:#}").contains("unknown field `foreign_authority`"));
    }
    fixture.expected = original_expected;
    fixture.expected["cases"][0]["roh_metrics"]["additional_native_observation"] =
        serde_json::json!({"nested": [1, null, {"flag": true}]});
    write_fixture(&fixture);
    let before = std::fs::read(fixture.scratch.path().join("expected.json"))
        .expect("capture dynamic metrics");
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("arbitrary nested metrics are decoded and completely compared");
    assert!(error.to_string().contains("truth drifted for case `plink2_roh_output`"));
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
        validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("order and identical duplicates retain set/map semantics")
            .validated_case_count,
        9
    );
    fixture.expected["cases"][0]["demography_metrics"]["status"] = "complete".into();
    write_fixture(&fixture);
    assert!(
        validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("final correct duplicate supersedes earlier altered row")
            .valid
    );
    fixture.expected["cases"][9]["demography_metrics"]["status"] = "complete".into();
    write_fixture(&fixture);
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("changed winning duplicate alters complete truth");
    assert!(error
        .to_string()
        .contains("truth drifted for case `ibdne_demography_insufficient_data`"));
    fixture.expected = original;
    fixture.expected["cases"][0]["case_id"] = "foreign-segment-case".into();
    fixture.manifest["cases"][0]["artifact_root_path"] =
        fixture.scratch.path().to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("expected identity admission precedes missing raw observations");
    assert!(error.to_string().contains("case ids do not match manifest"));
}

#[test]
fn interval_relatedness_and_inference_drift_reject_with_unchanged_aggregate_sizes() {
    let mut fixture = fixture();
    let original = fixture.expected.clone();
    for (index, family, section, row, field, value, count, case_id) in [
        (
            0,
            "roh_output_truth",
            "segments",
            true,
            "sample_id",
            serde_json::json!("foreign_sample"),
            "segment_count",
            "plink2_roh_output",
        ),
        (
            1,
            "ibd_output_truth",
            "rows",
            true,
            "sample_b",
            serde_json::json!("sample_c"),
            "pair_count",
            "germline_ibd_complete",
        ),
        (
            2,
            "ibd_output_truth",
            "insufficient_overlap_probe",
            false,
            "insufficient_reason",
            serde_json::json!("foreign_overlap_reason"),
            "pair_count",
            "germline_ibd_insufficient_overlap",
        ),
        (
            7,
            "demography_output_truth",
            "ne_estimates",
            true,
            "ne",
            serde_json::json!(12500.0),
            "estimate_count",
            "ibdne_demography_complete",
        ),
        (
            8,
            "demography_output_truth",
            "insufficient_data_probe",
            false,
            "insufficient_reason",
            serde_json::json!("foreign_data_reason"),
            "estimate_count",
            "ibdne_demography_insufficient_data",
        ),
    ] {
        fixture.expected = original.clone();
        if row {
            fixture.expected["cases"][index][family][section][0][field] = value;
        } else {
            fixture.expected["cases"][index][family][section][field] = value;
        }
        assert_eq!(
            fixture.expected["cases"][index][family][count],
            original["cases"][index][family][count]
        );
        if row {
            assert_eq!(
                fixture.expected["cases"][index][family][section]
                    .as_array()
                    .expect("changed rows")
                    .len(),
                original["cases"][index][family][section].as_array().expect("original rows").len()
            );
        }
        write_fixture(&fixture);
        let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
            .expect_err(
                "complete intervals and inference cannot be replaced with aggregate counts",
            );
        assert!(error.to_string().contains(&format!("truth drifted for case `{case_id}`")));
    }
}

#[test]
fn command_decode_precedes_later_native_acquisition_without_mutating_observations() {
    let mut fixture = fixture();
    let raw = fixture
        .root
        .join(fixture.manifest["cases"][0]["artifact_root_path"].as_str().expect("artifact path"));
    let isolated = fixture.scratch.path().join("native-roh");
    std::fs::create_dir(&isolated).expect("create isolated evidence");
    for entry in std::fs::read_dir(raw).expect("read native evidence") {
        let entry = entry.expect("native evidence entry");
        std::fs::copy(entry.path(), isolated.join(entry.file_name()))
            .expect("copy native evidence");
    }
    let command = isolated.join("raw.command.json");
    let original = std::fs::read(&command).expect("read command metadata");
    std::fs::write(&command, b"invalid JSON").expect("write malformed command metadata");
    let log = isolated.join("raw.log");
    std::fs::write(&log, b"").expect("write empty existing native log");
    fixture.manifest["cases"][0]["artifact_root_path"] = isolated.to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("raw command decode precedes later native log admission");
    assert!(format!("{error:#}").contains("raw.command.json"));
    assert!(!format!("{error:#}").contains("ROH log must not be empty"));
    assert_eq!(std::fs::read(&command).expect("read failed observation"), b"invalid JSON");
    assert_eq!(std::fs::read(&log).expect("read failed log"), b"");
    std::fs::write(&command, original).expect("repair only isolated command metadata");
    let error = validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
        .expect_err("decoded command exposes the next canonical native error");
    assert_eq!(error.to_string(), "ROH log must not be empty");
    assert_eq!(std::fs::read(&log).expect("read failed log"), b"");
}

#[test]
fn absolute_references_and_backslash_display_paths_retain_local_resolution() {
    let mut fixture = fixture();
    for source in fixture.manifest["source_paths"].as_array_mut().expect("sources") {
        *source = fixture
            .root
            .join(source.as_str().expect("source path"))
            .to_string_lossy()
            .as_ref()
            .into();
    }
    for case in fixture.manifest["cases"].as_array_mut().expect("cases") {
        case["artifact_root_path"] = fixture
            .root
            .join(case["artifact_root_path"].as_str().expect("artifact path"))
            .to_string_lossy()
            .as_ref()
            .into();
    }
    fixture.manifest["expected_path"] =
        fixture.scratch.path().join("expected.json").to_string_lossy().as_ref().into();
    write_fixture(&fixture);
    assert!(
        validate_segments_demography_truth_manifest_path(&fixture.root, &fixture.path)
            .expect("absolute native references retain their declared behavior")
            .valid
    );
    #[cfg(unix)]
    {
        let decorated = fixture.scratch.path().join("manifest\\segments.toml");
        std::fs::write(&decorated, toml::to_string(&fixture.manifest).expect("encode declaration"))
            .expect("write literal backslash filename");
        let report = validate_segments_demography_truth_manifest_path(&fixture.root, &decorated)
            .expect("report normalization does not change the observed path");
        assert_eq!(report.manifest_path, decorated.to_string_lossy().replace('\\', "/"));
    }
}
