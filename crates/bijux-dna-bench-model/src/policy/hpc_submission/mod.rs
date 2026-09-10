//! Pure selection and dependency admission for one scheduler submission scope.
mod dependencies;
mod selection;

use crate::PlannedJob;
use anyhow::Result;
pub use selection::{hpc_stage_domain, HpcSubmissionSelector};

/// A selected job borrows the current campaign plan; it is not a scheduler receipt.
#[derive(Debug)]
pub struct HpcSelectedJob<'a> {
    pub name: &'a str,
    pub planned: &'a PlannedJob,
    pub depends_on_names: &'a [String],
}

/// Admitted, ordered selected jobs and their in-scope dependencies.
/// Declared dependencies outside the selected scope remain in each job's declarations.
#[derive(Debug)]
pub struct HpcSubmissionPlan<'a> {
    jobs: Vec<HpcSelectedJob<'a>>,
    dependencies: Vec<Vec<usize>>,
}

impl HpcSubmissionPlan<'_> {
    #[must_use]
    pub fn jobs(&self) -> &[HpcSelectedJob<'_>] {
        &self.jobs
    }

    /// Resolve dependencies against exactly the receipts preceding this job.
    /// # Errors
    /// Rejects an invalid position, incomplete receipt prefix or blank scheduler identity.
    pub fn scheduler_dependencies(
        &self,
        index: usize,
        submitted: &[String],
    ) -> Result<Vec<String>> {
        anyhow::ensure!(
            index < self.jobs.len() && submitted.len() == index,
            "scheduler receipts must match the preceding submission prefix"
        );
        anyhow::ensure!(
            submitted.iter().all(|id| !id.trim().is_empty()),
            "blank scheduler receipt identity"
        );
        self.dependencies[index]
            .iter()
            .map(|prior| {
                submitted
                    .get(*prior)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("missing admitted dependency receipt"))
            })
            .collect()
    }
}

/// Select and admit the complete local dependency graph before scheduler or filesystem effects.
/// # Errors
/// Rejects an empty/invalid scope, ambiguous job names, unknown declarations and forward dependencies.
pub fn plan_hpc_submission<'a>(
    jobs: &'a [PlannedJob],
    selector: &HpcSubmissionSelector,
) -> Result<HpcSubmissionPlan<'a>> {
    let selected = selection::select(jobs, selector)?;
    let dependencies = dependencies::admit(jobs, &selected)?;
    Ok(HpcSubmissionPlan { jobs: selected, dependencies })
}
