use std::collections::{BTreeMap, BTreeSet};
use std::process::Output;
use std::sync::mpsc;
use std::time::Instant;

use anyhow::{ensure, Context, Result};

use super::{
    DependencyProcessOutput, DependentProcessSpec, ParallelProcessOutput, ParallelProcessSpec,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Pending,
    Running,
    Passed,
    Failed,
    Blocked,
}

fn dependencies(specs: &[DependentProcessSpec], max_parallelism: usize) -> Result<Vec<Vec<usize>>> {
    ensure!(max_parallelism > 0, "dependency process parallelism must be positive");
    let mut indices = BTreeMap::new();
    for (index, spec) in specs.iter().enumerate() {
        ensure!(!spec.process.label.is_empty(), "dependency process label must not be empty");
        ensure!(
            indices.insert(spec.process.label.as_str(), index).is_none(),
            "duplicate dependency process label: {}",
            spec.process.label
        );
    }
    let dependencies = specs
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            let mut seen = BTreeSet::new();
            spec.dependencies
                .iter()
                .map(|dependency| {
                    let dependency_index =
                        *indices.get(dependency.as_str()).with_context(|| {
                            format!("unknown dependency {dependency} for {}", spec.process.label)
                        })?;
                    ensure!(
                        dependency_index != index && seen.insert(dependency_index),
                        "self or duplicate dependency for {}",
                        spec.process.label
                    );
                    Ok(dependency_index)
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    let mut resolved = BTreeSet::new();
    while resolved.len() < specs.len() {
        let ready = dependencies
            .iter()
            .enumerate()
            .filter_map(|(index, dependencies)| {
                (!resolved.contains(&index) && dependencies.iter().all(|id| resolved.contains(id)))
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        ensure!(!ready.is_empty(), "dependency process graph contains a cycle");
        resolved.extend(ready);
    }
    Ok(dependencies)
}

pub(super) fn collect<F>(
    specs: Vec<DependentProcessSpec>,
    max_parallelism: usize,
    run: F,
) -> Result<Vec<DependencyProcessOutput>>
where
    F: Fn(&ParallelProcessSpec) -> Result<Output> + Sync,
{
    let dependencies = dependencies(&specs, max_parallelism)?;
    let mut states = vec![State::Pending; specs.len()];
    let mut outputs = (0..specs.len()).map(|_| None).collect::<Vec<_>>();
    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        let mut active = 0usize;
        let mut first_error = None;
        loop {
            if first_error.is_none() {
                // Propagate failure to a fixed point before admitting independent ready work.
                loop {
                    let blocked = (0..specs.len())
                        .filter(|&index| {
                            states[index] == State::Pending
                                && dependencies[index]
                                    .iter()
                                    .any(|&id| matches!(states[id], State::Failed | State::Blocked))
                        })
                        .collect::<Vec<_>>();
                    if blocked.is_empty() {
                        break;
                    }
                    for index in blocked {
                        states[index] = State::Blocked;
                        outputs[index] = Some(DependencyProcessOutput::BlockedDependency {
                            label: specs[index].process.label.clone(),
                        });
                    }
                }
                for index in 0..specs.len() {
                    if active == max_parallelism {
                        break;
                    }
                    if states[index] != State::Pending
                        || !dependencies[index].iter().all(|&id| states[id] == State::Passed)
                    {
                        continue;
                    }
                    states[index] = State::Running;
                    active += 1;
                    let sender = sender.clone();
                    let spec = &specs[index].process;
                    let run = &run;
                    scope.spawn(move || {
                        let started = Instant::now();
                        // Every admitted child must send a terminal result even if its executor
                        // panics; the coordinator retains the sender for later slot admission.
                        let result =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(spec)))
                                .unwrap_or_else(|_| {
                                    Err(anyhow::anyhow!(
                                        "dependency process {} executor panicked",
                                        spec.label
                                    ))
                                })
                                .map(|output| ParallelProcessOutput {
                                    label: spec.label.clone(),
                                    output,
                                    duration: started.elapsed(),
                                });
                        let _ = sender.send((index, result));
                    });
                }
            }
            if active == 0 {
                break;
            }
            let (index, result) =
                receiver.recv().context("dependency process completion channel closed")?;
            active -= 1;
            match result {
                Ok(output) => {
                    states[index] =
                        if output.output.status.success() { State::Passed } else { State::Failed };
                    outputs[index] = Some(DependencyProcessOutput::Completed(output));
                }
                Err(error) => {
                    states[index] = State::Failed;
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        outputs
            .into_iter()
            .enumerate()
            .map(|(index, output)| {
                output.with_context(|| {
                    format!("dependency process {} was not collected", specs[index].process.label)
                })
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Condvar, Mutex};
    use std::time::Duration;

    use super::*;

    fn spec(label: &str, dependencies: &[&str]) -> DependentProcessSpec {
        DependentProcessSpec {
            process: ParallelProcessSpec {
                label: label.into(),
                program: "sh".into(),
                args: vec!["-c".into(), "exit 0".into()],
                envs: vec![],
                current_dir: None,
            },
            dependencies: dependencies.iter().map(|id| (*id).into()).collect(),
        }
    }

    fn output(success: bool) -> Result<Output> {
        Ok(std::process::Command::new("sh")
            .args(["-c", if success { "exit 0" } else { "exit 7" }])
            .output()?)
    }

    #[test]
    fn consumer_starts_while_an_independent_peer_is_still_running() -> Result<()> {
        let signal = (Mutex::new(false), Condvar::new());
        let outputs = collect(
            vec![spec("producer", &[]), spec("peer", &[]), spec("consumer", &["producer"])],
            2,
            |spec| {
                if spec.label == "peer" {
                    let (ready, _) = signal
                        .1
                        .wait_timeout_while(
                            signal.0.lock().unwrap(),
                            Duration::from_secs(10),
                            |ready| !*ready,
                        )
                        .unwrap();
                    ensure!(*ready, "consumer waited for the independent peer batch barrier");
                } else if spec.label == "consumer" {
                    *signal.0.lock().unwrap() = true;
                    signal.1.notify_all();
                }
                output(true)
            },
        )?;
        assert_eq!(outputs.len(), 3);
        for (result, expected) in outputs.iter().zip(["producer", "peer", "consumer"]) {
            assert!(
                matches!(result, DependencyProcessOutput::Completed(output) if output.label == expected)
            );
        }
        Ok(())
    }

    #[test]
    fn failed_dependencies_block_descendants_but_allow_independent_work() -> Result<()> {
        let launched = Mutex::new(Vec::new());
        let outputs = collect(
            vec![
                spec("failed", &[]),
                spec("child", &["failed"]),
                spec("descendant", &["child"]),
                spec("independent", &[]),
            ],
            2,
            |spec| {
                launched.lock().unwrap().push(spec.label.clone());
                output(spec.label != "failed")
            },
        )?;
        assert!(
            matches!(&outputs[1], DependencyProcessOutput::BlockedDependency { label } if label == "child")
        );
        assert!(
            matches!(&outputs[2], DependencyProcessOutput::BlockedDependency { label } if label == "descendant")
        );
        let mut launched = launched.into_inner().unwrap();
        launched.sort();
        assert_eq!(launched, ["failed", "independent"]);
        Ok(())
    }

    #[test]
    fn malformed_graphs_are_rejected_before_any_process_runs() {
        for graph in [
            vec![spec("same", &[]), spec("same", &[])],
            vec![spec("unknown", &["absent"])],
            vec![spec("self", &["self"])],
            vec![spec("a", &["b"]), spec("b", &["a"])],
            vec![spec("a", &[]), spec("b", &["a", "a"])],
        ] {
            assert!(collect(graph, 2, |_| panic!("invalid graph launched")).is_err());
        }
        assert!(collect(vec![spec("a", &[])], 0, |_| panic!("invalid budget launched")).is_err());
    }

    #[test]
    fn active_children_are_drained_after_a_launch_error() -> Result<()> {
        let launched = Mutex::new(Vec::new());
        let completed = Mutex::new(Vec::new());
        let result = collect(
            vec![spec("missing", &[]), spec("peer", &[]), spec("pending", &[])],
            2,
            |spec| {
                launched.lock().unwrap().push(spec.label.clone());
                if spec.label == "missing" {
                    anyhow::bail!("executable missing");
                }
                let result = output(true);
                completed.lock().unwrap().push(spec.label.clone());
                result
            },
        );
        assert!(result.is_err());
        let mut launched = launched.into_inner().unwrap();
        launched.sort();
        // A completed peer may release a slot before the launch error is observed.
        assert!(launched.contains(&"peer".to_string()));
        assert!(launched.contains(&"missing".to_string()));
        assert!(completed.into_inner().unwrap().contains(&"peer".to_string()));
        Ok(())
    }

    #[test]
    fn panicked_executor_returns_an_error_after_draining_peers() {
        let completed = Mutex::new(Vec::new());
        let result = collect(vec![spec("panicked", &[]), spec("peer", &[])], 2, |spec| {
            assert_ne!(spec.label, "panicked", "synthetic executor panic");
            let result = output(true);
            completed.lock().unwrap().push(spec.label.clone());
            result
        });
        assert!(result.is_err());
        assert_eq!(completed.into_inner().unwrap(), ["peer"]);
    }

    #[test]
    fn ready_work_respects_process_budget_and_join_dependencies() -> Result<()> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let active = AtomicUsize::new(0);
        let maximum = AtomicUsize::new(0);
        let completed = Mutex::new(BTreeSet::new());
        let mut graph =
            (0..20).map(|index| spec(&format!("producer-{index}"), &[])).collect::<Vec<_>>();
        let mut join = spec("join", &[]);
        join.dependencies = graph.iter().map(|spec| spec.process.label.clone()).collect();
        graph.push(join);
        let outputs = collect(graph, 2, |spec| {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            maximum.fetch_max(current, Ordering::SeqCst);
            if spec.label == "join" {
                ensure!(
                    completed.lock().unwrap().len() == 20,
                    "join started before all producers passed"
                );
            }
            let result = output(true);
            completed.lock().unwrap().insert(spec.label.clone());
            active.fetch_sub(1, Ordering::SeqCst);
            result
        })?;
        assert_eq!(outputs.len(), 21);
        assert!(maximum.load(Ordering::SeqCst) <= 2);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        Ok(())
    }
}
