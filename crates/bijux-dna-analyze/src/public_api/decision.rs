pub mod compare {
    pub use crate::decision::compare::*;
}

pub mod ranking {
    pub use crate::decision::score::*;
}

pub use crate::decision::compare::compare_runs;
pub use crate::decision::score::{
    build_rankings, decision_trace_for_input, print_rank_explain, RankInput, RankingEntry,
    RankingMode, ScoreBreakdown,
};
