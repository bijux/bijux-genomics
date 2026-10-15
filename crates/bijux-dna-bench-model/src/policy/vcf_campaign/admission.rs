//! Owner: VCF campaign requested-goal admission.
use super::{anyhow, BTreeSet, Result, VCF_GOALS_CATALOG};

pub(super) fn selected_goal_ids(raw: Option<&str>) -> Result<Vec<String>> {
    let available =
        VCF_GOALS_CATALOG.iter().map(|goal| goal.goal_id.to_string()).collect::<BTreeSet<_>>();
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
        return Err(anyhow!("vcf-benchmark-campaign requires at least one goal"));
    }
    let unknown =
        selected.iter().filter(|goal| !available.contains(*goal)).cloned().collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!("unknown vcf goals requested: {}", unknown.join(",")));
    }
    let mut ordered = VCF_GOALS_CATALOG
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
