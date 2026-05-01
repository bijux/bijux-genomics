use std::path::Path;

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

use crate::artifacts::{
    BuildTaxonomyDbReportV1, BuildTaxonomyDbSourceEntryV1, BUILD_TAXONOMY_DB_REPORT_SCHEMA_VERSION,
};

/// Build taxonomy database identity from source files.
///
/// # Errors
/// Returns an error when sources are missing or cannot be parsed.
pub fn build_taxonomy_db(
    taxonomy_sources: &[&Path],
    database_family: Option<&str>,
) -> Result<BuildTaxonomyDbReportV1> {
    if taxonomy_sources.is_empty() {
        return Err(anyhow!("fastq.build_taxonomy_db requires at least one taxonomy source"));
    }

    let mut sources = Vec::with_capacity(taxonomy_sources.len());
    for source in taxonomy_sources {
        if !source.exists() {
            return Err(anyhow!("taxonomy source missing: {}", source.display()));
        }
        let raw = std::fs::read_to_string(source)?;
        let digest = bijux_dna_infra::hash_file_sha256(source)
            .map_err(|err| anyhow!("hash {}: {err}", source.display()))?;
        let record_count = raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .count() as u64;
        sources.push(BuildTaxonomyDbSourceEntryV1 {
            path: source.display().to_string(),
            sha256: digest,
            record_count,
        });
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));

    let source_record_count = sources.iter().map(|entry| entry.record_count).sum::<u64>();
    let mut hasher = Sha256::new();
    for entry in &sources {
        hasher.update(entry.path.as_bytes());
        hasher.update(b"=");
        hasher.update(entry.sha256.as_bytes());
        hasher.update(b"\n");
    }
    let database_hash =
        hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();

    Ok(BuildTaxonomyDbReportV1 {
        schema_version: BUILD_TAXONOMY_DB_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.build_taxonomy_db".to_string(),
        stage_id: "fastq.build_taxonomy_db".to_string(),
        tool_id: "bijux".to_string(),
        database_family: database_family.unwrap_or("kraken2").to_string(),
        source_record_count,
        database_hash,
        sources,
    })
}

#[cfg(test)]
mod tests {
    use super::build_taxonomy_db;

    #[test]
    fn build_taxonomy_db_counts_records() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-taxonomy-db")?;
        let source = temp.path().join("taxonomy.tsv");
        std::fs::write(&source, "#comment\n9606\tHomo sapiens\n562\tEscherichia coli\n")?;

        let report = build_taxonomy_db(&[source.as_path()], Some("kraken2"))?;
        assert_eq!(report.source_record_count, 2);
        assert_eq!(report.sources.len(), 1);
        assert!(!report.database_hash.is_empty());
        Ok(())
    }
}
