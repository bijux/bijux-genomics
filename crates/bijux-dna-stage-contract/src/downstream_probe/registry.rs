use std::collections::BTreeMap;

use anyhow::{bail, Result};
use bijux_dna_core::ids::StageId;

use super::{
    DownstreamProbe, DownstreamProbeAssertionV1, DownstreamProbeV1, DOWNSTREAM_PROBE_SCHEMA_V1,
};
use crate::{canonical_tool_bindings, ToolBinding};

#[derive(Debug, Clone, Default)]
pub struct DownstreamProbeRegistry {
    by_stage: BTreeMap<StageId, DownstreamProbeV1>,
    stage_by_probe: BTreeMap<String, StageId>,
}

impl DownstreamProbeRegistry {
    /// Adds one stage-owned consumer probe and refuses ambiguous ownership.
    ///
    /// # Errors
    /// Returns an error when the probe is malformed or reuses a stage/probe identity.
    pub fn insert(&mut self, probe: DownstreamProbeV1) -> Result<()> {
        validate_probe(&probe)?;
        if self.by_stage.contains_key(probe.producer_stage_id()) {
            bail!("duplicate downstream probe for stage `{}`", probe.producer_stage_id());
        }
        if let Some(stage_id) = self.stage_by_probe.get(probe.probe_id()) {
            bail!("downstream probe `{}` is already owned by stage `{stage_id}`", probe.probe_id());
        }
        self.stage_by_probe.insert(probe.probe_id.clone(), probe.producer_stage_id.clone());
        self.by_stage.insert(probe.producer_stage_id.clone(), probe);
        Ok(())
    }

    #[must_use]
    pub fn probe_for_stage(&self, stage_id: &StageId) -> Option<&DownstreamProbeV1> {
        self.by_stage.get(stage_id)
    }

    #[must_use]
    pub fn probe(&self, probe_id: &str) -> Option<&DownstreamProbeV1> {
        self.stage_by_probe.get(probe_id).and_then(|stage_id| self.by_stage.get(stage_id))
    }

    #[must_use]
    pub fn probes(&self) -> impl ExactSizeIterator<Item = &DownstreamProbeV1> {
        self.by_stage.values()
    }
}

/// Derives one tool-independent consumer probe for every canonical benchmark stage.
///
/// # Errors
/// Returns an error if tools bound to one stage disagree about their normalized output or probe.
pub fn canonical_downstream_probes() -> Result<DownstreamProbeRegistry> {
    let bindings = canonical_tool_bindings()?;
    downstream_probes_for_bindings(&bindings)
}

/// Derives and validates tool-independent probes for an explicit binding registry.
///
/// # Errors
/// Returns an error if tools bound to one stage disagree about their normalized output or probe.
pub fn downstream_probes_for_bindings(
    bindings: &crate::ToolBindingRegistry,
) -> Result<DownstreamProbeRegistry> {
    let mut declarations = BTreeMap::<StageId, (String, String)>::new();
    for binding in bindings.bindings() {
        let declaration = (
            binding.downstream_probe_id().to_string(),
            binding.normalized_output_contract_id().to_string(),
        );
        if let Some(existing) = declarations.insert(binding.stage_id().clone(), declaration.clone())
        {
            if existing != declaration {
                bail!(
                    "stage `{}` bindings disagree about downstream consumer semantics: existing={existing:?}, binding {}={declaration:?}",
                    binding.stage_id(),
                    binding.binding_id()
                );
            }
        }
    }

    let mut registry = DownstreamProbeRegistry::default();
    for (producer_stage_id, (probe_id, input_contract_id)) in declarations {
        registry.insert(DownstreamProbeV1 {
            schema_version: DOWNSTREAM_PROBE_SCHEMA_V1.to_string(),
            probe_id,
            producer_stage_id,
            input_contract_id,
            assertions: vec![
                DownstreamProbeAssertionV1::NormalizedIdentity,
                DownstreamProbeAssertionV1::ScientificMetrics,
                DownstreamProbeAssertionV1::DeclaredArtifacts,
                DownstreamProbeAssertionV1::ScientificInvariants,
            ],
        })?;
    }
    Ok(registry)
}

pub(super) fn validate_probe(probe: &impl DownstreamProbe) -> Result<()> {
    if probe.schema_version() != DOWNSTREAM_PROBE_SCHEMA_V1 {
        bail!("unsupported downstream probe schema `{}`", probe.schema_version());
    }
    for (value, field) in [
        (probe.probe_id(), "probe_id"),
        (probe.producer_stage_id().as_str(), "producer_stage_id"),
        (probe.input_contract_id(), "input_contract_id"),
    ] {
        if value.trim().is_empty() {
            bail!("downstream probe {field} must not be empty");
        }
    }
    let expected = [
        DownstreamProbeAssertionV1::NormalizedIdentity,
        DownstreamProbeAssertionV1::ScientificMetrics,
        DownstreamProbeAssertionV1::DeclaredArtifacts,
        DownstreamProbeAssertionV1::ScientificInvariants,
    ];
    if probe.assertions() != expected {
        bail!("downstream probe must declare the complete semantic assertion set");
    }
    Ok(())
}
