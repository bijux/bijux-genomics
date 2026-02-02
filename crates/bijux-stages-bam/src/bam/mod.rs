//! BAM stage planners.

use anyhow::Result;
use bijux_core::StagePlanV1;

pub mod authenticity;
pub mod bias_mitigation;
pub mod complexity;
pub mod contamination;
pub mod coverage;
pub mod damage;
pub mod filter;
pub mod genotyping;
pub mod haplogroups;
pub mod kinship;
pub mod markdup;
pub mod qc_pre;
pub mod recalibration;
pub mod sex;
pub mod validate;

fn ensure_required_outputs(plan: StagePlanV1, required: &[&str]) -> Result<StagePlanV1> {
    let mut missing = Vec::new();
    for name in required {
        if !plan.io.outputs.iter().any(|output| output.name == *name) {
            missing.push(*name);
        }
    }
    if !missing.is_empty() {
        anyhow::bail!(
            "stage {} missing required outputs: {}",
            plan.stage_id.0,
            missing.join(", ")
        );
    }
    Ok(plan)
}
