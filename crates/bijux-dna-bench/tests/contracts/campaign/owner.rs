#![allow(clippy::expect_used)]

use super::CampaignContext;
use crate::repo::hpc_campaign::environment::{command_available_in_path, parse_env_line};
use crate::repo::hpc_campaign::profiles::write_campaign_profiles;
use bijux_dna_bench_model::validate_hpc_campaign_config_source as validate_confidential_config;
use bijux_dna_bench_model::{
    CampaignDryRunReport, CampaignPreflightReport, CampaignSecurity, ResourceTemplates,
};
use std::path::{Path, PathBuf};
const ENV_DEFAULT_PATH: &str = "configs/hpc/.env";
fn default_resource_template_map(
) -> std::collections::BTreeMap<String, bijux_dna_bench_model::ResourceTemplate> {
    ResourceTemplates::default().templates
}
fn context() -> CampaignContext {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    CampaignContext {
        campaign_root: root.join("benchmarks/configs/hpc/campaign"),
        workspace_root: root,
        process_environment: [
            "BIJUX_SLURM_ACCOUNT",
            "SLURM_ACCOUNT",
            "BIJUX_SLURM_PROJECT",
            "BIJUX_HPC_PROJECT",
            "SLURM_PROJECT",
        ]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok().map(|value| (key.to_owned(), value)))
        .collect(),
        executable_search_path: std::env::var_os("PATH"),
        timestamp: "1704067200".to_string(),
    }
}
fn campaign_dry_run(
    config: &Path,
    env: Option<&Path>,
    policy: Option<&Path>,
) -> anyhow::Result<CampaignDryRunReport> {
    super::campaign_dry_run(&context(), config, env, policy)
}
fn campaign_preflight(
    config: &Path,
    env: Option<&Path>,
    policy: Option<&Path>,
) -> anyhow::Result<CampaignPreflightReport> {
    super::campaign_preflight(&context(), config, env, policy)
}

use std::ffi::OsString;

#[test]
fn parse_env_line_supports_export_prefix() {
    let row = parse_env_line("export BIJUX_SLURM_ACCOUNT=proj-123").expect("parsed");
    assert_eq!(row.0, "BIJUX_SLURM_ACCOUNT");
    assert_eq!(row.1, "proj-123");
}

#[test]
fn command_available_in_path_detects_present_binary() {
    let root = tempfile::tempdir().expect("tempdir");
    let tool_dir = root.path().join("bin");
    bijux_dna_infra::ensure_dir(&tool_dir).expect("create bin dir");
    let tool_path = tool_dir.join("apptainer");
    bijux_dna_infra::write_bytes(&tool_path, b"#!/bin/sh\n").expect("write tool");
    let path = OsString::from(tool_dir.as_os_str());
    assert!(command_available_in_path("apptainer", Some(path.as_os_str())));
}

#[test]
fn command_available_in_path_rejects_missing_binary() {
    let root = tempfile::tempdir().expect("tempdir");
    let tool_dir = root.path().join("bin");
    bijux_dna_infra::ensure_dir(&tool_dir).expect("create bin dir");
    let path = OsString::from(tool_dir.as_os_str());
    assert!(!command_available_in_path("sbatch", Some(path.as_os_str())));
    assert!(!command_available_in_path("sbatch", None));
}

#[test]
fn campaign_security_defaults_to_mock_backend() {
    let security = CampaignSecurity::default();
    assert_eq!(security.encryption_backend, "mock-envelope-v1");
    assert!(!security.encrypt_operator_outputs);
}

#[test]
fn confidential_config_rejects_secret_values() {
    let err = validate_confidential_config("token = \"abcd\"").expect_err("must reject secret");
    assert!(err.to_string().contains("sensitive key `token`"));
}

#[test]
fn confidential_config_rejects_tracked_slurm_account_fields() {
    let err = validate_confidential_config(
        r#"
[slurm]
account = "lunarc-account"
project = "lunarc-project"
"#,
    )
    .expect_err("must reject tracked slurm account fields");
    assert!(err.to_string().contains("confidential slurm `account`"));
    assert!(err.to_string().contains("confidential slurm `project`"));
}

#[test]
fn campaign_templates_require_core_tokens() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[output_templates]
log = "{campaign}/{job_id}.log"
out = "{campaign}/{job_id}.out"
err = "{campaign}/{job_id}.err"
results = "{campaign}/{job_id}.results"
code = "{campaign}/{job_id}.code"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let err = campaign_preflight(&config_path, None, None).expect_err("must reject missing tokens");
    assert!(err.to_string().contains("must include required token"));
}

#[test]
fn campaign_preflight_flags_unsupported_encryption_backend() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_backend = "unsupported-v9"
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let report = campaign_preflight(&config_path, None, None).expect("preflight");
    assert!(!report.ok);
    assert!(report
        .checks
        .iter()
        .any(|check| check.name == "encryption_backend_supported" && !check.ok));
}

#[test]
fn campaign_preflight_flags_invalid_age_recipient_and_missing_identity_files() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_backend = "age-cli"
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let report = campaign_preflight(&config_path, None, None).expect("preflight");
    assert!(!report.ok);
    assert!(report.checks.iter().any(|check| check.name == "age_recipients_valid" && !check.ok));
    assert!(report
        .checks
        .iter()
        .any(|check| check.name == "age_identity_files_configured" && !check.ok));
}

#[test]
fn campaign_dry_run_expands_templates() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 8
mem_gb = 32
walltime = "02:00:00"
scratch_gb = 64

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
resource_template = "standard"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let report =
        campaign_dry_run(&config_path, None, Some(root.path().join("missing.policy").as_path()))
            .expect("dry run");
    assert!(report.env_file_path.ends_with(ENV_DEFAULT_PATH));
    assert!(!report.user_policies_applied);
    assert_eq!(report.planned_jobs.len(), 1);
    let job = &report.planned_jobs[0];
    assert!(job
        .outputs
        .log
        .contains("/shared/logs/mini/fastq/fastq.validate_reads/seqkit_v2/sample-1/"));
    assert!(job.outputs.results.ends_with(".results"));
    assert!(job.outputs.code.ends_with(".code"));
    assert!(report.security.redacted_env_keys.is_empty());
}

#[test]
fn campaign_dry_run_carries_redaction_keys_into_security_plan() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 8
mem_gb = 32
walltime = "02:00:00"
scratch_gb = 64

[security]
encryption_backend = "mock-envelope-v1"
encryption_recipients = ["alice"]
redacted_env_keys = ["BIJUX_API_TOKEN", "PRIVATE_PATH"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let report = campaign_dry_run(&config_path, None, None).expect("dry run");
    assert_eq!(
        report.security.redacted_env_keys,
        vec!["BIJUX_API_TOKEN".to_string(), "PRIVATE_PATH".to_string()]
    );
}

#[test]
fn campaign_resource_stage_defaults_choose_template_without_job_policy() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 8
mem_gb = 32
walltime = "02:00:00"
scratch_gb = 64

[resources.templates.fastq_small]
cpus = 4
mem_gb = 16
walltime = "01:00:00"
scratch_gb = 16

[resources.stage_defaults]
fastq = "fastq_small"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let report =
        campaign_dry_run(&config_path, None, Some(root.path().join("missing.policy").as_path()))
            .expect("dry run");
    assert_eq!(report.planned_jobs.len(), 1);
    assert_eq!(report.planned_jobs[0].resource_template, "fastq_small");
    assert_eq!(report.planned_jobs[0].resources.cpus, 4);
}

#[test]
fn campaign_preflight_detects_missing_resource_template() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
[campaign]
id = "mini"
domain = "bam"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "bam.align"
tool = "bwa"
sample = "sample-1"
resource_template = "missing"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let report =
        campaign_preflight(&config_path, None, Some(root.path().join("missing.policy").as_path()))
            .expect("preflight");
    assert!(!report.ok);
    assert!(report
        .checks
        .iter()
        .any(|check| check.name.starts_with("job_template_present") && !check.ok));
    assert!(default_resource_template_map().contains_key("standard"));
}

#[test]
fn campaign_preflight_rejects_unknown_stage_ids() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "cross"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
name = "bam_sort_sample_1"
stage = "bam.sort"
tool = "samtools"
sample = "sample-1"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let err = campaign_preflight(&config_path, None, None).expect_err("must reject stale stage");
    assert!(err.to_string().contains("unknown stage `bam.sort`"));
}

#[test]
fn campaign_dry_run_applies_corpus_size_scaling_policy() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 4
mem_gb = 8
walltime = "01:00:00"
scratch_gb = 32

[resources.corpus_size_scaling.stage_large]
cpu_percent = 250
mem_percent = 200
walltime_percent = 150
scratch_percent = 125

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
corpus_size_class = "stage_large"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let report = campaign_dry_run(&config_path, None, None).expect("dry run");
    assert_eq!(report.planned_jobs.len(), 1);
    let resources = &report.planned_jobs[0].resources;
    assert_eq!(resources.cpus, 10);
    assert_eq!(resources.mem_gb, 16);
    assert_eq!(resources.walltime, "01:30:00");
    assert_eq!(resources.scratch_gb, 40);
    assert_eq!(report.planned_jobs[0].corpus_size_class.as_deref(), Some("stage_large"));
}

#[test]
fn campaign_preflight_flags_unknown_corpus_size_class() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 4
mem_gb = 8
walltime = "01:00:00"
scratch_gb = 32

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
corpus_size_class = "stage_large"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");
    let report = campaign_preflight(&config_path, None, None).expect("preflight");
    assert!(!report.ok);
    assert!(report
        .checks
        .iter()
        .any(|check| check.name.starts_with("job_corpus_size_class_present") && !check.ok));
}

#[test]
fn campaign_reports_mark_user_policies_when_file_exists() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let policy_path = root.path().join("user.policy.toml");
    let config = r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#;
    let policy_toml = r#"
[slurm]
partition = "debug"
"#;
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");
    bijux_dna_infra::write_bytes(&policy_path, policy_toml).expect("write policy");

    let report = campaign_dry_run(&config_path, None, Some(&policy_path)).expect("dry run");
    assert!(report.user_policies_applied);
    assert_eq!(report.user_policy_path, policy_path.display().to_string());
    assert_eq!(report.resolved_slurm.partition, "debug");
}

#[test]
fn campaign_preflight_passes_when_layout_roots_are_writable() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let env_file_path = root.path().join("campaign.env");
    let tool_dir = root.path().join("bin");
    let corpora_root = root.path().join("corpora");
    let databases_root = root.path().join("databases");
    let images_root = root.path().join("images");
    let scratch_root = root.path().join("scratch");
    let logs_root = root.path().join("logs");
    let encrypted_results_root = root.path().join("results");
    let encrypted_code_root = root.path().join("code");
    let appraiser_imports_root = root.path().join("imports");
    let baselines_root = root.path().join("baselines");
    for dir in [
        &tool_dir,
        &corpora_root,
        &databases_root,
        &images_root,
        &scratch_root,
        &logs_root,
        &encrypted_results_root,
        &encrypted_code_root,
        &appraiser_imports_root,
        &baselines_root,
    ] {
        bijux_dna_infra::ensure_dir(dir).expect("create dir");
    }
    for command in ["apptainer", "sbatch", "squeue", "sacct"] {
        bijux_dna_infra::write_bytes(tool_dir.join(command), b"#!/bin/sh\n").expect("write tool");
    }

    let config = format!(
        r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "{corpora_root}"
databases_root = "{databases_root}"
images_root = "{images_root}"
scratch_root = "{scratch_root}"
logs_root = "{logs_root}"
encrypted_results_root = "{encrypted_results_root}"
encrypted_code_root = "{encrypted_code_root}"
appraiser_imports_root = "{appraiser_imports_root}"
baselines_root = "{baselines_root}"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
        corpora_root = corpora_root.display(),
        databases_root = databases_root.display(),
        images_root = images_root.display(),
        scratch_root = scratch_root.display(),
        logs_root = logs_root.display(),
        encrypted_results_root = encrypted_results_root.display(),
        encrypted_code_root = encrypted_code_root.display(),
        appraiser_imports_root = appraiser_imports_root.display(),
        baselines_root = baselines_root.display()
    );
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");
    bijux_dna_infra::write_bytes(
        &env_file_path,
        "BIJUX_SLURM_ACCOUNT=account-local\nBIJUX_SLURM_PROJECT=project-local\n",
    )
    .expect("write env");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&env_file_path).expect("env metadata").permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&env_file_path, perms).expect("set env mode");
    }
    let mut observations = context();
    observations.executable_search_path = Some(tool_dir.as_os_str().to_owned());
    let report = super::campaign_preflight(&observations, &config_path, Some(&env_file_path), None)
        .expect("preflight");
    assert!(report.ok);
}

#[test]
#[cfg(unix)]
fn campaign_preflight_flags_world_readable_env_file() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().expect("tempdir");
    let config_path = root.path().join("campaign.toml");
    let env_file_path = root.path().join("campaign.env");
    for name in [
        "corpora",
        "databases",
        "images",
        "scratch",
        "logs",
        "results",
        "code",
        "imports",
        "baselines",
    ] {
        bijux_dna_infra::ensure_dir(root.path().join(name)).expect("create dir");
    }
    let config = format!(
        r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
        root = root.path().display()
    );
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");
    bijux_dna_infra::write_bytes(
        &env_file_path,
        "BIJUX_SLURM_ACCOUNT=account-local\nBIJUX_SLURM_PROJECT=project-local\n",
    )
    .expect("write env");
    let mut perms = std::fs::metadata(&env_file_path).expect("env metadata").permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&env_file_path, perms).expect("set env mode");

    let report = campaign_preflight(&config_path, Some(&env_file_path), None).expect("preflight");
    assert!(!report.ok);
    assert!(report.checks.iter().any(|check| check.name == "env_file_private" && !check.ok));
}

#[test]
fn campaign_resolves_partition_from_site_profile_file() {
    let root = tempfile::tempdir().expect("tempdir");
    let config_dir = root.path().join("campaign");
    let profiles_dir = config_dir.join("site-profiles");
    bijux_dna_infra::ensure_dir(&profiles_dir).expect("create profile dir");
    let config_path = config_dir.join("mini.toml");
    let env_file_path = root.path().join("campaign.env");

    bijux_dna_infra::write_bytes(
        profiles_dir.join("generic.toml"),
        "partition = \"debug\"\nqos = \"short\"\n",
    )
    .expect("write profile");
    bijux_dna_infra::write_bytes(
        &env_file_path,
        "BIJUX_SLURM_ACCOUNT=account-local\nBIJUX_SLURM_PROJECT=project-local\n",
    )
    .expect("write env");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&env_file_path).expect("env metadata").permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&env_file_path, perms).expect("set env mode");
    }
    for name in [
        "corpora",
        "databases",
        "images",
        "scratch",
        "logs",
        "results",
        "code",
        "imports",
        "baselines",
    ] {
        bijux_dna_infra::ensure_dir(root.path().join(name)).expect("create dir");
    }
    let config = format!(
        r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
        root = root.path().display()
    );
    bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

    let report = campaign_dry_run(&config_path, Some(&env_file_path), None).expect("dry run");
    assert_eq!(report.resolved_slurm.partition, "debug");
    assert_eq!(report.resolved_slurm.qos, "short");
}

#[test]
fn write_campaign_profiles_emits_expected_files() {
    let root = tempfile::tempdir().expect("tempdir");
    let written = write_campaign_profiles(root.path()).expect("write profiles");
    assert_eq!(written.len(), 5);
    assert!(written.iter().any(|path| path.ends_with("lunarc-small.toml")));
    assert!(written.iter().any(|path| path.ends_with("lunarc-fastq-bam-local-ready.toml")));
    assert!(written.iter().any(|path| path.ends_with("lunarc-fastq-bam-vcf-local-ready.toml")));
    assert!(written.iter().any(|path| path.ends_with("generic-small.toml")));
    assert!(written.iter().any(|path| path.ends_with("cross-mini.toml")));
    for path in written {
        assert!(path.is_file());
    }
}

#[test]
fn lunarc_fastq_bam_local_ready_profile_expands_prepared_roots_and_jobs() {
    let root = tempfile::tempdir().expect("tempdir");
    let written = write_campaign_profiles(root.path()).expect("write profiles");
    let config_path = written
        .into_iter()
        .find(|path| path.ends_with("lunarc-fastq-bam-local-ready.toml"))
        .expect("local-ready profile");

    let report = campaign_dry_run(
        &config_path,
        Some(root.path().join("missing.env").as_path()),
        Some(root.path().join("missing.policy").as_path()),
    )
    .expect("dry run");

    assert_eq!(report.campaign_id, "adna-equus-caballus-local-ready");
    assert_eq!(report.domain, "cross");
    assert_eq!(report.layout.corpora_root, "/mnt/shared/bijux/corpora");
    assert_eq!(report.layout.databases_root, "/mnt/shared/bijux/databases");
    assert_eq!(report.layout.images_root, "/mnt/shared/bijux/images");
    assert_eq!(report.resolved_slurm.site_profile, "lunarc");
    assert_eq!(report.resolved_slurm.partition, "main");
    assert_eq!(report.resolved_slurm.qos, "normal");
    assert_eq!(report.planned_jobs.len(), 9);
    assert_eq!(report.planned_jobs[0].stage, "fastq.validate_reads");
    assert_eq!(report.planned_jobs[1].stage, "fastq.profile_read_lengths");
    assert_eq!(report.planned_jobs[2].stage, "fastq.detect_adapters");
    assert_eq!(report.planned_jobs[3].stage, "fastq.trim_reads");
    assert_eq!(report.planned_jobs[4].stage, "fastq.merge_pairs");
    assert_eq!(report.planned_jobs[5].stage, "fastq.filter_reads");
    assert_eq!(report.planned_jobs[6].stage, "bam.align");
    assert_eq!(report.planned_jobs[7].stage, "bam.markdup");
    assert_eq!(report.planned_jobs[8].stage, "bam.qc_pre");
    assert_eq!(report.planned_jobs[6].resource_template, "bam_align");
    assert_eq!(report.planned_jobs[7].resource_template, "bam_markdup");
    assert_eq!(report.planned_jobs[8].resource_template, "bam_qc");
}

#[test]
fn lunarc_fastq_bam_vcf_local_ready_profile_expands_prepared_roots_and_jobs() {
    let root = tempfile::tempdir().expect("tempdir");
    let written = write_campaign_profiles(root.path()).expect("write profiles");
    let config_path = written
        .into_iter()
        .find(|path| path.ends_with("lunarc-fastq-bam-vcf-local-ready.toml"))
        .expect("local-ready vcf profile");

    let report = campaign_dry_run(
        &config_path,
        Some(root.path().join("missing.env").as_path()),
        Some(root.path().join("missing.policy").as_path()),
    )
    .expect("dry run");

    let stage_ids: Vec<_> = report.planned_jobs.iter().map(|job| job.stage.as_str()).collect();
    assert_eq!(report.campaign_id, "adna-equus-caballus-fastq-bam-vcf-local-ready");
    assert_eq!(report.domain, "cross");
    assert_eq!(report.layout.corpora_root, "/mnt/shared/bijux/corpora");
    assert_eq!(report.layout.databases_root, "/mnt/shared/bijux/databases");
    assert_eq!(report.layout.images_root, "/mnt/shared/bijux/images");
    assert_eq!(report.resolved_slurm.site_profile, "lunarc");
    assert_eq!(report.resolved_slurm.partition, "main");
    assert_eq!(report.resolved_slurm.qos, "normal");
    assert_eq!(report.planned_jobs.len(), 16);
    assert_eq!(
        stage_ids,
        vec![
            "fastq.validate_reads",
            "fastq.profile_read_lengths",
            "fastq.detect_adapters",
            "fastq.trim_reads",
            "fastq.merge_pairs",
            "fastq.filter_reads",
            "bam.align",
            "bam.markdup",
            "bam.qc_pre",
            "bam.damage",
            "bam.coverage",
            "bam.duplication_metrics",
            "vcf.call",
            "vcf.filter",
            "vcf.stats",
            "vcf.qc",
        ]
    );
    assert_eq!(report.planned_jobs[6].resource_template, "bam_align");
    assert_eq!(report.planned_jobs[7].resource_template, "bam_markdup");
    assert_eq!(report.planned_jobs[12].resource_template, "vcf_call");
    assert_eq!(report.planned_jobs[13].resource_template, "vcf_qc");
    assert_eq!(report.planned_jobs[14].resource_template, "vcf_qc");
    assert_eq!(report.planned_jobs[15].resource_template, "vcf_qc");
    assert!(report.planned_jobs.iter().all(|job| {
        !matches!(
            job.stage.as_str(),
            "fastq.index_reference" | "vcf.prepare_reference_panel" | "bam.index_reference"
        )
    }));
}

#[test]
fn campaign_redaction_masks_short_unicode_values_without_byte_boundary_panics() {
    use super::policies::redacted;
    for value in ["", "  "] {
        assert_eq!(redacted(Some(value)), "<missing>");
    }
    for value in ["a", "abcd", "é", "项目账号", "🔐🧬🔑🧪"] {
        assert_eq!(redacted(Some(value)), "****");
    }
    assert_eq!(redacted(Some(" abcdef ")), "ab***ef");
    assert_eq!(redacted(Some("账号保密项目")), "账号***项目");
    assert_eq!(redacted(Some("éabc😀")), "éa***c😀");
}

#[test]
fn redaction_observation_uses_current_campaign_environment_decoding() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let path = temporary.path().join("campaign.env");
    std::fs::write(
        &path,
        "export API_TOKEN='quoted-secret'\nCUSTOM=\"explicit-secret\"\nOTHER=ordinary\n",
    )?;
    let observed = crate::collect_hpc_redaction_needles(&path, &["CUSTOM".into()])?;
    assert_eq!(observed, ["explicit-secret", "quoted-secret"]);
    std::fs::write(&path, "API_TOKEN=replacement-secret\n")?;
    assert_eq!(crate::collect_hpc_redaction_needles(&path, &[])?, ["replacement-secret"]);
    std::fs::remove_file(&path)?;
    assert!(crate::collect_hpc_redaction_needles(&path, &[])?.is_empty());
    Ok(())
}
