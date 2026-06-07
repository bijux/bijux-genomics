use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_bam::metrics::{
    parse_addeam_json, parse_contamination_json, parse_damageprofiler_json,
    parse_mapdamage2_misincorporation, parse_mosdepth_summary, parse_ngsbriggs_json,
    parse_picard_gc_bias_metrics, parse_picard_insert_size_metrics, parse_pmdtools_json,
    parse_preseq_estimates, parse_pydamage_json, parse_samtools_depth,
    parse_samtools_depth_with_uniformity, parse_samtools_flagstat, parse_samtools_idxstats,
    parse_samtools_stats, parse_sex_json,
};

const BAM_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.bam_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BamRawParserFixtureCase {
    stage_id: &'static str,
    tool_id: &'static str,
    parser_id: &'static str,
    raw_file: &'static str,
    expected_file: &'static str,
}

const BAM_RAW_PARSER_FIXTURE_CASES: &[BamRawParserFixtureCase] = &[
    BamRawParserFixtureCase {
        stage_id: "bam.qc_pre",
        tool_id: "samtools",
        parser_id: "parse_samtools_flagstat",
        raw_file: "raw.flagstat.txt",
        expected_file: "expected.flagstat.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.mapping_summary",
        tool_id: "samtools",
        parser_id: "parse_samtools_stats",
        raw_file: "raw.samtools-stats.txt",
        expected_file: "expected.samtools-stats.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.endogenous_content",
        tool_id: "samtools",
        parser_id: "parse_samtools_idxstats",
        raw_file: "raw.idxstats.txt",
        expected_file: "expected.idxstats.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.coverage",
        tool_id: "mosdepth",
        parser_id: "parse_mosdepth_summary",
        raw_file: "raw.mosdepth-summary.txt",
        expected_file: "expected.mosdepth-summary.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.coverage",
        tool_id: "samtools",
        parser_id: "parse_samtools_depth",
        raw_file: "raw.depth.txt",
        expected_file: "expected.depth.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.coverage",
        tool_id: "samtools",
        parser_id: "parse_samtools_depth_with_uniformity",
        raw_file: "raw.depth.txt",
        expected_file: "expected.depth-with-uniformity.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.complexity",
        tool_id: "preseq",
        parser_id: "parse_preseq_estimates",
        raw_file: "raw.preseq.txt",
        expected_file: "expected.preseq.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.damage",
        tool_id: "pydamage",
        parser_id: "parse_pydamage_json",
        raw_file: "raw.pydamage.json",
        expected_file: "expected.pydamage.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.authenticity",
        tool_id: "damageprofiler",
        parser_id: "parse_damageprofiler_json",
        raw_file: "raw.damageprofiler.json",
        expected_file: "expected.damageprofiler.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.damage",
        tool_id: "mapdamage2",
        parser_id: "parse_mapdamage2_misincorporation",
        raw_file: "raw.mapdamage2.txt",
        expected_file: "expected.mapdamage2.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.contamination",
        tool_id: "schmutzi",
        parser_id: "parse_contamination_json",
        raw_file: "raw.contamination.json",
        expected_file: "expected.contamination.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.sex",
        tool_id: "rxy",
        parser_id: "parse_sex_json",
        raw_file: "raw.sex.json",
        expected_file: "expected.sex.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.insert_size",
        tool_id: "picard",
        parser_id: "parse_picard_insert_size_metrics",
        raw_file: "raw.insert-size.metrics.txt",
        expected_file: "expected.insert-size.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.gc_bias",
        tool_id: "picard",
        parser_id: "parse_picard_gc_bias_metrics",
        raw_file: "raw.gc-bias.metrics.txt",
        expected_file: "expected.gc-bias.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.authenticity",
        tool_id: "pmdtools",
        parser_id: "parse_pmdtools_json",
        raw_file: "raw.pmdtools.json",
        expected_file: "expected.pmdtools.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.damage",
        tool_id: "ngsbriggs",
        parser_id: "parse_ngsbriggs_json",
        raw_file: "raw.ngsbriggs.json",
        expected_file: "expected.ngsbriggs.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.damage",
        tool_id: "addeam",
        parser_id: "parse_addeam_json",
        raw_file: "raw.addeam.json",
        expected_file: "expected.addeam.json",
    },
];

#[test]
fn bam_raw_parser_fixture_bank_covers_every_governed_case() {
    assert_eq!(BAM_RAW_PARSER_FIXTURE_CASES.len(), 17);
    for case in BAM_RAW_PARSER_FIXTURE_CASES {
        let dir = fixture_dir(case);
        assert!(dir.exists(), "missing fixture directory: {}", dir.display());
        assert!(
            dir.join(case.raw_file).exists(),
            "missing raw fixture `{}` for {} / {} / {}",
            case.raw_file,
            case.stage_id,
            case.tool_id,
            case.parser_id
        );
        assert!(
            dir.join(case.expected_file).exists(),
            "missing expected fixture `{}` for {} / {} / {}",
            case.expected_file,
            case.stage_id,
            case.tool_id,
            case.parser_id
        );
    }
}

#[test]
fn bam_raw_parser_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in BAM_RAW_PARSER_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = parse_case(case)?;
        assert_json_matches(
            &observed,
            &expected,
            &format!("{} / {} / {}", case.stage_id, case.tool_id, case.parser_id),
        );
    }
    Ok(())
}

fn parse_case(case: &BamRawParserFixtureCase) -> Result<serde_json::Value> {
    let raw_path = fixture_dir(case).join(case.raw_file);
    let normalized = match case.parser_id {
        "parse_samtools_flagstat" => serde_json::to_value(parse_samtools_flagstat(&raw_path)?)?,
        "parse_samtools_stats" => {
            let (fragment_length, mapq_summary) = parse_samtools_stats(&raw_path)?;
            serde_json::json!({
                "fragment_length": fragment_length,
                "mapq_summary": mapq_summary,
            })
        }
        "parse_samtools_idxstats" => serde_json::to_value(parse_samtools_idxstats(&raw_path)?)?,
        "parse_mosdepth_summary" => serde_json::to_value(parse_mosdepth_summary(&raw_path)?)?,
        "parse_samtools_depth" => serde_json::to_value(parse_samtools_depth(&raw_path)?)?,
        "parse_samtools_depth_with_uniformity" => {
            let (coverage, uniformity) = parse_samtools_depth_with_uniformity(&raw_path)?;
            serde_json::json!({
                "coverage": coverage,
                "uniformity": uniformity,
            })
        }
        "parse_preseq_estimates" => serde_json::to_value(parse_preseq_estimates(&raw_path)?)?,
        "parse_pydamage_json" => serde_json::to_value(parse_pydamage_json(&raw_path)?)?,
        "parse_damageprofiler_json" => serde_json::to_value(parse_damageprofiler_json(&raw_path)?)?,
        "parse_mapdamage2_misincorporation" => {
            serde_json::to_value(parse_mapdamage2_misincorporation(&raw_path)?)?
        }
        "parse_contamination_json" => serde_json::to_value(parse_contamination_json(&raw_path)?)?,
        "parse_sex_json" => serde_json::to_value(parse_sex_json(&raw_path)?)?,
        "parse_picard_insert_size_metrics" => {
            serde_json::to_value(parse_picard_insert_size_metrics(&raw_path)?)?
        }
        "parse_picard_gc_bias_metrics" => {
            serde_json::to_value(parse_picard_gc_bias_metrics(&raw_path)?)?
        }
        "parse_pmdtools_json" => serde_json::to_value(parse_pmdtools_json(&raw_path)?)?,
        "parse_ngsbriggs_json" => serde_json::to_value(parse_ngsbriggs_json(&raw_path)?)?,
        "parse_addeam_json" => serde_json::to_value(parse_addeam_json(&raw_path)?)?,
        parser_id => {
            return Err(anyhow::anyhow!(
                "unsupported BAM raw parser fixture parser_id `{parser_id}`"
            ));
        }
    };

    Ok(serde_json::json!({
        "schema_version": BAM_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage_id,
        "tool_id": case.tool_id,
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &BamRawParserFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join(case.expected_file);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &BamRawParserFixtureCase) -> PathBuf {
    repo_root().join("benchmarks/tests/fixtures/bench/parsers/bam").join(case.stage_id).join(case.tool_id)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn assert_json_matches(observed: &serde_json::Value, expected: &serde_json::Value, context: &str) {
    assert!(
        json_matches(observed, expected),
        "BAM raw parser fixture mismatch for {context}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}

fn json_matches(observed: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (observed, expected) {
        (serde_json::Value::Null, serde_json::Value::Null)
        | (serde_json::Value::Bool(_), serde_json::Value::Bool(_))
        | (serde_json::Value::String(_), serde_json::Value::String(_)) => observed == expected,
        (serde_json::Value::Number(left), serde_json::Value::Number(right)) => {
            match (left.as_i64(), right.as_i64()) {
                (Some(left), Some(right)) => left == right,
                _ => left
                    .as_f64()
                    .zip(right.as_f64())
                    .is_some_and(|(left, right)| (left - right).abs() <= 1e-12),
            }
        }
        (serde_json::Value::Array(left), serde_json::Value::Array(right)) => {
            left.len() == right.len()
                && left.iter().zip(right).all(|(left, right)| json_matches(left, right))
        }
        (serde_json::Value::Object(left), serde_json::Value::Object(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, left_value)| {
                    right.get(key).is_some_and(|right_value| json_matches(left_value, right_value))
                })
        }
        _ => false,
    }
}
