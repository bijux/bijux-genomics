use std::collections::BTreeMap;

use anyhow::{bail, Result};
use bijux_dna_core::ids::{StageId, ToolId};

use super::{validate_tool_binding, ToolBindingV1};

/// Deterministic registry keyed by the explicit stage/tool pair.
#[derive(Debug, Clone, Default)]
pub struct ToolBindingRegistry {
    bindings: BTreeMap<(StageId, ToolId), ToolBindingV1>,
}

impl ToolBindingRegistry {
    /// Adds one conforming binding and refuses duplicate stage/tool ownership.
    ///
    /// # Errors
    ///
    /// Returns an error when the binding is invalid or its stage/tool pair is already registered.
    pub fn insert(&mut self, binding: ToolBindingV1) -> Result<()> {
        validate_tool_binding(&binding)?;
        let key = (binding.stage_id.clone(), binding.tool_id.clone());
        if self.bindings.contains_key(&key) {
            bail!("duplicate tool binding for stage `{}` and tool `{}`", key.0, key.1);
        }
        self.bindings.insert(key, binding);
        Ok(())
    }

    /// Returns the binding for an exact stage/tool pair.
    #[must_use]
    pub fn binding(&self, stage_id: &StageId, tool_id: &ToolId) -> Option<&ToolBindingV1> {
        self.bindings.get(&(stage_id.clone(), tool_id.clone()))
    }

    /// Iterates in stable stage/tool order.
    #[must_use]
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = &ToolBindingV1> {
        self.bindings.values()
    }

    /// Returns every stage binding for a tool without assuming one-tool/one-stage ownership.
    #[must_use]
    pub fn bindings_for_tool(&self, tool_id: &ToolId) -> Vec<&ToolBindingV1> {
        self.bindings
            .iter()
            .filter_map(|((_, registered_tool), binding)| {
                (registered_tool == tool_id).then_some(binding)
            })
            .collect()
    }

    /// Returns every tool binding for a stage in stable tool order.
    #[must_use]
    pub fn bindings_for_stage(&self, stage_id: &StageId) -> Vec<&ToolBindingV1> {
        self.bindings
            .iter()
            .filter_map(|((registered_stage, _), binding)| {
                (registered_stage == stage_id).then_some(binding)
            })
            .collect()
    }
}
