//! Owner: bijux-dna-bench-vcf
//! Admitted catalog and supported binding inputs; acquisition belongs to the caller.
use anyhow::{anyhow, bail, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Explicit fixture effect supplied by the benchmark composition owner.
pub type ReferenceFastaMaterializer = fn(&Path, &Path) -> Result<(PathBuf, PathBuf)>;
use bijux_dna_bench_model::{
    validate_local_vcf_stage_catalog_config, LocalVcfStageCatalogConfig, VcfStageCatalogRow,
    VcfStageMatrixRow, LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1,
};

/// Explicit immutable input shared across tool-family adapter projections.
#[derive(Debug, Clone)]
pub struct VcfAdapterInputs {
    reference_fasta: ReferenceFastaMaterializer,
    catalog: Vec<VcfStageCatalogRow>,
    bindings: Vec<VcfStageMatrixRow>,
}
impl VcfAdapterInputs {
    /// Admit a catalog and its stage/tool/corpus/asset bindings before adapter effects.
    ///
    /// # Errors
    /// Rejects invalid catalogs, empty bindings, unknown stages and duplicate, blank or padded identities.
    pub fn new(
        catalog: Vec<VcfStageCatalogRow>,
        bindings: Vec<VcfStageMatrixRow>,
        reference_fasta: ReferenceFastaMaterializer,
    ) -> Result<Self> {
        let config = LocalVcfStageCatalogConfig {
            schema_version: LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1.to_string(),
            rows: catalog,
        };
        validate_local_vcf_stage_catalog_config(&config)?;
        if bindings.is_empty() {
            bail!("VCF adapter inputs require supported bindings");
        }
        let stages = config.rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
        let mut keys = BTreeSet::new();
        for row in &bindings {
            if !stages.contains(row.stage_id.as_str()) {
                bail!("VCF adapter binding has unknown stage `{}`", row.stage_id);
            }
            if [
                &row.stage_id,
                &row.tool_id,
                &row.corpus_id,
                &row.asset_profile_id,
                &row.adapter_id,
                &row.parser_id,
            ]
            .into_iter()
            .any(|v| v.trim().is_empty() || v.trim() != *v)
            {
                bail!("VCF adapter binding contains a blank or padded identity");
            }
            if !keys.insert((&row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id)) {
                bail!("VCF adapter inputs repeat a stage/tool/corpus/asset binding");
            }
            if row.expected_outputs.is_empty()
                || row.expected_outputs.iter().any(|v| v.trim().is_empty() || v.trim() != *v)
                || row.expected_outputs.iter().collect::<BTreeSet<_>>().len()
                    != row.expected_outputs.len()
            {
                bail!("VCF adapter binding has invalid output identities");
            }
        }
        Ok(Self { catalog: config.rows, bindings, reference_fasta })
    }
    /// Invoke the caller-selected reference materializer with explicit paths.
    ///
    /// # Errors
    /// Propagates reference admission or publication failure from the supplied effect owner.
    pub fn materialize_reference_fasta(
        &self,
        source: &Path,
        destination: &Path,
    ) -> Result<(PathBuf, PathBuf)> {
        (self.reference_fasta)(source, destination)
    }
    /// Borrow the admitted stage catalog.
    #[must_use]
    pub fn catalog(&self) -> &[VcfStageCatalogRow] {
        &self.catalog
    }
    /// Borrow admitted supported tool bindings.
    #[must_use]
    pub fn bindings(&self) -> &[VcfStageMatrixRow] {
        &self.bindings
    }
}

/// Project explicitly supported stage/tool pairs onto their benchmark templates.
///
/// # Errors
/// Rejects absent templates, duplicate templates and duplicate projected binding identities.
pub fn project_supported_vcf_bindings(
    templates: &[VcfStageMatrixRow],
    pairs: impl IntoIterator<Item = (String, String)>,
) -> Result<Vec<VcfStageMatrixRow>> {
    let mut by_stage = BTreeMap::new();
    for row in templates {
        if by_stage.insert(row.stage_id.as_str(), row).is_some() {
            bail!("VCF benchmark templates repeat stage `{}`", row.stage_id);
        }
    }
    let mut rows = pairs
        .into_iter()
        .map(|(stage, tool)| {
            let template = by_stage.get(stage.as_str()).ok_or_else(|| {
                anyhow!("supported VCF binding `{stage}` / `{tool}` has no stage contract template")
            })?;
            let mut row = (*template).clone();
            row.tool_id = tool;
            Ok(row)
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.asset_profile_id.cmp(&right.asset_profile_id))
    });
    let keys = rows
        .iter()
        .map(|row| (&row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id))
        .collect::<BTreeSet<_>>();
    if keys.len() != rows.len() {
        bail!(
            "VCF benchmark bindings must keep exactly one row per stage/tool/corpus/asset binding"
        );
    }
    Ok(rows)
}

/// Resolved identity fields shared by tool-family report construction.
pub(crate) struct AdapterBindingIdentity {
    pub(crate) adapter: String,
    pub(crate) parser: String,
    pub(crate) corpus: String,
    pub(crate) asset_profile: String,
}

pub(crate) fn resolve_adapter_binding_identity(
    stage: bijux_dna_domain_vcf::taxonomy::VcfDomainStage,
    stage_id: &str,
    tool_id: &str,
    binding: Option<&VcfStageMatrixRow>,
    default_asset_profile: &str,
) -> Result<AdapterBindingIdentity> {
    use bijux_dna_stages_vcf::stage_specs::{
        vcf_domain_stage_adapter_id, vcf_domain_stage_parser_id,
    };
    let adapter_id = binding
        .map(|row| row.adapter_id.clone())
        .or_else(|| vcf_domain_stage_adapter_id(stage).map(str::to_string))
        .ok_or_else(|| anyhow!("VCF {stage_id} / {tool_id} row is missing adapter id"))?;
    let parser_id = binding
        .map(|row| row.parser_id.clone())
        .or_else(|| vcf_domain_stage_parser_id(stage).map(str::to_string))
        .ok_or_else(|| anyhow!("VCF {stage_id} / {tool_id} row is missing parser id"))?;
    Ok(AdapterBindingIdentity {
        adapter: adapter_id,
        parser: parser_id,
        corpus: binding
            .map_or_else(|| "vcf_production_regression".to_string(), |row| row.corpus_id.clone()),
        asset_profile: binding
            .map_or_else(|| default_asset_profile.to_string(), |row| row.asset_profile_id.clone()),
    })
}
