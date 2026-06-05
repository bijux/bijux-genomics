use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::{
    benchmark_corpus_assignment_for_stage_tool, governed_benchmark_stage_tool_bindings,
    BenchmarkCorpusFamily,
};

#[test]
fn benchmark_corpus_assignment_covers_every_governed_bam_stage_tool_binding() {
    for (stage_id, tool_id) in governed_benchmark_stage_tool_bindings() {
        let assignment = benchmark_corpus_assignment_for_stage_tool(
            &StageId::new((*stage_id).to_string()),
            &ToolId::new((*tool_id).to_string()),
        );
        assert!(
            assignment.is_some(),
            "missing BAM benchmark corpus assignment for `{stage_id}` / `{tool_id}`"
        );
    }
}

#[test]
fn benchmark_corpus_assignment_routes_general_adna_genotyping_and_kinship_rows() {
    assert_eq!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("bam.align".to_string()),
            &ToolId::new("bwa".to_string()),
        )
        .map(|assignment| assignment.assigned_family()),
        Some(BenchmarkCorpusFamily::Corpus01)
    );
    assert_eq!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("bam.qc_pre".to_string()),
            &ToolId::new("samtools".to_string()),
        )
        .map(|assignment| assignment.assigned_family()),
        Some(BenchmarkCorpusFamily::BamMini)
    );
    assert_eq!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("bam.authenticity".to_string()),
            &ToolId::new("authenticct".to_string()),
        )
        .map(|assignment| assignment.assigned_family()),
        Some(BenchmarkCorpusFamily::AdnaBam)
    );
    assert_eq!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("bam.genotyping".to_string()),
            &ToolId::new("angsd".to_string()),
        )
        .map(|assignment| assignment.assigned_family()),
        Some(BenchmarkCorpusFamily::Genotyping)
    );
    assert_eq!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("bam.kinship".to_string()),
            &ToolId::new("king".to_string()),
        )
        .map(|assignment| assignment.assigned_family()),
        Some(BenchmarkCorpusFamily::Kinship)
    );
}

#[test]
fn benchmark_corpus_assignment_rejects_ungoverned_tool_stage_pairs() {
    assert!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("bam.haplogroups".to_string()),
            &ToolId::new("samtools".to_string()),
        )
        .is_none()
    );
    assert!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("bam.authenticity".to_string()),
            &ToolId::new("pydamage".to_string()),
        )
        .is_none()
    );
}
