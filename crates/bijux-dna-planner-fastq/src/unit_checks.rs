use super::*;
use bijux_dna_core::ids::ToolId;

#[test]
fn select_trim_tools_dedup_and_sort() {
    let tools = vec![
        "fastp".to_string(),
        "FASTP".to_string(),
        "bbduk".to_string(),
    ];
    match select_trim_tools(&tools, false) {
        Ok(normalized) => {
            assert_eq!(normalized, vec!["bbduk".to_string(), "fastp".to_string()]);
        }
        Err(err) => panic!("normalize failed: {err}"),
    }
}

#[test]
fn select_trim_tools_rejects_tools_outside_execution_support() {
    let tools = vec!["seqpurge".to_string()];
    match select_trim_tools(&tools, false) {
        Ok(_) => panic!("expected failure"),
        Err(err) => assert!(err.to_string().contains("unsupported tool")),
    }
}

#[test]
fn select_trim_tools_keeps_contract_even_when_opt_in_flag_is_set() {
    let tools = vec!["seqpurge".to_string()];
    match select_trim_tools(&tools, true) {
        Ok(_) => panic!("expected failure"),
        Err(err) => assert!(err.to_string().contains("unsupported tool")),
    }
}

#[test]
fn select_tools_rejects_empty() {
    match select_validate_tools(&[]) {
        Ok(_) => panic!("expected empty failure"),
        Err(err) => assert!(err.to_string().contains("no tools specified")),
    }
}

#[test]
fn stage_status_comes_from_domain_execution_support() {
    assert_eq!(stage_status("fastq.validate_reads").as_deref(), Some("supported"));
    assert_eq!(stage_status("fastq.infer_asvs").as_deref(), Some("planned"));
    assert!(stage_status("fastq.unknown_stage").is_none());
}

#[test]
fn benchmark_query_context_uses_stage_contract_hash_for_governed_stages() {
    let context = bench_query_context_for_stage(&StageId::from_static("fastq.trim_reads"))
        .expect("governed stage contract hash should be available");

    assert!(context.params_hash.is_none());
    assert!(context.image_digest.is_none());
    assert!(context.stage_contract_hash.is_some());
}

#[test]
fn benchmark_query_context_stays_empty_for_unknown_stages() {
    let context = bench_query_context_for_stage(&StageId::new("fastq.unknown".to_string()))
        .expect("unknown stages should not fail benchmark query context construction");

    assert!(context.is_empty());
}

#[test]
fn stage_tool_capability_no_longer_treats_planned_bindings_as_plannable() {
    let capability = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.infer_asvs"),
        &ToolId::from_static("dada2"),
    )
    .expect("declared FASTQ ASV binding must still surface a capability row");

    assert!(capability.declared);
    assert!(!capability.plannable);
    assert!(!capability.runnable);
    assert!(!capability.parse_normalized);
    assert!(!capability.benchmark_normalized);
    assert!(!capability.comparable);
}

#[test]
fn stage_tool_capability_uses_manifest_normalization_modes() {
    let detect_adapters = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.detect_adapters"),
        &ToolId::from_static("fastqc"),
    )
    .expect("fastqc detect-adapters capability must exist");
    assert!(detect_adapters.parse_normalized);
    assert!(!detect_adapters.benchmark_normalized);

    let trim_reads = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.trim_reads"),
        &ToolId::from_static("fastp"),
    )
    .expect("fastp trim capability must exist");
    assert!(trim_reads.parse_normalized);
    assert!(trim_reads.benchmark_normalized);
    assert!(!trim_reads.comparable);
}

#[test]
fn mixed_normalization_stages_only_mark_observer_specialized_tools_comparable() {
    let fastqc = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.profile_overrepresented_sequences"),
        &ToolId::from_static("fastqc"),
    )
    .expect("fastqc overrepresented capability must exist");
    let seqkit = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.profile_overrepresented_sequences"),
        &ToolId::from_static("seqkit"),
    )
    .expect("seqkit overrepresented capability must exist");

    assert!(fastqc.parse_normalized);
    assert!(fastqc.benchmark_normalized);
    assert!(fastqc.comparable);

    assert!(seqkit.runnable);
    assert!(!seqkit.parse_normalized);
    assert!(!seqkit.benchmark_normalized);
    assert!(!seqkit.comparable);
}
