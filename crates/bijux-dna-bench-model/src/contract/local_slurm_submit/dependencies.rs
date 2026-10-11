//! Acyclic admission after submission identities and dependency closure are validated.

use crate::BenchError;
use std::collections::{BTreeMap, VecDeque};

pub(super) fn validate_acyclic<'a>(
    jobs: impl Iterator<Item = (&'a str, &'a [String])>,
) -> Result<(), BenchError> {
    let nodes = jobs.collect::<BTreeMap<_, _>>();
    let mut remaining =
        nodes.iter().map(|(&id, deps)| (id, deps.len())).collect::<BTreeMap<_, _>>();
    let mut dependents = BTreeMap::<&str, Vec<&str>>::new();
    for (&id, dependencies) in &nodes {
        for dependency in *dependencies {
            dependents.entry(dependency.as_str()).or_default().push(id);
        }
    }
    let mut ready = remaining
        .iter()
        .filter_map(|(&id, &count)| (count == 0).then_some(id))
        .collect::<VecDeque<_>>();
    let mut visited = 0;
    while let Some(id) = ready.pop_front() {
        visited += 1;
        if let Some(children) = dependents.get(id) {
            for child in children {
                let count = remaining.get_mut(child).ok_or_else(|| {
                    BenchError::InvalidPolicy("Slurm dependency references an unknown job".into())
                })?;
                *count -= 1;
                if *count == 0 {
                    ready.push_back(child);
                }
            }
        }
    }
    if visited != nodes.len() {
        return Err(BenchError::InvalidPolicy(
            "Slurm submission dependencies must be acyclic".into(),
        ));
    }
    Ok(())
}
