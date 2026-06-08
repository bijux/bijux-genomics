use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::{parse_bcftools_stage_metrics, VcfDomainStage};

const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfBcftoolsFixtureCase {
    stage: VcfDomainStage,
    parser_id: &'static str,
    required_files: &'static [&'static str],
}

const VCF_BCFTOOLS_FIXTURE_CASES: &[VcfBcftoolsFixtureCase] = &[
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::Call,
        parser_id: "parse_bcftools_call_metrics",
        required_files: &["raw.calls.vcf", "expected.normalized.json"],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::CallDiploid,
        parser_id: "parse_bcftools_call_diploid_metrics",
        required_files: &["raw.diploid.vcf", "expected.normalized.json"],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::CallGl,
        parser_id: "parse_bcftools_call_gl_metrics",
        required_files: &["raw.gl.vcf", "expected.normalized.json"],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::CallPseudohaploid,
        parser_id: "parse_bcftools_call_pseudohaploid_metrics",
        required_files: &["raw.pseudohaploid.vcf", "raw.command.json", "expected.normalized.json"],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::DamageFilter,
        parser_id: "parse_bcftools_damage_filter_metrics",
        required_files: &[
            "raw.damage_filtered.vcf",
            "raw.damage_filter_summary.json",
            "raw.damage_filter_counts.json",
            "expected.normalized.json",
        ],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::Filter,
        parser_id: "parse_bcftools_filter_metrics",
        required_files: &[
            "raw.filtered.vcf",
            "raw.filter_explain.json",
            "expected.normalized.json",
        ],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::GlPropagation,
        parser_id: "parse_bcftools_gl_propagation_metrics",
        required_files: &["raw.input.vcf", "raw.propagated.vcf", "expected.normalized.json"],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::Postprocess,
        parser_id: "parse_bcftools_postprocess_metrics",
        required_files: &[
            "raw.postprocess.vcf",
            "raw.postprocess.vcf.tbi",
            "raw.validate_outputs.json",
            "raw.final_manifest.json",
            "expected.normalized.json",
        ],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::PrepareReferencePanel,
        parser_id: "parse_bcftools_prepare_reference_panel_metrics",
        required_files: &[
            "raw.panel.vcf",
            "raw.panel.vcf.tbi",
            "raw.panel_manifest.json",
            "expected.normalized.json",
        ],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::Stats,
        parser_id: "parse_bcftools_stats_metrics",
        required_files: &["raw.bcftools_stats.txt", "expected.normalized.json"],
    },
    VcfBcftoolsFixtureCase {
        stage: VcfDomainStage::Qc,
        parser_id: "parse_bcftools_qc_metrics",
        required_files: &[
            "raw.sample_missingness.tsv",
            "raw.variant_missingness.tsv",
            "raw.allele_frequency.tsv",
            "raw.heterozygosity.tsv",
            "raw.hwe.tsv",
            "raw.thresholds.json",
            "expected.normalized.json",
        ],
    },
];

#[test]
fn vcf_bcftools_fixture_bank_covers_governed_stage_set() {
    let governed_stage_ids =
        VCF_BCFTOOLS_FIXTURE_CASES.iter().map(|case| case.stage.as_str()).collect::<Vec<_>>();
    assert_eq!(
        governed_stage_ids,
        vec![
            "vcf.call",
            "vcf.call_diploid",
            "vcf.call_gl",
            "vcf.call_pseudohaploid",
            "vcf.damage_filter",
            "vcf.filter",
            "vcf.gl_propagation",
            "vcf.postprocess",
            "vcf.prepare_reference_panel",
            "vcf.stats",
            "vcf.qc",
        ]
    );
    for case in VCF_BCFTOOLS_FIXTURE_CASES {
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
fn vcf_bcftools_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in VCF_BCFTOOLS_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(&observed, &expected, case.stage.as_str());
    }
    Ok(())
}

fn render_case(case: &VcfBcftoolsFixtureCase) -> Result<serde_json::Value> {
    let normalized = parse_bcftools_stage_metrics(case.stage, &fixture_dir(case))
        .with_context(|| format!("parse fixture stage `{}`", case.stage.as_str()))?;
    Ok(serde_json::json!({
        "schema_version": VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage.as_str(),
        "tool_id": "bcftools",
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &VcfBcftoolsFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join("expected.normalized.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &VcfBcftoolsFixtureCase) -> PathBuf {
    repo_root()
        .join("benchmarks/tests/fixtures/bench/parsers/vcf/bcftools")
        .join(case.stage.as_str())
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
        "VCF bcftools parser fixture mismatch for {stage_id}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}
