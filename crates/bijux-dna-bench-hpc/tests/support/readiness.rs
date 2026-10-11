use anyhow::{anyhow, Result};
use bijux_dna_bench_hpc::readiness::{
    ports::{
        AmpliconFixtureObservation, CampaignPlanObservation, CorpusEvidence,
        CorpusSamplesObservation, DamageFixtureObservation, EdnaFixtureObservation,
        FoundationPreparationObservation, HarnessEvidence, LocalCampaignEvidence,
        MockSubmissionObservation, PipelineEvidence, ReadinessDomain, SlurmArtifactEvidence,
        StageEvidence, TaxonomyFixtureObservation, ToolComparisonObservation,
    },
    ReadinessInputPaths,
};
use bijux_dna_bench_hpc::watchdog::{LocalDagWatchdogScenario, LocalDagWatchdogSimulationReport};
use bijux_dna_bench_model::{
    BenchmarkLocalCorpusSkipReport, BenchmarkLocalCorpusStageCompatibilityReport,
    BenchmarkLocalPipelineDagValidationReport, BenchmarkLocalSlurmDependencyCheckReport,
    BenchmarkLocalSlurmDryRunReport, BenchmarkLocalSlurmScriptBodyReport,
    BenchmarkLocalSlurmShellSyntaxReport, BenchmarkLocalSlurmSubmitManifest,
    BenchmarkLocalStageCommandManifest, BenchmarkLocalStageFakeFailureManifest,
    BenchmarkLocalStageFakeRunManifest, BenchmarkLocalStageInventory,
    BenchmarkLocalStageManifestCompletionReport, BenchmarkLocalStageOutputCompletionReport,
    BenchmarkLocalStageRuntimeMetricsReport, BenchmarkLocalSummaryReport,
    BenchmarkStageResultManifest,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
#[derive(Default)]
pub(super) struct UnavailableEvidence {
    pub(super) calls: RefCell<Vec<&'static str>>,
}
impl UnavailableEvidence {
    fn unavailable<T>(&self, method: &'static str) -> Result<T> {
        self.calls.borrow_mut().push(method);
        Err(anyhow!("provider unavailable: {method}"))
    }
}
impl StageEvidence for UnavailableEvidence {
    fn load_local_stage_inventory(
        &self,
        _repo_root: &Path,
        _domain: ReadinessDomain,
    ) -> Result<BenchmarkLocalStageInventory> {
        self.unavailable("load_local_stage_inventory")
    }
    fn materialize_local_stage(&self, _repo_root: &Path, _stage_id: &str) -> Result<PathBuf> {
        self.unavailable("materialize_local_stage")
    }
}
impl HarnessEvidence for UnavailableEvidence {
    fn render_local_stage_commands(
        &self,
        _repo_root: &Path,
        _output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageCommandManifest> {
        self.unavailable("render_local_stage_commands")
    }
    fn fake_run_local_stage_commands(
        &self,
        _repo_root: &Path,
        _output_root: PathBuf,
    ) -> Result<BenchmarkLocalStageFakeRunManifest> {
        self.unavailable("fake_run_local_stage_commands")
    }
    fn fake_run_local_stage_failures(
        &self,
        _repo_root: &Path,
        _output_root: PathBuf,
        _stage_ids: &[String],
        _exit_code: i32,
    ) -> Result<BenchmarkLocalStageFakeFailureManifest> {
        self.unavailable("fake_run_local_stage_failures")
    }
    fn check_local_stage_output_completion(
        &self,
        _repo_root: &Path,
        _fake_run_root: PathBuf,
        _report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageOutputCompletionReport> {
        self.unavailable("check_local_stage_output_completion")
    }
    fn check_local_stage_manifest_completion(
        &self,
        _repo_root: &Path,
        _fake_run_root: PathBuf,
        _report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageManifestCompletionReport> {
        self.unavailable("check_local_stage_manifest_completion")
    }
    fn load_validated_stage_result_manifest_path(
        &self,
        _manifest_path: &Path,
    ) -> Result<BenchmarkStageResultManifest> {
        self.unavailable("load_validated_stage_result_manifest_path")
    }
    fn collect_local_stage_runtime_metrics(
        &self,
        _repo_root: &Path,
        _fake_run_root: PathBuf,
        _report_output_path: PathBuf,
    ) -> Result<BenchmarkLocalStageRuntimeMetricsReport> {
        self.unavailable("collect_local_stage_runtime_metrics")
    }
    fn render_local_tool_comparison_template(
        &self,
        _repo_root: &Path,
        _fake_run_root: PathBuf,
        _output_path: PathBuf,
    ) -> Result<ToolComparisonObservation> {
        self.unavailable("render_local_tool_comparison_template")
    }
    fn render_local_benchmark_summary(
        &self,
        _repo_root: &Path,
        _fake_run_root: PathBuf,
        _json_output_path: PathBuf,
        _markdown_output_path: PathBuf,
    ) -> Result<BenchmarkLocalSummaryReport> {
        self.unavailable("render_local_benchmark_summary")
    }
}
impl CorpusEvidence for UnavailableEvidence {
    fn validate_fastq_corpus_fixture_manifest_path(
        &self,
        _repo_root: &Path,
        _manifest_path: &Path,
    ) -> Result<CorpusSamplesObservation> {
        self.unavailable("validate_fastq_corpus_fixture_manifest_path")
    }
    fn validate_bam_corpus_fixture_manifest_path(
        &self,
        _repo_root: &Path,
        _manifest_path: &Path,
    ) -> Result<CorpusSamplesObservation> {
        self.unavailable("validate_bam_corpus_fixture_manifest_path")
    }
    fn validate_bam_damage_fixture_manifest_path(
        &self,
        _repo_root: &Path,
        _manifest_path: &Path,
    ) -> Result<DamageFixtureObservation> {
        self.unavailable("validate_bam_damage_fixture_manifest_path")
    }
    fn validate_edna_corpus_fixture_manifest_path(
        &self,
        _repo_root: &Path,
        _manifest_path: &Path,
    ) -> Result<EdnaFixtureObservation> {
        self.unavailable("validate_edna_corpus_fixture_manifest_path")
    }
    fn validate_amplicon_corpus_fixture_manifest_path(
        &self,
        _repo_root: &Path,
        _manifest_path: &Path,
    ) -> Result<AmpliconFixtureObservation> {
        self.unavailable("validate_amplicon_corpus_fixture_manifest_path")
    }
    fn validate_taxonomy_database_fixture_manifest_path(
        &self,
        _repo_root: &Path,
        _manifest_path: &Path,
    ) -> Result<TaxonomyFixtureObservation> {
        self.unavailable("validate_taxonomy_database_fixture_manifest_path")
    }
    fn validate_corpus_stage_compatibility_path(
        &self,
        _repo_root: &Path,
        _matrix_path: &Path,
    ) -> Result<BenchmarkLocalCorpusStageCompatibilityReport> {
        self.unavailable("validate_corpus_stage_compatibility_path")
    }
    fn render_corpus_skip_report_path(
        &self,
        _repo_root: &Path,
        _matrix_path: &Path,
        _output_path: &Path,
    ) -> Result<BenchmarkLocalCorpusSkipReport> {
        self.unavailable("render_corpus_skip_report_path")
    }
}
impl PipelineEvidence for UnavailableEvidence {
    fn validate_pipeline_dag_path(
        &self,
        _repo_root: &Path,
        _config_path: &Path,
        _output_path: &Path,
    ) -> Result<BenchmarkLocalPipelineDagValidationReport> {
        self.unavailable("validate_pipeline_dag_path")
    }
    fn simulate_dag_watchdog_path(
        &self,
        _repo_root: &Path,
        _scenario: LocalDagWatchdogScenario,
        _output_path: &Path,
    ) -> Result<LocalDagWatchdogSimulationReport> {
        self.unavailable("simulate_dag_watchdog_path")
    }
}
impl SlurmArtifactEvidence for UnavailableEvidence {
    fn render_local_slurm_scripts(
        &self,
        _repo_root: &Path,
        _domain: ReadinessDomain,
        _output_root: PathBuf,
    ) -> Result<BenchmarkLocalSlurmDryRunReport> {
        self.unavailable("render_local_slurm_scripts")
    }
    fn validate_slurm_script_bodies(
        &self,
        _repo_root: &Path,
        _root_path: PathBuf,
        _report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmScriptBodyReport> {
        self.unavailable("validate_slurm_script_bodies")
    }
    fn validate_slurm_shell_syntax(
        &self,
        _repo_root: &Path,
        _root_path: PathBuf,
        _report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmShellSyntaxReport> {
        self.unavailable("validate_slurm_shell_syntax")
    }
    fn render_slurm_submit_manifest(
        &self,
        _repo_root: &Path,
        _root_path: PathBuf,
        _output_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmSubmitManifest> {
        self.unavailable("render_slurm_submit_manifest")
    }
    fn validate_slurm_dependencies(
        &self,
        _repo_root: &Path,
        _root_path: PathBuf,
        _manifest_path: PathBuf,
        _report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmDependencyCheckReport> {
        self.unavailable("validate_slurm_dependencies")
    }
}
impl LocalCampaignEvidence for UnavailableEvidence {
    fn campaign_dry_run(
        &self,
        _config_path: &Path,
        _env_file_policy: Option<&Path>,
        _user_policy_path: Option<&Path>,
    ) -> Result<CampaignPlanObservation> {
        self.unavailable("campaign_dry_run")
    }
    fn prepare_foundation(
        &self,
        _config_path: &Path,
        _env_file_policy: Option<&Path>,
        _user_policy_path: Option<&Path>,
        _dry_run: bool,
    ) -> Result<FoundationPreparationObservation> {
        self.unavailable("prepare_foundation")
    }
    fn submit_mock_campaign(
        &self,
        _config_path: &Path,
        _env_file_policy: Option<&Path>,
        _user_policy_path: Option<&Path>,
    ) -> Result<MockSubmissionObservation> {
        self.unavailable("submit_mock_campaign")
    }
}
pub(super) fn paths() -> ReadinessInputPaths<'static> {
    ReadinessInputPaths {
        fastq_fixture: "inputs/fastq.toml",
        bam_fixture: "inputs/bam.toml",
        damage_fixture: "inputs/damage.toml",
        edna_fixture: "inputs/edna.toml",
        amplicon_fixture: "inputs/amplicon.toml",
        taxonomy_fixture: "inputs/taxonomy.toml",
        corpus_stage_compatibility: "inputs/compatibility.toml",
        fake_run_root: "runs/fake",
    }
}
