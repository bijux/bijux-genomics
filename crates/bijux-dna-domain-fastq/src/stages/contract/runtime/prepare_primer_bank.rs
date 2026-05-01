use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{PreparePrimerBankReportV1, PREPARE_PRIMER_BANK_REPORT_SCHEMA_VERSION};
use crate::banks::{amplicon_governance_path, load_primer_bank};

/// Prepare governed primer-bank metadata for amplicon workflows.
///
/// # Errors
/// Returns an error when the primer governance file cannot be resolved or validated.
pub fn prepare_primer_bank(
    governance_file: Option<&Path>,
    assay_family: Option<&str>,
) -> Result<PreparePrimerBankReportV1> {
    let governance_path = governance_file
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repository_root().join(amplicon_governance_path()));
    let bank = load_primer_bank(&governance_path)?;
    let governance_hash = bijux_dna_infra::hash_file_sha256(&governance_path)
        .map_err(|err| anyhow!("hash {}: {err}", governance_path.display()))?;

    let mut primer_set_ids = bank.primer_sets.iter().map(|set| set.id.clone()).collect::<Vec<_>>();
    primer_set_ids.sort();

    Ok(PreparePrimerBankReportV1 {
        schema_version: PREPARE_PRIMER_BANK_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.prepare_primer_bank".to_string(),
        stage_id: "fastq.prepare_primer_bank".to_string(),
        tool_id: "bijux".to_string(),
        bank_id: bank.bank_id,
        bank_version: bank.version,
        governance_hash,
        selection_logic: bank.selection_logic,
        primer_set_count: primer_set_ids.len() as u64,
        primer_set_ids,
        assay_family: assay_family.unwrap_or("marker_amplicon").to_string(),
    })
}

fn repository_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::prepare_primer_bank;

    #[test]
    fn prepare_primer_bank_loads_governed_bank() -> anyhow::Result<()> {
        let report = prepare_primer_bank(None, None)?;
        assert_eq!(report.stage_id, "fastq.prepare_primer_bank");
        assert!(report.primer_set_count > 0);
        assert!(!report.governance_hash.is_empty());
        Ok(())
    }
}
