use anyhow::{anyhow, Result};
use bijux_dna_bench_hpc::dry_run::{
    ports::{DryRunEvidenceProvider, DryRunOutputPolicy},
    DryRunContext, DryRunInputPaths,
};
use bijux_dna_bench_model::{
    BenchmarkLocalHpcAssetStagingManifest, BenchmarkLocalHpcCandidateRunManifest,
    BenchmarkLocalHpcDependencySimulationReport, BenchmarkLocalHpcExecutionResolverReport,
    BenchmarkLocalHpcPipelineNodeArrayReport, BenchmarkLocalHpcResultCollectionSimulationReport,
    BenchmarkLocalHpcResumeSimulationReport, BenchmarkLocalHpcScratchLayout,
    BenchmarkLocalHpcStageBenchmarkArrayReport,
};
use std::{
    cell::{Cell, RefCell},
    path::Path,
};
#[derive(Default)]
pub struct Evidence {
    pub calls: RefCell<Vec<&'static str>>,
    pub generation: Cell<u32>,
}
impl Evidence {
    fn unavailable<T>(&self, provider: &'static str) -> Result<T> {
        self.calls.borrow_mut().push(provider);
        Err(anyhow!("{provider} unavailable at generation {}", self.generation.get()))
    }
}
impl DryRunEvidenceProvider for Evidence {
    fn asset(&self, _: &Path) -> Result<BenchmarkLocalHpcAssetStagingManifest> {
        self.unavailable("asset")
    }
    fn scratch(&self, _: &Path) -> Result<BenchmarkLocalHpcScratchLayout> {
        self.unavailable("scratch")
    }
    fn execution(&self, _: &Path) -> Result<BenchmarkLocalHpcExecutionResolverReport> {
        self.unavailable("execution")
    }
    fn stage_benchmark(&self, _: &Path) -> Result<BenchmarkLocalHpcStageBenchmarkArrayReport> {
        self.unavailable("stage_benchmark")
    }
    fn pipeline_node(&self, _: &Path) -> Result<BenchmarkLocalHpcPipelineNodeArrayReport> {
        self.unavailable("pipeline_node")
    }
    fn dependency(&self, _: &Path) -> Result<BenchmarkLocalHpcDependencySimulationReport> {
        self.unavailable("dependency")
    }
    fn resume(&self, _: &Path) -> Result<BenchmarkLocalHpcResumeSimulationReport> {
        self.unavailable("resume")
    }
    fn result_collection(
        &self,
        _: &Path,
    ) -> Result<BenchmarkLocalHpcResultCollectionSimulationReport> {
        self.unavailable("result_collection")
    }
    fn candidate(&self, _: &Path) -> Result<BenchmarkLocalHpcCandidateRunManifest> {
        self.unavailable("candidate")
    }
}
pub struct OutputPolicy;
impl DryRunOutputPolicy for OutputPolicy {
    fn admit(&self, root: &Path, output: &Path) -> Result<()> {
        if output.starts_with(root.join("reports")) {
            Ok(())
        } else {
            Err(anyhow!("outside admitted report root"))
        }
    }
    fn display(&self, root: &Path, output: &Path) -> String {
        output.strip_prefix(root).unwrap_or(output).to_string_lossy().replace('\\', "/")
    }
}
pub fn context<'a>(root: &'a Path, evidence: &'a Evidence) -> DryRunContext<'a> {
    DryRunContext {
        repo_root: root,
        evidence,
        output: &OutputPolicy,
        paths: DryRunInputPaths {
            asset: "provider/asset.json",
            scratch: "provider/scratch.json",
            execution: "provider/execution.json",
            stage_benchmark: "provider/stage_benchmark.json",
            pipeline_node: "provider/pipeline_node.json",
            dependency: "provider/dependency.json",
            resume: "provider/resume.json",
            result_collection: "provider/result_collection.json",
            candidate: "provider/candidate.json",
        },
    }
}
