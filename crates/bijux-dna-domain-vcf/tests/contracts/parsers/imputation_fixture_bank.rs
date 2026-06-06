use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::{parse_imputation_stage_metrics, VcfDomainStage};

const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfImputationFixtureCase {
    tool_id: &'static str,
    stage: VcfDomainStage,
    parser_id: &'static str,
    required_files: &'static [&'static str],
}

const IMPUTE_REQUIRED_FILES: &[&str] = &[
    "raw.command.json",
    "raw.imputed.vcf",
    "raw.imputation_qc.json",
    "raw.imputation_manifest.json",
    "raw.imputation_accept.json",
    "raw.truth.vcf",
    "expected.normalized.json",
];

const IMPUTATION_REQUIRED_FILES: &[&str] = &[
    "raw.command.json",
    "raw.imputed.vcf",
    "raw.imputation_qc.json",
    "raw.imputation_manifest.json",
    "raw.imputation_accept.json",
    "raw.orchestration_manifest.json",
    "raw.truth.vcf",
    "expected.normalized.json",
];

const VCF_IMPUTATION_FIXTURE_CASES: &[VcfImputationFixtureCase] = &[
    VcfImputationFixtureCase {
        tool_id: "beagle",
        stage: VcfDomainStage::Impute,
        parser_id: "parse_beagle_impute_metrics",
        required_files: IMPUTE_REQUIRED_FILES,
    },
    VcfImputationFixtureCase {
        tool_id: "beagle",
        stage: VcfDomainStage::Imputation,
        parser_id: "parse_beagle_imputation_metrics",
        required_files: IMPUTATION_REQUIRED_FILES,
    },
    VcfImputationFixtureCase {
        tool_id: "glimpse",
        stage: VcfDomainStage::Impute,
        parser_id: "parse_glimpse_impute_metrics",
        required_files: IMPUTE_REQUIRED_FILES,
    },
    VcfImputationFixtureCase {
        tool_id: "glimpse",
        stage: VcfDomainStage::Imputation,
        parser_id: "parse_glimpse_imputation_metrics",
        required_files: IMPUTATION_REQUIRED_FILES,
    },
    VcfImputationFixtureCase {
        tool_id: "impute5",
        stage: VcfDomainStage::Impute,
        parser_id: "parse_impute5_impute_metrics",
        required_files: IMPUTE_REQUIRED_FILES,
    },
    VcfImputationFixtureCase {
        tool_id: "impute5",
        stage: VcfDomainStage::Imputation,
        parser_id: "parse_impute5_imputation_metrics",
        required_files: IMPUTATION_REQUIRED_FILES,
    },
    VcfImputationFixtureCase {
        tool_id: "minimac4",
        stage: VcfDomainStage::Impute,
        parser_id: "parse_minimac4_impute_metrics",
        required_files: IMPUTE_REQUIRED_FILES,
    },
    VcfImputationFixtureCase {
        tool_id: "minimac4",
        stage: VcfDomainStage::Imputation,
        parser_id: "parse_minimac4_imputation_metrics",
        required_files: IMPUTATION_REQUIRED_FILES,
    },
];

#[test]
fn vcf_imputation_fixture_bank_covers_governed_tool_stage_rows() {
    let governed_rows = VCF_IMPUTATION_FIXTURE_CASES
        .iter()
        .map(|case| format!("{}:{}", case.tool_id, case.stage.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        governed_rows,
        vec![
            "beagle:vcf.impute",
            "beagle:vcf.imputation",
            "glimpse:vcf.impute",
            "glimpse:vcf.imputation",
            "impute5:vcf.impute",
            "impute5:vcf.imputation",
            "minimac4:vcf.impute",
            "minimac4:vcf.imputation",
        ]
    );
    for case in VCF_IMPUTATION_FIXTURE_CASES {
        let dir = fixture_dir(case);
        assert!(dir.exists(), "missing fixture directory: {}", dir.display());
        for file_name in case.required_files {
            assert!(
                dir.join(file_name).exists(),
                "missing fixture file `{file_name}` for {} / {}",
                case.tool_id,
                case.stage.as_str()
            );
        }
    }
}

#[test]
fn vcf_imputation_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in VCF_IMPUTATION_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(&observed, &expected, case.tool_id, case.stage.as_str());
    }
    Ok(())
}

#[test]
fn vcf_imputation_fixture_bank_rejects_truth_drift() -> Result<()> {
    let case = VCF_IMPUTATION_FIXTURE_CASES
        .iter()
        .find(|case| case.tool_id == "beagle" && case.stage == VcfDomainStage::Impute)
        .copied()
        .expect("governed beagle impute fixture");
    let dir = unique_temp_dir(case.tool_id, case.stage.as_str())?;
    copy_fixture_dir(&fixture_dir(&case), &dir)?;
    fs::write(
        dir.join("raw.truth.vcf"),
        "##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\nchr1\t10\t.\tA\tG\t60\tPASS\t.\tGT\t1/1\n",
    )
    .with_context(|| format!("install truth drift probe under {}", dir.display()))?;
    let error = parse_imputation_stage_metrics(case.tool_id, case.stage, &dir)
        .expect_err("truth drift must fail");
    let message = error.to_string();
    assert!(
        message.contains("masked truth match count drifted"),
        "unexpected truth drift failure: {message}"
    );
    let _ = fs::remove_dir_all(&dir);
    Ok(())
}

fn render_case(case: &VcfImputationFixtureCase) -> Result<serde_json::Value> {
    let normalized =
        parse_imputation_stage_metrics(case.tool_id, case.stage, &fixture_dir(case))
            .with_context(|| format!("parse {} / {}", case.tool_id, case.stage.as_str()))?;
    Ok(serde_json::json!({
        "schema_version": VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage.as_str(),
        "tool_id": case.tool_id,
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &VcfImputationFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join("expected.normalized.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &VcfImputationFixtureCase) -> PathBuf {
    repo_root()
        .join("tests/fixtures/bench/parsers/vcf/imputation")
        .join(case.tool_id)
        .join(case.stage.as_str())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn unique_temp_dir(tool_id: &str, stage_id: &str) -> Result<PathBuf> {
    let stamp =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = std::env::temp_dir()
        .join(format!("bijux-vcf-imputation-{tool_id}-{}-{stamp}", stage_id.replace('.', "_")));
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

fn assert_json_matches(
    observed: &serde_json::Value,
    expected: &serde_json::Value,
    tool_id: &str,
    stage_id: &str,
) {
    assert!(
        observed == expected,
        "VCF imputation parser fixture mismatch for {tool_id} / {stage_id}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}
