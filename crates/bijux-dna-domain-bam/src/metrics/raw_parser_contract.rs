use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::{
    parse_addeam_json, parse_contamination_json, parse_damageprofiler_json,
    parse_mapdamage2_misincorporation, parse_mosdepth_summary, parse_ngsbriggs_json,
    parse_picard_gc_bias_metrics, parse_picard_insert_size_metrics, parse_pmdtools_json,
    parse_preseq_estimates, parse_pydamage_json, parse_samtools_depth,
    parse_samtools_depth_with_uniformity, parse_samtools_flagstat, parse_samtools_idxstats,
    parse_samtools_stats, parse_sex_json,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BamRawParserFailureClass {
    MissingRawOutput,
    EmptyRawOutput,
    MalformedRawOutput,
    UnexpectedSuccess,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BamRawParserFailureContractRow {
    pub expected_failure_class: BamRawParserFailureClass,
    pub observed_failure_class: BamRawParserFailureClass,
    pub stage_id: String,
    pub tool_id: String,
    pub parser_id: String,
    pub observed_error: String,
    pub passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BamRawParserFixtureCase {
    stage_id: &'static str,
    tool_id: &'static str,
    parser_id: &'static str,
    raw_file: &'static str,
}

const BAM_RAW_PARSER_FIXTURE_CASES: &[BamRawParserFixtureCase] = &[
    BamRawParserFixtureCase {
        stage_id: "bam.qc_pre",
        tool_id: "samtools",
        parser_id: "parse_samtools_flagstat",
        raw_file: "raw.flagstat.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.mapping_summary",
        tool_id: "samtools",
        parser_id: "parse_samtools_stats",
        raw_file: "raw.samtools-stats.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.endogenous_content",
        tool_id: "samtools",
        parser_id: "parse_samtools_idxstats",
        raw_file: "raw.idxstats.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.coverage",
        tool_id: "mosdepth",
        parser_id: "parse_mosdepth_summary",
        raw_file: "raw.mosdepth-summary.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.coverage",
        tool_id: "samtools",
        parser_id: "parse_samtools_depth",
        raw_file: "raw.depth.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.coverage",
        tool_id: "samtools",
        parser_id: "parse_samtools_depth_with_uniformity",
        raw_file: "raw.depth.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.complexity",
        tool_id: "preseq",
        parser_id: "parse_preseq_estimates",
        raw_file: "raw.preseq.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.damage",
        tool_id: "pydamage",
        parser_id: "parse_pydamage_json",
        raw_file: "raw.pydamage.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.authenticity",
        tool_id: "damageprofiler",
        parser_id: "parse_damageprofiler_json",
        raw_file: "raw.damageprofiler.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.damage",
        tool_id: "mapdamage2",
        parser_id: "parse_mapdamage2_misincorporation",
        raw_file: "raw.mapdamage2.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.contamination",
        tool_id: "schmutzi",
        parser_id: "parse_contamination_json",
        raw_file: "raw.contamination.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.sex",
        tool_id: "rxy",
        parser_id: "parse_sex_json",
        raw_file: "raw.sex.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.insert_size",
        tool_id: "picard",
        parser_id: "parse_picard_insert_size_metrics",
        raw_file: "raw.insert-size.metrics.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.gc_bias",
        tool_id: "picard",
        parser_id: "parse_picard_gc_bias_metrics",
        raw_file: "raw.gc-bias.metrics.txt",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.authenticity",
        tool_id: "pmdtools",
        parser_id: "parse_pmdtools_json",
        raw_file: "raw.pmdtools.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.damage",
        tool_id: "ngsbriggs",
        parser_id: "parse_ngsbriggs_json",
        raw_file: "raw.ngsbriggs.json",
    },
    BamRawParserFixtureCase {
        stage_id: "bam.damage",
        tool_id: "addeam",
        parser_id: "parse_addeam_json",
        raw_file: "raw.addeam.json",
    },
];

pub fn evaluate_bam_raw_parser_failure_contracts(
    repo_root: &Path,
    scratch_root: &Path,
) -> Result<Vec<BamRawParserFailureContractRow>> {
    BAM_RAW_PARSER_FIXTURE_CASES
        .iter()
        .flat_map(|case| {
            [
                BamRawParserFailureClass::MissingRawOutput,
                BamRawParserFailureClass::EmptyRawOutput,
                BamRawParserFailureClass::MalformedRawOutput,
            ]
            .into_iter()
            .map(move |failure_class| evaluate_case(repo_root, scratch_root, case, failure_class))
        })
        .collect()
}

fn evaluate_case(
    repo_root: &Path,
    scratch_root: &Path,
    case: &BamRawParserFixtureCase,
    expected_failure_class: BamRawParserFailureClass,
) -> Result<BamRawParserFailureContractRow> {
    let temp = bijux_dna_infra::temp_dir_in(scratch_root, "bam-parser-failure-")
        .map_err(anyhow::Error::from)
        .context("create BAM parser failure probe root")?;
    let raw_path = materialize_failure_probe(repo_root, temp.path(), case, expected_failure_class)?;
    let observed = parse_case(case, &raw_path);
    let (observed_failure_class, observed_error) = classify_observed_failure(&raw_path, observed);
    let passed =
        observed_failure_class == expected_failure_class && !observed_error.trim().is_empty();

    Ok(BamRawParserFailureContractRow {
        expected_failure_class,
        observed_failure_class,
        stage_id: case.stage_id.to_string(),
        tool_id: case.tool_id.to_string(),
        parser_id: case.parser_id.to_string(),
        observed_error,
        passed,
    })
}

fn materialize_failure_probe(
    repo_root: &Path,
    temp_root: &Path,
    case: &BamRawParserFixtureCase,
    failure_class: BamRawParserFailureClass,
) -> Result<PathBuf> {
    let path = temp_root.join(case.raw_file);
    match failure_class {
        BamRawParserFailureClass::MissingRawOutput => Ok(path),
        BamRawParserFailureClass::EmptyRawOutput => {
            fs::write(&path, "").with_context(|| format!("write {}", path.display()))?;
            Ok(path)
        }
        BamRawParserFailureClass::MalformedRawOutput => {
            let payload = malformed_payload(repo_root, case)?;
            fs::write(&path, payload).with_context(|| format!("write {}", path.display()))?;
            Ok(path)
        }
        BamRawParserFailureClass::UnexpectedSuccess => Ok(path),
    }
}

fn malformed_payload(repo_root: &Path, case: &BamRawParserFixtureCase) -> Result<String> {
    let fixture_path = fixture_dir(repo_root, case).join(case.raw_file);
    let payload = if case.raw_file.ends_with(".json") {
        "{ malformed json".to_string()
    } else {
        "not a valid parser fixture".to_string()
    };
    if payload.is_empty() {
        return Err(anyhow!("malformed payload generation failed for {}", fixture_path.display()));
    }
    Ok(payload)
}

fn classify_observed_failure(
    raw_path: &Path,
    observed: Result<()>,
) -> (BamRawParserFailureClass, String) {
    match observed {
        Ok(()) => (
            BamRawParserFailureClass::UnexpectedSuccess,
            format!("parser unexpectedly accepted {}", raw_path.display()),
        ),
        Err(error) => {
            let failure_class = if !raw_path.exists() {
                BamRawParserFailureClass::MissingRawOutput
            } else if raw_path.metadata().map(|metadata| metadata.len()).unwrap_or(1) == 0 {
                BamRawParserFailureClass::EmptyRawOutput
            } else {
                BamRawParserFailureClass::MalformedRawOutput
            };
            (failure_class, error.to_string())
        }
    }
}

fn parse_case(case: &BamRawParserFixtureCase, raw_path: &Path) -> Result<()> {
    match case.parser_id {
        "parse_samtools_flagstat" => {
            parse_samtools_flagstat(raw_path)?;
        }
        "parse_samtools_stats" => {
            parse_samtools_stats(raw_path)?;
        }
        "parse_samtools_idxstats" => {
            parse_samtools_idxstats(raw_path)?;
        }
        "parse_mosdepth_summary" => {
            parse_mosdepth_summary(raw_path)?;
        }
        "parse_samtools_depth" => {
            parse_samtools_depth(raw_path)?;
        }
        "parse_samtools_depth_with_uniformity" => {
            parse_samtools_depth_with_uniformity(raw_path)?;
        }
        "parse_preseq_estimates" => {
            parse_preseq_estimates(raw_path)?;
        }
        "parse_pydamage_json" => {
            parse_pydamage_json(raw_path)?;
        }
        "parse_damageprofiler_json" => {
            parse_damageprofiler_json(raw_path)?;
        }
        "parse_mapdamage2_misincorporation" => {
            parse_mapdamage2_misincorporation(raw_path)?;
        }
        "parse_contamination_json" => {
            parse_contamination_json(raw_path)?;
        }
        "parse_sex_json" => {
            parse_sex_json(raw_path)?;
        }
        "parse_picard_insert_size_metrics" => {
            parse_picard_insert_size_metrics(raw_path)?;
        }
        "parse_picard_gc_bias_metrics" => {
            parse_picard_gc_bias_metrics(raw_path)?;
        }
        "parse_pmdtools_json" => {
            parse_pmdtools_json(raw_path)?;
        }
        "parse_ngsbriggs_json" => {
            parse_ngsbriggs_json(raw_path)?;
        }
        "parse_addeam_json" => {
            parse_addeam_json(raw_path)?;
        }
        parser_id => {
            return Err(anyhow!(
                "unsupported BAM raw parser failure contract parser_id `{parser_id}`"
            ));
        }
    }
    Ok(())
}

fn fixture_dir(repo_root: &Path, case: &BamRawParserFixtureCase) -> PathBuf {
    repo_root.join("benchmarks/tests/fixtures/bench/parsers/bam").join(case.stage_id).join(case.tool_id)
}

#[cfg(test)]
mod tests {
    use super::{evaluate_bam_raw_parser_failure_contracts, BamRawParserFailureClass};
    use anyhow::Result;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn governed_bam_raw_parser_failure_contracts_classify_missing_empty_and_malformed_inputs(
    ) -> Result<()> {
        let root = repo_root();
        let scratch_root = root.join("artifacts/contract-probes");
        let rows = evaluate_bam_raw_parser_failure_contracts(&root, &scratch_root)?;

        assert_eq!(rows.len(), 51);
        let failed_rows = rows.iter().filter(|row| !row.passed).collect::<Vec<_>>();
        assert!(failed_rows.is_empty(), "failed rows: {failed_rows:#?}");
        assert_eq!(
            rows.iter()
                .filter(
                    |row| row.expected_failure_class == BamRawParserFailureClass::MissingRawOutput
                )
                .count(),
            17
        );
        assert_eq!(
            rows.iter()
                .filter(|row| row.expected_failure_class == BamRawParserFailureClass::EmptyRawOutput)
                .count(),
            17
        );
        assert_eq!(
            rows.iter()
                .filter(|row| row.expected_failure_class
                    == BamRawParserFailureClass::MalformedRawOutput)
                .count(),
            17
        );
        assert!(rows.iter().all(|row| !row.observed_error.trim().is_empty()));
        assert!(rows.iter().any(|row| {
            row.stage_id == "bam.qc_pre"
                && row.tool_id == "samtools"
                && row.expected_failure_class == BamRawParserFailureClass::MalformedRawOutput
                && row.observed_error.contains("flagstat summary missing `in total` line")
        }));
        assert!(rows.iter().any(|row| {
            row.stage_id == "bam.coverage"
                && row.tool_id == "mosdepth"
                && row.expected_failure_class == BamRawParserFailureClass::EmptyRawOutput
                && row
                    .observed_error
                    .contains("mosdepth summary missing total/genome/all coverage row")
        }));
        Ok(())
    }
}
