#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use std::fs;

fn markdown_link_targets(path: &str) -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut targets = BTreeSet::new();
    for line in raw.lines() {
        let mut rest = line;
        while let Some((_, suffix)) = rest.split_once("](") {
            if let Some((target, tail)) = suffix.split_once(')') {
                targets.insert(target.to_string());
                rest = tail;
            } else {
                break;
            }
        }
    }
    targets
}

fn publication_index_targets() -> BTreeSet<String> {
    let root = support::workspace_root();
    fs::read_dir(root.join("assets/publications"))
        .unwrap_or_else(|err| panic!("read assets/publications: {err}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|path| {
            format!(
                "../../assets/publications/{}/index.md",
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_else(|| panic!("invalid UTF-8 publication path {}", path.display()))
            )
        })
        .collect()
}

#[test]
fn policy__contracts__root_science_docs_policy__science_index_links_root_docs_exactly() {
    let expected = BTreeSet::from([
        "SCIENTIFIC_DECISIONS.md".to_string(),
        "SCIENTIFIC_DEFAULTS.md".to_string(),
        "PUBLICATION_ASSETS.md".to_string(),
        "TOOL_INDEX.md".to_string(),
        "DOMAIN_COVERAGE.generated.md".to_string(),
        "TOOL_STAGE_CITATIONS.md".to_string(),
        "VALIDITY_LIMITS.md".to_string(),
        "fastq/index.md".to_string(),
        "bam/index.md".to_string(),
        "vcf/index.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/20-science/index.md");
    assert_eq!(
        expected, documented,
        "The root science index must link the governed root docs and domain landing pages exactly"
    );
}

#[test]
fn policy__contracts__root_science_docs_policy__scientific_decisions_link_governed_surfaces() {
    let expected = BTreeSet::from([
        "../../assets/publications/adna-methods-2024/index.md".to_string(),
        "PUBLICATION_ASSETS.md".to_string(),
        "fastq/SCIENTIFIC_SPEC.md".to_string(),
        "bam/METHODOLOGICAL_INTENT.md".to_string(),
        "vcf/index.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/20-science/SCIENTIFIC_DECISIONS.md");
    assert_eq!(
        expected, documented,
        "Scientific decisions must link the governed publication asset surface and domain science authorities exactly"
    );
}

#[test]
fn policy__contracts__root_science_docs_policy__scientific_defaults_link_governed_references() {
    let expected = BTreeSet::from([
        "fastq/GOLD_PIPELINE_SPEC.md".to_string(),
        "fastq/TOOLS_ROSTER.md".to_string(),
        "fastq/STAGE_ASSUMPTIONS.md".to_string(),
        "bam/STAGE_ASSUMPTIONS.md".to_string(),
        "VALIDITY_LIMITS.md".to_string(),
        "vcf/STAGE_CATALOG.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/20-science/SCIENTIFIC_DEFAULTS.md");
    assert_eq!(
        expected, documented,
        "Scientific defaults must link the governed reference surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_science_docs_policy__validity_limits_link_domain_authorities() {
    let expected = BTreeSet::from([
        "SCIENTIFIC_DEFAULTS.md".to_string(),
        "fastq/SCIENTIFIC_SPEC.md".to_string(),
        "bam/METHODOLOGICAL_INTENT.md".to_string(),
        "vcf/POPULATION_STRUCTURE.md".to_string(),
        "vcf/ROH.md".to_string(),
        "vcf/IBD.md".to_string(),
        "vcf/DEMOGRAPHY.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/20-science/VALIDITY_LIMITS.md");
    assert_eq!(
        expected, documented,
        "Scientific validity limits must link the governed domain authority docs exactly"
    );
}

#[test]
fn policy__contracts__root_science_docs_policy__tool_stage_citations_link_governed_ledgers() {
    let expected = BTreeSet::from([
        "fastq/REFERENCES.md".to_string(),
        "bam/REFERENCES.md".to_string(),
        "vcf/REFERENCES.md".to_string(),
        "../../science/docs/upstream/papers/TOOL_PAPER_MAP.tsv".to_string(),
        "../../science/docs/upstream/github-repos/MANIFEST.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("docs/20-science/TOOL_STAGE_CITATIONS.md");
    assert_eq!(
        expected, documented,
        "Tool-stage citations must link the governed domain ledgers and upstream evidence maps exactly"
    );
}

#[test]
fn policy__contracts__root_science_docs_policy__publication_assets_list_publication_indexes_exactly(
) {
    let expected = publication_index_targets();
    let documented = markdown_link_targets("docs/20-science/PUBLICATION_ASSETS.md")
        .into_iter()
        .filter(|target| target.starts_with("../../assets/publications/"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "Publication assets must list the governed publication indexes exactly"
    );
}
