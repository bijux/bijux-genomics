use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use crate::resolution::validate_sha256;
use crate::runtime_config::{
    load_toml, workspace_root, BundlesConfig, GeneticMapBankConfig, OrganellarPolicyConfig,
    ReferenceBankConfig, ReferenceSetConfig,
};
use crate::{
    BundleEntry, ContigNormalizationPolicy, GeneticMapBankEntry, MaterializedIndexArtifact,
    OrganellarPolicy, ReferenceBankEntry, ReferenceBundle, ReferenceMaterializationReport,
    ReferenceProvenance, ReferenceSet,
};

/// # Errors
/// Returns an error if reference bank config cannot be read or entry is missing.
pub fn resolve_reference_bank(species: &str, build: &str) -> Result<ReferenceBankEntry> {
    let path = workspace_root().join("configs/runtime/reference_bank.toml");
    let cfg: ReferenceBankConfig = load_toml(&path)?;
    let entry = cfg
        .reference
        .into_iter()
        .find(|row| row.species_id == species && row.build_id == build)
        .ok_or_else(|| anyhow!("reference bank entry missing for {species}:{build}"))?;
    validate_sha256(&entry.fasta_sha256, "reference_bank fasta_sha256")?;
    if entry.license_id.trim().is_empty() || entry.license_url.trim().is_empty() {
        bail!("reference bank entry for {species}:{build} missing license metadata");
    }
    Ok(entry)
}

/// # Errors
/// Returns an error if genetic map bank config cannot be read or no matching map exists.
pub fn resolve_genetic_map_bank(
    species: &str,
    build: &str,
    panel_id: Option<&str>,
) -> Result<GeneticMapBankEntry> {
    let path = workspace_root().join("configs/runtime/genetic_map_bank.toml");
    let cfg: GeneticMapBankConfig = load_toml(&path)?;
    let mut candidates = cfg
        .map
        .into_iter()
        .filter(|entry| entry.species_id == species && entry.build_id == build)
        .collect::<Vec<_>>();
    if let Some(panel) = panel_id {
        candidates.retain(|entry| entry.panel_id.as_deref() == Some(panel));
    }
    let selected = candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("genetic map bank entry missing for {species}:{build}"))?;
    validate_sha256(&selected.map_sha256, "genetic_map_bank map_sha256")?;
    Ok(selected)
}

/// # Errors
/// Returns an error if organellar policy cannot be loaded.
pub fn resolve_organellar_policy(species: &str, build: &str) -> Result<OrganellarPolicy> {
    let path = workspace_root().join("configs/runtime/organellar_policy.toml");
    let cfg: OrganellarPolicyConfig = load_toml(&path)?;
    cfg.policy
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("organellar policy missing for {species}:{build}"))
}

/// # Errors
/// Returns an error if default reference set cannot be loaded for species/usecase.
pub fn resolve_default_reference_set(species: &str, usecase: &str) -> Result<ReferenceSet> {
    let path = workspace_root().join("configs/runtime/reference_sets.toml");
    let cfg: ReferenceSetConfig = load_toml(&path)?;
    cfg.set
        .into_iter()
        .find(|entry| entry.species_id == species && entry.usecase == usecase)
        .ok_or_else(|| anyhow!("default reference set missing for {species}:{usecase}"))
}

/// # Errors
/// Returns an error when the reference bundle cannot be resolved or violates contract.
pub fn resolve_reference_bundle(species: &str, build: &str) -> Result<ReferenceBundle> {
    let bundle = resolve_bundle_entry(species, build)?;
    validate_bundle_digests(&bundle)?;
    validate_bundle_contigs(&bundle)?;
    let normalization_policy = match bundle.normalization_policy.as_str() {
        "strict_only" => ContigNormalizationPolicy::StrictOnly,
        "deterministic_remap" => ContigNormalizationPolicy::DeterministicRemap,
        other => bail!("unknown normalization policy: {other}"),
    };
    if normalization_policy == ContigNormalizationPolicy::DeterministicRemap
        && bundle.remap.is_empty()
    {
        bail!("deterministic_remap requires non-empty remap table");
    }
    Ok(ReferenceBundle {
        bundle_id: bundle.bundle_id.clone(),
        species_id: bundle.species_id.clone(),
        build_id: bundle.build_id.clone(),
        fasta: bundle.fasta.clone(),
        fai: bundle.fai.clone(),
        dict: bundle.dict.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        contigs: bundle.contigs.iter().map(|contig| contig.name.clone()).collect(),
        mask_bed: bundle.mask_bed.clone(),
        regions_bed: bundle.regions_bed.clone(),
        source_lock_sha256: bundle.source_lock_sha256.clone(),
        bundle_lock_sha256: bundle.bundle_lock_sha256.clone(),
        normalization_policy,
        remap_table: bundle.remap.clone(),
    })
}

/// # Errors
/// Returns an error if policy forbids remapping and contig names differ.
pub fn normalize_contig_name(bundle: &ReferenceBundle, contig: &str) -> Result<String> {
    let contig = contig.trim();
    if contig.is_empty() {
        bail!("contig name must not be empty");
    }
    match bundle.normalization_policy {
        ContigNormalizationPolicy::StrictOnly => {
            if bundle.contigs.iter().any(|canonical| canonical == contig) {
                return Ok(contig.to_string());
            }
            bail!("contig {contig} not declared for bundle {}", bundle.bundle_id);
        }
        ContigNormalizationPolicy::DeterministicRemap => {
            let normalized = bundle
                .remap_table
                .get(contig)
                .cloned()
                .or_else(|| {
                    if bundle.remap_table.values().any(|value| value == contig) {
                        Some(contig.to_string())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow!("contig {contig} not in deterministic remap table"))?;
            if bundle.contigs.iter().any(|canonical| canonical == &normalized) {
                return Ok(normalized);
            }
            bail!(
                "contig {contig} normalized to {normalized}, which is not declared for bundle {}",
                bundle.bundle_id
            );
        }
    }
}

#[must_use]
pub fn reference_provenance(
    _species: &str,
    _build: &str,
    bundle: &ReferenceBundle,
) -> ReferenceProvenance {
    ReferenceProvenance {
        species_id: bundle.species_id.clone(),
        build_id: bundle.build_id.clone(),
        bundle_id: bundle.bundle_id.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        source_lock_sha256: bundle.source_lock_sha256.clone(),
        bundle_lock_sha256: bundle.bundle_lock_sha256.clone(),
    }
}

/// # Errors
/// Returns an error if reference contracts are invalid or offline materialization is disallowed.
pub fn materialize_reference_bank(
    species: &str,
    build: &str,
    materialization_root: &Path,
    offline: bool,
    allow_fixture_materialization: bool,
) -> Result<ReferenceMaterializationReport> {
    let bank = resolve_reference_bank(species, build)?;
    let bundle = resolve_reference_bundle(species, build)?;
    if offline && !allow_fixture_materialization {
        bail!(
            "offline materialization refused for {species}:{build}; enable fixture materialization explicitly"
        );
    }
    if bank.required_indexes.is_empty() {
        bail!("reference bank entry for {species}:{build} must declare required indexes");
    }

    let root = materialization_root.join(species).join(build).join("refs");
    let raw = root.join("raw");
    let normalized = root.join("normalized");
    let derived = root.join("derived");
    for dir in [&raw, &normalized, &derived] {
        std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    }

    let fasta = raw.join("reference.fa");
    let fasta_contents = format!(
        ">synthetic_reference|species={species}|build={build}|mode=fixture\nACGTACGTACGTACGT\n"
    );
    std::fs::write(&fasta, fasta_contents.as_bytes())
        .with_context(|| format!("write {}", fasta.display()))?;

    let mut index_artifacts = Vec::new();
    for required in &bank.required_indexes {
        let artifact = write_index_artifact(required, &fasta, &normalized)?;
        index_artifacts.push(artifact);
    }
    let dict_path = normalized.join("reference.dict");
    std::fs::write(&dict_path, format!("@HD\tVN:1.0\n@SQ\tSN:{}\tLN:16\n", bundle.contigs[0]))
        .with_context(|| format!("write {}", dict_path.display()))?;
    index_artifacts.push(MaterializedIndexArtifact {
        tool_id: "samtools_dict".to_string(),
        path: dict_path,
        status: "fixture".to_string(),
    });

    Ok(ReferenceMaterializationReport {
        schema_version: "bijux.reference_materialization.v1".to_string(),
        species_id: species.to_string(),
        build_id: build.to_string(),
        source_url: bank.fasta_url,
        declared_sha256: bank.fasta_sha256,
        license_id: bank.license_id,
        license_url: bank.license_url,
        materialization_root: root,
        mode: if offline { "offline_fixture" } else { "online_fixture" }.to_string(),
        bundle_id: bundle.bundle_id,
        index_artifacts,
    })
}

pub(crate) fn resolve_bundle_entry(species: &str, build: &str) -> Result<BundleEntry> {
    let path = workspace_root().join("configs/runtime/reference_bundles.toml");
    let cfg: BundlesConfig = load_toml(&path)?;
    cfg.bundle
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("no reference bundle found for {species}:{build}"))
}

fn validate_bundle_digests(bundle: &BundleEntry) -> Result<()> {
    validate_sha256(&bundle.contig_set_digest, "contig_set_digest")?;
    validate_sha256(&bundle.source_lock_sha256, "source_lock_sha256")?;
    validate_sha256(&bundle.bundle_lock_sha256, "bundle_lock_sha256")
}

fn validate_bundle_contigs(bundle: &BundleEntry) -> Result<()> {
    if bundle.contigs.is_empty() {
        bail!("reference bundle {} must declare at least one contig", bundle.bundle_id);
    }
    for contig in &bundle.contigs {
        if contig.name.trim().is_empty() {
            bail!("reference bundle {} contains an empty contig name", bundle.bundle_id);
        }
        if contig.length_bp == 0 {
            bail!(
                "reference bundle {} contig {} must have positive length",
                bundle.bundle_id,
                contig.name
            );
        }
    }
    Ok(())
}

fn write_index_artifact(
    required: &str,
    fasta: &Path,
    normalized: &Path,
) -> Result<MaterializedIndexArtifact> {
    let (tool_id, target, payload): (&str, PathBuf, String) = match required {
        "samtools_faidx" => (
            "samtools_faidx",
            normalized.join("reference.fa.fai"),
            "synthetic_reference\t16\t0\t16\t17\n".to_string(),
        ),
        "bwa_index" => {
            ("bwa_index", normalized.join("reference.fa.bwt"), "synthetic-bwa-index\n".to_string())
        }
        "bowtie2_index" => (
            "bowtie2_index",
            normalized.join("reference.fa.1.bt2"),
            "synthetic-bowtie2-index\n".to_string(),
        ),
        other => bail!("unsupported required index tool: {other}"),
    };
    let header = format!("# source={}\n", fasta.display());
    std::fs::write(&target, format!("{header}{payload}").as_bytes())
        .with_context(|| format!("write {}", target.display()))?;
    Ok(MaterializedIndexArtifact {
        tool_id: tool_id.to_string(),
        path: target,
        status: "fixture".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{materialize_reference_bank, validate_bundle_contigs, validate_bundle_digests};
    use crate::runtime_config::{BundleEntry, ContigEntry, SupportedFeatureEntry};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn validate_bundle_digests_rejects_invalid_contig_set_digest() {
        let bundle = BundleEntry {
            bundle_id: "bundle".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            fasta: "ref.fa.gz".to_string(),
            fai: "ref.fa.gz.fai".to_string(),
            dict: "ref.dict".to_string(),
            contig_set_digest: "not-a-digest".to_string(),
            mask_bed: None,
            regions_bed: None,
            source_lock_sha256: "a".repeat(64),
            bundle_lock_sha256: "b".repeat(64),
            normalization_policy: "strict_only".to_string(),
            remap: BTreeMap::new(),
            sex_system: "xy".to_string(),
            par_policy: "unsupported".to_string(),
            default_coverage_regime: None,
            supported_features: SupportedFeatureEntry::default(),
            contigs: vec![ContigEntry { name: "1".to_string(), length_bp: 1 }],
        };

        let Err(error) = validate_bundle_digests(&bundle) else {
            panic!("invalid contig digest must fail");
        };

        assert!(error.to_string().contains("contig_set_digest"));
    }

    #[test]
    fn validate_bundle_contigs_rejects_empty_contig_sets() {
        let bundle = BundleEntry {
            bundle_id: "bundle".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            fasta: "ref.fa.gz".to_string(),
            fai: "ref.fa.gz.fai".to_string(),
            dict: "ref.dict".to_string(),
            contig_set_digest: "a".repeat(64),
            mask_bed: None,
            regions_bed: None,
            source_lock_sha256: "a".repeat(64),
            bundle_lock_sha256: "b".repeat(64),
            normalization_policy: "strict_only".to_string(),
            remap: BTreeMap::new(),
            sex_system: "xy".to_string(),
            par_policy: "unsupported".to_string(),
            default_coverage_regime: None,
            supported_features: SupportedFeatureEntry::default(),
            contigs: Vec::new(),
        };

        let Err(error) = validate_bundle_contigs(&bundle) else {
            panic!("empty contig set must fail");
        };

        assert!(error.to_string().contains("at least one contig"));
    }

    #[test]
    fn resolve_reference_bundle_preserves_declared_contigs() {
        let bundle = super::resolve_reference_bundle("Homo sapiens", "GRCh38")
            .unwrap_or_else(|error| panic!("resolve reference bundle: {error}"));

        assert!(bundle.contigs.iter().any(|contig| contig == "1"));
        assert!(bundle.contigs.iter().any(|contig| contig == "X"));
    }

    #[test]
    fn strict_contig_normalization_rejects_undeclared_contigs() {
        let bundle = super::resolve_reference_bundle("Homo sapiens", "GRCh38")
            .unwrap_or_else(|error| panic!("resolve reference bundle: {error}"));

        let Err(error) = super::normalize_contig_name(&bundle, "chr1") else {
            panic!("strict bundle must reject non-canonical contig");
        };

        assert!(error.to_string().contains("not declared"));
    }

    #[test]
    fn reference_provenance_uses_resolved_bundle_identity() {
        let bundle = super::resolve_reference_bundle("Homo sapiens", "GRCh38")
            .unwrap_or_else(|error| panic!("resolve reference bundle: {error}"));

        let provenance = super::reference_provenance("wrong species", "wrong build", &bundle);

        assert_eq!(provenance.species_id, bundle.species_id);
        assert_eq!(provenance.build_id, bundle.build_id);
    }

    #[test]
    fn reference_materialization_offline_refusal_requires_fixture_opt_in() {
        let temp = make_temp_dir("offline-refusal");
        let Err(error) = materialize_reference_bank("Homo sapiens", "GRCh38", &temp, true, false)
        else {
            panic!("offline materialization without fixture opt-in must fail");
        };

        assert!(error.to_string().contains("offline materialization refused"));
    }

    #[test]
    fn reference_materialization_writes_fixture_indexes() {
        let temp = make_temp_dir("reference-indexes");
        let report = materialize_reference_bank("Homo sapiens", "GRCh38", &temp, true, true)
            .unwrap_or_else(|error| panic!("materialize reference bank: {error}"));

        assert_eq!(report.schema_version, "bijux.reference_materialization.v1");
        assert_eq!(report.mode, "offline_fixture");
        assert!(report.index_artifacts.iter().any(|artifact| artifact.tool_id == "samtools_faidx"));
        assert!(report.index_artifacts.iter().all(|artifact| artifact.path.exists()));
    }

    fn make_temp_dir(label: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0_u128, |value| value.as_nanos());
        path.push(format!("bijux-db-ref-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|error| panic!("create temp dir {}: {error}", path.display()));
        path
    }
}
