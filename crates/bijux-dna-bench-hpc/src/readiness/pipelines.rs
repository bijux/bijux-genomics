use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::PIPELINE_DAG_GOALS;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalPipelineDagValidationReport as LocalPipelineDagValidationReport;

pub(in crate::readiness) fn evaluate_pipeline_dag_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    for &(goal_id, config_path, output_path) in PIPELINE_DAG_GOALS {
        match context.backend.validate_pipeline_dag_path(
            repo_root,
            &repo_root.join(config_path),
            &repo_root.join(output_path),
        ) {
            Ok(report) => evaluate_pipeline_dag_goal(checks, goal_id, &report),
            Err(err) => checks.push(fail_check(
                goal_id,
                "pipeline_dags",
                format!("{config_path} validates as an acyclic governed local DAG"),
                Some(output_path.to_string()),
                format!("{err:#}"),
            )),
        }
    }
}

pub(in crate::readiness) fn evaluate_pipeline_dag_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    goal_id: u32,
    report: &LocalPipelineDagValidationReport,
) {
    if report.valid && report.acyclic && report.node_count > 0 {
        checks.push(ok_check(
            goal_id,
            "pipeline_dags",
            format!("{} validates as an acyclic governed local DAG", report.pipeline_id),
            Some(report.output_path.clone()),
            format!(
                "pipeline `{}` validates {} nodes and {} edges",
                report.pipeline_id, report.node_count, report.edge_count
            ),
        ));
    } else {
        checks.push(fail_check(
            goal_id,
            "pipeline_dags",
            format!("{} validates as an acyclic governed local DAG", report.pipeline_id),
            Some(report.output_path.clone()),
            format!(
                "valid={} acyclic={} node_count={}",
                report.valid, report.acyclic, report.node_count
            ),
        ));
    }
}
