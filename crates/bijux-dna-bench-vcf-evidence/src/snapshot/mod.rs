//! Admit once and select multiple stages without filesystem or process effects.
mod admission;
mod identity;
mod rows;
use anyhow::{anyhow, Result};
pub use identity::VcfStageReadinessBindingKey;
pub use rows::{ActiveVcfBinding, VcfEvidenceRows, VcfStageReadinessBinding};
use std::collections::BTreeMap;

/// Immutable, unambiguous evidence indexed by stage and full corpus/asset binding.
#[derive(Debug, Clone)]
pub struct VcfEvidenceSnapshot {
    stages: BTreeMap<String, Vec<VcfStageReadinessBinding>>,
}

impl VcfEvidenceSnapshot {
    /// Validate all identities and joins before exposing any stage observation.
    ///
    /// # Errors
    /// Rejects duplicate, foreign or contradictory evidence. Missing surfaces remain observable.
    pub fn admit(rows: VcfEvidenceRows) -> Result<Self> {
        Ok(Self { stages: admission::admit(rows)? })
    }

    /// Select an already admitted stage without reacquiring or regenerating evidence.
    ///
    /// # Errors
    /// Returns an error when no retained binding names this stage.
    pub fn stage_bindings(&self, stage_id: &str) -> Result<&[VcfStageReadinessBinding]> {
        self.stages
            .get(stage_id)
            .map(Vec::as_slice)
            .ok_or_else(|| anyhow!("VCF stage readiness is missing retained `{stage_id}` bindings"))
    }
}
