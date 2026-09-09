//! Owner: bijux-dna-bench-model
//! Pure selected-tool scope projection for HPC execution resolution.

use std::collections::{BTreeMap, BTreeSet};

use crate::BenchmarkLocalAllDomainSlurmSubmitJob;

#[derive(Debug, Clone, Default)]
struct SelectedToolScopeAccumulator {
    domains: BTreeSet<String>,
    selected_stage_ids: BTreeSet<String>,
    selected_job_ids: BTreeSet<String>,
    selected_result_ids: BTreeSet<String>,
    selected_pipeline_ids: BTreeSet<String>,
    selected_node_ids: BTreeSet<String>,
}

/// Deterministic execution scope projected from the selected submission jobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkLocalHpcToolScope {
    pub tool_id: String,
    pub lookup_tool_id: String,
    pub domains: Vec<String>,
    pub selected_stage_ids: Vec<String>,
    pub selected_job_ids: Vec<String>,
    pub selected_result_ids: Vec<String>,
    pub selected_pipeline_ids: Vec<String>,
    pub selected_node_ids: Vec<String>,
}

/// Group selected jobs by declared tool identity without repository access.
/// Lookup aliases do not merge distinct declared tool identities.
#[must_use]
pub fn project_benchmark_local_hpc_tool_scopes(
    jobs: &[BenchmarkLocalAllDomainSlurmSubmitJob],
) -> Vec<BenchmarkLocalHpcToolScope> {
    let mut scope_by_tool = BTreeMap::<String, SelectedToolScopeAccumulator>::new();
    for job in jobs {
        let scope = scope_by_tool.entry(job.tool_id.clone()).or_default();
        scope.domains.insert(job.domain.clone());
        scope.selected_stage_ids.insert(job.stage_id.clone());
        scope.selected_job_ids.insert(job.job_id_local.clone());
        if let Some(result_id) = &job.result_id {
            scope.selected_result_ids.insert(result_id.clone());
        }
        if let Some(pipeline_id) = &job.pipeline_id {
            scope.selected_pipeline_ids.insert(pipeline_id.clone());
        }
        if let Some(node_id) = &job.node_id {
            scope.selected_node_ids.insert(node_id.clone());
        }
    }

    let mut scopes = scope_by_tool
        .into_iter()
        .map(|(tool_id, scope)| BenchmarkLocalHpcToolScope {
            lookup_tool_id: canonical_lookup_tool_id(&tool_id),
            tool_id,
            domains: scope.domains.into_iter().collect(),
            selected_stage_ids: scope.selected_stage_ids.into_iter().collect(),
            selected_job_ids: scope.selected_job_ids.into_iter().collect(),
            selected_result_ids: scope.selected_result_ids.into_iter().collect(),
            selected_pipeline_ids: scope.selected_pipeline_ids.into_iter().collect(),
            selected_node_ids: scope.selected_node_ids.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    scopes.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    scopes
}

fn canonical_lookup_tool_id(tool_id: &str) -> String {
    match tool_id {
        "bijux-dna" => "bijux_dna".to_string(),
        _ => tool_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BenchmarkLocalSlurmSubmitResources;

    fn job(id: &str, dependencies: &[&str]) -> BenchmarkLocalAllDomainSlurmSubmitJob {
        BenchmarkLocalAllDomainSlurmSubmitJob {
            job_id_local: id.to_string(),
            domain: "fastq".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            pipeline_id: None,
            node_id: None,
            tool_id: "fastp".to_string(),
            corpus_id: "corpus".to_string(),
            asset_profile_id: "profile".to_string(),
            result_id: Some(format!("result-{id}")),
            script_path: format!("{id}.sh"),
            stdout: format!("{id}.out"),
            stderr: format!("{id}.err"),
            outputs: vec![format!("{id}.json")],
            dependencies: dependencies.iter().map(|id| (*id).to_string()).collect(),
            resources: BenchmarkLocalSlurmSubmitResources {
                cpus_per_task: 1,
                memory_mb: 128,
                time_limit: "00:01:00".to_string(),
            },
        }
    }

    #[test]
    fn scopes_preserve_declared_identity_and_deduplicate_sorted_membership() {
        let mut alpha = job("job-z", &[]);
        alpha.tool_id = "bijux-dna".into();
        let mut beta = job("job-a", &[]);
        beta.tool_id = "bijux-dna".into();
        beta.domain = "bam".into();
        beta.stage_id = "bam.coverage".into();
        beta.result_id = None;
        beta.pipeline_id = Some("pipeline".into());
        beta.node_id = Some("node".into());
        let mut alias = job("job-alias", &[]);
        alias.tool_id = "bijux_dna".into();
        let jobs = vec![alpha.clone(), beta.clone(), alias.clone(), alpha];
        let scopes = project_benchmark_local_hpc_tool_scopes(&jobs);
        assert_eq!(scopes.len(), 2);
        assert_eq!(scopes[0].tool_id, "bijux-dna");
        assert_eq!(scopes[1].tool_id, "bijux_dna");
        assert_eq!(scopes[0].lookup_tool_id, scopes[1].lookup_tool_id);
        assert_eq!(scopes[0].domains, ["bam", "fastq"]);
        assert_eq!(scopes[0].selected_job_ids, ["job-a", "job-z"]);
        assert_eq!(scopes[0].selected_stage_ids, ["bam.coverage", "fastq.validate_reads"]);
        assert_eq!(scopes[0].selected_result_ids, ["result-job-z"]);
        assert_eq!(scopes[0].selected_pipeline_ids, ["pipeline"]);
        assert_eq!(scopes[0].selected_node_ids, ["node"]);
        assert_eq!(scopes[1].selected_job_ids, ["job-alias"]);
        let mut reordered = jobs;
        reordered.reverse();
        assert_eq!(project_benchmark_local_hpc_tool_scopes(&reordered), scopes);
    }

    #[test]
    fn scope_projection_does_not_resolve_transport_or_mutate_selected_jobs() -> anyhow::Result<()> {
        assert!(project_benchmark_local_hpc_tool_scopes(&[]).is_empty());
        let mut selected = job("job", &[]);
        let before = project_benchmark_local_hpc_tool_scopes(&[selected.clone()]);
        selected.script_path = "unavailable/script.sh".into();
        selected.outputs.clear();
        selected.resources.cpus_per_task = 0;
        selected.dependencies = vec!["not-in-scope".into()];
        let jobs = vec![selected];
        let bytes = serde_json::to_vec(&jobs)?;
        assert_eq!(project_benchmark_local_hpc_tool_scopes(&jobs), before);
        assert_eq!(serde_json::to_vec(&jobs)?, bytes);
        Ok(())
    }
}
