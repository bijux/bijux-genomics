use anyhow::{anyhow, Result};
use bijux_dna_bench_hpc::candidate::{
    ports::{CandidateEvidenceProvider, CandidateResultPaths},
    CandidateContext,
};
use bijux_dna_bench_model::{
    BenchmarkLocalAllDomainSlurmSubmitJob, BenchmarkLocalHpcAssetStagingInput,
    BenchmarkLocalHpcAssetStagingJob, BenchmarkLocalHpcAssetStagingManifest,
    BenchmarkLocalHpcExecutionResolverReport, BenchmarkLocalHpcExecutionResolverRow,
    BenchmarkLocalHpcScratchJobResources, BenchmarkLocalSlurmSubmitResources, HpcJobResourceHints,
    BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1, BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
pub struct Evidence {
    pub jobs: Vec<BenchmarkLocalAllDomainSlurmSubmitJob>,
    pub assets: BenchmarkLocalHpcAssetStagingManifest,
    pub execution: BenchmarkLocalHpcExecutionResolverReport,
    pub resources: HpcJobResourceHints,
    pub calls: RefCell<Vec<&'static str>>,
    pub path_calls: RefCell<Vec<String>>,
    pub reject_paths: bool,
}
pub fn evidence() -> Evidence {
    let jobs = vec![job("bam", "single"), job("fastq", "large"), job("fastq", "small")];
    let assets = BenchmarkLocalHpcAssetStagingManifest {
        schema_version: BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1.into(),
        output_path: "reports/assets.json".into(),
        staging_root: "artifacts/staged".into(),
        selected_job_count: 3,
        staged_input_count: 3,
        unique_source_path_count: 3,
        jobs: jobs
            .iter()
            .zip([3, 20, 2])
            .map(|(job, bytes)| BenchmarkLocalHpcAssetStagingJob {
                result_id: job.job_id_local.clone(),
                domain: job.domain.clone(),
                stage_id: job.stage_id.clone(),
                tool_id: job.tool_id.clone(),
                corpus_id: job.corpus_id.clone(),
                asset_profile_id: job.asset_profile_id.clone(),
                staged_inputs: vec![input(&job.job_id_local, bytes)],
            })
            .collect(),
    };
    let rows =
        ["bam", "fastq"].into_iter().map(|domain| execution_row(domain, &jobs)).collect::<Vec<_>>();
    let execution = BenchmarkLocalHpcExecutionResolverReport {
        schema_version: BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1.into(),
        output_path: "reports/execution.tsv".into(),
        row_count: 2,
        resolution_counts: BTreeMap::from([("host_binary".into(), 2)]),
        unclassified_tool_count: 0,
        rows,
    };
    let resources = jobs
        .iter()
        .map(|job| {
            (
                (job.domain.clone(), job.stage_id.clone(), job.tool_id.clone()),
                BenchmarkLocalHpcScratchJobResources {
                    cpus_per_task: 1,
                    memory_mb: 1024,
                    time_limit: "00:10:00".into(),
                    scratch_gb: 1,
                },
            )
        })
        .collect();
    Evidence {
        jobs,
        assets,
        execution,
        resources,
        calls: RefCell::default(),
        path_calls: RefCell::default(),
        reject_paths: false,
    }
}
fn job(domain: &str, label: &str) -> BenchmarkLocalAllDomainSlurmSubmitJob {
    let id = format!("{domain}-{label}");
    BenchmarkLocalAllDomainSlurmSubmitJob {
        job_id_local: id.clone(),
        domain: domain.into(),
        stage_id: format!("{domain}.stage"),
        pipeline_id: None,
        node_id: None,
        tool_id: format!("{domain}-tool"),
        corpus_id: "small-inputs".into(),
        asset_profile_id: "local".into(),
        result_id: Some(id.clone()),
        script_path: format!("artifacts/jobs/{id}/run.sh"),
        stdout: format!("artifacts/jobs/{id}/stdout.log"),
        stderr: format!("artifacts/jobs/{id}/stderr.log"),
        outputs: vec![format!("artifacts/jobs/{id}/output.dat")],
        dependencies: Vec::new(),
        resources: BenchmarkLocalSlurmSubmitResources {
            cpus_per_task: 1,
            memory_mb: 1024,
            time_limit: "00:10:00".into(),
        },
    }
}
pub fn input(id: &str, bytes: u64) -> BenchmarkLocalHpcAssetStagingInput {
    let source_path = format!("fixtures/{id}.dat");
    BenchmarkLocalHpcAssetStagingInput {
        artifact_id: Some("input".into()),
        artifact_role: Some("reads".into()),
        source_kind: "file".into(),
        staged_path: format!("artifacts/staged/{source_path}"),
        source_path,
        checksum_sha256: "a".repeat(64),
        size_bytes: bytes,
        member_count: 1,
        consumer_result_id: id.into(),
    }
}
fn execution_row(
    domain: &str,
    jobs: &[BenchmarkLocalAllDomainSlurmSubmitJob],
) -> BenchmarkLocalHpcExecutionResolverRow {
    let selected = jobs.iter().filter(|job| job.domain == domain).collect::<Vec<_>>();
    let ids = selected
        .iter()
        .map(|job| job.job_id_local.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    BenchmarkLocalHpcExecutionResolverRow {
        tool_id: format!("{domain}-tool"),
        lookup_tool_id: format!("{domain}-tool"),
        execution_mode: "host".into(),
        execution_mode_summary: "local host fixture".into(),
        domains: vec![domain.into()],
        selected_stage_ids: vec![format!("{domain}.stage")],
        selected_job_count: ids.len(),
        selected_job_ids: ids.clone(),
        selected_result_ids: ids,
        selected_pipeline_ids: Vec::new(),
        selected_node_ids: Vec::new(),
        install_kind: "host".into(),
        resolution_kind: "host_binary".into(),
        resolution_target: format!("tools/{domain}-tool"),
        command_entrypoint: Some(format!("{domain}-tool")),
        runtime_probe_paths: Vec::new(),
        registry_paths: Vec::new(),
        unavailable_reason: None,
    }
}
pub fn context<'a>(root: &'a Path, evidence: &'a Evidence) -> CandidateContext<'a> {
    CandidateContext { repo_root: root, evidence }
}
impl CandidateEvidenceProvider for Evidence {
    fn assets(&self, _: &Path) -> Result<BenchmarkLocalHpcAssetStagingManifest> {
        self.calls.borrow_mut().push("assets");
        Ok(self.assets.clone())
    }
    fn execution(&self, _: &Path) -> Result<BenchmarkLocalHpcExecutionResolverReport> {
        self.calls.borrow_mut().push("execution");
        Ok(self.execution.clone())
    }
    fn jobs(&self, _: &Path) -> Result<Vec<BenchmarkLocalAllDomainSlurmSubmitJob>> {
        self.calls.borrow_mut().push("jobs");
        Ok(self.jobs.clone())
    }
    fn resources(&self, _: &Path) -> Result<HpcJobResourceHints> {
        self.calls.borrow_mut().push("resources");
        Ok(self.resources.clone())
    }
    fn result_paths(
        &self,
        root: &Path,
        job: &BenchmarkLocalAllDomainSlurmSubmitJob,
    ) -> Result<CandidateResultPaths> {
        self.path_calls.borrow_mut().push(job.job_id_local.clone());
        if self.reject_paths {
            return Err(anyhow!("inconsistent result destinations"));
        }
        let result_root = root.join("artifacts/jobs").join(&job.job_id_local);
        Ok(CandidateResultPaths {
            stage_result_manifest_path: result_root.join("stage-result.json"),
            result_root,
        })
    }
    fn admit_output(&self, root: &Path, output: &Path) -> Result<()> {
        if output.starts_with(root.join("reports")) {
            Ok(())
        } else {
            Err(anyhow!("outside admitted report root"))
        }
    }
}
