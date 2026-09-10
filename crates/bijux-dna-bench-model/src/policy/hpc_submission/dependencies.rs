use super::HpcSelectedJob;
use crate::PlannedJob;
use anyhow::{ensure, Result};
use std::collections::BTreeMap;

pub(super) fn admit(
    all: &[PlannedJob],
    selected: &[HpcSelectedJob<'_>],
) -> Result<Vec<Vec<usize>>> {
    let mut names = BTreeMap::new();
    for job in all {
        ensure!(!job.job_name.trim().is_empty(), "blank campaign job name");
        ensure!(
            names.insert(job.job_name.as_str(), job).is_none(),
            "duplicate campaign job name: {}",
            job.job_name
        );
    }
    let positions: BTreeMap<_, _> =
        selected.iter().enumerate().map(|(index, job)| (job.name, index)).collect();
    let mut last_by_sample = BTreeMap::new();
    let mut graph = Vec::with_capacity(selected.len());
    for (index, job) in selected.iter().enumerate() {
        let mut dependencies = Vec::new();
        for name in job.depends_on_names {
            ensure!(
                names.contains_key(name.as_str()),
                "unknown dependency {name} for {}",
                job.name
            );
            // Scoped submission has always excluded jobs outside its selected domain/stage.
            // Preserve that boundary, but never silently discard an in-scope dependency.
            if let Some(prior) = positions.get(name.as_str()) {
                ensure!(
                    *prior < index,
                    "dependency {name} must precede {} in the submission plan",
                    job.name
                );
                dependencies.push(*prior);
            }
        }
        if let Some(prior) = last_by_sample.insert(job.planned.sample.as_str(), index) {
            if !dependencies.contains(&prior) {
                dependencies.push(prior);
            }
        }
        graph.push(dependencies);
    }
    Ok(graph)
}
