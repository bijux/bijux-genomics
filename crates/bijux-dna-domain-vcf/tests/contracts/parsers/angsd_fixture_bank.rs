use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::{parse_angsd_stage_metrics, VcfDomainStage};

const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfAngsdFixtureCase {
    stage: VcfDomainStage,
    parser_id: &'static str,
    required_files: &'static [&'static str],
}

const VCF_ANGSD_FIXTURE_CASES: &[VcfAngsdFixtureCase] = &[
    VcfAngsdFixtureCase {
        stage: VcfDomainStage::CallGl,
        parser_id: "parse_angsd_call_gl_metrics",
        required_files: &["raw.gl_sites.vcf", "raw.command.json", "expected.normalized.json"],
    },
    VcfAngsdFixtureCase {
        stage: VcfDomainStage::CallPseudohaploid,
        parser_id: "parse_angsd_call_pseudohaploid_metrics",
        required_files: &["raw.pseudohaploid.vcf", "raw.command.json", "expected.normalized.json"],
    },
    VcfAngsdFixtureCase {
        stage: VcfDomainStage::DamageFilter,
        parser_id: "parse_angsd_damage_filter_metrics",
        required_files: &[
            "raw.damage_report.txt",
            "raw.damage_bias_audit_report.json",
            "raw.command.json",
            "expected.normalized.json",
        ],
    },
    VcfAngsdFixtureCase {
        stage: VcfDomainStage::GlPropagation,
        parser_id: "parse_angsd_gl_propagation_metrics",
        required_files: &[
            "raw.input.vcf",
            "raw.gl_propagated.vcf",
            "raw.gl_propagation_report.json",
            "raw.command.json",
            "expected.normalized.json",
        ],
    },
];

#[test]
fn vcf_angsd_fixture_bank_covers_governed_stage_set() {
    let governed_stage_ids =
        VCF_ANGSD_FIXTURE_CASES.iter().map(|case| case.stage.as_str()).collect::<Vec<_>>();
    assert_eq!(
        governed_stage_ids,
        vec!["vcf.call_gl", "vcf.call_pseudohaploid", "vcf.damage_filter", "vcf.gl_propagation",]
    );
    for case in VCF_ANGSD_FIXTURE_CASES {
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
fn vcf_angsd_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in VCF_ANGSD_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(&observed, &expected, case.stage.as_str());
    }
    Ok(())
}

fn render_case(case: &VcfAngsdFixtureCase) -> Result<serde_json::Value> {
    let normalized = parse_angsd_stage_metrics(case.stage, &fixture_dir(case))
        .with_context(|| format!("parse fixture stage `{}`", case.stage.as_str()))?;
    Ok(serde_json::json!({
        "schema_version": VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage.as_str(),
        "tool_id": "angsd",
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &VcfAngsdFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join("expected.normalized.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &VcfAngsdFixtureCase) -> PathBuf {
    repo_root().join("tests/fixtures/bench/parsers/vcf/angsd").join(case.stage.as_str())
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
        "VCF angsd parser fixture mismatch for {stage_id}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}
