//! Owner: bijux-dna-bench-workflows
//! Explicit fixture operations and admitted germline tool bindings.
use anyhow::{bail, Result};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeMap;
use std::path::Path;

/// Explicit alignment encoding; a filename does not determine its native format.
#[derive(Clone, Copy, Debug)]
pub enum AlignmentFixtureFormat {
    Sam,
    Bam,
}

/// Alignment fixture effects supplied by the composition owner.
#[derive(Clone, Copy)]
pub struct AlignmentFixtureOperations {
    encode: fn(&Path, &Path, &Path) -> Result<()>,
    reference_intervals: fn(&Path, AlignmentFixtureFormat, &Path) -> Result<()>,
}
impl AlignmentFixtureOperations {
    #[must_use]
    pub fn new(
        encode: fn(&Path, &Path, &Path) -> Result<()>,
        reference_intervals: fn(&Path, AlignmentFixtureFormat, &Path) -> Result<()>,
    ) -> Self {
        Self { encode, reference_intervals }
    }
    /// # Errors
    /// Propagates alignment admission, native encoding and publication failures.
    pub fn encode_coordinate_bam(&self, sam: &Path, bam: &Path, index: &Path) -> Result<()> {
        (self.encode)(sam, bam, index)
    }
    /// # Errors
    /// Propagates reference-header admission and interval publication failures.
    pub fn write_reference_intervals(
        &self,
        alignment: &Path,
        format: AlignmentFixtureFormat,
        bed: &Path,
    ) -> Result<()> {
        (self.reference_intervals)(alignment, format, bed)
    }
}

/// Four explicitly admitted VCF tool bindings and fixture effects for germline composition.
pub struct CoreGermlineWorkflowInputs {
    alignment: AlignmentFixtureOperations,
    vcf_tools: BTreeMap<String, String>,
}
impl CoreGermlineWorkflowInputs {
    /// # Errors
    /// Rejects missing, foreign, duplicated, blank or padded stage/tool observations before effects.
    pub fn new(
        alignment: AlignmentFixtureOperations,
        pairs: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Self> {
        let required = [stage::VCF_CALL, stage::VCF_FILTER, stage::VCF_STATS, stage::VCF_QC];
        let mut vcf_tools = BTreeMap::new();
        for (stage, tool) in pairs {
            if !required.contains(&stage.as_str()) || tool.trim().is_empty() || tool.trim() != tool
            {
                bail!("invalid germline VCF stage/tool binding");
            }
            if vcf_tools.insert(stage, tool).is_some() {
                bail!("duplicate germline VCF binding");
            }
        }
        if vcf_tools.len() != required.len() {
            bail!("germline workflow requires all four VCF tool bindings");
        }
        Ok(Self { alignment, vcf_tools })
    }
    #[must_use]
    pub fn alignment(&self) -> &AlignmentFixtureOperations {
        &self.alignment
    }
    #[must_use]
    pub fn vcf_tools(&self) -> &BTreeMap<String, String> {
        &self.vcf_tools
    }
}
