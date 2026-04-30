use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    contract::validate_suite, BenchmarkCorpusManifest, BenchmarkSuiteSpec, CorpusDomain,
    CorpusScale,
};
use bijux_dna_domain_fastq::execution_support::{
    benchmark_cohort_stage_ids, execution_support_for_stage,
};
use bijux_dna_domain_fastq::{
    admitted_execution_tools_for_stage, stage_parameter_ids, STAGE_TRIM_READS,
};

fn suite_dir() -> PathBuf {
    bijux_dna_bench::bench_suites_dir()
}

fn checked_in_suites() -> Result<Vec<(PathBuf, BenchmarkSuiteSpec)>> {
    let mut suites = Vec::new();
    for entry in fs::read_dir(suite_dir()).context("read suite dir")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let suite: BenchmarkSuiteSpec =
            toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
        suites.push((path, suite));
    }
    Ok(suites)
}

fn checked_in_corpora() -> Result<Vec<BenchmarkCorpusManifest>> {
    bijux_dna_bench::load_corpus_catalog()
}

#[test]
fn checked_in_suite_catalog_uses_governed_schema_and_stage_ids() -> Result<()> {
    for entry in fs::read_dir(suite_dir()).context("read suite dir")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        assert!(
            raw.contains("schema_version = \"bijux.bench.suite.v1\""),
            "{} must use the governed bench suite schema id",
            path.display()
        );
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| panic!("suite file name is not valid UTF-8: {}", path.display()))
            .starts_with("fastq_")
            || path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| panic!("suite file name is not valid UTF-8: {}", path.display()))
                .contains("fastq")
        {
            for legacy in ["validate_pre", "trim", "filter", "stats", "qc_post"] {
                assert!(
                    !raw.contains(&format!("stage = \"{legacy}\"")),
                    "{} must use canonical FASTQ stage ids instead of legacy alias {}",
                    path.display(),
                    legacy
                );
            }
            assert!(
                !raw.contains("tools = [\"multiqc\", \"samtools\"]"),
                "{} must not benchmark samtools under fastq.report_qc",
                path.display()
            );
        }
    }
    Ok(())
}

#[test]
fn checked_in_suite_catalog_deserializes_and_validates() -> Result<()> {
    for (path, suite) in checked_in_suites()? {
        validate_suite(&suite).with_context(|| format!("validate {}", path.display()))?;
    }
    Ok(())
}

#[test]
fn checked_in_fastq_suite_catalog_covers_governed_benchmark_stages() -> Result<()> {
    let covered = checked_in_suites()?
        .into_iter()
        .flat_map(|(_path, suite)| suite.stages.into_iter().map(|stage| stage.stage))
        .filter(|stage| stage.starts_with("fastq."))
        .collect::<std::collections::BTreeSet<_>>();
    let expected = benchmark_cohort_stage_ids()
        .into_iter()
        .filter(|stage_id| {
            execution_support_for_stage(stage_id)
                .map(|support| {
                    support.execution_status == bijux_dna_domain_fastq::ExecutionStatus::Closed
                })
                .unwrap_or(false)
        })
        .map(|stage_id| stage_id.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    for stage_id in expected {
        assert!(
            covered.contains(&stage_id),
            "checked-in bench suites must cover governed FASTQ benchmark stage {stage_id}"
        );
    }
    Ok(())
}

#[test]
fn checked_in_suite_catalog_exercises_structured_param_bindings() -> Result<()> {
    let has_param_bindings = checked_in_suites()?.into_iter().any(|(_path, suite)| {
        suite.stages.into_iter().any(|stage| !stage.param_bindings.is_empty())
    });
    assert!(has_param_bindings, "checked-in bench suites must exercise structured param_bindings");
    Ok(())
}

#[test]
fn checked_in_suite_catalog_exercises_stage_and_tool_param_bindings() -> Result<()> {
    let mut has_stage_scoped = false;
    let mut has_tool_scoped = false;
    for (_path, suite) in checked_in_suites()? {
        for stage in suite.stages {
            for binding in stage.param_bindings {
                if binding.tool.is_some() {
                    has_tool_scoped = true;
                }
                if binding.tool.is_none() && binding.stage_instance_id.is_some() {
                    has_stage_scoped = true;
                }
            }
        }
    }
    assert!(has_stage_scoped, "checked-in suites must exercise stage-scoped param_bindings");
    assert!(has_tool_scoped, "checked-in suites must exercise tool-scoped param_bindings");
    Ok(())
}

#[test]
fn checked_in_suites_only_bind_manifest_declared_stage_parameters() -> Result<()> {
    for (path, suite) in checked_in_suites()? {
        for stage in suite.stages {
            let has_stage_bindings =
                stage.param_bindings.iter().any(|binding| binding.tool.is_none());
            if !stage.stage.starts_with("fastq.") || !has_stage_bindings {
                continue;
            }
            let declared = stage_parameter_ids(&stage.stage).ok_or_else(|| {
                anyhow::anyhow!(
                    "{} references stage {} without a declared parameter registry entry",
                    path.display(),
                    stage.stage
                )
            })?;
            for binding in stage.param_bindings {
                if binding.tool.is_some() {
                    continue;
                }
                for key in binding.values.keys() {
                    assert!(
                        declared.contains(key.as_str()),
                        "{} binds undeclared stage parameter {} for {}",
                        path.display(),
                        key,
                        stage.stage
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn checked_in_fastq_suite_catalog_exercises_multi_tool_validation_cohorts() -> Result<()> {
    let has_multi_tool_validation_suite = checked_in_suites()?.into_iter().any(|(_path, suite)| {
        suite
            .stages
            .into_iter()
            .any(|stage| stage.stage == "fastq.validate_reads" && stage.tools.len() > 1)
    });
    assert!(
        has_multi_tool_validation_suite,
        "checked-in FASTQ suites must exercise a multi-tool validation cohort"
    );
    Ok(())
}

#[test]
fn checked_in_fastq_suite_catalog_uses_multiple_remove_duplicates_suites() -> Result<()> {
    let suite_count = checked_in_suites()?
        .into_iter()
        .filter(|(_path, suite)| {
            suite.stages.iter().any(|stage| stage.stage == "fastq.remove_duplicates")
        })
        .count();
    assert!(
        suite_count >= 2,
        "checked-in FASTQ suites must exercise fastq.remove_duplicates in more than one suite"
    );
    Ok(())
}

#[test]
fn checked_in_fastq_suite_catalog_covers_all_admitted_trim_backends() -> Result<()> {
    let covered = checked_in_suites()?
        .into_iter()
        .flat_map(|(_path, suite)| suite.stages.into_iter())
        .filter(|stage| stage.stage == STAGE_TRIM_READS.as_str())
        .flat_map(|stage| stage.tools.into_iter())
        .collect::<std::collections::BTreeSet<_>>();
    let expected = admitted_execution_tools_for_stage(&STAGE_TRIM_READS)
        .into_iter()
        .map(|tool_id| tool_id.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        covered, expected,
        "checked-in FASTQ suites must cover every admitted fastq.trim_reads backend"
    );
    Ok(())
}

#[test]
fn checked_in_fastq_suite_catalog_exercises_full_trim_branch_join() -> Result<()> {
    let expected = admitted_execution_tools_for_stage(&STAGE_TRIM_READS)
        .into_iter()
        .map(|tool_id| tool_id.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let has_full_join = checked_in_suites()?.into_iter().any(|(_path, suite)| {
        let mut trim_tools = std::collections::BTreeSet::new();
        let mut has_report_qc_join = false;
        for stage in suite.stages {
            if stage.stage == STAGE_TRIM_READS.as_str() {
                trim_tools.extend(stage.tools);
            }
            if stage.stage == "fastq.report_qc" {
                has_report_qc_join = true;
            }
        }
        has_report_qc_join && trim_tools == expected
    });
    assert!(
        has_full_join,
        "checked-in FASTQ suites must include a report_qc branch-join DAG that covers every admitted fastq.trim_reads backend"
    );
    Ok(())
}

#[test]
fn checked_in_corpus_catalog_contains_fastq_ci_small_case_matrix() -> Result<()> {
    let corpora = checked_in_corpora()?;
    let Some(fastq_ci_small) = corpora
        .iter()
        .find(|corpus| corpus.domain == CorpusDomain::Fastq && corpus.scale == CorpusScale::CiSmall)
    else {
        anyhow::bail!("checked-in corpus catalog must include a fastq ci-small manifest");
    };

    let tags = fastq_ci_small
        .datasets
        .iter()
        .flat_map(|dataset| dataset.case_tags.iter().map(String::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    for required in ["valid", "truncated", "adapter-heavy", "low-complexity", "umi", "contaminant", "sparse", "empty"] {
        assert!(
            tags.contains(required),
            "fastq ci-small corpus must cover required case tag {required}"
        );
    }
    Ok(())
}

#[test]
fn checked_in_corpus_catalog_contains_bam_ci_small_case_matrix() -> Result<()> {
    let corpora = checked_in_corpora()?;
    let Some(bam_ci_small) = corpora
        .iter()
        .find(|corpus| corpus.domain == CorpusDomain::Bam && corpus.scale == CorpusScale::CiSmall)
    else {
        anyhow::bail!("checked-in corpus catalog must include a bam ci-small manifest");
    };

    let tags = bam_ci_small
        .datasets
        .iter()
        .flat_map(|dataset| dataset.case_tags.iter().map(String::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        "valid",
        "missing-index",
        "duplicate-heavy",
        "low-coverage",
        "damage-like",
        "reference-mismatch",
    ] {
        assert!(
            tags.contains(required),
            "bam ci-small corpus must cover required case tag {required}"
        );
    }
    Ok(())
}

#[test]
fn checked_in_corpus_catalog_contains_vcf_ci_small_case_matrix() -> Result<()> {
    let corpora = checked_in_corpora()?;
    let Some(vcf_ci_small) = corpora
        .iter()
        .find(|corpus| corpus.domain == CorpusDomain::Vcf && corpus.scale == CorpusScale::CiSmall)
    else {
        anyhow::bail!("checked-in corpus catalog must include a vcf ci-small manifest");
    };

    let tags = vcf_ci_small
        .datasets
        .iter()
        .flat_map(|dataset| dataset.case_tags.iter().map(String::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        "valid",
        "malformed-header",
        "contig-alias",
        "multisample",
        "low-coverage-gl",
        "phased",
        "imputed",
        "panel-mismatch",
    ] {
        assert!(
            tags.contains(required),
            "vcf ci-small corpus must cover required case tag {required}"
        );
    }
    Ok(())
}

#[test]
fn checked_in_corpus_catalog_contains_fastq_local_medium_stress_matrix() -> Result<()> {
    let corpora = checked_in_corpora()?;
    let Some(fastq_local_medium) = corpora.iter().find(|corpus| {
        corpus.domain == CorpusDomain::Fastq && corpus.scale == CorpusScale::LocalMedium
    }) else {
        anyhow::bail!("checked-in corpus catalog must include a fastq local-medium manifest");
    };

    let tags = fastq_local_medium
        .datasets
        .iter()
        .flat_map(|dataset| dataset.case_tags.iter().map(String::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    for required in ["chunking", "edna", "depletion", "aggregate-qc"] {
        assert!(
            tags.contains(required),
            "fastq local-medium corpus must cover required stress tag {required}"
        );
    }
    Ok(())
}

#[test]
fn checked_in_corpus_catalog_contains_bam_local_medium_stress_matrix() -> Result<()> {
    let corpora = checked_in_corpora()?;
    let Some(bam_local_medium) = corpora
        .iter()
        .find(|corpus| corpus.domain == CorpusDomain::Bam && corpus.scale == CorpusScale::LocalMedium)
    else {
        anyhow::bail!("checked-in corpus catalog must include a bam local-medium manifest");
    };

    let tags = bam_local_medium
        .datasets
        .iter()
        .flat_map(|dataset| dataset.case_tags.iter().map(String::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    for required in ["sort", "index", "markdup", "coverage", "damage"] {
        assert!(
            tags.contains(required),
            "bam local-medium corpus must cover required stress tag {required}"
        );
    }
    Ok(())
}

#[test]
fn checked_in_corpus_catalog_contains_vcf_local_medium_stress_matrix() -> Result<()> {
    let corpora = checked_in_corpora()?;
    let Some(vcf_local_medium) = corpora
        .iter()
        .find(|corpus| corpus.domain == CorpusDomain::Vcf && corpus.scale == CorpusScale::LocalMedium)
    else {
        anyhow::bail!("checked-in corpus catalog must include a vcf local-medium manifest");
    };

    let tags = vcf_local_medium
        .datasets
        .iter()
        .flat_map(|dataset| dataset.case_tags.iter().map(String::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    for required in ["multisample-filtering", "normalization", "annotation", "evidence-size"] {
        assert!(
            tags.contains(required),
            "vcf local-medium corpus must cover required stress tag {required}"
        );
    }
    Ok(())
}

#[test]
fn checked_in_corpus_catalog_labels_truth_set_presence_and_absence() -> Result<()> {
    let corpora = checked_in_corpora()?;
    let mut has_available_truth = false;
    let mut has_unavailable_truth = false;

    for corpus in &corpora {
        for dataset in &corpus.datasets {
            if dataset.truth_set.note.trim().is_empty() {
                anyhow::bail!(
                    "dataset {} in corpus {} must document truth_set note",
                    dataset.dataset_id,
                    corpus.corpus_id
                );
            }
            match dataset.truth_set.status {
                bijux_dna_bench::TruthSetStatus::Available => {
                    has_available_truth = true;
                    assert!(
                        dataset.truth_set.truth_set_id.is_some(),
                        "available truth_set must include truth_set_id for {}",
                        dataset.dataset_id
                    );
                }
                bijux_dna_bench::TruthSetStatus::Unavailable => {
                    has_unavailable_truth = true;
                    assert!(
                        dataset
                            .truth_set
                            .note
                            .to_ascii_lowercase()
                            .contains("no truth set"),
                        "unavailable truth_set note must state absence for {}",
                        dataset.dataset_id
                    );
                }
            }
        }
    }

    assert!(has_available_truth, "corpus catalog must include at least one available truth set");
    assert!(has_unavailable_truth, "corpus catalog must include at least one unavailable truth set");
    Ok(())
}
