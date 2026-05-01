use anyhow::Result;
use std::path::{Path, PathBuf};

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("workspace root"))
}

#[test]
fn adapter_bank_publishes_complete_production_provenance() -> Result<()> {
    let root = workspace_root()?;
    let bank = bijux_dna_domain_fastq::load_adapter_bank(
        &root.join(bijux_dna_domain_fastq::adapter_bank_path()),
    )?;
    let presets = bijux_dna_domain_fastq::load_adapter_presets(
        &root.join(bijux_dna_domain_fastq::adapter_presets_path()),
        &bank,
    )?;

    assert_eq!(bank.provenance_status, "complete");
    assert!(!bank.license.trim().is_empty());
    assert_eq!(bank.source_checksum_sha256.len(), 64);
    assert!(
        bank.applicable_assays.iter().any(|assay| assay == "amplicon"),
        "adapter bank must disclose amplicon applicability when primer-aware workflows reuse the bank"
    );
    assert!(
        presets.presets.iter().all(|preset| !preset.selection_logic.trim().is_empty()),
        "every adapter preset must publish explicit selection logic"
    );
    Ok(())
}

#[test]
fn primer_bank_registry_is_literature_and_checksum_locked() -> Result<()> {
    let root = workspace_root()?;
    let bank = bijux_dna_domain_fastq::load_primer_bank(
        &root.join(bijux_dna_domain_fastq::amplicon_governance_path()),
    )?;

    assert_eq!(bank.provenance_status, "complete");
    assert_eq!(bank.license, "CC-BY-4.0");
    assert_eq!(bank.primer_sets.len(), 3);
    assert!(
        bank.primer_sets.iter().all(|primer_set| primer_set.primer_sha256.len() == 64),
        "every primer set must carry a locked fasta checksum"
    );
    assert!(
        bank.primer_sets.iter().all(|primer_set| !primer_set.doi_status.trim().is_empty()),
        "production primer banks must surface an explicit provenance status"
    );
    assert!(
        bank.primer_sets.iter().all(|primer_set| !primer_set.primary_locator.trim().is_empty()),
        "production primer banks must carry a machine-readable primary locator"
    );
    Ok(())
}
