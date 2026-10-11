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

#[test]
fn repository_bank_contexts_preserve_provenance_without_changing_current_directory() -> Result<()> {
    use bijux_dna_domain_fastq::banks::{
        adapter_bank_context_at, contaminant_bank_context_at, polyx_bank_context_at,
    };
    let root = workspace_root()?;
    let cwd = std::env::current_dir()?;
    let adapter = adapter_bank_context_at(&root, None, None, None, &[], &[])?
        .ok_or_else(|| anyhow::anyhow!("adapter provenance absent"))?;
    let polyx = polyx_bank_context_at(&root, None)?
        .ok_or_else(|| anyhow::anyhow!("polyX provenance absent"))?;
    let contaminant = contaminant_bank_context_at(&root, None)?
        .ok_or_else(|| anyhow::anyhow!("contaminant provenance absent"))?;
    for (context, path) in [
        (&adapter, bijux_dna_domain_fastq::adapter_bank_path()),
        (&polyx, bijux_dna_domain_fastq::polyx_bank_path()),
        (&contaminant, bijux_dna_domain_fastq::contaminant_motifs_path()),
    ] {
        assert_eq!(context["bank_hash"], bijux_dna_infra::hash_file_sha256(&root.join(path))?);
        assert!(context["enabled_entries"].as_array().is_some_and(|entries| !entries.is_empty()));
    }
    let references =
        contaminant["references"].as_array().ok_or_else(|| anyhow::anyhow!("references absent"))?;
    assert!(!references.is_empty());
    for reference in references {
        let file = reference["file"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("reference filename absent"))?;
        let path = root.join(bijux_dna_domain_fastq::contaminant_references_dir()).join(file);
        assert_eq!(reference["fasta"], std::fs::read_to_string(&path)?);
        assert_eq!(reference["sha256"], bijux_dna_infra::hash_file_sha256(&path)?);
    }
    assert_eq!(std::env::current_dir()?, cwd);
    Ok(())
}

#[test]
fn repository_bank_contexts_reject_missing_or_changed_supplied_sources() -> Result<()> {
    use bijux_dna_domain_fastq::banks::{
        adapter_bank_context_at, contaminant_bank_context_at, polyx_bank_context_at,
    };
    let root = workspace_root()?;
    let scratch = bijux_dna_testkit::tempdir_for("bank-context-source-admission");
    let target = scratch.path();
    assert!(adapter_bank_context_at(target, None, None, None, &[], &[]).is_err());
    assert!(polyx_bank_context_at(target, None).is_err());
    assert!(contaminant_bank_context_at(target, None).is_err());
    let original = contaminant_bank_context_at(&root, None)?
        .ok_or_else(|| anyhow::anyhow!("source contaminant provenance absent"))?;
    for path in [
        bijux_dna_domain_fastq::contaminant_motifs_path(),
        bijux_dna_domain_fastq::contaminant_presets_path(),
    ] {
        let destination = target.join(&path);
        std::fs::create_dir_all(
            destination.parent().ok_or_else(|| anyhow::anyhow!("bank parent absent"))?,
        )?;
        std::fs::copy(root.join(path), destination)?;
    }
    assert!(
        contaminant_bank_context_at(target, None).is_err(),
        "references must use the supplied root"
    );
    let reference_dir = bijux_dna_domain_fastq::contaminant_references_dir();
    std::fs::create_dir_all(target.join(&reference_dir))?;
    let references = original["references"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("source references absent"))?;
    for reference in references {
        let file = reference["file"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("reference filename absent"))?;
        std::fs::copy(
            root.join(&reference_dir).join(file),
            target.join(&reference_dir).join(file),
        )?;
    }
    assert_eq!(contaminant_bank_context_at(target, None)?, Some(original.clone()));
    let first = references
        .first()
        .and_then(|reference| reference["file"].as_str())
        .ok_or_else(|| anyhow::anyhow!("source reference absent"))?;
    std::fs::write(target.join(reference_dir).join(first), ">changed\nACGT\n")?;
    assert!(
        contaminant_bank_context_at(target, None).is_err(),
        "changed FASTA must fail the existing preset digest contract"
    );
    assert_eq!(contaminant_bank_context_at(&root, None)?, Some(original));
    Ok(())
}
