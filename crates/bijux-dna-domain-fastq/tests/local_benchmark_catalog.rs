use std::collections::BTreeSet;

use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::execution_support::{
    benchmark_cohort_stage_ids, execution_support_for_stage, ExecutionStatus,
};
use bijux_dna_domain_fastq::{FASTQ_LOCAL_BENCH_STAGE_ID_CATALOG, FASTQ_STAGE_ID_CATALOG};

fn local_stage_ids() -> BTreeSet<String> {
    FASTQ_LOCAL_BENCH_STAGE_ID_CATALOG.iter().map(|stage_id| (*stage_id).to_string()).collect()
}

#[test]
fn local_fastq_benchmark_catalog_stays_unique_and_complete() {
    let local = local_stage_ids();
    assert_eq!(
        local.len(),
        FASTQ_LOCAL_BENCH_STAGE_ID_CATALOG.len(),
        "FASTQ local benchmark stage catalog must not contain duplicate stage IDs",
    );
    assert_eq!(
        local.len(),
        27,
        "FASTQ local benchmark stage catalog must cover the governed 27-stage local readiness slice",
    );
}

#[test]
fn local_fastq_benchmark_catalog_stays_inside_domain_stage_catalog() {
    let local = local_stage_ids();
    let full = FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|stage_id| (*stage_id).to_string())
        .collect::<BTreeSet<_>>();

    assert!(
        local.is_subset(&full),
        "FASTQ local benchmark stage catalog must stay inside the full FASTQ stage catalog",
    );
}

#[test]
fn local_fastq_benchmark_catalog_matches_benchmark_surface_plus_local_dry_runs() {
    let local = local_stage_ids();
    let benchmark_surface = benchmark_cohort_stage_ids()
        .into_iter()
        .map(|stage_id| stage_id.to_string())
        .collect::<BTreeSet<_>>();
    let expected = benchmark_surface
        .union(&BTreeSet::from([
            "fastq.detect_duplicates_premerge".to_string(),
            "fastq.estimate_library_complexity_prealign".to_string(),
        ]))
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        local, expected,
        "FASTQ local benchmark stage catalog must equal benchmark-relevant stages plus local advisory-readiness stages",
    );
}

#[test]
fn local_advisory_stage_statuses_remain_honest_in_execution_support() {
    let detect_duplicates =
        execution_support_for_stage(&StageId::from_static("fastq.detect_duplicates_premerge"))
            .unwrap_or_else(|| {
                panic!("fastq.detect_duplicates_premerge must stay in FASTQ execution support")
            });
    assert_eq!(
        detect_duplicates.execution_status,
        ExecutionStatus::Closed,
        "fastq.detect_duplicates_premerge must leave declared-only execution support once its governed runtime closes",
    );

    let estimate_complexity = execution_support_for_stage(&StageId::from_static(
        "fastq.estimate_library_complexity_prealign",
    ))
    .unwrap_or_else(|| {
        panic!("fastq.estimate_library_complexity_prealign must stay in FASTQ execution support")
    });
    assert_eq!(
        estimate_complexity.execution_status,
        ExecutionStatus::DeclaredOnly,
        "fastq.estimate_library_complexity_prealign must remain declared-only until runtime support closes",
    );
}
