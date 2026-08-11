use std::collections::HashSet;

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
    matches!(parts.as_slice(), [prefix, suffix]
        if matches!(*prefix, "fastq" | "bam" | "vcf") && !suffix.is_empty())
}

fn is_identifier_literal(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn tool_literals_after_identifier(line: &str) -> Vec<String> {
    let Some(position) = line.find("tool_id") else {
        return Vec::new();
    };
    let suffix = &line[position + "tool_id".len()..];
    let trimmed = suffix.trim_start();
    let source = if trimmed.starts_with('"') {
        let Some((_, value)) = trimmed.split_once(':') else {
            return Vec::new();
        };
        value
    } else {
        suffix
    };
    extract_string_literals(source)
        .into_iter()
        .filter(|literal| is_identifier_literal(literal))
        .collect()
}

#[test]
fn benchmark_references_canonical_ids() {
    let crate_root = crate::support::crate_root("bijux-dna-bench")
        .unwrap_or_else(|err| panic!("resolve benchmark crate root: {err}"));
    let repository_root = crate_root
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("benchmark crate root is not under workspace crates"));
    let closure = bijux_dna_bench::derive_benchmark_stage_tool_closure(repository_root)
        .unwrap_or_else(|err| panic!("derive stage/tool closure: {err}"));
    let mut allowed_stages = bijux_dna_domain_fastq::FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|stage| (*stage).to_string())
        .collect::<HashSet<_>>();
    allowed_stages.extend(
        bijux_dna_domain_bam::BamStage::all().iter().map(|stage| stage.as_str().to_string()),
    );
    allowed_stages.extend(closure.rows.iter().map(|row| row.stage_id.clone()));

    let mut allowed_tools = HashSet::new();
    for stage in bijux_dna_domain_fastq::FASTQ_STAGE_ID_CATALOG {
        if let Some(json) = bijux_dna_domain_fastq::stage_contract_json(stage) {
            if let Some(tools) = json.get("tool_ids").and_then(|value| value.as_array()) {
                allowed_tools
                    .extend(tools.iter().filter_map(|tool| tool.as_str()).map(str::to_string));
            }
        }
    }
    for stage in bijux_dna_domain_bam::BamStage::all() {
        if let Some(json) = bijux_dna_domain_bam::stage_contract_json(stage.as_str()) {
            if let Some(tools) = json.get("tool_ids").and_then(|value| value.as_array()) {
                allowed_tools
                    .extend(tools.iter().filter_map(|tool| tool.as_str()).map(str::to_string));
            }
        }
    }
    allowed_tools.extend(closure.rows.iter().map(|row| row.tool_id.clone()));
    let allowed_families = bijux_dna_bench::BAM_STAGE_FAMILIES
        .iter()
        .chain(bijux_dna_bench::FASTQ_STAGE_FAMILIES)
        .chain(bijux_dna_bench::VCF_STAGE_FAMILIES)
        .map(|family| family.family_id)
        .collect::<HashSet<_>>();

    let root = crate::support::crate_root("bijux-dna-bench")
        .unwrap_or_else(|err| panic!("resolve benchmark crate root: {err}"))
        .join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let contents = bijux_dna_testkit::workspace_support::strip_rust_test_modules(
            &std::fs::read_to_string(entry.path()).expect("read source"),
        );
        for literal in extract_string_literals(&contents) {
            if is_stage_id(&literal)
                && !allowed_stages.contains(&literal)
                && !allowed_families.contains(literal.as_str())
            {
                offenders.push(format!(
                    "{}: unowned stage id literal {}",
                    entry.path().display(),
                    literal
                ));
            }
        }
        for line in contents.lines() {
            for literal in tool_literals_after_identifier(line) {
                if !allowed_tools.contains(&literal) {
                    offenders.push(format!(
                        "{}: unowned tool id literal {}",
                        entry.path().display(),
                        literal
                    ));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "benchmark must use canonical stage ids:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn stage_id_shape_excludes_domain_prefixes() {
    assert!(is_stage_id("fastq.validate_reads"));
    assert!(is_stage_id("bam.align"));
    assert!(is_stage_id("vcf.impute"));
    assert!(!is_stage_id("fastq."));
    assert!(!is_stage_id("bam."));
    assert!(!is_stage_id("vcf."));
}

#[test]
fn tool_scanner_reads_values_after_the_tool_identifier() {
    assert_eq!(
        tool_literals_after_identifier(
            "identity.run.domain != \"bam\" || identity.tool_id != \"picard\""
        ),
        ["picard"]
    );
    assert_eq!(
        tool_literals_after_identifier(
            "matches!(identity.tool_id.as_str(), \"bowtie2\" | \"bwa\")"
        ),
        ["bowtie2", "bwa"]
    );
    assert_eq!(
        tool_literals_after_identifier("(\"strategy_id\".to_string(), identity.tool_id.clone())"),
        Vec::<String>::new()
    );
    assert_eq!(tool_literals_after_identifier("{\"tool_id\": \"samtools\"}"), ["samtools"]);
}
