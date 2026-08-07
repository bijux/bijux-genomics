use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use bijux_dna_containers::api::{
    admit_container_pipeline_site, admit_historical_container_pipeline_replay, ContainerGovernance,
    ContainerPathPolicy, ContainerPipelineSiteAdmissionRequest, ContainerPreflightPolicy,
    ContainerRuntimePolicy, ContainerSitePolicy, ContainerSlurmPolicy,
    HistoricalContainerPipelineReplayRequest,
};
use sha2::{Digest, Sha256};

struct SiteFixture {
    root: tempfile::TempDir,
    governance: ContainerGovernance,
    repository: PathBuf,
    commit: String,
    image: PathBuf,
    image_sha256: String,
    input: PathBuf,
    input_sha256: String,
    pipeline: PathBuf,
}

impl SiteFixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        let study = root.path().join("study");
        let containers = study.join("containers");
        let data = study.join("data");
        let results = study.join("results");
        let finalization = study.join("finalization");
        let repository = root.path().join("repository");
        let frozen = root.path().join("frozen");
        for path in [
            containers.join("apptainer"),
            data.clone(),
            results.clone(),
            finalization.clone(),
            repository.clone(),
            frozen.clone(),
            study.join("scratch"),
        ] {
            std::fs::create_dir_all(path)?;
        }

        git(&repository, &["init", "--quiet"])?;
        std::fs::write(repository.join("README.md"), "governed repository\n")?;
        git(&repository, &["add", "README.md"])?;
        git(
            &repository,
            &[
                "-c",
                "user.name=Bijux Test",
                "-c",
                "user.email=bijux@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "initial",
            ],
        )?;
        let commit = git(&repository, &["rev-parse", "HEAD"])?;

        let image = containers.join("apptainer/bwa.sif");
        std::fs::write(&image, b"immutable image")?;
        let image_sha256 = sha256(b"immutable image");
        let input = data.join("reads.fastq.gz");
        std::fs::write(&input, b"immutable reads")?;
        let input_sha256 = sha256(b"immutable reads");
        let pipeline = root.path().join("pipeline.yml");

        let governance = ContainerGovernance {
            schema_version: "bijux.container.governance.v1".to_owned(),
            site: ContainerSitePolicy {
                id: "test-site".to_owned(),
                hostname_regex: "test-node".to_owned(),
            },
            paths: ContainerPathPolicy {
                study_root: study.clone(),
                containers_root: containers,
                data_root: data,
                results_root: results,
                finalization_root: finalization,
                active_repository: repository.clone(),
                frozen_repositories: frozen,
            },
            runtime: ContainerRuntimePolicy { apptainer_architecture: "linux/amd64".to_owned() },
            slurm: ContainerSlurmPolicy {
                account: "science".to_owned(),
                scratch_environment: None,
                scratch_root: Some(study.join("scratch")),
                partitions: vec!["compute".to_owned()],
                quota_command: vec!["quota".to_owned()],
            },
            preflight: ContainerPreflightPolicy {
                minimum_available_bytes: 1,
                minimum_available_inodes: 1,
                maximum_clock_skew_seconds: 1,
                required_commands: vec!["git".to_owned()],
                required_contracts: vec!["pipeline".to_owned()],
            },
        };

        let fixture = Self {
            root,
            governance,
            repository,
            commit,
            image,
            image_sha256,
            input,
            input_sha256,
            pipeline,
        };
        fixture.write_pipeline(&fixture.repository, "science", "compute", None)?;
        Ok(fixture)
    }

    fn write_pipeline(
        &self,
        repository: &Path,
        account: &str,
        partition: &str,
        result_root: Option<&Path>,
    ) -> Result<()> {
        let result_root = result_root.map_or_else(
            || self.governance.paths.results_root.join("container-validation/run-1200-bwa"),
            Path::to_path_buf,
        );
        let contract = serde_json::json!({
            "schema_version": "bijux.container.pipeline.v2",
            "run": {
                "id": "run-1200",
                "kind": "qualification",
                "campaign": "governed-site-test",
                "corpus": "immutable-corpus",
                "sample": "sample-a",
                "domain": "fastq",
                "stage": "fastq.align_reads",
                "tool": "bwa",
                "operator": "bijux",
                "submitted_at_utc": "2026-08-07T00:00:00Z"
            },
            "repository": {
                "path": repository,
                "commit": self.commit,
                "clean": true
            },
            "image": {"path": self.image, "sha256": self.image_sha256},
            "data": {"inputs": [{
                "id": "reads",
                "kind": "data",
                "path": self.input,
                "mount_path": "/input/reads.fastq.gz",
                "sha256": self.input_sha256
            }]},
            "commands": {
                "version": ["bwa", "version"],
                "help": ["bwa", "help"],
                "real": ["bwa", "mem"],
                "negative": ["bwa", "invalid"]
            },
            "outputs": {
                "result_root": result_root,
                "mount_path": "/result",
                "required_files": ["result.json"],
                "collection_rule": "required_files_and_metrics"
            },
            "resources": {
                "account": account,
                "partition": partition,
                "cpus": 2,
                "memory_mb": 1024,
                "walltime": "00:05:00",
                "scratch_mb": 1024,
                "retry": {
                    "max_attempts": 1,
                    "backoff_seconds": 0,
                    "retryable_exit_codes": []
                }
            },
            "execution": {
                "locale": "C",
                "timezone": "UTC",
                "runtime_network": false,
                "threads": 2,
                "random_seed": null,
                "cache_policy": "cold",
                "determinism": "deterministic"
            },
            "dependencies": [],
            "acceptance": {
                "version_regex": "^1[.]0$",
                "negative_exit_nonzero": true,
                "metrics": [{
                    "id": "records",
                    "path": "result.json",
                    "json_pointer": "/records",
                    "operator": "ge",
                    "expected": 1,
                    "unit": "records"
                }]
            }
        });
        std::fs::write(&self.pipeline, serde_json::to_vec_pretty(&contract)?)?;
        Ok(())
    }

    fn request(&self, hostname: &str) -> ContainerPipelineSiteAdmissionRequest {
        ContainerPipelineSiteAdmissionRequest {
            governance: self.governance.clone(),
            hostname: hostname.to_owned(),
            pipeline_path: self.pipeline.clone(),
        }
    }
}

#[test]
fn admits_active_and_commit_addressed_frozen_repositories() -> Result<()> {
    let fixture = SiteFixture::new()?;
    let admission = admit_container_pipeline_site(&fixture.request("test-node"))?;
    assert_eq!(admission.run_id, "run-1200");
    assert_eq!(admission.repository_commit, fixture.commit);
    assert_eq!(admission.inputs.len(), 1);

    let frozen = fixture.governance.paths.frozen_repositories.join(&fixture.commit);
    let output = Command::new("git")
        .args(["clone", "--quiet"])
        .arg(&fixture.repository)
        .arg(&frozen)
        .output()?;
    ensure!(
        output.status.success(),
        "git clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fixture.write_pipeline(&frozen, "science", "compute", None)?;
    let frozen_admission = admit_container_pipeline_site(&fixture.request("test-node"))?;
    assert_eq!(frozen_admission.repository_path, frozen.canonicalize()?);
    Ok(())
}

#[test]
fn rejects_site_identity_and_immutable_file_drift() -> Result<()> {
    let fixture = SiteFixture::new()?;
    let host_error = admit_container_pipeline_site(&fixture.request("other-node"))
        .err()
        .context("pipeline admitted outside governed host")?;
    assert!(format!("{host_error:#}").contains("outside governed site"));

    std::fs::write(&fixture.image, b"changed image")?;
    let digest_error = admit_container_pipeline_site(&fixture.request("test-node"))
        .err()
        .context("pipeline admitted a changed image")?;
    assert!(format!("{digest_error:#}").contains("digest differs"));
    Ok(())
}

#[test]
fn rejects_dirty_repository_resources_and_result_layout() -> Result<()> {
    let fixture = SiteFixture::new()?;
    std::fs::write(fixture.repository.join("untracked.txt"), "dirty\n")?;
    let dirty_error = admit_container_pipeline_site(&fixture.request("test-node"))
        .err()
        .context("pipeline admitted a dirty repository")?;
    assert!(format!("{dirty_error:#}").contains("worktree is not clean"));
    std::fs::remove_file(fixture.repository.join("untracked.txt"))?;

    fixture.write_pipeline(&fixture.repository, "other", "compute", None)?;
    let account_error = admit_container_pipeline_site(&fixture.request("test-node"))
        .err()
        .context("pipeline admitted an ungoverned account")?;
    assert!(format!("{account_error:#}").contains("resources.account"));

    let wrong_result = fixture.governance.paths.results_root.join("container-validation/run-1200");
    fixture.write_pipeline(&fixture.repository, "science", "compute", Some(&wrong_result))?;
    let result_error = admit_container_pipeline_site(&fixture.request("test-node"))
        .err()
        .context("current pipeline admitted a legacy result name")?;
    assert!(format!("{result_error:#}").contains("run-1200-bwa"));
    Ok(())
}

#[test]
fn admits_only_the_exact_benchmark_result_layout() -> Result<()> {
    let fixture = SiteFixture::new()?;
    let mut contract: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&fixture.pipeline)?)?;
    contract["run"]["kind"] = serde_json::json!("benchmark");
    contract["outputs"]["result_root"] = serde_json::json!(fixture
        .governance
        .paths
        .results_root
        .join("benchmarks/run-1200-fastq-align-reads/bwa"));
    contract["parameters"] = serde_json::json!({
        "schema_version": "bijux.benchmark.parameter_evidence.v1",
        "parameter_schema_version": "bijux.stage.parameters.v1",
        "defaults_provider": "stage-contract",
        "overrides_file": "config-pipeline-parameters.yml",
        "overrides_sha256": "0".repeat(64),
        "effective_file": "effective-parameters.json",
        "effective_sha256": "1".repeat(64)
    });
    std::fs::write(&fixture.pipeline, serde_json::to_vec_pretty(&contract)?)?;
    let admission = admit_container_pipeline_site(&fixture.request("test-node"))?;
    assert_eq!(admission.evidence_kind, bijux_dna_containers::api::PipelineEvidenceKind::Benchmark);

    contract["outputs"]["result_root"] =
        serde_json::json!(fixture.governance.paths.results_root.join("benchmarks/run-1200/bwa"));
    std::fs::write(&fixture.pipeline, serde_json::to_vec_pretty(&contract)?)?;
    let error = admit_container_pipeline_site(&fixture.request("test-node"))
        .err()
        .context("benchmark pipeline admitted a noncanonical result root")?;
    assert!(format!("{error:#}").contains("run-1200-fastq-align-reads"));
    Ok(())
}

#[test]
fn historical_replay_allows_only_repository_and_image_currentization() -> Result<()> {
    let fixture = SiteFixture::new()?;
    let frozen = fixture.governance.paths.frozen_repositories.join(&fixture.commit);
    let output = Command::new("git")
        .args(["clone", "--quiet"])
        .arg(&fixture.repository)
        .arg(&frozen)
        .output()?;
    ensure!(output.status.success(), "git clone failed");

    let source_path = fixture.root.path().join("source-pipeline.json");
    let replay_path = fixture.root.path().join("replay-pipeline.json");
    let source = legacy_pipeline(
        &fixture,
        &fixture.root.path().join("retired-repository"),
        &"a".repeat(40),
        &fixture.root.path().join("retired-bwa.sif"),
        &"b".repeat(64),
    );
    let replay =
        legacy_pipeline(&fixture, &frozen, &fixture.commit, &fixture.image, &fixture.image_sha256);
    std::fs::write(&source_path, serde_json::to_vec_pretty(&source)?)?;
    std::fs::write(&replay_path, serde_json::to_vec_pretty(&replay)?)?;
    let request = HistoricalContainerPipelineReplayRequest {
        governance: fixture.governance.clone(),
        hostname: "test-node".to_owned(),
        source_pipeline_path: source_path.canonicalize()?,
        replay_pipeline_path: replay_path.canonicalize()?,
    };
    let admission = admit_historical_container_pipeline_replay(&request)?;
    assert_eq!(admission.site.run_id, "run-1200");
    assert_ne!(admission.source_pipeline_sha256, admission.replay_pipeline_sha256);

    let mut changed = replay;
    changed["commands"]["real"] = serde_json::json!(["bwa", "index"]);
    std::fs::write(&request.replay_pipeline_path, serde_json::to_vec_pretty(&changed)?)?;
    let error = admit_historical_container_pipeline_replay(&request)
        .err()
        .context("historical replay admitted changed scientific intent")?;
    assert!(format!("{error:#}").contains("outside repository and image identity"));
    Ok(())
}

fn legacy_pipeline(
    fixture: &SiteFixture,
    repository: &Path,
    commit: &str,
    image: &Path,
    image_sha256: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "bijux.container.pipeline.v1",
        "run": {
            "id": "run-1200",
            "campaign": "governed-replay-test",
            "corpus": "immutable-corpus",
            "sample": "sample-a",
            "domain": "fastq",
            "stage": "fastq.align_reads",
            "tool": "bwa",
            "operator": "bijux"
        },
        "repository": {"path": repository, "commit": commit, "clean": true},
        "image": {"path": image, "sha256": image_sha256},
        "data": {"inputs": [{
            "id": "reads",
            "kind": "data",
            "path": fixture.input,
            "sha256": fixture.input_sha256
        }]},
        "commands": {
            "version": ["bwa", "version"],
            "help": ["bwa", "help"],
            "real": ["bwa", "mem"],
            "negative": ["bwa", "invalid"]
        },
        "outputs": {
            "result_root": fixture
                .governance
                .paths
                .results_root
                .join("container-validation/bwa/run-1200"),
            "required_files": ["result.json"]
        },
        "resources": {
            "account": "science",
            "partition": "compute",
            "cpus": 2,
            "memory_mb": 1024,
            "walltime": "00:05:00",
            "scratch_mb": 1024
        },
        "execution": {"locale": "C", "timezone": "UTC", "runtime_network": false},
        "acceptance": {"version_regex": "^1[.]0$", "negative_exit_nonzero": true}
    })
}

fn sha256(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn git(repository: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git").arg("-C").arg(repository).args(arguments).output()?;
    ensure!(
        output.status.success(),
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
