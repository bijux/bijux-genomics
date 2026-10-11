use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
pub(super) fn ok_check(
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> LocalHpcDryRunReadyGoalCheck {
    LocalHpcDryRunReadyGoalCheck {
        goal_id,
        surface: surface.to_string(),
        output_path,
        ok: true,
        detail: detail.into(),
    }
}

pub(super) fn fail_check(
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> LocalHpcDryRunReadyGoalCheck {
    LocalHpcDryRunReadyGoalCheck {
        goal_id,
        surface: surface.to_string(),
        output_path,
        ok: false,
        detail: detail.into(),
    }
}

pub(super) fn error_detail(error: Option<&anyhow::Error>) -> String {
    error.map_or_else(|| "validation failed".to_string(), |error| format!("{error:#}"))
}

pub(super) fn blocked_by(surface: &str, error: Option<&anyhow::Error>) -> String {
    match error {
        Some(error) => format!("{surface} validation failed earlier: {error:#}"),
        None => format!("{surface} validation failed earlier"),
    }
}
