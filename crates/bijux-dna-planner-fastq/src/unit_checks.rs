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
