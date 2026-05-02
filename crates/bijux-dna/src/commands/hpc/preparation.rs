use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::hpc::{campaign_dry_run, sha256_hex};

const PREPARATION_GRAPH_SCHEMA_VERSION: &str = "bijux.hpc.preparation_graph.v1";
const PREPARATION_APPLY_SCHEMA_VERSION: &str = "bijux.hpc.preparation_apply.v1";

#[derive(Debug, Clone, Serialize)]
pub struct PreparationDependencyGraphReport {
    pub schema_version: &'static str,
    pub campaign_id: String,
    pub domain: String,
    pub ready: bool,
    pub nodes: Vec<PreparationDependencyNode>,
    pub missing_prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreparationDependencyNode {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreparationApplyReport {
    pub schema_version: &'static str,
    pub campaign_id: String,
    pub domain: String,
    pub dry_run: bool,
    pub actions: Vec<PreparationApplyAction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreparationApplyAction {
    pub surface: String,
    pub root_path: String,
    pub lock_path: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreparationLock {
    schema_version: String,
    surface: String,
    campaign_id: String,
    domain: String,
    fingerprint: String,
}

pub fn preparation_dependency_graph(
    config_path: &Path,
    env_file_override: Option<&Path>,
    user_override_path: Option<&Path>,
) -> Result<PreparationDependencyGraphReport> {
    let report = campaign_dry_run(config_path, env_file_override, user_override_path)?;
    let mut nodes = Vec::new();
    nodes.push(PreparationDependencyNode {
        id: "corpora".to_string(),
        kind: "corpus_root".to_string(),
        path: report.layout.corpora_root.clone(),
        depends_on: Vec::new(),
    });
    nodes.push(PreparationDependencyNode {
        id: "databases".to_string(),
        kind: "database_root".to_string(),
        path: report.layout.databases_root.clone(),
        depends_on: Vec::new(),
    });
    nodes.push(PreparationDependencyNode {
        id: "images".to_string(),
        kind: "image_root".to_string(),
        path: report.layout.images_root.clone(),
        depends_on: Vec::new(),
    });

    let mut by_name = BTreeMap::new();
    for job in &report.planned_jobs {
        by_name.insert(job.job_name.clone(), job.job_id.clone());
    }

    let mut missing_prerequisites = Vec::new();
    for job in &report.planned_jobs {
        let mut depends_on = vec!["corpora".to_string(), "databases".to_string(), "images".to_string()];
        for dep in &job.depends_on {
            if let Some(resolved) = by_name.get(dep) {
                depends_on.push(resolved.clone());
            } else {
                missing_prerequisites.push(format!(
                    "job {} references unknown dependency `{dep}`",
                    job.job_id
                ));
            }
        }
        nodes.push(PreparationDependencyNode {
            id: job.job_id.clone(),
            kind: "planned_job".to_string(),
            path: job.outputs.results.clone(),
            depends_on,
        });
    }
    missing_prerequisites.sort();
    missing_prerequisites.dedup();
    Ok(PreparationDependencyGraphReport {
        schema_version: PREPARATION_GRAPH_SCHEMA_VERSION,
        campaign_id: report.campaign_id,
        domain: report.domain,
        ready: missing_prerequisites.is_empty(),
        nodes,
        missing_prerequisites,
    })
}

fn preparation_surfaces(report: &crate::commands::hpc::CampaignDryRunReport) -> [(&'static str, &str); 3] {
    [
        ("corpora", report.layout.corpora_root.as_str()),
        ("databases", report.layout.databases_root.as_str()),
        ("images", report.layout.images_root.as_str()),
    ]
}

fn lock_path(root: &Path) -> PathBuf {
    root.join(".bijux.prepare.lock.json")
}

fn surface_fingerprint(campaign_id: &str, domain: &str, surface: &str, root_path: &str) -> String {
    let payload = format!("{campaign_id}|{domain}|{surface}|{root_path}");
    sha256_hex(payload.as_bytes())
}

pub fn prepare_foundation(
    config_path: &Path,
    env_file_override: Option<&Path>,
    user_override_path: Option<&Path>,
    dry_run: bool,
) -> Result<PreparationApplyReport> {
    let report = campaign_dry_run(config_path, env_file_override, user_override_path)?;
    let mut actions = Vec::new();
    for (surface, root_str) in preparation_surfaces(&report) {
        let root = Path::new(root_str);
        if !dry_run {
            bijux_dna_infra::ensure_dir(root).with_context(|| format!("create {}", root.display()))?;
        }
        let lock = lock_path(root);
        let fingerprint = surface_fingerprint(&report.campaign_id, &report.domain, surface, root_str);
        let state = PreparationLock {
            schema_version: PREPARATION_APPLY_SCHEMA_VERSION.to_string(),
            surface: surface.to_string(),
            campaign_id: report.campaign_id.clone(),
            domain: report.domain.clone(),
            fingerprint: fingerprint.clone(),
        };
        let existing_fingerprint = if lock.is_file() {
            let raw = std::fs::read_to_string(&lock).with_context(|| format!("read {}", lock.display()))?;
            let parsed = serde_json::from_str::<PreparationLock>(&raw).ok();
            parsed.map(|value| value.fingerprint)
        } else {
            None
        };
        let action = if existing_fingerprint.as_deref() == Some(fingerprint.as_str()) {
            "reused"
        } else if dry_run {
            "would_prepare"
        } else {
            let payload =
                serde_json::to_vec_pretty(&state).context("serialize preparation lock")?;
            bijux_dna_api::v1::api::run::atomic_write_bytes(&lock, &payload)
                .with_context(|| format!("write {}", lock.display()))?;
            "prepared"
        };
        actions.push(PreparationApplyAction {
            surface: surface.to_string(),
            root_path: root.display().to_string(),
            lock_path: lock.display().to_string(),
            action: action.to_string(),
        });
    }
    Ok(PreparationApplyReport {
        schema_version: PREPARATION_APPLY_SCHEMA_VERSION,
        campaign_id: report.campaign_id,
        domain: report.domain,
        dry_run,
        actions,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{prepare_foundation, preparation_dependency_graph};

    #[test]
    fn preparation_graph_contains_preparation_roots_and_job_edges() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
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

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_recipients = ["alice"]

[[jobs]]
name = "fastq_validate"
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"

[[jobs]]
name = "bam_sort"
stage = "bam.sort"
tool = "samtools"
sample = "sample-1"
depends_on = ["fastq_validate"]
"#,
            root = root.path().display()
        );
        std::fs::write(&config_path, config).expect("write config");
        let graph = preparation_dependency_graph(&config_path, None, None).expect("graph");
        assert!(graph.ready);
        assert!(graph.nodes.iter().any(|node| node.id == "corpora"));
        assert!(graph.nodes.iter().any(|node| node.id == "databases"));
        assert!(graph.nodes.iter().any(|node| node.id == "images"));
        assert_eq!(graph.nodes.len(), 5);
        let bam_node = graph
            .nodes
            .iter()
            .find(|node| node.kind == "planned_job" && node.path.contains("bam.sort"))
            .expect("bam node");
        assert!(bam_node.depends_on.iter().any(|dep| dep.starts_with("dryrun-0001")));
    }

    #[test]
    fn prepare_foundation_is_idempotent_after_first_run() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
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

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_recipients = ["alice"]

[[jobs]]
name = "fastq_validate"
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
            root = root.path().display()
        );
        std::fs::write(&config_path, config).expect("write config");

        let first = prepare_foundation(&config_path, None, None, false).expect("prepare 1");
        assert!(first.actions.iter().all(|action| action.action == "prepared"));
        let second = prepare_foundation(&config_path, None, None, false).expect("prepare 2");
        assert!(second.actions.iter().all(|action| action.action == "reused"));
    }

    #[test]
    fn prepare_foundation_dry_run_does_not_write_lockfiles() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
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

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_recipients = ["alice"]

[[jobs]]
name = "fastq_validate"
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
            root = root.path().display()
        );
        std::fs::write(&config_path, config).expect("write config");
        let report = prepare_foundation(&config_path, None, None, true).expect("dry run");
        assert!(report.dry_run);
        assert!(report.actions.iter().all(|action| action.action == "would_prepare"));
        for dir in ["corpora", "databases", "images"] {
            assert!(!root.path().join(dir).join(".bijux.prepare.lock.json").exists());
        }
    }
}
