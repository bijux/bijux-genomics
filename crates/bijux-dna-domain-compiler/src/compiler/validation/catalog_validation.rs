use super::{
    bail, is_unspecified, read_yaml, AdapterBank, BTreeMap, BTreeSet, ContaminationDbBank,
    DomainArtifactVocabulary, DomainMetricVocabulary, Path, ReferenceBank, Result,
};

pub(super) struct DomainVocabularies {
    pub(super) artifact_vocab: BTreeMap<String, BTreeSet<String>>,
    pub(super) metric_vocab: BTreeMap<String, BTreeSet<String>>,
}

fn artifact_ids(doc: DomainArtifactVocabulary) -> Vec<String> {
    if doc.artifact_ids.is_empty() {
        doc.artifacts.into_iter().map(|entry| entry.id).collect()
    } else {
        doc.artifact_ids
    }
}

fn metric_ids(doc: DomainMetricVocabulary) -> Vec<String> {
    if doc.metric_ids.is_empty() {
        doc.metrics.into_iter().map(|entry| entry.id).collect()
    } else {
        doc.metric_ids
    }
}

fn collect_unique_ids(path: &Path, field: &str, ids: Vec<String>) -> Result<BTreeSet<String>> {
    let mut unique_ids = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() {
            bail!("{} has empty {field} entry", path.display());
        }
        if !unique_ids.insert(id.clone()) {
            bail!("{} has duplicate {field} entry `{id}`", path.display());
        }
    }
    Ok(unique_ids)
}

pub(super) fn validate_reference_catalogs(workspace_root: &Path) -> Result<()> {
    let adapter_bank_path =
        workspace_root.join("assets").join("reference").join("adapters").join("bank.v1.yaml");
    let reference_bank_path =
        workspace_root.join("assets").join("reference").join("references").join("bank.v1.yaml");
    let contamination_db_bank_path = workspace_root
        .join("assets")
        .join("reference")
        .join("contaminants")
        .join("db_bank.v1.yaml");

    super::super::compile::require_exists(&adapter_bank_path)?;
    super::super::compile::require_exists(&reference_bank_path)?;
    super::super::compile::require_exists(&contamination_db_bank_path)?;

    let adapter_bank: AdapterBank = read_yaml(&adapter_bank_path)?;
    if adapter_bank.schema_version.trim().is_empty()
        || adapter_bank.bank_id.trim().is_empty()
        || adapter_bank.provenance_status.trim().is_empty()
        || adapter_bank.adapters.is_empty()
    {
        bail!("{} missing required adapter bank fields", adapter_bank_path.display());
    }
    if adapter_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            adapter_bank_path.display()
        );
    }
    if adapter_bank.version.trim().is_empty() {
        bail!("{} missing adapter bank version", adapter_bank_path.display());
    }
    for entry in &adapter_bank.adapters {
        if entry.id.trim().is_empty()
            || is_unspecified(&entry.rationale)
            || is_unspecified(&entry.source)
        {
            bail!("{} adapter entries require id/source/rationale", adapter_bank_path.display());
        }
    }

    let reference_bank: ReferenceBank = read_yaml(&reference_bank_path)?;
    if reference_bank.schema_version.trim().is_empty()
        || reference_bank.bank_id.trim().is_empty()
        || reference_bank.version.trim().is_empty()
        || reference_bank.provenance_status.trim().is_empty()
        || reference_bank.references.is_empty()
    {
        bail!("{} missing required reference bank fields", reference_bank_path.display());
    }
    if reference_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            reference_bank_path.display()
        );
    }
    for entry in &reference_bank.references {
        if entry.id.trim().is_empty()
            || entry.kind.trim().is_empty()
            || is_unspecified(&entry.source)
            || is_unspecified(&entry.rationale)
        {
            bail!(
                "{} reference entries require id/kind/source/rationale",
                reference_bank_path.display()
            );
        }
    }

    let contamination_db_bank: ContaminationDbBank = read_yaml(&contamination_db_bank_path)?;
    if contamination_db_bank.schema_version.trim().is_empty()
        || contamination_db_bank.bank_id.trim().is_empty()
        || contamination_db_bank.version.trim().is_empty()
        || contamination_db_bank.provenance_status.trim().is_empty()
        || contamination_db_bank.databases.is_empty()
    {
        bail!(
            "{} missing required contamination db bank fields",
            contamination_db_bank_path.display()
        );
    }
    if contamination_db_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            contamination_db_bank_path.display()
        );
    }
    for entry in &contamination_db_bank.databases {
        if entry.id.trim().is_empty()
            || entry.db_version.trim().is_empty()
            || entry.digest.trim().is_empty()
            || is_unspecified(&entry.source)
            || is_unspecified(&entry.rationale)
        {
            bail!(
                "{} contamination database entries require id/version/digest/source/rationale",
                contamination_db_bank_path.display()
            );
        }
    }

    Ok(())
}

pub(super) fn validate_domain_vocabularies(domain_dir: &Path) -> Result<DomainVocabularies> {
    let mut artifact_vocab = BTreeMap::<String, BTreeSet<String>>::new();
    let mut metric_vocab = BTreeMap::<String, BTreeSet<String>>::new();

    for dom in ["fastq", "bam", "vcf"] {
        let artifacts_path = domain_dir.join(dom).join("artifacts.yaml");
        let metrics_path = domain_dir.join(dom).join("metrics.yaml");
        let artifacts: DomainArtifactVocabulary = read_yaml(&artifacts_path)?;
        let metrics: DomainMetricVocabulary = read_yaml(&metrics_path)?;
        if artifacts.domain != dom {
            bail!(
                "{} domain mismatch: expected {}, got {}",
                artifacts_path.display(),
                dom,
                artifacts.domain
            );
        }
        if metrics.domain != dom {
            bail!(
                "{} domain mismatch: expected {}, got {}",
                metrics_path.display(),
                dom,
                metrics.domain
            );
        }
        let artifact_ids = artifact_ids(artifacts);
        let metric_ids = metric_ids(metrics);
        if artifact_ids.is_empty() {
            bail!("{} missing artifact_ids", artifacts_path.display());
        }
        if metric_ids.is_empty() {
            bail!("{} missing metric_ids", metrics_path.display());
        }
        artifact_vocab.insert(
            dom.to_string(),
            collect_unique_ids(&artifacts_path, "artifact_ids", artifact_ids)?,
        );
        metric_vocab
            .insert(dom.to_string(), collect_unique_ids(&metrics_path, "metric_ids", metric_ids)?);
    }

    Ok(DomainVocabularies { artifact_vocab, metric_vocab })
}
