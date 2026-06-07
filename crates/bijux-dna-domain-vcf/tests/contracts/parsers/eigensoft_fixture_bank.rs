use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::{parse_eigensoft_stage_metrics, VcfDomainStage};

const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfEigensoftFixtureCase {
    stage: VcfDomainStage,
    fixture_dir_name: &'static str,
    parser_id: &'static str,
    required_files: &'static [&'static str],
}

const VCF_EIGENSOFT_FIXTURE_CASES: &[VcfEigensoftFixtureCase] = &[
    VcfEigensoftFixtureCase {
        stage: VcfDomainStage::Pca,
        fixture_dir_name: "pca",
        parser_id: "parse_eigensoft_pca_metrics",
        required_files: &[
            "raw.command.json",
            "raw.geno",
            "raw.snp",
            "raw.ind",
            "raw.evec",
            "raw.eval",
            "raw.smartpca.log",
            "raw.sample_metadata.tsv",
            "expected.normalized.json",
        ],
    },
    VcfEigensoftFixtureCase {
        stage: VcfDomainStage::PopulationStructure,
        fixture_dir_name: "population_structure",
        parser_id: "parse_eigensoft_population_structure_metrics",
        required_files: &[
            "raw.command.json",
            "raw.geno",
            "raw.snp",
            "raw.ind",
            "raw.evec",
            "raw.eval",
            "raw.smartpca.log",
            "raw.sample_metadata.tsv",
            "raw.population_structure.json",
            "expected.normalized.json",
        ],
    },
];

#[test]
fn vcf_eigensoft_fixture_bank_covers_governed_stage_set() {
    let governed_stage_ids =
        VCF_EIGENSOFT_FIXTURE_CASES.iter().map(|case| case.stage.as_str()).collect::<Vec<_>>();
    assert_eq!(governed_stage_ids, vec!["vcf.pca", "vcf.population_structure"]);
    for case in VCF_EIGENSOFT_FIXTURE_CASES {
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
fn vcf_eigensoft_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in VCF_EIGENSOFT_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(&observed, &expected, case.stage.as_str());
    }
    Ok(())
}

fn render_case(case: &VcfEigensoftFixtureCase) -> Result<serde_json::Value> {
    let normalized = parse_eigensoft_stage_metrics(case.stage, &fixture_dir(case))
        .with_context(|| format!("parse fixture stage `{}`", case.stage.as_str()))?;
    Ok(serde_json::json!({
        "schema_version": VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage.as_str(),
        "tool_id": "eigensoft",
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &VcfEigensoftFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join("expected.normalized.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &VcfEigensoftFixtureCase) -> PathBuf {
    repo_root().join("benchmarks/tests/fixtures/bench/parsers/vcf/eigensoft").join(case.fixture_dir_name)
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
        "VCF eigensoft parser fixture mismatch for {stage_id}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}
