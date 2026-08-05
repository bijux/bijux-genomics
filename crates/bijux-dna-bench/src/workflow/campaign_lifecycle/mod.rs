//! Owner: bijux-dna-bench
//! Campaign lifecycle planning from governed benchmark declarations.

mod planning;
mod storage;

pub use planning::{plan_stage_campaign_attempts, verify_campaign_plan_source};
pub use storage::ensure_stage_campaign_attempts;
