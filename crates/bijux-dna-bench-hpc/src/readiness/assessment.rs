use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn check_goal<F>(
    goal_id: u32,
    category: &str,
    surface: &str,
    output_path: Option<&str>,
    check: F,
) -> BenchLocalHpcSubmissionReadyGoalCheck
where
    F: FnOnce() -> Result<String>,
{
    match check() {
        Ok(detail) => {
            ok_check(goal_id, category, surface, output_path.map(ToString::to_string), detail)
        }
        Err(err) => fail_check(
            goal_id,
            category,
            surface,
            output_path.map(ToString::to_string),
            format!("{err:#}"),
        ),
    }
}

pub(in crate::readiness) fn ok_check(
    goal_id: u32,
    category: &str,
    surface: impl Into<String>,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> BenchLocalHpcSubmissionReadyGoalCheck {
    BenchLocalHpcSubmissionReadyGoalCheck {
        goal_id,
        category: category.to_string(),
        surface: surface.into(),
        output_path,
        ok: true,
        detail: detail.into(),
    }
}

pub(in crate::readiness) fn fail_check(
    goal_id: u32,
    category: &str,
    surface: impl Into<String>,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> BenchLocalHpcSubmissionReadyGoalCheck {
    BenchLocalHpcSubmissionReadyGoalCheck {
        goal_id,
        category: category.to_string(),
        surface: surface.into(),
        output_path,
        ok: false,
        detail: detail.into(),
    }
}
