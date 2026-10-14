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
use anyhow::{anyhow, ensure, Context, Result};
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
    assess_bam_tool_definition, load_bam_domain_tool_definition, BamDomainToolDefinition,
    BamToolRegistryObservation,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
/// Definition results live for one serving request; failures keep canonical planner semantics.
pub(in crate::commands::benchmark::readiness) struct ServingAdapter {
    repo_root: PathBuf,
    registry: BamToolRegistryObservation,
    definitions: RefCell<BTreeMap<ToolId, Result<BamDomainToolDefinition>>>,
}
impl ServingAdapter {
    pub(in crate::commands::benchmark::readiness) fn new(repo_root: &Path) -> Self {
        Self {
            repo_root: repo_root.to_path_buf(),
            registry: BamToolRegistryObservation::new(repo_root),
            definitions: RefCell::new(BTreeMap::new()),
        }
    }

    pub(in crate::commands::benchmark::readiness) fn context(&self) -> ServingContext<'_> {
        ServingContext { repo_root: &self.repo_root, evidence: self }
    }

    fn ensure_owned_root(&self, root: &Path) -> Result<()> {
        ensure!(
            root == self.repo_root.as_path(),
            "serving observation belongs to repository root {}, requested {}",
            self.repo_root.display(),
            root.display()
        );
        Ok(())
    }
}
impl ServingEvidence for ServingAdapter {
    fn corpus_declarations(&self, root: &Path) -> Result<Vec<CorpusDeclaration>> {
        self.ensure_owned_root(root)?;
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
        self.ensure_owned_root(root)?;
        load_benchmark_integrated_container_tool_ids(root)
    }
    fn stage_inventory(&self, root: &Path, domain: Domain) -> Result<StageInventory> {
        self.ensure_owned_root(root)?;
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
    fn fastq_tools(&self, root: &Path, stage_id: &str) -> Result<Vec<String>> {
        self.ensure_owned_root(root)?;
        Ok(stage_tool_bindings_for_stage(&StageId::new(stage_id))
            .into_iter()
            .map(|binding| binding.tool_id.as_str().to_string())
            .collect())
    }
    fn fastq_assessment(
        &self,
        root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<FastqAssessment> {
        self.ensure_owned_root(root)?;
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
    fn bam_stage(&self, root: &Path, stage_id: &str) -> Result<BamStageAssessment> {
        self.ensure_owned_root(root)?;
        let stage = BamStage::try_from(stage_id)
            .with_context(|| format!("resolve BAM stage `{stage_id}`"))?;
        let completeness = bam_stage_completeness(stage);
        Ok(BamStageAssessment {
            tools: self
                .registry
                .allowed_tools_for_stage(stage)?
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
        self.ensure_owned_root(root)?;
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

#[cfg(test)]
mod tests {
    use super::ServingAdapter;
    use anyhow::{Context, Result};
    use bijux_dna_bench_bindings::ports::ServingEvidence;
    use std::path::Path;

    fn repository(source: &Path, destination: &Path, tool: &str) -> Result<()> {
        std::fs::create_dir_all(destination.join("crates/bijux-dna"))?;
        std::fs::create_dir_all(destination.join("configs/ci/registry"))?;
        for relative in ["Cargo.toml", "Cargo.lock", "crates/bijux-dna/Cargo.toml"] {
            std::fs::copy(source.join(relative), destination.join(relative))?;
        }
        std::fs::write(
            destination.join("configs/ci/registry/tool_registry.toml"),
            format!("[[tools]]\nid = '{tool}'\nstage_ids = ['bam.align']\n"),
        )?;
        Ok(())
    }

    #[test]
    fn serving_ports_refuse_cross_root_reuse_before_cached_registry_or_definitions() -> Result<()> {
        let source = crate::commands::support::workspace_root::resolve_repo_root()?;
        let alpha = tempfile::tempdir()?;
        let beta = tempfile::tempdir()?;
        repository(&source, alpha.path(), "request_alpha_tool")?;
        repository(&source, beta.path(), "request_beta_tool")?;
        let observed_alpha = ServingAdapter::new(alpha.path());
        let observed_beta = ServingAdapter::new(beta.path());
        assert_eq!(observed_alpha.context().repo_root, alpha.path());
        let alpha_stage = observed_alpha.bam_stage(alpha.path(), "bam.align")?;
        let beta_stage = observed_beta.bam_stage(beta.path(), "bam.align")?;
        assert!(alpha_stage.tools.iter().any(|tool| tool == "request_alpha_tool"));
        assert!(!alpha_stage.tools.iter().any(|tool| tool == "request_beta_tool"));
        assert!(beta_stage.tools.iter().any(|tool| tool == "request_beta_tool"));
        let error = observed_alpha
            .bam_stage(beta.path(), "bam.align")
            .err()
            .context("registry observation crossed source authority")?;
        assert!(error.to_string().contains("serving observation belongs to repository root"));
        let error = observed_alpha
            .bam_assessment(beta.path(), "bam.align", "bwa")
            .err()
            .context("definition observation crossed source authority")?;
        assert!(error.to_string().contains("serving observation belongs to repository root"));
        assert!(observed_alpha.fastq_tools(beta.path(), "fastq.validate_reads").is_err());
        assert!(observed_alpha.corpus_declarations(beta.path()).is_err());
        assert!(observed_alpha.integrated_tools(beta.path()).is_err());
        assert!(observed_alpha
            .stage_inventory(beta.path(), bijux_dna_bench_bindings::observations::Domain::Fastq)
            .is_err());
        assert!(observed_alpha
            .fastq_assessment(beta.path(), "fastq.validate_reads", "fastqc")
            .is_err());
        Ok(())
    }

    #[test]
    fn serving_fastq_demand_is_lazy_and_bam_registry_failure_remains_retryable() -> Result<()> {
        let source = crate::commands::support::workspace_root::resolve_repo_root()?;
        let root = tempfile::tempdir()?;
        let observed = ServingAdapter::new(root.path());
        assert!(!observed.fastq_tools(root.path(), "fastq.validate_reads")?.is_empty());
        assert!(observed.bam_stage(root.path(), "bam.align").is_err());
        repository(&source, root.path(), "request_repaired_tool")?;
        let registry = root.path().join("configs/ci/registry/tool_registry.toml");
        std::fs::write(&registry, "tools = 'invalid schema'\n")?;
        assert!(!observed.fastq_tools(root.path(), "fastq.validate_reads")?.is_empty());
        let error = observed
            .bam_stage(root.path(), "bam.align")
            .err()
            .context("BAM demand accepted a malformed tools declaration")?;
        assert!(format!("{error:#}").contains("must declare tools"));
        std::fs::write(
            &registry,
            "[[tools]]\nid = 'request_repaired_tool'\nstage_ids = ['bam.align']\n",
        )?;
        let first = observed.bam_stage(root.path(), "bam.align")?;
        assert!(first.tools.iter().any(|tool| tool == "request_repaired_tool"));
        std::fs::remove_file(&registry)?;
        assert_eq!(observed.bam_stage(root.path(), "bam.align")?.tools, first.tools);
        assert!(ServingAdapter::new(root.path()).bam_stage(root.path(), "bam.align").is_err());
        Ok(())
    }
}
