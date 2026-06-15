use std::collections::BTreeSet;

use bijux_dna_domain_fastq::execution_support::{
    benchmark_cohort_stage_ids, comparable_benchmark_stage_ids, plannable_stage_ids,
    runnable_stage_ids, BenchmarkSupport, NormalizationSupport, PlanningSupport, RuntimeSupport,
};
use bijux_dna_domain_fastq::{
    all_stage_execution_support, execution_closed_stage_ids, execution_declared_only_stage_ids,
    FASTQ_STAGE_ID_CATALOG,
};

fn stage_support(stage_id: &str) -> bijux_dna_domain_fastq::StageExecutionSupport {
    all_stage_execution_support()
        .into_iter()
        .find(|support| support.stage_id.as_str() == stage_id)
        .unwrap_or_else(|| panic!("{stage_id} must stay in the execution support manifest"))
}

#[test]
fn execution_support_manifest_covers_every_fastq_stage() {
    let domain_stage_ids =
        FASTQ_STAGE_ID_CATALOG.iter().map(|stage| (*stage).to_string()).collect::<BTreeSet<_>>();
    let execution_stage_ids = all_stage_execution_support()
        .into_iter()
        .map(|stage| stage.stage_id.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        execution_stage_ids, domain_stage_ids,
        "execution support drifted from FASTQ stage catalog",
    );
}

#[test]
fn closed_stages_have_defaults_and_declared_only_stages_do_not() {
    for support in all_stage_execution_support() {
        match support.execution_status {
            bijux_dna_domain_fastq::ExecutionStatus::Closed => {
                assert!(
                    support.default_tool.is_some(),
                    "closed stage {} must declare a default tool",
                    support.stage_id.as_str(),
                );
                assert!(
                    !support.admitted_tools.is_empty(),
                    "closed stage {} must admit at least one tool",
                    support.stage_id.as_str(),
                );
            }
            bijux_dna_domain_fastq::ExecutionStatus::DeclaredOnly => {
                assert!(
                    support.default_tool.is_none(),
                    "declared-only stage {} must not expose a default tool",
                    support.stage_id.as_str(),
                );
                assert!(
                    support.admitted_tools.is_empty(),
                    "declared-only stage {} must not admit execution tools",
                    support.stage_id.as_str(),
                );
            }
        }
    }
}

#[test]
fn execution_support_separates_closed_and_declared_only_stage_sets() {
    let closed = execution_closed_stage_ids()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();
    let declared_only = execution_declared_only_stage_ids()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();

    assert!(
        closed.contains("fastq.infer_asvs"),
        "governed FASTQ ASV inference must appear in the closed execution set once its runtime contract closes",
    );
    assert!(
        closed.contains("fastq.detect_duplicates_premerge"),
        "fixture-backed FASTQ duplicate signaling must appear in the closed execution set once its governed runtime closes",
    );
    assert!(
        !declared_only.contains("fastq.infer_asvs"),
        "closed ASV inference must leave the declared-only execution set",
    );
    assert!(
        !declared_only.contains("fastq.detect_duplicates_premerge"),
        "closed duplicate signaling must leave the declared-only execution set",
    );
}

#[test]
fn execution_support_tracks_capability_axes_without_optimistic_defaults() {
    for support in all_stage_execution_support() {
        match support.planning_support {
            PlanningSupport::DeclaredOnly => {
                assert!(
                    !support.is_plannable(),
                    "declared-only stage {} must not report planner readiness",
                    support.stage_id.as_str(),
                );
            }
            PlanningSupport::StageFamily => {
                assert!(
                    support.is_plannable(),
                    "governed stage family {} must report planner readiness",
                    support.stage_id.as_str(),
                );
            }
        }

        match support.runtime_support {
            RuntimeSupport::DeclaredOnly => {
                assert!(
                    !support.is_runnable(),
                    "declared-only stage {} must not report runtime readiness",
                    support.stage_id.as_str(),
                );
            }
            RuntimeSupport::Runnable => {
                assert!(
                    support.is_runnable(),
                    "runnable stage {} must report runtime readiness",
                    support.stage_id.as_str(),
                );
            }
        }
    }
}

#[test]
fn execution_support_distinguishes_generic_mixed_and_comparable_normalization() {
    let mut by_stage = all_stage_execution_support()
        .into_iter()
        .map(|support| (support.stage_id.to_string(), support))
        .collect::<std::collections::BTreeMap<_, _>>();

    assert_eq!(
        by_stage.remove("fastq.detect_adapters").map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
    assert_eq!(
        by_stage.remove("fastq.deplete_host").map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
    assert_eq!(
        by_stage.remove("fastq.deplete_rrna").map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
    assert_eq!(
        by_stage
            .remove("fastq.deplete_reference_contaminants")
            .map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
    assert_eq!(
        by_stage.remove("fastq.validate_reads").map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
    assert_eq!(
        by_stage.remove("fastq.trim_reads").map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
    assert_eq!(
        by_stage.remove("fastq.screen_taxonomy").map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
    assert_eq!(
        by_stage.remove("fastq.remove_duplicates").map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
    assert_eq!(
        by_stage.remove("fastq.infer_asvs").map(|support| support.normalization_support),
        Some(NormalizationSupport::ObserverSpecialized),
    );
}

#[test]
fn execution_support_reports_benchmark_stage_sets_from_manifest_truth() {
    let cohort = benchmark_cohort_stage_ids()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();
    let comparable = comparable_benchmark_stage_ids()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();
    let plannable =
        plannable_stage_ids().into_iter().map(|stage| stage.to_string()).collect::<BTreeSet<_>>();
    let runnable =
        runnable_stage_ids().into_iter().map(|stage| stage.to_string()).collect::<BTreeSet<_>>();

    assert!(plannable.contains("fastq.trim_reads"));
    assert!(runnable.contains("fastq.trim_reads"));
    assert!(cohort.contains("fastq.trim_reads"));
    assert!(cohort.contains("fastq.trim_polyg_tails"));
    assert!(cohort.contains("fastq.validate_reads"));
    assert!(cohort.contains("fastq.deplete_rrna"));
    assert!(cohort.contains("fastq.deplete_host"));
    assert!(cohort.contains("fastq.deplete_reference_contaminants"));
    assert!(cohort.contains("fastq.extract_umis"));
    assert!(cohort.contains("fastq.profile_reads"));
    assert!(comparable.contains("fastq.validate_reads"));
    assert!(comparable.contains("fastq.detect_adapters"));
    assert!(comparable.contains("fastq.report_qc"));
    assert!(!comparable.contains("fastq.trim_reads"));
    assert!(plannable.contains("fastq.infer_asvs"));
    assert!(runnable.contains("fastq.infer_asvs"));

    let support = stage_support("fastq.profile_reads");
    assert_eq!(support.benchmark_support, BenchmarkSupport::Cohort);
}
