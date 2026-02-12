use std::collections::HashSet;
use std::path::Path;

fn extract_string_literals(contents: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut in_string = false;
    let mut current = String::new();
    let mut chars = contents.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_string {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                continue;
            }
            if ch == '"' {
                in_string = false;
                literals.push(current.clone());
                current.clear();
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_string = true;
        }
    }
    literals
}

fn is_stage_id(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    matches!(parts.as_slice(), [prefix, _] if *prefix == "fastq" || *prefix == "bam")
}

#[test]
fn benchmark_references_canonical_ids() {
    let mut allowed_stages = HashSet::new();
    for stage in bijux_dna_domain_fastq::stages::ids::STAGES {
        allowed_stages.insert(stage.as_str().to_string());
    }
    for stage in bijux_dna_domain_bam::BamStage::all() {
        allowed_stages.insert(stage.as_str().to_string());
    }

    let mut allowed_tools = HashSet::new();
    for stage in bijux_dna_domain_fastq::stages::ids::STAGES {
        if let Some(json) = bijux_dna_domain_fastq::stage_contract_json(stage.as_str()) {
            if let Some(tools) = json.get("tool_ids").and_then(|value| value.as_array()) {
                for tool in tools {
                    if let Some(tool) = tool.as_str() {
                        allowed_tools.insert(tool.to_string());
                    }
                }
            }
        }
    }
    for stage in bijux_dna_domain_bam::BamStage::all() {
        if let Some(json) = bijux_dna_domain_bam::stage_contract_json(stage.as_str()) {
            if let Some(tools) = json.get("tool_ids").and_then(|value| value.as_array()) {
                for tool in tools {
                    if let Some(tool) = tool.as_str() {
                        allowed_tools.insert(tool.to_string());
                    }
                }
            }
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let contents = std::fs::read_to_string(entry.path()).expect("read source");
        for literal in extract_string_literals(&contents) {
            if is_stage_id(&literal) && !allowed_stages.contains(&literal) {
                offenders.push(format!(
                    "{}: unowned stage id literal {}",
                    entry.path().display(),
                    literal
                ));
            }
        }
        for line in contents.lines() {
            let trimmed = line.trim();
            if !trimmed.contains("tool_id") {
                continue;
            }
            let Some(start) = trimmed.find('"') else {
                continue;
            };
            let Some(end) = trimmed[start + 1..].find('"') else {
                continue;
            };
            let literal = &trimmed[start + 1..start + 1 + end];
            if !allowed_tools.contains(literal) {
                offenders.push(format!(
                    "{}: unowned tool id literal {}",
                    entry.path().display(),
                    literal
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "benchmark must use canonical stage ids:\n{}",
        offenders.join("\n")
    );
}
