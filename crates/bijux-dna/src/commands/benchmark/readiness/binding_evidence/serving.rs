use super::super::{
    active_scope::include_fastq_active_benchmark_pair,
    catalog::load_benchmark_integrated_container_tool_ids,
};
use crate::commands::benchmark::{
    local_corpus_stage_compatibility::{
        load_corpus_stage_compatibility_declarations, DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
    },
    local_stage_inventory::{load_local_stage_inventory, BenchLocalDomain},
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_bindings::{
    observations::{
        BamStageAssessment, BamToolAssessment, CorpusDeclaration, Domain, FastqAssessment,
        StageInventory,
    },
    ports::ServingEvidence,
    ServingContext,
};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::{
    bam_scientific_report_contract_for_stage, bam_stage_completeness, BamStage,
};
use bijux_dna_domain_fastq::{
    benchmark_readiness_for_stage_tool, stage_tool_bindings_for_stage,
    stage_tool_capability_contract, BenchmarkReadinessLevel, RuntimeNormalizationLevel,
};
use bijux_dna_planner_bam::stage_api::{
    allowed_tools_for_stage, assess_bam_tool_definition, load_bam_domain_tool_definition,
    BamDomainToolDefinition,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
/// Definition results live for one serving request; failures keep canonical planner semantics.
#[derive(Default)]
pub(in crate::commands::benchmark::readiness) struct ServingAdapter {
    definitions: RefCell<BTreeMap<ToolId, Result<BamDomainToolDefinition>>>,
}
impl ServingAdapter {
    pub(in crate::commands::benchmark::readiness) fn context<'a>(
        &'a self,
        repo_root: &'a Path,
    ) -> ServingContext<'a> {
        ServingContext { repo_root, evidence: self }
    }
}
impl ServingEvidence for ServingAdapter {
    fn corpus_declarations(&self, root: &Path) -> Result<Vec<CorpusDeclaration>> {
        Ok(load_corpus_stage_compatibility_declarations(
            &root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
        )?
        .into_iter()
        .map(|row| CorpusDeclaration {
            stage_id: row.stage_id,
            compatibility_kind: row.compatibility_kind,
            fixture_id: row.fixture_id,
            benchmark_scope_id: row.benchmark_scope_id,
        })
        .collect())
    }
    fn integrated_tools(&self, root: &Path) -> Result<BTreeSet<String>> {
        load_benchmark_integrated_container_tool_ids(root)
    }
    fn stage_inventory(&self, root: &Path, domain: Domain) -> Result<StageInventory> {
        let inventory = load_local_stage_inventory(
            root,
            match domain {
                Domain::Fastq => BenchLocalDomain::Fastq,
                Domain::Bam => BenchLocalDomain::Bam,
            },
        )?;
        Ok(StageInventory {
            stage_count: inventory.stage_count,
            stages: inventory.stages.into_iter().map(|stage| stage.stage_id).collect(),
        })
    }
    fn fastq_tools(&self, _root: &Path, stage_id: &str) -> Result<Vec<String>> {
        Ok(stage_tool_bindings_for_stage(&StageId::new(stage_id))
            .into_iter()
            .map(|binding| binding.tool_id.as_str().to_string())
            .collect())
    }
    fn fastq_assessment(
        &self,
        _root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<FastqAssessment> {
        let stage = StageId::new(stage_id);
        let tool = ToolId::new(tool_id);
        let normalization = runtime_normalization_for_stage_tool(&stage, &tool);
        let capability =
            stage_tool_capability_contract(&stage, &tool, normalization).ok_or_else(|| {
                anyhow!("missing FASTQ capability contract for `{stage_id}` / `{tool_id}`")
            })?;
        let readiness = benchmark_readiness_for_stage_tool(&stage, &tool, normalization)
            .ok_or_else(|| {
                anyhow!("missing FASTQ benchmark readiness for `{stage_id}` / `{tool_id}`")
            })?;
        Ok(FastqAssessment {
            readiness: benchmark_readiness_label(readiness),
            active: include_fastq_active_benchmark_pair(stage_id, tool_id),
            runnable: capability.runnable,
            plannable: capability.plannable,
            declared: capability.declared,
            comparable: capability.comparable,
            benchmark_normalized: capability.benchmark_normalized,
            parse_normalized: capability.parse_normalized,
        })
    }
    fn bam_stage(&self, _root: &Path, stage_id: &str) -> Result<BamStageAssessment> {
        let stage = BamStage::try_from(stage_id)
            .with_context(|| format!("resolve BAM stage `{stage_id}`"))?;
        let completeness = bam_stage_completeness(stage);
        Ok(BamStageAssessment {
            tools: allowed_tools_for_stage(stage)
                .into_iter()
                .map(|tool| tool.as_str().to_string())
                .collect(),
            has_parser_fixtures: completeness.has_parser_fixtures,
            has_scientific_report_contract: bam_scientific_report_contract_for_stage(
                stage.as_str(),
            )
            .is_some(),
            has_artifact_contract: completeness.has_artifact_contract,
        })
    }
    fn bam_assessment(
        &self,
        root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<BamToolAssessment> {
        let tool = ToolId::new(tool_id);
        let mut definitions = self.definitions.borrow_mut();
        let definition = definitions
            .entry(tool.clone())
            .or_insert_with(|| load_bam_domain_tool_definition(root, &tool))
            .as_ref()
            .ok();
        let readiness = assess_bam_tool_definition(definition, &StageId::new(stage_id));
        Ok(BamToolAssessment {
            support: readiness.support.as_str(),
            adapter: readiness.adapter.as_str(),
        })
    }
}
fn runtime_normalization_for_stage_tool(
    stage_id: &StageId,
    tool_id: &bijux_dna_core::ids::ToolId,
) -> RuntimeNormalizationLevel {
    match bijux_dna_stages_fastq::runtime_interpretation_for_stage_tool(stage_id, tool_id)
        .unwrap_or(bijux_dna_stages_fastq::RuntimeInterpretationLevel::GenericEnvelope)
    {
        bijux_dna_stages_fastq::RuntimeInterpretationLevel::GenericEnvelope => {
            RuntimeNormalizationLevel::GenericEnvelope
        }
        bijux_dna_stages_fastq::RuntimeInterpretationLevel::ObserverSpecialized => {
            RuntimeNormalizationLevel::ObserverSpecialized
        }
    }
}
fn benchmark_readiness_label(readiness: BenchmarkReadinessLevel) -> &'static str {
    match readiness {
        BenchmarkReadinessLevel::PlannedContract => "planned_contract",
        BenchmarkReadinessLevel::GovernedExecution => "governed_execution",
        BenchmarkReadinessLevel::GovernedBenchmarkCohort => "governed_benchmark_cohort",
        BenchmarkReadinessLevel::ObserverSpecializedBenchmark => "observer_specialized_benchmark",
    }
}
