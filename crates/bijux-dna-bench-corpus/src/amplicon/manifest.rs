//! Owner: bijux-dna-bench-corpus
//! AMPLICON corpus manifest ownership.
use super::{
    anyhow, fs, AmpliconCorpusFixtureManifest, AmpliconCorpusSampleKind,
    AmpliconGovernanceDocument, BTreeSet, Path, Result, AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION,
};
use anyhow::Context;

/// Load the typed corpus document from its explicit local path.
///
/// # Errors
/// Rejects unreadable documents, malformed syntax and incompatible typed fields.
pub fn load_amplicon_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<AmpliconCorpusFixtureManifest> {
    load_amplicon_corpus_fixture_manifest_path_impl(manifest_path)
}

/// Validate the typed corpus declaration before observing its payloads.
///
/// # Errors
/// Rejects unsupported schemas, inconsistent identities and incomplete corpus declarations.
pub fn validate_amplicon_corpus_fixture_manifest_contract(
    manifest: &AmpliconCorpusFixtureManifest,
) -> Result<()> {
    validate_amplicon_corpus_fixture_manifest_contract_impl(manifest)
}

pub(super) fn load_amplicon_corpus_fixture_manifest_path_impl(
    manifest_path: &Path,
) -> Result<AmpliconCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn validate_amplicon_corpus_fixture_manifest_contract_impl(
    manifest: &AmpliconCorpusFixtureManifest,
) -> Result<()> {
    validate_manifest_identity(manifest)?;
    let control_sample_ids = validate_manifest_samples(manifest)?;
    validate_manifest_controls(manifest, &control_sample_ids)?;
    validate_manifest_abundance(manifest)?;

    Ok(())
}

pub(super) fn validate_amplicon_governance_contract(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    primer_fasta_path: &Path,
    amplicon_governance_path: &Path,
) -> Result<()> {
    if !primer_fasta_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture primer FASTA is missing: {}",
            primer_fasta_path.display()
        ));
    }
    if !amplicon_governance_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture governance file is missing: {}",
            amplicon_governance_path.display()
        ));
    }

    let raw = fs::read_to_string(amplicon_governance_path)
        .with_context(|| format!("read {}", amplicon_governance_path.display()))?;
    let governance: AmpliconGovernanceDocument = toml::from_str(&raw)
        .with_context(|| format!("parse {}", amplicon_governance_path.display()))?;
    let marker = governance.markers.get(&manifest.marker_id).ok_or_else(|| {
        anyhow!(
            "amplicon governance does not declare marker `{}` in {}",
            manifest.marker_id,
            amplicon_governance_path.display()
        )
    })?;
    if marker.primer_set_id != manifest.primer_set_id {
        return Err(anyhow!(
            "amplicon corpus fixture marker `{}` expects primer set `{}` but manifest declared `{}`",
            manifest.marker_id,
            marker.primer_set_id,
            manifest.primer_set_id
        ));
    }
    if !marker.applicable_assays.iter().any(|assay| assay == &manifest.assay_id) {
        return Err(anyhow!(
            "amplicon corpus fixture assay `{}` is not allowed for marker `{}`",
            manifest.assay_id,
            manifest.marker_id
        ));
    }
    let governed_primer_fasta = if marker.primer_fasta.is_absolute() {
        marker.primer_fasta.clone()
    } else {
        repo_root.join(&marker.primer_fasta)
    };
    let manifest_primer_fasta = primer_fasta_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", primer_fasta_path.display()))?;
    let governed_primer_fasta = governed_primer_fasta
        .canonicalize()
        .with_context(|| format!("canonicalize {}", governed_primer_fasta.display()))?;
    if governed_primer_fasta != manifest_primer_fasta {
        return Err(anyhow!(
            "amplicon corpus fixture primer FASTA `{}` does not match governance path `{}` for marker `{}`",
            manifest_primer_fasta.display(),
            governed_primer_fasta.display(),
            manifest.marker_id
        ));
    }
    Ok(())
}

fn validate_manifest_identity(manifest: &AmpliconCorpusFixtureManifest) -> Result<()> {
    if manifest.schema_version != AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported amplicon corpus fixture schema `{}`",
            manifest.schema_version
        ));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.assay_id.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `assay_id`"));
    }
    if manifest.marker_id.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `marker_id`"));
    }
    if manifest.target_region.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `target_region`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `description`"));
    }
    if manifest.primer_set_id.trim().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `primer_set_id`"));
    }
    if manifest.forward_primer_id.trim().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `forward_primer_id`"
        ));
    }
    if manifest.reverse_primer_id.trim().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `reverse_primer_id`"
        ));
    }
    if manifest.primer_fasta.as_os_str().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `primer_fasta`"));
    }
    if manifest.primers_tsv_path.as_os_str().is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare a non-empty `primers_tsv_path`"));
    }
    if manifest.expected_asvs_path.as_os_str().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `expected_asvs_path`"
        ));
    }
    if manifest.chimera_controls_fasta_path.as_os_str().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `chimera_controls_fasta_path`"
        ));
    }
    if manifest.chimera_expectations_path.as_os_str().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `chimera_expectations_path`"
        ));
    }
    if manifest.amplicon_governance_path.as_os_str().is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture must declare a non-empty `amplicon_governance_path`"
        ));
    }
    if manifest.controls.is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare at least one control"));
    }
    if manifest.abundance_tables.is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare at least one abundance table"));
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!("amplicon corpus fixture must declare at least one sample"));
    }

    Ok(())
}

fn validate_manifest_samples(manifest: &AmpliconCorpusFixtureManifest) -> Result<BTreeSet<&str>> {
    let mut sample_ids = BTreeSet::new();
    let mut biological_sample_count = 0_usize;
    let mut control_sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture samples must declare a non-empty `sample_id`"
            ));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!(
                "amplicon corpus fixture repeats sample_id `{}`",
                sample.sample_id
            ));
        }
        if sample.expected_read_count == 0 {
            return Err(anyhow!(
                "amplicon corpus fixture sample `{}` must declare a positive `expected_read_count`",
                sample.sample_id
            ));
        }
        if sample.source_paths.is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture sample `{}` must declare at least one `source_paths` entry",
                sample.sample_id
            ));
        }
        match sample.sample_kind {
            AmpliconCorpusSampleKind::Biological => {
                biological_sample_count = biological_sample_count.saturating_add(1);
            }
            AmpliconCorpusSampleKind::Control => {
                control_sample_ids.insert(sample.sample_id.as_str());
            }
        }
    }
    if biological_sample_count == 0 {
        return Err(anyhow!("amplicon corpus fixture must declare at least one biological sample"));
    }

    Ok(control_sample_ids)
}

fn validate_manifest_controls(
    manifest: &AmpliconCorpusFixtureManifest,
    control_sample_ids: &BTreeSet<&str>,
) -> Result<()> {
    let declared_sample_ids =
        manifest.samples.iter().map(|sample| sample.sample_id.as_str()).collect::<BTreeSet<_>>();
    let mut declared_controls = BTreeSet::new();
    for control in &manifest.controls {
        if control.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture controls must declare a non-empty `sample_id`"
            ));
        }
        if !declared_controls.insert(control.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture repeats control sample_id `{}`",
                control.sample_id
            ));
        }
        if !declared_sample_ids.contains(control.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture control `{}` does not match a declared sample",
                control.sample_id
            ));
        }
        if !control_sample_ids.contains(control.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture control `{}` must reference a sample with `sample_kind = \"control\"`",
                control.sample_id
            ));
        }
        if control.control_kind.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture control `{}` must declare a non-empty `control_kind`",
                control.sample_id
            ));
        }
        if control.purpose.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture control `{}` must declare a non-empty `purpose`",
                control.sample_id
            ));
        }
    }

    for sample in &manifest.samples {
        if matches!(sample.sample_kind, AmpliconCorpusSampleKind::Control)
            && !declared_controls.contains(sample.sample_id.as_str())
        {
            return Err(anyhow!(
                "amplicon corpus fixture control sample `{}` must also appear in `controls`",
                sample.sample_id
            ));
        }
    }

    Ok(())
}

fn validate_manifest_abundance(manifest: &AmpliconCorpusFixtureManifest) -> Result<()> {
    let biological_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Biological))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut abundance_table_ids = BTreeSet::new();
    for table in &manifest.abundance_tables {
        if table.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture abundance tables must declare a non-empty `sample_id`"
            ));
        }
        if !abundance_table_ids.insert(table.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture repeats abundance table sample_id `{}`",
                table.sample_id
            ));
        }
        if table.table_kind.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a non-empty `table_kind`",
                table.sample_id
            ));
        }
        if table.table_path.as_os_str().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a non-empty `table_path`",
                table.sample_id
            ));
        }
        if table.expected_row_count == 0 {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a positive `expected_row_count`",
                table.sample_id
            ));
        }
        if table.expected_sample_count == 0 {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a positive `expected_sample_count`",
                table.sample_id
            ));
        }
        if table.expected_feature_count == 0 {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare a positive `expected_feature_count`",
                table.sample_id
            ));
        }
        if table.source_paths.is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must declare at least one `source_paths` entry",
                table.sample_id
            ));
        }
        if !table.table_kind.eq("otu_abundance") {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` must currently use `table_kind = \"otu_abundance\"`",
                table.sample_id
            ));
        }
        let expected_sample_count = usize::try_from(table.expected_sample_count).map_err(|_| {
            anyhow!(
                "amplicon corpus fixture abundance table `{}` uses an unsupported `expected_sample_count`",
                table.sample_id
            )
        })?;
        if biological_sample_ids.len() < expected_sample_count {
            return Err(anyhow!(
                "amplicon corpus fixture abundance table `{}` expects {} biological samples but only {} are declared",
                table.sample_id,
                table.expected_sample_count,
                biological_sample_ids.len()
            ));
        }
    }
    Ok(())
}
