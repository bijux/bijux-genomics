use super::acquisition::{
    collect_vcf_readiness_evidence_with_observations, ActiveEvidenceProvider,
};
use crate::{adapters::observations::VcfAdapterObservations, inputs::VcfAdapterInputs};
use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::{
    model::command_reports::VcfRenderedCommandsReport, snapshot::VcfEvidenceSnapshot,
};
use std::{cell::OnceCell, path::Path};
/// Input declarations supplied by the application composition authority.
pub type AdapterInputProvider = fn(&Path) -> Result<VcfAdapterInputs>;
/// One immutable, successfully admitted evidence snapshot shared by a local request.
/// Failed acquisition is not cached. Each independent request gets a separate session.
pub struct VcfReadinessEvidenceSession<'a> {
    repo_root: &'a Path,
    run_id: &'a str,
    inputs: AdapterInputProvider,
    active: ActiveEvidenceProvider,
    evidence: OnceCell<(VcfRenderedCommandsReport, VcfEvidenceSnapshot)>,
}
impl<'a> VcfReadinessEvidenceSession<'a> {
    /// Bind repository and acquisition authorities without reading files or publishing outputs.
    #[must_use]
    pub const fn new(
        repo_root: &'a Path,
        run_id: &'a str,
        inputs: AdapterInputProvider,
        active: ActiveEvidenceProvider,
    ) -> Self {
        Self { repo_root, run_id, inputs, active, evidence: OnceCell::new() }
    }
    /// Borrow the same successful evidence observation for every stage in this request.
    ///
    /// # Errors
    /// Propagates input/active binding acquisition, semantic admission or publication failure.
    pub fn evidence(&self) -> Result<&(VcfRenderedCommandsReport, VcfEvidenceSnapshot)> {
        if self.evidence.get().is_none() {
            let inputs = (self.inputs)(self.repo_root)?;
            let observations = VcfAdapterObservations::new(&inputs, self.repo_root);
            let evidence = collect_vcf_readiness_evidence_with_observations(
                &observations,
                self.run_id,
                self.active,
            )?;
            self.evidence
                .set(evidence)
                .map_err(|_| anyhow!("VCF readiness evidence was initialized recursively"))?;
        }
        self.evidence.get().ok_or_else(|| anyhow!("VCF readiness evidence was not initialized"))
    }
}
