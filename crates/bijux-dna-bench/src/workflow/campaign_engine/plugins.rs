//! Owner: bijux-dna-bench
//! Dependency-injected stage-plugin lifecycle over canonical tool-binding authority.

use std::collections::BTreeSet;

use anyhow::{bail, Context, Result};
use bijux_dna_core::ids::StageId;
use bijux_dna_stage_contract::{
    downstream_probes_for_bindings, evaluate_downstream_probe, ArtifactRef,
    DownstreamProbeEvaluation, StageInvocationV1, StagePlanV1, StagePlugin, StagePluginOutputV1,
    ToolBinding, ToolBindingRegistry,
};

pub struct BenchmarkStageNormalization {
    pub output: StagePluginOutputV1,
    pub downstream: DownstreamProbeEvaluation,
}

pub struct BenchmarkStagePluginRegistry<'a> {
    plugins: Vec<&'a dyn StagePlugin>,
}

impl<'a> BenchmarkStagePluginRegistry<'a> {
    #[must_use]
    pub fn new(plugins: Vec<&'a dyn StagePlugin>) -> Self {
        Self { plugins }
    }

    /// Validates that every binding-backed stage resolves to exactly one injected plugin.
    ///
    /// # Errors
    /// Returns an error when a stage has missing or ambiguous plugin ownership.
    pub fn validate_for_bindings(&self, bindings: &ToolBindingRegistry) -> Result<()> {
        let stages =
            bindings.bindings().map(|binding| binding.stage_id().clone()).collect::<BTreeSet<_>>();
        for stage_id in stages {
            self.plugin_for_stage(&stage_id)?;
        }
        Ok(())
    }

    /// Materializes one binding-backed invocation through its domain plugin.
    ///
    /// # Errors
    /// Returns an error when binding authority, plan outputs, or plugin ownership do not close.
    pub fn materialize(
        &self,
        bindings: &ToolBindingRegistry,
        plan: &StagePlanV1,
    ) -> Result<StageInvocationV1> {
        let binding = binding_for_plan(bindings, plan)?;
        validate_plan_outputs(binding, plan)?;
        self.plugin_for_stage(&plan.stage_id)?.materialize(plan)
    }

    /// Normalizes one collected output set and evaluates its stage-owned downstream probe.
    ///
    /// Scientific failures remain in the returned evaluation. Authority failures are errors.
    ///
    /// # Errors
    /// Returns an error when binding, plugin, normalized artifact, or probe authority differs.
    pub fn normalize(
        &self,
        bindings: &ToolBindingRegistry,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<BenchmarkStageNormalization> {
        let binding = binding_for_plan(bindings, plan)?;
        validate_plan_outputs(binding, plan)?;
        let output = self.plugin_for_stage(&plan.stage_id)?.parse_outputs(plan, outputs)?;
        let probes = downstream_probes_for_bindings(bindings)?;
        let probe = probes
            .probe_for_stage(&plan.stage_id)
            .with_context(|| format!("stage {} has no downstream probe", plan.stage_id))?;
        let downstream = evaluate_downstream_probe(probe, binding, &output)?;
        Ok(BenchmarkStageNormalization { output, downstream })
    }

    fn plugin_for_stage(&self, stage_id: &StageId) -> Result<&'a dyn StagePlugin> {
        let mut matching =
            self.plugins.iter().copied().filter(|plugin| plugin.handles_stage(stage_id.as_str()));
        let plugin = matching
            .next()
            .with_context(|| format!("stage {stage_id} has no injected stage plugin"))?;
        if matching.next().is_some() {
            bail!("stage {stage_id} has ambiguous injected stage plugin ownership");
        }
        Ok(plugin)
    }
}

fn binding_for_plan<'a>(
    bindings: &'a ToolBindingRegistry,
    plan: &StagePlanV1,
) -> Result<&'a bijux_dna_stage_contract::ToolBindingV1> {
    bindings.binding(&plan.stage_id, &plan.tool_id).with_context(|| {
        format!("plan has no admitted binding: {} / {}", plan.stage_id, plan.tool_id)
    })
}

fn validate_plan_outputs(binding: &impl ToolBinding, plan: &StagePlanV1) -> Result<()> {
    let planned = plan
        .io
        .outputs
        .iter()
        .map(|artifact| (artifact.name.as_str(), artifact.artifact_contract_id.as_deref()))
        .collect::<BTreeSet<_>>();
    let declared = binding
        .outputs()
        .iter()
        .map(|output| (output.name.as_str(), Some(output.artifact_contract_id.as_str())))
        .collect::<BTreeSet<_>>();
    if plan.io.outputs.len() != binding.outputs().len() || planned != declared {
        bail!(
            "stage plan outputs differ from binding authority: {} / {}",
            plan.stage_id,
            plan.tool_id
        );
    }
    Ok(())
}
