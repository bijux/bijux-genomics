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
fn select_trim_tools_blocks_experimental_by_default() {
    let tools = vec!["seqpurge".to_string()];
    match select_trim_tools(&tools, false) {
        Ok(_) => panic!("expected failure"),
        Err(err) => assert!(err.to_string().contains("unsupported tool")),
    }
}

#[test]
fn select_trim_tools_allows_experimental_when_enabled() {
    let tools = vec!["seqpurge".to_string()];
    match select_trim_tools(&tools, true) {
        Ok(normalized) => assert_eq!(normalized, vec!["seqpurge".to_string()]),
        Err(err) => panic!("normalize failed: {err}"),
    }
}

#[test]
fn select_tools_rejects_empty() {
    match select_validate_tools(&[]) {
        Ok(_) => panic!("expected empty failure"),
        Err(err) => assert!(err.to_string().contains("no tools specified")),
    }
}
