use std::collections::{BTreeMap, BTreeSet};
use std::process::Output;
use std::sync::mpsc;
use std::time::Instant;

use anyhow::{ensure, Context, Result};

use super::{
    DependencyProcessOutput, DependentProcessSpec, ParallelProcessOutput, ParallelProcessSpec,
    ProvisionedResourceUnits, WeightedProcessSpec,
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
    collect_weighted(
        specs
            .into_iter()
            .map(|process| WeightedProcessSpec {
                process,
                resource_units: 1,
                terminal_dependencies: Vec::new(),
            })
            .collect(),
        max_parallelism,
        max_parallelism,
        run,
    )
}

pub(super) fn collect_weighted<F>(
    specs: Vec<WeightedProcessSpec>,
    max_parallelism: usize,
    resource_capacity: usize,
    run: F,
) -> Result<Vec<DependencyProcessOutput>>
where
    F: Fn(&ParallelProcessSpec) -> Result<Output> + Sync,
{
    collect_provisioned(specs, max_parallelism, resource_capacity, None, run)
}

pub(super) struct AuxiliaryReservation {
    pub capacity: ProvisionedResourceUnits,
    pub demands: Vec<ProvisionedResourceUnits>,
}

impl ProvisionedResourceUnits {
    fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            memory_bytes: self.memory_bytes.checked_add(other.memory_bytes)?,
            scratch_bytes: self.scratch_bytes.checked_add(other.scratch_bytes)?,
            io_slots: self.io_slots.checked_add(other.io_slots)?,
        })
    }
    fn checked_sub(self, other: Self) -> Option<Self> {
        Some(Self {
            memory_bytes: self.memory_bytes.checked_sub(other.memory_bytes)?,
            scratch_bytes: self.scratch_bytes.checked_sub(other.scratch_bytes)?,
            io_slots: self.io_slots.checked_sub(other.io_slots)?,
        })
    }
    fn fits(self, capacity: Self) -> bool {
        self.memory_bytes <= capacity.memory_bytes
            && self.scratch_bytes <= capacity.scratch_bytes
            && self.io_slots <= capacity.io_slots
    }
}

pub(super) fn collect_provisioned<F>(
    specs: Vec<WeightedProcessSpec>,
    max_parallelism: usize,
    resource_capacity: usize,
    auxiliary: Option<AuxiliaryReservation>,
    run: F,
) -> Result<Vec<DependencyProcessOutput>>
where
    F: Fn(&ParallelProcessSpec) -> Result<Output> + Sync,
{
    if let Some(reservation) = &auxiliary {
        ensure!(
            reservation.demands.len() == specs.len()
                && reservation.capacity.memory_bytes > 0
                && reservation.capacity.io_slots > 0,
            "invalid provisioned resource roster/capacity"
        );
        for demand in &reservation.demands {
            ensure!(
                demand.memory_bytes > 0 && demand.io_slots > 0 && demand.fits(reservation.capacity),
                "declared process does not fit provisioned resources"
            );
        }
        reservation
            .demands
            .iter()
            .try_fold(ProvisionedResourceUnits::default(), |total, demand| {
                total.checked_add(*demand)
            })
            .context("declared provisioned resources overflow")?;
    }
    ensure!(resource_capacity > 0, "weighted resource capacity must be positive");
    for spec in &specs {
        ensure!(
            spec.resource_units > 0 && spec.resource_units <= resource_capacity,
            "weighted process {} does not fit the resource capacity",
            spec.process.process.label
        );
    }
    specs
        .iter()
        .try_fold(0usize, |total, spec| total.checked_add(spec.resource_units))
        .context("declared weighted process units overflow")?;
    let combined = specs
        .iter()
        .map(|spec| DependentProcessSpec {
            process: spec.process.process.clone(),
            dependencies: spec
                .process
                .dependencies
                .iter()
                .chain(&spec.terminal_dependencies)
                .cloned()
                .collect(),
        })
        .collect::<Vec<_>>();
    let combined_dependencies = dependencies(&combined, max_parallelism)?;
    let dependencies = specs
        .iter()
        .zip(&combined_dependencies)
        .map(|(spec, dependencies)| dependencies[..spec.process.dependencies.len()].to_vec())
        .collect::<Vec<_>>();
    let terminal = specs
        .iter()
        .zip(&combined_dependencies)
        .map(|(spec, dependencies)| dependencies[spec.process.dependencies.len()..].to_vec())
        .collect::<Vec<_>>();

    let mut states = vec![State::Pending; specs.len()];
    let mut outputs = (0..specs.len()).map(|_| None).collect::<Vec<_>>();
    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        let mut active = 0usize;
        let mut occupied_units = 0usize;
        let mut occupied_resources = ProvisionedResourceUnits::default();
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
                            label: specs[index].process.process.label.clone(),
                        });
                    }
                }
                for index in 0..specs.len() {
                    if active == max_parallelism {
                        break;
                    }
                    if states[index] != State::Pending
                        || !dependencies[index].iter().all(|&id| states[id] == State::Passed)
                        || !terminal[index].iter().all(|&id| {
                            matches!(states[id], State::Passed | State::Failed | State::Blocked)
                        })
                        || occupied_units
                            .checked_add(specs[index].resource_units)
                            .is_none_or(|units| units > resource_capacity)
                        || auxiliary.as_ref().is_some_and(|reservation| {
                            occupied_resources
                                .checked_add(reservation.demands[index])
                                .is_none_or(|total| !total.fits(reservation.capacity))
                        })
                    {
                        continue;
                    }
                    if let Some(reservation) = &auxiliary {
                        occupied_resources = occupied_resources
                            .checked_add(reservation.demands[index])
                            .context("admitted resource reservation overflow")?;
                    }
                    states[index] = State::Running;
                    active += 1;
                    occupied_units += specs[index].resource_units;
                    let sender = sender.clone();
                    let spec = &specs[index].process.process;
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
            occupied_units -= specs[index].resource_units;
            if let Some(reservation) = &auxiliary {
                occupied_resources = occupied_resources
                    .checked_sub(reservation.demands[index])
                    .context("resource reservation released more than once")?;
            }
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
        ensure!(
            occupied_resources == ProvisionedResourceUnits::default(),
            "provisioned reservations were not drained"
        );
        if let Some(error) = first_error {
            return Err(error);
        }
        outputs
            .into_iter()
            .enumerate()
            .map(|(index, output)| {
                output.with_context(|| {
                    format!(
                        "dependency process {} was not collected",
                        specs[index].process.process.label
                    )
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

    fn weighted(
        label: &str,
        weight: usize,
        success: &[&str],
        terminal: &[&str],
    ) -> WeightedProcessSpec {
        WeightedProcessSpec {
            process: spec(label, success),
            resource_units: weight,
            terminal_dependencies: terminal.iter().map(|value| (*value).to_string()).collect(),
        }
    }

    fn demand(memory_bytes: u64, scratch_bytes: u64, io_slots: usize) -> ProvisionedResourceUnits {
        ProvisionedResourceUnits { memory_bytes, scratch_bytes, io_slots }
    }

    #[test]
    fn provisioned_ram_disk_and_io_each_defer_cpu_ready_work_until_completion() -> Result<()> {
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        for capacity in [demand(3, 10, 10), demand(10, 3, 10), demand(10, 10, 2)] {
            let signal = (Mutex::new(false), Condvar::new());
            let completed = AtomicBool::new(false);
            let active = AtomicU64::new(0);
            let maximum = AtomicU64::new(0);
            let outputs = collect_provisioned(
                vec![
                    weighted("producer", 1, &[], &[]),
                    weighted("peer", 1, &[], &[]),
                    weighted("consumer", 1, &["producer"], &[]),
                    weighted("fitting", 1, &[], &[]),
                ],
                4,
                4,
                Some(AuxiliaryReservation {
                    capacity,
                    demands: vec![
                        demand(2, 2, 1),
                        demand(1, 1, 1),
                        demand(2, 2, 1),
                        demand(1, 1, 1),
                    ],
                }),
                |spec| {
                    let weight =
                        if spec.label == "producer" || spec.label == "consumer" { 2 } else { 1 };
                    let current = active.fetch_add(weight, Ordering::SeqCst) + weight;
                    maximum.fetch_max(current, Ordering::SeqCst);
                    if spec.label == "peer" {
                        let (ready, _) = signal
                            .1
                            .wait_timeout_while(
                                signal.0.lock().map_err(|_| {
                                    anyhow::anyhow!("scheduler test mutex was poisoned")
                                })?,
                                Duration::from_secs(10),
                                |ready| !*ready,
                            )
                            .map_err(|_| {
                                anyhow::anyhow!("scheduler test condition mutex was poisoned")
                            })?;
                        ensure!(*ready, "consumer failed to refill reserved slot");
                    } else if spec.label == "consumer" {
                        completed.store(true, Ordering::SeqCst);
                        *signal
                            .0
                            .lock()
                            .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))? =
                            true;
                        signal.1.notify_all();
                    } else if spec.label == "fitting" {
                        ensure!(
                            completed.load(Ordering::SeqCst),
                            "CPU-only admission ignored RAM/IO reservations"
                        );
                    }
                    let output = output(true);
                    active.fetch_sub(weight, Ordering::SeqCst);
                    output
                },
            )?;
            assert_eq!(outputs.len(), 4);
            assert!(maximum.load(Ordering::SeqCst) <= 3);
            assert_eq!(active.load(Ordering::SeqCst), 0);
        }
        Ok(())
    }

    #[test]
    fn provisioned_unknown_zero_overcapacity_and_overflow_fail_before_launch() {
        for reservation in [
            AuxiliaryReservation { capacity: demand(10, 10, 2), demands: vec![demand(0, 1, 1)] },
            AuxiliaryReservation { capacity: demand(10, 10, 2), demands: vec![demand(11, 1, 1)] },
            AuxiliaryReservation { capacity: demand(10, 10, 2), demands: vec![demand(1, 11, 1)] },
            AuxiliaryReservation { capacity: demand(10, 10, 2), demands: vec![demand(1, 1, 0)] },
            AuxiliaryReservation { capacity: demand(10, 10, 2), demands: vec![demand(1, 1, 3)] },
            AuxiliaryReservation { capacity: demand(10, 10, 2), demands: vec![] },
        ] {
            assert!(collect_provisioned(
                vec![weighted("refused", 1, &[], &[])],
                1,
                1,
                Some(reservation),
                |_| panic!("invalid provisioned child launched")
            )
            .is_err());
        }
        for (capacity, demands) in [
            (demand(u64::MAX, 0, 2), vec![demand(u64::MAX, 0, 1), demand(1, 0, 1)]),
            (demand(2, u64::MAX, 2), vec![demand(1, u64::MAX, 1), demand(1, 1, 1)]),
            (demand(2, 2, usize::MAX), vec![demand(1, 1, usize::MAX), demand(1, 1, 1)]),
        ] {
            assert!(collect_provisioned(
                vec![weighted("large", 1, &[], &[]), weighted("small", 1, &[], &[])],
                2,
                2,
                Some(AuxiliaryReservation { capacity, demands }),
                |_| panic!("overflowed demand launched")
            )
            .is_err());
        }
    }

    #[test]
    fn provisioned_peers_drain_after_launch_error_and_executor_panic() -> Result<()> {
        use std::sync::atomic::{AtomicBool, Ordering};
        for panicked in [false, true] {
            let finished = AtomicBool::new(false);
            let result = collect_provisioned(
                vec![weighted("failed", 1, &[], &[]), weighted("peer", 1, &[], &[])],
                2,
                2,
                Some(AuxiliaryReservation {
                    capacity: demand(2, 2, 2),
                    demands: vec![demand(1, 1, 1), demand(1, 1, 1)],
                }),
                |spec| {
                    if spec.label == "failed" {
                        assert!(!panicked, "executor failure");
                        anyhow::bail!("launcher failure");
                    }
                    finished.store(true, Ordering::SeqCst);
                    output(true)
                },
            );
            assert!(result.is_err());
            assert!(finished.load(Ordering::SeqCst));
        }
        Ok(())
    }

    #[test]
    fn failed_success_consumer_is_blocked_while_terminal_consumers_run() -> Result<()> {
        let outputs = collect_weighted(
            vec![
                weighted("producer", 2, &[], &[]),
                weighted("success", 1, &["producer"], &[]),
                weighted("terminal", 1, &[], &["producer"]),
                weighted("blocked-terminal", 1, &[], &["success"]),
            ],
            3,
            3,
            |spec| output(spec.label != "producer"),
        )?;
        assert!(matches!(&outputs[1], DependencyProcessOutput::BlockedDependency { .. }));
        assert!(
            matches!(&outputs[2], DependencyProcessOutput::Completed(value) if value.output.status.success())
        );
        assert!(
            matches!(&outputs[3], DependencyProcessOutput::Completed(value) if value.output.status.success())
        );
        Ok(())
    }

    #[test]
    fn weighted_admission_rejects_bad_weights_and_mixed_dependency_cycles_before_launch() {
        for (specs, capacity) in [
            (vec![weighted("zero", 0, &[], &[])], 2),
            (vec![weighted("large", 3, &[], &[])], 2),
            (vec![weighted("a", 1, &[], &["b"]), weighted("b", 1, &["a"], &[])], 2),
            (vec![weighted("a", 1, &[], &["missing"])], 2),
            (vec![weighted("a", 1, &[], &["a"])], 2),
            (vec![weighted("a", 1, &[], &[]), weighted("b", 1, &["a"], &["a"])], 2),
            (vec![weighted("a", usize::MAX, &[], &[]), weighted("b", 1, &[], &[])], usize::MAX),
            (vec![weighted("a", 1, &[], &[])], 0),
        ] {
            assert!(collect_weighted(specs, 2, capacity, |_| panic!(
                "invalid weighted process launched"
            ))
            .is_err());
        }
    }

    #[test]
    fn fitting_work_refills_weighted_slots_while_an_independent_peer_waits() -> Result<()> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let signal = (Mutex::new(false), Condvar::new());
        let active = AtomicUsize::new(0);
        let maximum = AtomicUsize::new(0);
        let outputs = collect_weighted(
            vec![
                weighted("producer", 2, &[], &[]),
                weighted("peer", 1, &[], &[]),
                weighted("consumer", 2, &["producer"], &[]),
            ],
            3,
            3,
            |spec| {
                let weight = if spec.label == "peer" { 1 } else { 2 };
                let current = active.fetch_add(weight, Ordering::SeqCst) + weight;
                maximum.fetch_max(current, Ordering::SeqCst);
                if spec.label == "peer" {
                    let (ready, _) = signal
                        .1
                        .wait_timeout_while(
                            signal.0.lock().map_err(|_| {
                                anyhow::anyhow!("scheduler test mutex was poisoned")
                            })?,
                            Duration::from_secs(10),
                            |value| !*value,
                        )
                        .map_err(|_| {
                            anyhow::anyhow!("scheduler test condition mutex was poisoned")
                        })?;
                    ensure!(*ready, "fitting consumer waited for peer");
                } else if spec.label == "consumer" {
                    *signal
                        .0
                        .lock()
                        .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))? = true;
                    signal.1.notify_all();
                }
                let result = output(true);
                active.fetch_sub(weight, Ordering::SeqCst);
                result
            },
        )?;
        assert_eq!(outputs.len(), 3);
        assert!(maximum.load(Ordering::SeqCst) <= 3);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        Ok(())
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
                            signal.0.lock().map_err(|_| {
                                anyhow::anyhow!("scheduler test mutex was poisoned")
                            })?,
                            Duration::from_secs(10),
                            |ready| !*ready,
                        )
                        .map_err(|_| {
                            anyhow::anyhow!("scheduler test condition mutex was poisoned")
                        })?;
                    ensure!(*ready, "consumer waited for the independent peer batch barrier");
                } else if spec.label == "consumer" {
                    *signal
                        .0
                        .lock()
                        .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))? = true;
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
                launched
                    .lock()
                    .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?
                    .push(spec.label.clone());
                output(spec.label != "failed")
            },
        )?;
        assert!(
            matches!(&outputs[1], DependencyProcessOutput::BlockedDependency { label } if label == "child")
        );
        assert!(
            matches!(&outputs[2], DependencyProcessOutput::BlockedDependency { label } if label == "descendant")
        );
        let mut launched = launched
            .into_inner()
            .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?;
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
                launched
                    .lock()
                    .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?
                    .push(spec.label.clone());
                if spec.label == "missing" {
                    anyhow::bail!("executable missing");
                }
                let result = output(true);
                completed
                    .lock()
                    .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?
                    .push(spec.label.clone());
                result
            },
        );
        assert!(result.is_err());
        let mut launched = launched
            .into_inner()
            .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?;
        launched.sort();
        // A completed peer may release a slot before the launch error is observed.
        assert!(launched.contains(&"peer".to_string()));
        assert!(launched.contains(&"missing".to_string()));
        assert!(completed
            .into_inner()
            .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?
            .contains(&"peer".to_string()));
        Ok(())
    }

    #[test]
    fn panicked_executor_returns_an_error_after_draining_peers() -> Result<()> {
        let completed = Mutex::new(Vec::new());
        let result = collect(vec![spec("panicked", &[]), spec("peer", &[])], 2, |spec| {
            assert_ne!(spec.label, "panicked", "synthetic executor panic");
            let result = output(true);
            completed
                .lock()
                .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?
                .push(spec.label.clone());
            result
        });
        assert!(result.is_err());
        assert_eq!(
            completed
                .into_inner()
                .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?,
            ["peer"]
        );
        Ok(())
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
                    completed
                        .lock()
                        .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?
                        .len()
                        == 20,
                    "join started before all producers passed"
                );
            }
            let result = output(true);
            completed
                .lock()
                .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))?
                .insert(spec.label.clone());
            active.fetch_sub(1, Ordering::SeqCst);
            result
        })?;
        assert_eq!(outputs.len(), 21);
        assert!(maximum.load(Ordering::SeqCst) <= 2);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        Ok(())
    }
}
