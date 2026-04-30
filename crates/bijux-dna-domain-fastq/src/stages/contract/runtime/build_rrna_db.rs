use std::path::Path;

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

use crate::artifacts::{
    BuildRrnaDbReportV1, BuildRrnaDbSourceEntryV1, BUILD_RRNA_DB_REPORT_SCHEMA_VERSION,
};

/// Build rRNA database identity from source files.
///
/// # Errors
/// Returns an error when sources are missing or cannot be parsed.
pub fn build_rrna_db(rrna_sources: &[&Path], database_family: Option<&str>) -> Result<BuildRrnaDbReportV1> {
    if rrna_sources.is_empty() {
        return Err(anyhow!("fastq.build_rrna_db requires at least one rRNA source"));
    }

    let mut sources = Vec::with_capacity(rrna_sources.len());
    for source in rrna_sources {
        if !source.exists() {
            return Err(anyhow!("rRNA source missing: {}", source.display()));
        }
        let raw = std::fs::read_to_string(source)?;
        let digest = bijux_dna_infra::hash_file_sha256(source)
            .map_err(|err| anyhow!("hash {}: {err}", source.display()))?;
        let sequence_count = raw
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with('>'))
            .count() as u64;
        sources.push(BuildRrnaDbSourceEntryV1 {
            path: source.display().to_string(),
            sha256: digest,
            sequence_count,
        });
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));

    let source_sequence_count = sources.iter().map(|entry| entry.sequence_count).sum::<u64>();
    let mut hasher = Sha256::new();
    for entry in &sources {
        hasher.update(entry.path.as_bytes());
        hasher.update(b"=");
        hasher.update(entry.sha256.as_bytes());
        hasher.update(b"\n");
    }
    let database_hash = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    Ok(BuildRrnaDbReportV1 {
        schema_version: BUILD_RRNA_DB_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.build_rrna_db".to_string(),
        stage_id: "fastq.build_rrna_db".to_string(),
        tool_id: "bijux".to_string(),
        database_family: database_family.unwrap_or("sortmerna").to_string(),
        source_sequence_count,
        database_hash,
        sources,
    })
}

#[cfg(test)]
mod tests {
    use super::build_rrna_db;

    #[test]
    fn build_rrna_db_counts_sequences() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-rrna-db")?;
        let source = temp.path().join("rrna.fa");
        std::fs::write(&source, ">r1\nACGT\n>r2\nTGCA\n")?;

        let report = build_rrna_db(&[source.as_path()], Some("sortmerna"))?;
        assert_eq!(report.source_sequence_count, 2);
        assert_eq!(report.sources.len(), 1);
        assert!(!report.database_hash.is_empty());
        Ok(())
    }
}
