use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::parse_phasing_stage_metrics;

const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfPhasingFixtureCase {
    tool_id: &'static str,
    parser_id: &'static str,
    required_files: &'static [&'static str],
}

const VCF_PHASING_FIXTURE_CASES: &[VcfPhasingFixtureCase] = &[
    VcfPhasingFixtureCase {
        tool_id: "shapeit5",
        parser_id: "parse_shapeit5_phasing_metrics",
        required_files: &[
            "raw.command.json",
            "raw.phased.vcf",
            "raw.unphased.vcf",
            "raw.phase_block_stats.tsv",
            "raw.switch_error_proxy.tsv",
            "raw.phasing_qc.json",
            "raw.phasing_manifest.json",
            "expected.normalized.json",
        ],
    },
    VcfPhasingFixtureCase {
        tool_id: "eagle",
        parser_id: "parse_eagle_phasing_metrics",
        required_files: &[
            "raw.command.json",
            "raw.phased.vcf",
            "raw.unphased.vcf",
            "raw.phase_block_stats.tsv",
            "raw.switch_error_proxy.tsv",
            "raw.phasing_qc.json",
            "raw.phasing_manifest.json",
            "expected.normalized.json",
        ],
    },
    VcfPhasingFixtureCase {
        tool_id: "beagle",
        parser_id: "parse_beagle_phasing_metrics",
        required_files: &[
            "raw.command.json",
            "raw.phased.vcf",
            "raw.unphased.vcf",
            "raw.phase_block_stats.tsv",
            "raw.switch_error_proxy.tsv",
            "raw.phasing_qc.json",
            "raw.phasing_manifest.json",
            "expected.normalized.json",
        ],
    },
];

#[test]
fn vcf_phasing_fixture_bank_covers_governed_tool_set() {
    let governed_tool_ids =
        VCF_PHASING_FIXTURE_CASES.iter().map(|case| case.tool_id).collect::<Vec<_>>();
    assert_eq!(governed_tool_ids, vec!["shapeit5", "eagle", "beagle"]);
    for case in VCF_PHASING_FIXTURE_CASES {
        let dir = fixture_dir(case);
        assert!(dir.exists(), "missing fixture directory: {}", dir.display());
        for file_name in case.required_files {
            assert!(
                dir.join(file_name).exists(),
                "missing fixture file `{file_name}` for tool `{}`",
                case.tool_id
            );
        }
    }
}

#[test]
fn vcf_phasing_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in VCF_PHASING_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(&observed, &expected, case.tool_id);
    }
    Ok(())
}

#[test]
fn vcf_phasing_fixture_bank_rejects_all_unphased_output() -> Result<()> {
    for case in VCF_PHASING_FIXTURE_CASES {
        let dir = unique_temp_dir(case.tool_id)?;
        copy_fixture_dir(&fixture_dir(case), &dir)?;
        fs::copy(dir.join("raw.unphased.vcf"), dir.join("raw.phased.vcf"))
            .with_context(|| format!("install unphased probe for `{}`", case.tool_id))?;
        let error = match parse_phasing_stage_metrics(case.tool_id, &dir) {
            Ok(_) => panic!("all-unphased phasing output must fail"),
            Err(error) => error,
        };
        let message = error.to_string();
        assert!(
            message.contains("contains no phased genotypes"),
            "unexpected unphased failure for {}: {message}",
            case.tool_id
        );
        let _ = fs::remove_dir_all(&dir);
    }
    Ok(())
}

fn render_case(case: &VcfPhasingFixtureCase) -> Result<serde_json::Value> {
    let normalized = parse_phasing_stage_metrics(case.tool_id, &fixture_dir(case))
        .with_context(|| format!("parse phasing fixture for `{}`", case.tool_id))?;
    Ok(serde_json::json!({
        "schema_version": VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": "vcf.phasing",
        "tool_id": case.tool_id,
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &VcfPhasingFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join("expected.normalized.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &VcfPhasingFixtureCase) -> PathBuf {
    repo_root().join("benchmarks/tests/fixtures/bench/parsers/vcf/phasing").join(case.tool_id)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|err| panic!("canonicalize repo root: {err}"))
}

fn unique_temp_dir(tool_id: &str) -> Result<PathBuf> {
    let stamp =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = std::env::temp_dir().join(format!("bijux-vcf-phasing-{tool_id}-{stamp}"));
    fs::create_dir_all(&path).with_context(|| format!("create {}", path.display()))?;
    Ok(path)
}

fn copy_fixture_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to).with_context(|| format!("create {}", to.display()))?;
    for entry in fs::read_dir(from).with_context(|| format!("read {}", from.display()))? {
        let entry = entry.with_context(|| format!("read entry under {}", from.display()))?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());
        if source_path.is_dir() {
            copy_fixture_dir(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!("copy {} to {}", source_path.display(), target_path.display())
            })?;
        }
    }
    Ok(())
}

fn assert_json_matches(observed: &serde_json::Value, expected: &serde_json::Value, tool_id: &str) {
    assert!(
        observed == expected,
        "VCF phasing parser fixture mismatch for {tool_id}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}
