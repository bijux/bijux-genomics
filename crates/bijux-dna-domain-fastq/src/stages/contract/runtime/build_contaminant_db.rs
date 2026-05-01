use std::path::Path;

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

use crate::artifacts::{
    BuildContaminantDbReportV1, BuildContaminantDbSourceEntryV1,
    BUILD_CONTAMINANT_DB_REPORT_SCHEMA_VERSION,
};

/// Build contaminant database identity from source files.
///
/// # Errors
/// Returns an error when sources are missing or cannot be parsed.
pub fn build_contaminant_db(
    contaminant_sources: &[&Path],
    database_family: Option<&str>,
) -> Result<BuildContaminantDbReportV1> {
    if contaminant_sources.is_empty() {
        return Err(anyhow!("fastq.build_contaminant_db requires at least one contaminant source"));
    }

    let mut sources = Vec::with_capacity(contaminant_sources.len());
    for source in contaminant_sources {
        if !source.exists() {
            return Err(anyhow!("contaminant source missing: {}", source.display()));
        }
        let raw = std::fs::read_to_string(source)?;
        let digest = bijux_dna_infra::hash_file_sha256(source)
            .map_err(|err| anyhow!("hash {}: {err}", source.display()))?;
        let sequence_count =
            raw.lines().map(str::trim).filter(|line| line.starts_with('>')).count() as u64;
        sources.push(BuildContaminantDbSourceEntryV1 {
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
    let database_hash =
        hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();

    Ok(BuildContaminantDbReportV1 {
        schema_version: BUILD_CONTAMINANT_DB_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.build_contaminant_db".to_string(),
        stage_id: "fastq.build_contaminant_db".to_string(),
        tool_id: "bijux".to_string(),
        database_family: database_family.unwrap_or("bowtie2").to_string(),
        source_sequence_count,
        database_hash,
        sources,
    })
}

#[cfg(test)]
mod tests {
    use super::build_contaminant_db;

    #[test]
    fn build_contaminant_db_counts_sequences() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-contaminant-db")?;
        let source_a = temp.path().join("a.fa");
        let source_b = temp.path().join("b.fa");
        std::fs::write(&source_a, ">c1\nACGT\n>c2\nTGCA\n")?;
        std::fs::write(&source_b, ">c3\nAAAA\n")?;

        let report =
            build_contaminant_db(&[source_a.as_path(), source_b.as_path()], Some("bowtie2"))?;
        assert_eq!(report.source_sequence_count, 3);
        assert_eq!(report.sources.len(), 2);
        assert!(!report.database_hash.is_empty());
        Ok(())
    }
}
