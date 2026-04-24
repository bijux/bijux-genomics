use bijux_dna_core::ids::StageId;

#[test]
fn newly_benchmarked_fastq_stage_families_publish_scenarios() {
    for stage_id in [
        "fastq.detect_adapters",
        "fastq.deplete_rrna",
        "fastq.deplete_host",
        "fastq.deplete_reference_contaminants",
        "fastq.extract_umis",
        "fastq.trim_polyg_tails",
        "fastq.report_qc",
    ] {
        let stage = StageId::new(stage_id.to_string());
        let scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&stage);
        assert!(!scenarios.is_empty(), "{stage_id} must publish at least one benchmark scenario");
    }
}
