use crate::decision::compare::CompareRobustStats;
use crate::decision::score::{decision_trace_for_input, RankInput, RankingMode};
use crate::decision::DecisionTrace;

#[must_use]
pub fn decision_trace_for_ranking(mode: RankingMode, input: &RankInput) -> DecisionTrace {
    decision_trace_for_input(mode, input)
}

#[must_use]
pub fn decision_trace_for_compare(stats: &CompareRobustStats) -> DecisionTrace {
    crate::decision::compare::trace_for_robust_stats(stats)
}
