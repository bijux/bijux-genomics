use crate::decision::score::{RankingEntry, RankingMode};

pub(super) fn annotate_why_not_first(entries: &mut [RankingEntry], mode: RankingMode) {
    if entries.is_empty() {
        return;
    }
    let best_score = entries[0].score;
    for entry in entries.iter_mut().skip(1) {
        let explain = match mode {
            RankingMode::FastestAcceptable => "runtime is slower".to_string(),
            RankingMode::MostConservative => "retention/quality sum is lower".to_string(),
            RankingMode::BalancedPareto => "composite score is lower".to_string(),
        };
        entry.why_not_first.push(explain);
        if (entry.score - best_score).abs() <= f64::EPSILON {
            entry
                .why_not_first
                .push("tie broken by tool_id".to_string());
        }
    }
}
