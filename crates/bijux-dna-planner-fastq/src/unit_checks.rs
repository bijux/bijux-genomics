use super::*;

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
