use super::HpcSelectedJob;
use crate::PlannedJob;
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

/// Explicit campaign scope supplied by the command adapter.
#[derive(Debug, Clone)]
pub enum HpcSubmissionSelector {
    All,
    Stage { stage: String, tool: Option<String>, sample: Option<String> },
    Domain { domain: String },
    Cross { domains: Vec<String> },
}

#[must_use]
pub fn hpc_stage_domain(stage: &str) -> &str {
    stage.split('.').next().unwrap_or("unknown")
}

pub(super) fn select<'a>(
    jobs: &'a [PlannedJob],
    selector: &HpcSubmissionSelector,
) -> Result<Vec<HpcSelectedJob<'a>>> {
    let selected: Vec<_> = jobs
        .iter()
        .filter(|job| match selector {
            HpcSubmissionSelector::All => true,
            HpcSubmissionSelector::Stage { stage, tool, sample } => {
                job.stage == *stage
                    && tool.as_ref().is_none_or(|tool| job.tool == *tool)
                    && sample.as_ref().is_none_or(|sample| job.sample == *sample)
            }
            HpcSubmissionSelector::Domain { domain } => hpc_stage_domain(&job.stage) == domain,
            HpcSubmissionSelector::Cross { domains } => {
                domains.iter().any(|domain| hpc_stage_domain(&job.stage) == domain)
            }
        })
        .map(|planned| HpcSelectedJob {
            name: &planned.job_name,
            planned,
            depends_on_names: &planned.depends_on,
        })
        .collect();
    if matches!(selector, HpcSubmissionSelector::Cross { .. }) {
        let domains: BTreeSet<_> =
            selected.iter().map(|job| hpc_stage_domain(&job.planned.stage)).collect();
        ensure!(
            domains.len() >= 2,
            "cross-domain submission requires jobs from at least two domains; found {}",
            domains.len()
        );
    }
    ensure!(!selected.is_empty(), "no jobs matched submission selector");
    Ok(selected)
}
