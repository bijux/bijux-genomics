use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::BAM_STAGE_GOALS;
use crate::readiness::declarations::FASTQ_STAGE_GOALS;
use crate::readiness::paths::absolutize;
use crate::readiness::paths::path_relative_to_repo;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_local_stage_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    for &(stage_id, goal_id, expected_root) in
        FASTQ_STAGE_GOALS.iter().chain(BAM_STAGE_GOALS.iter())
    {
        let expected_root_path = repo_root.join(expected_root);
        match context.backend.materialize_local_stage(repo_root, stage_id) {
            Ok(path) => {
                let absolute_path = absolutize(repo_root, &path);
                if !absolute_path.exists() {
                    checks.push(fail_check(
                        goal_id,
                        "local_smokes",
                        format!("{stage_id} local artifact materializes under governed target"),
                        Some(path_relative_to_repo(repo_root, &absolute_path)),
                        format!("materialized path `{}` does not exist", absolute_path.display()),
                    ));
                } else if !absolute_path.starts_with(&expected_root_path) {
                    checks.push(fail_check(
                        goal_id,
                        "local_smokes",
                        format!("{stage_id} local artifact materializes under governed target"),
                        Some(path_relative_to_repo(repo_root, &absolute_path)),
                        format!(
                            "materialized path `{}` is outside governed root `{}`",
                            absolute_path.display(),
                            expected_root_path.display()
                        ),
                    ));
                } else {
                    checks.push(ok_check(
                        goal_id,
                        "local_smokes",
                        format!("{stage_id} local artifact materializes under governed target"),
                        Some(path_relative_to_repo(repo_root, &absolute_path)),
                        format!(
                            "materialized `{}` under `{}`",
                            path_relative_to_repo(repo_root, &absolute_path),
                            expected_root
                        ),
                    ));
                }
            }
            Err(err) => {
                checks.push(fail_check(
                    goal_id,
                    "local_smokes",
                    format!("{stage_id} local artifact materializes under governed target"),
                    Some(expected_root.to_string()),
                    format!("{err:#}"),
                ));
            }
        }
    }
}
