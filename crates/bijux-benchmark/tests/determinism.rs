use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_benchmark::{gate, summarize, BenchRunOptions, BenchmarkSuiteSpec};
use bijux_benchmark_model::{GatePolicy, GatePolicyOverrides};
use bijux_core::contract::canonical::to_canonical_json_bytes;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bench_bundle")
}

fn load_observations() -> Result<Vec<bijux_benchmark::BenchmarkObservation>> {
    let path = fixture_root().join("observations.jsonl");
    let raw = fs::read_to_string(path)?;
    let mut observations = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        observations.push(serde_json::from_str(line)?);
    }
    Ok(observations)
}

fn suite_from_observations(
    observations: &[bijux_benchmark::BenchmarkObservation],
) -> BenchmarkSuiteSpec {
    let first = observations.first().expect("observations");
    BenchmarkSuiteSpec::v1(
        "suite-fixture".to_string(),
        vec![bijux_benchmark::DatasetSpec {
            id: first.dataset_id.clone(),
            hash: "hash".to_string(),
            size: 1,
            origin: "fixture".to_string(),
            class_label: first.dataset_class.clone(),
            read_layout: first.read_layout.clone(),
        }],
        vec![first.stage_id.clone()],
        vec![first.tool_id.clone()],
        vec![first.params_hash.clone()],
        bijux_benchmark::ReplicatePolicy {
            count: 1,
            warmup: 0,
            seeds: vec![1],
        },
        bijux_benchmark::DiversityRequirements {
            min_dataset_count: 1,
            min_classes: 1,
            min_read_layouts: 1,
        },
        vec![bijux_benchmark::StratificationRequirement {
            key: "dataset_class".to_string(),
            required_values: vec![first.dataset_class.clone()],
        }],
        bijux_benchmark::AnalysisRequirements {
            require_bootstrap: false,
            require_outlier_detection: false,
            min_replicates_for_bootstrap: 5,
        },
    )
}

#[test]
fn benchmark_outputs_are_deterministic() -> Result<()> {
    let observations = load_observations()?;
    let suite = suite_from_observations(&observations);

    let summary_a = summarize(&suite, &observations, &BenchRunOptions::default())?;
    let summary_b = summarize(&suite, &observations, &BenchRunOptions::default())?;
    let canon_a = to_canonical_json_bytes(&summary_a)?;
    let canon_b = to_canonical_json_bytes(&summary_b)?;
    assert_eq!(canon_a, canon_b);

    let policy = GatePolicy {
        objective: "runtime".to_string(),
        required_metrics: vec!["runtime_s".to_string()],
        thresholds: BTreeMap::new(),
        allowed_regressions: BTreeMap::new(),
        must_not_regress: Vec::new(),
        semantics_overrides: BTreeMap::new(),
        stage_overrides: BTreeMap::new(),
    };
    let decisions_a = gate(&policy, &summary_a);
    let decisions_b = gate(&policy, &summary_b);
    let canon_decisions_a = to_canonical_json_bytes(&decisions_a)?;
    let canon_decisions_b = to_canonical_json_bytes(&decisions_b)?;
    assert_eq!(canon_decisions_a, canon_decisions_b);
    Ok(())
}
