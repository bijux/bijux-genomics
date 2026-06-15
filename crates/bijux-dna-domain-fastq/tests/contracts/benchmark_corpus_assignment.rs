use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::{
    benchmark_corpus_assignment_for_stage_tool, stage_tool_governance_profiles_for_stage,
    BenchmarkCorpusFamily, STAGES,
};

#[test]
fn benchmark_corpus_assignment_covers_every_governed_fastq_stage_tool_binding() {
    for stage in STAGES {
        let stage_id = StageId::new(stage.as_str().to_string());
        let tool_ids = stage_tool_governance_profiles_for_stage(&stage_id)
            .into_iter()
            .map(|profile| profile.tool_id)
            .collect::<Vec<_>>();
        assert!(
            !tool_ids.is_empty(),
            "FASTQ benchmark corpus routing expects at least one governed tool for `{}`",
            stage.as_str()
        );
        for tool_id in tool_ids {
            let assignment = benchmark_corpus_assignment_for_stage_tool(&stage_id, &tool_id);
            assert!(
                assignment.is_some(),
                "missing FASTQ benchmark corpus assignment for `{}` / `{}`",
                stage_id.as_str(),
                tool_id.as_str()
            );
        }
    }
}

#[test]
fn benchmark_corpus_assignment_routes_general_taxonomy_and_amplicon_stages() {
    assert_eq!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("fastq.validate_reads".to_string()),
            &ToolId::new("fastqc".to_string()),
        )
        .and_then(|assignment| assignment.assigned_family()),
        Some(BenchmarkCorpusFamily::Corpus01)
    );
    assert_eq!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("fastq.screen_taxonomy".to_string()),
            &ToolId::new("kraken2".to_string()),
        )
        .and_then(|assignment| assignment.assigned_family()),
        Some(BenchmarkCorpusFamily::Corpus02)
    );
    assert_eq!(
        benchmark_corpus_assignment_for_stage_tool(
            &StageId::new("fastq.normalize_primers".to_string()),
            &ToolId::new("cutadapt".to_string()),
        )
        .and_then(|assignment| assignment.assigned_family()),
        Some(BenchmarkCorpusFamily::Corpus03)
    );
}

#[test]
fn benchmark_corpus_assignment_routes_every_taxonomy_classifier_to_corpus_02() {
    let stage_id = StageId::new("fastq.screen_taxonomy".to_string());
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert_eq!(
            benchmark_corpus_assignment_for_stage_tool(
                &stage_id,
                &ToolId::new(tool_id.to_string()),
            )
            .and_then(|assignment| assignment.assigned_family()),
            Some(BenchmarkCorpusFamily::Corpus02),
            "taxonomy classifier `{tool_id}` must stay on the governed eDNA corpus family"
        );
    }
}

#[test]
fn benchmark_corpus_assignment_routes_every_amplicon_binding_to_corpus_03() {
    for (stage_id, tool_id) in [
        ("fastq.normalize_primers", "cutadapt"),
        ("fastq.remove_chimeras", "vsearch"),
        ("fastq.infer_asvs", "dada2"),
        ("fastq.cluster_otus", "vsearch"),
        ("fastq.normalize_abundance", "seqkit"),
    ] {
        assert_eq!(
            benchmark_corpus_assignment_for_stage_tool(
                &StageId::new(stage_id.to_string()),
                &ToolId::new(tool_id.to_string()),
            )
            .and_then(|assignment| assignment.assigned_family()),
            Some(BenchmarkCorpusFamily::Corpus03),
            "amplicon binding `{stage_id}` / `{tool_id}` must stay on the governed amplicon corpus family"
        );
    }
}

#[test]
fn benchmark_corpus_assignment_preserves_precise_exclusion_reasons() {
    let index_reference = benchmark_corpus_assignment_for_stage_tool(
        &StageId::new("fastq.index_reference".to_string()),
        &ToolId::new("bowtie2_build".to_string()),
    )
    .unwrap_or_else(|| panic!("index-reference assignment"));
    assert_eq!(index_reference.benchmark_scope_id(), Some("reference-index-assets"));

    let overrepresented = benchmark_corpus_assignment_for_stage_tool(
        &StageId::new("fastq.profile_overrepresented_sequences".to_string()),
        &ToolId::new("fastqc".to_string()),
    )
    .unwrap_or_else(|| panic!("overrepresented assignment"));
    assert_eq!(overrepresented.assigned_family(), Some(BenchmarkCorpusFamily::Corpus01));

    let report_qc = benchmark_corpus_assignment_for_stage_tool(
        &StageId::new("fastq.report_qc".to_string()),
        &ToolId::new("multiqc".to_string()),
    )
    .unwrap_or_else(|| panic!("report-qc assignment"));
    assert_eq!(report_qc.exclusion_reason_code(), Some("governed_multiqc_bundle_fixture_missing"));
}
