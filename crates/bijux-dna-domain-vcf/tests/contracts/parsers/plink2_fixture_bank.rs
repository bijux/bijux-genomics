use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::{parse_plink2_stage_metrics, VcfDomainStage};

const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfPlink2FixtureCase {
    stage: VcfDomainStage,
    parser_id: &'static str,
    required_files: &'static [&'static str],
}

const VCF_PLINK2_FIXTURE_CASES: &[VcfPlink2FixtureCase] = &[
    VcfPlink2FixtureCase {
        stage: VcfDomainStage::Qc,
        parser_id: "parse_plink2_qc_metrics",
        required_files: &[
            "raw.smiss",
            "raw.vmiss",
            "raw.afreq",
            "raw.het",
            "raw.hardy",
            "raw.log",
            "expected.normalized.json",
        ],
    },
    VcfPlink2FixtureCase {
        stage: VcfDomainStage::Pca,
        parser_id: "parse_plink2_pca_metrics",
        required_files: &[
            "raw.eigenvec",
            "raw.eigenval",
            "raw.pca_manifest.json",
            "raw.log",
            "expected.normalized.json",
        ],
    },
    VcfPlink2FixtureCase {
        stage: VcfDomainStage::Admixture,
        parser_id: "parse_plink2_admixture_metrics",
        required_files: &[
            "raw.q_matrix.tsv",
            "raw.k_selection.json",
            "raw.log",
            "expected.normalized.json",
        ],
    },
    VcfPlink2FixtureCase {
        stage: VcfDomainStage::PopulationStructure,
        parser_id: "parse_plink2_population_structure_metrics",
        required_files: &[
            "raw.prune.in",
            "raw.prune.out",
            "raw.eigenvec",
            "raw.eigenval",
            "raw.pca.json",
            "raw.admixture.json",
            "raw.population_structure.json",
            "raw.prune.log",
            "raw.pca.log",
            "expected.normalized.json",
        ],
    },
    VcfPlink2FixtureCase {
        stage: VcfDomainStage::Roh,
        parser_id: "parse_plink2_roh_metrics",
        required_files: &["raw.command.json", "raw.hom", "raw.log", "expected.normalized.json"],
    },
];

#[test]
fn vcf_plink2_fixture_bank_covers_governed_stage_set() {
    let governed_stage_ids =
        VCF_PLINK2_FIXTURE_CASES.iter().map(|case| case.stage.as_str()).collect::<Vec<_>>();
    assert_eq!(
        governed_stage_ids,
        vec!["vcf.qc", "vcf.pca", "vcf.admixture", "vcf.population_structure", "vcf.roh"]
    );
    for case in VCF_PLINK2_FIXTURE_CASES {
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
fn vcf_plink2_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in VCF_PLINK2_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(&observed, &expected, case.stage.as_str());
    }
    Ok(())
}

fn render_case(case: &VcfPlink2FixtureCase) -> Result<serde_json::Value> {
    let normalized = parse_plink2_stage_metrics(case.stage, &fixture_dir(case))
        .with_context(|| format!("parse fixture stage `{}`", case.stage.as_str()))?;
    Ok(serde_json::json!({
        "schema_version": VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage.as_str(),
        "tool_id": "plink2",
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &VcfPlink2FixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join("expected.normalized.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &VcfPlink2FixtureCase) -> PathBuf {
    repo_root().join("benchmarks/tests/fixtures/bench/parsers/vcf/plink2").join(case.stage.as_str())
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
        "VCF plink2 parser fixture mismatch for {stage_id}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}
