use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::{parse_plink_stage_metrics, VcfDomainStage};

const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfPlinkFixtureCase {
    stage: VcfDomainStage,
    parser_id: &'static str,
    required_files: &'static [&'static str],
}

const VCF_PLINK_FIXTURE_CASES: &[VcfPlinkFixtureCase] = &[
    VcfPlinkFixtureCase {
        stage: VcfDomainStage::Qc,
        parser_id: "parse_plink_qc_metrics",
        required_files: &[
            "raw.imiss",
            "raw.lmiss",
            "raw.frq",
            "raw.het",
            "raw.hwe",
            "raw.log",
            "expected.normalized.json",
        ],
    },
    VcfPlinkFixtureCase {
        stage: VcfDomainStage::Admixture,
        parser_id: "parse_plink_admixture_prep_metrics",
        required_files: &[
            "raw.fam",
            "raw.bim",
            "raw.frq",
            "raw.imiss",
            "raw.lmiss",
            "raw.log",
            "expected.normalized.json",
        ],
    },
];

#[test]
fn vcf_plink_fixture_bank_covers_governed_stage_set() {
    let governed_stage_ids =
        VCF_PLINK_FIXTURE_CASES.iter().map(|case| case.stage.as_str()).collect::<Vec<_>>();
    assert_eq!(governed_stage_ids, vec!["vcf.qc", "vcf.admixture"]);
    for case in VCF_PLINK_FIXTURE_CASES {
        let dir = fixture_dir(case);
        assert!(dir.exists(), "missing fixture directory: {}", dir.display());
        for file_name in case.required_files {
            assert!(
                dir.join(file_name).exists(),
                "missing fixture file `{file_name}` for stage `{}`",
                case.stage.as_str()
            );
        }
    }
}

#[test]
fn vcf_plink_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in VCF_PLINK_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(&observed, &expected, case.stage.as_str());
    }
    Ok(())
}

fn render_case(case: &VcfPlinkFixtureCase) -> Result<serde_json::Value> {
    let normalized = parse_plink_stage_metrics(case.stage, &fixture_dir(case))
        .with_context(|| format!("parse fixture stage `{}`", case.stage.as_str()))?;
    Ok(serde_json::json!({
        "schema_version": VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage.as_str(),
        "tool_id": "plink",
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &VcfPlinkFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join("expected.normalized.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &VcfPlinkFixtureCase) -> PathBuf {
    repo_root().join("benchmarks/tests/fixtures/bench/parsers/vcf/plink").join(case.stage.as_str())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn assert_json_matches(observed: &serde_json::Value, expected: &serde_json::Value, stage_id: &str) {
    assert!(
        observed == expected,
        "VCF plink parser fixture mismatch for {stage_id}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}
