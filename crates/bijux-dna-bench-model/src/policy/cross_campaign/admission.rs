//! Owner: Cross-domain campaign requested-goal admission.
use super::{anyhow, BTreeSet, Result, CROSS_GOALS_CATALOG};

pub(super) fn selected_goal_ids(raw: Option<&str>) -> Result<Vec<String>> {
    let available =
        CROSS_GOALS_CATALOG.iter().map(|goal| goal.goal_id.to_string()).collect::<BTreeSet<_>>();
    let mut selected = if let Some(value) = raw {
        value
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(str::to_ascii_uppercase)
            .collect::<BTreeSet<_>>()
    } else {
        available.clone()
    };
    if selected.is_empty() {
        return Err(anyhow!("cross-benchmark-campaign requires at least one goal"));
    }
    let unknown =
        selected.iter().filter(|goal| !available.contains(*goal)).cloned().collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!("unknown cross goals requested: {}", unknown.join(",")));
    }
    let mut ordered = CROSS_GOALS_CATALOG
        .iter()
        .filter_map(|goal| {
            if selected.remove(goal.goal_id) {
                Some(goal.goal_id.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    ordered.sort();
    Ok(ordered)
}
