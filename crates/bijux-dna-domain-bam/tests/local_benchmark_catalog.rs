use std::collections::BTreeSet;

use bijux_dna_domain_bam::{BAM_LOCAL_BENCH_STAGE_ID_CATALOG, BAM_STAGE_ID_CATALOG};

fn local_stage_ids() -> BTreeSet<String> {
    BAM_LOCAL_BENCH_STAGE_ID_CATALOG.iter().map(|stage_id| (*stage_id).to_string()).collect()
}

#[test]
fn local_bam_benchmark_catalog_stays_unique_and_complete() {
    let local = local_stage_ids();
    assert_eq!(
        local.len(),
        BAM_LOCAL_BENCH_STAGE_ID_CATALOG.len(),
        "BAM local benchmark stage catalog must not contain duplicate stage IDs",
    );
    assert_eq!(
        local.len(),
        24,
        "BAM local benchmark stage catalog must cover the governed 24-stage local readiness slice",
    );
}

#[test]
fn local_bam_benchmark_catalog_matches_domain_stage_catalog() {
    let local = local_stage_ids();
    let full = BAM_STAGE_ID_CATALOG
        .iter()
        .map(|stage_id| (*stage_id).to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        local, full,
        "BAM local benchmark stage catalog must equal the full BAM stage catalog",
    );
}
