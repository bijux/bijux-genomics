use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};

use super::domain_workflow::{
    domain_directories, failure_block, list_block, read_utf8, regex, scalar_from_text, success_line,
};
use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;

fn parse_stage_catalog(path: &Path, const_name: &str) -> Result<BTreeSet<String>> {
    let text = read_utf8(path)?;
    let pattern = format!(
        r"(?s)pub\s+const\s+{}:\s*&\[\s*&str\s*\]\s*=\s*&\[(.*?)\];",
        regex::escape(const_name)
    );
    let captures = regex(&pattern)?
        .captures(&text)
        .ok_or_else(|| anyhow!("missing {const_name} in {}", path.display()))?;
    let body = captures
        .get(1)
        .map(|value| value.as_str())
        .ok_or_else(|| anyhow!("missing catalog body for {const_name}"))?;
    let item_re = regex(r#""([a-z0-9_.]+)""#)?;
    Ok(item_re
        .captures_iter(body)
        .filter_map(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
        .collect())
}

pub(super) fn check_rust_stage_catalog_parity(
    workspace: &Workspace,
) -> Result<DomainCommandOutcome> {
    let specs = [
        (
            "fastq",
            workspace.path("crates/bijux-dna-domain-fastq/src/id_catalog.rs"),
            "FASTQ_STAGE_ID_CATALOG",
        ),
        (
            "bam",
            workspace.path("crates/bijux-dna-domain-bam/src/types/mod.rs"),
            "BAM_STAGE_ID_CATALOG",
        ),
        (
            "vcf",
            workspace.path("crates/bijux-dna-domain-vcf/src/lib.rs"),
            "VCF_STAGE_ID_CATALOG",
        ),
    ];

    let mut errors = Vec::new();
    for (domain, path, const_name) in specs {
        let domain_ids = list_block(
            &read_utf8(&workspace.path(&format!("domain/{domain}/index.yaml")))?,
            "stage_ids",
        )?
        .into_iter()
        .collect::<BTreeSet<_>>();
        let rust_ids = parse_stage_catalog(&path, const_name)?;
        for missing in domain_ids.difference(&rust_ids) {
            errors.push(format!(
                "{}: {const_name} missing domain stage '{}'",
                workspace.rel(&path).display(),
                missing
            ));
        }
        for extra in rust_ids.difference(&domain_ids) {
            errors.push(format!(
                "{}: {const_name} has stale non-domain stage '{}'",
                workspace.rel(&path).display(),
                extra
            ));
        }
    }
    if errors.is_empty() {
        return success_line("rust stage catalog parity: OK");
    }
    failure_block("rust stage catalog parity check failed", errors)
}

pub(super) fn check_ssot_authority(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let doc = workspace.path("docs/10-architecture/SSOT.md");
    let doc_text = read_utf8(&doc)?;
    if !doc_text.contains("domain/*/**/*.yaml") || !doc_text.contains("source of truth") {
        return Ok(DomainCommandOutcome::failure(
            "ssot authority check: docs/10-architecture/SSOT.md must declare domain/*/**/*.yaml as source of truth\n",
        ));
    }

    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        let index_path = dom_dir.join("index.yaml");
        if !index_path.is_file() {
            continue;
        }
        let text = read_utf8(&index_path)?;
        let Some(version) = scalar_from_text(&text, "domain_version")? else {
            errors.push(format!(
                "{} missing domain_version: v1|v2",
                workspace.rel(&index_path).display()
            ));
            continue;
        };
        if !matches!(version.as_str(), "v1" | "v2") {
            errors.push(format!(
                "{} has invalid domain_version '{}' (expected v1|v2)",
                workspace.rel(&index_path).display(),
                version
            ));
        }
        if dom_dir.file_name().and_then(|name| name.to_str()) == Some("vcf") && version != "v2" {
            errors.push("domain/vcf/index.yaml must declare domain_version: v2".to_string());
        }
    }
    if errors.is_empty() {
        return success_line("ssot authority/version: OK");
    }
    failure_block("ssot authority check failed", errors)
}
