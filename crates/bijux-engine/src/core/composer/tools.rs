use anyhow::{anyhow, Result};

pub fn normalize_trim_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "fastp",
        "cutadapt",
        "bbduk",
        "adapterremoval",
        "trimmomatic",
        "trim_galore",
        "atropos",
        "seqpurge",
    ];
    let mut allowlist = allowed.to_vec();
    if std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_err() {
        allowlist.retain(|tool| *tool != "seqpurge");
    }
    normalize_tools_with_allowlist(tools, &allowlist)
}

pub fn normalize_validate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "seqtk",
        "fastqc",
        "fastqvalidator",
        "fastqvalidator_official",
        "fqtools",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["prinseq", "fastp", "seqkit"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["pear", "vsearch", "bbmerge", "flash2"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_correct_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["rcorrector", "spades", "bayeshammer", "lighter", "musket"];
    let mut allowlist = allowed.to_vec();
    if std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_err() {
        allowlist.retain(|tool| *tool == "rcorrector");
    }
    normalize_tools_with_allowlist(tools, &allowlist)
}

pub fn normalize_qc_post_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["fastqc", "multiqc"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["umi_tools"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "kraken2",
        "centrifuge",
        "metaphlan",
        "kaiju",
        "fastq_screen",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_stats_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["seqkit_stats"];
    normalize_tools_with_allowlist(tools, &allowed)
}

fn normalize_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn normalize_trim_tools_dedup_and_sort() {
        let tools = vec![
            "fastp".to_string(),
            "FASTP".to_string(),
            "cutadapt".to_string(),
        ];
        match normalize_trim_tool_list(&tools) {
            Ok(normalized) => {
                assert_eq!(
                    normalized,
                    vec!["cutadapt".to_string(), "fastp".to_string()]
                );
            }
            Err(err) => panic!("normalize failed: {err}"),
        }
    }

    #[test]
    fn normalize_trim_tools_blocks_experimental_by_default() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");
        let tools = vec!["seqpurge".to_string()];
        match normalize_trim_tool_list(&tools) {
            Ok(_) => panic!("expected failure"),
            Err(err) => assert!(err.to_string().contains("unsupported tool")),
        }
    }

    #[test]
    fn normalize_trim_tools_allows_experimental_when_enabled() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").ok();
        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
        let tools = vec!["seqpurge".to_string()];
        match normalize_trim_tool_list(&tools) {
            Ok(normalized) => assert_eq!(normalized, vec!["seqpurge".to_string()]),
            Err(err) => panic!("normalize failed: {err}"),
        }
        match prev {
            Some(value) => std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", value),
            None => std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS"),
        }
    }

    #[test]
    fn normalize_tools_rejects_empty() {
        match normalize_validate_tool_list(&[]) {
            Ok(_) => panic!("expected empty failure"),
            Err(err) => assert!(err.to_string().contains("no tools specified")),
        }
    }
}
