use bijux_dna_core::ids::{StageId, ToolId};

#[test]
fn stage_tool_maturity_distinguishes_planned_generic_and_observer_bindings() {
    let infer_stage = StageId::from_static("fastq.infer_asvs");
    assert_eq!(
        bijux_dna_planner_fastq::stage_api::stage_tool_maturity(
            &infer_stage,
            &ToolId::from_static("dada2"),
        ),
        Some(bijux_dna_planner_fastq::stage_api::StageToolMaturityLevel::ObserverNormalized)
    );

    let detect_stage = StageId::from_static("fastq.detect_adapters");
    assert_eq!(
        bijux_dna_planner_fastq::stage_api::stage_tool_maturity(
            &detect_stage,
            &ToolId::from_static("fastqc"),
        ),
        Some(bijux_dna_planner_fastq::stage_api::StageToolMaturityLevel::BenchmarkComparable)
    );

    let trim_stage = StageId::from_static("fastq.trim_reads");
    assert_eq!(
        bijux_dna_planner_fastq::stage_api::stage_tool_maturity(
            &trim_stage,
            &ToolId::from_static("fastp"),
        ),
        Some(bijux_dna_planner_fastq::stage_api::StageToolMaturityLevel::ObserverNormalized)
    );
}

#[test]
fn current_fastq_benchmark_bindings_do_not_overclaim_full_comparability() {
    let comparable = bijux_dna_planner_fastq::stage_api::benchmark_profiles_for_stage(
        &StageId::from_static("fastq.trim_reads"),
    )
    .into_iter()
    .chain(bijux_dna_planner_fastq::stage_api::benchmark_profiles_for_stage(&StageId::from_static(
        "fastq.screen_taxonomy",
    )))
    .filter_map(|profile| {
        bijux_dna_planner_fastq::stage_api::stage_tool_maturity(&profile.stage_id, &profile.tool_id)
    })
    .filter(|level| {
        *level == bijux_dna_planner_fastq::stage_api::StageToolMaturityLevel::BenchmarkComparable
    })
    .count();
    assert_eq!(
        comparable, 0,
        "benchmark cohorts must not be advertised as fully comparable until observer-specialized normalization exists"
    );
}
