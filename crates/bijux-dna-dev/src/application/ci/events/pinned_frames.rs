//! Minimal lifecycle derived from the pinned reporter's captured ignored-completion frames.

use anyhow::{Context, Result};
use serde_json::{json, Value};

use super::admit_shard_events;
use crate::application::ci::{CoverageAttestation, ExpectedTestInventory, ShardDiscoveryPlan};

const ORDINARY: &str = "transport::native_case_116";
const PENDING: &str = "transport::native_case_88";
const IGNORED: &str = "transport::native_case_0";

fn expected_inventory(ignored: bool) -> Result<ExpectedTestInventory> {
    ExpectedTestInventory::parse(&serde_json::to_vec(&json!({"test-count":3,"rust-suites":{
        "bijux-dna-api":{"package-name":"bijux-dna-api","binary-id":"bijux-dna-api",
            "binary-name":"bijux_dna_api","kind":"lib","status":"listed","testcases":{
                ORDINARY:{"ignored":false,"filter-match":{"status":"matches"}},
                PENDING:{"ignored":false,"filter-match":{"status":"matches"}},
                IGNORED:{"ignored":ignored,"filter-match":{"status":"matches"}}
            }}
    }}))?)
}

fn wire(name: &str) -> String {
    format!("bijux-dna-api::bijux_dna_api${name}")
}

fn records() -> Vec<Value> {
    let suite = json!({"crate":"bijux-dna-api","test_binary":"bijux_dna_api","kind":"lib"});
    // Two matching ordinary cases initialize running=2. The ignored completion and first
    // ordinary completion decrement it to zero while the other ordinary case is still live.
    vec![
        json!({"type":"suite","event":"started","test_count":3,"nextest":suite}),
        json!({"type":"test","event":"started","name":wire(ORDINARY)}),
        json!({"type":"test","event":"started","name":wire(IGNORED)}),
        json!({"type":"test","event":"ok","name":wire(IGNORED)}),
        json!({"type":"test","event":"started","name":wire(PENDING)}),
        json!({"type":"test","event":"ok","name":wire(ORDINARY)}),
        json!({"type":"suite","event":"ok","passed":2,"failed":0,"ignored":1,"measured":0,"filtered_out":0,"nextest":suite}),
        json!({"type":"suite","event":"started","test_count":3,"nextest":suite}),
        json!({"type":"test","event":"ok","name":wire(PENDING)}),
        json!({"type":"suite","event":"ok","passed":1,"failed":0,"ignored":1,"measured":0,"filtered_out":1,"nextest":suite}),
    ]
}

fn bytes(records: &[Value]) -> Vec<u8> {
    records.iter().map(Value::to_string).collect::<Vec<_>>().join("\n").into_bytes()
}

#[test]
fn pinned_ignored_completion_preserves_a_started_case_across_reporter_frames() -> Result<()> {
    let inventory = expected_inventory(true)?;
    let stream = bytes(&records());
    let cases = admit_shard_events(&inventory, &stream)?;
    assert_eq!(cases.len(), 3);
    assert!(cases.values().all(|passed| *passed));
    let plan = ShardDiscoveryPlan::from_inventory(&inventory, "hash:1/1", "0.9.100")?;
    let coverage =
        CoverageAttestation::reconcile_planned(&inventory, [("native", stream.as_slice(), &plan)]);
    assert!(coverage.complete);
    let value = serde_json::to_value(coverage)?;
    assert_eq!(value["observed_count"], 3);
    assert_eq!(value["expected_count"], 3);
    let ordinary_inventory = expected_inventory(false)?;
    let error = admit_shard_events(&ordinary_inventory, &stream)
        .err()
        .context("a suite without ignored cases must not flush with a pending case")?;
    assert!(error.to_string().contains("suite closed with missing test events"));
    Ok(())
}

#[test]
fn cross_frame_case_lifetimes_reject_missing_duplicate_and_closed_frame_events() -> Result<()> {
    let inventory = expected_inventory(true)?;
    let valid = records();
    let mut no_start = valid.clone();
    no_start.remove(4);
    let mut repeated_start = valid.clone();
    repeated_start.insert(8, valid[4].clone());
    let mut repeated_result = valid.clone();
    repeated_result.insert(9, valid[8].clone());
    let mut closed_result = valid.clone();
    closed_result.swap(7, 8);
    let mut repeated_close = valid.clone();
    repeated_close.insert(7, valid[6].clone());
    let mut premature_restart = valid.clone();
    premature_restart.insert(6, valid[7].clone());
    let mut different_count = valid.clone();
    different_count[7]["test_count"] = json!(2);
    let mut no_terminal = valid.clone();
    no_terminal.remove(8);
    no_terminal[8]["passed"] = json!(0);
    let error = admit_shard_events(&inventory, &bytes(&no_terminal))
        .err()
        .context("matching zero frame counters cannot replace the carried terminal event")?;
    assert!(error.to_string().contains("unfinished test cases across reporter frames"));
    for (name, stream) in [
        ("terminal-without-start", no_start),
        ("repeated-lifetime-start", repeated_start),
        ("duplicate-terminal", repeated_result),
        ("closed-frame-terminal", closed_result),
        ("duplicate-close", repeated_close),
        ("restart-before-close", premature_restart),
        ("changed-restart-count", different_count),
        ("missing-final-close", valid[..9].to_vec()),
        ("missing-restart-and-terminal", valid[..7].to_vec()),
    ] {
        assert!(admit_shard_events(&inventory, &bytes(&stream)).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn frame_counters_and_verdicts_must_match_only_that_frames_terminal_results() -> Result<()> {
    let inventory = expected_inventory(true)?;
    for (frame, field, value) in [
        (6, "passed", json!(1)),
        (9, "passed", json!(3)),
        (9, "failed", json!(1)),
        (9, "ignored", json!(2)),
        (9, "measured", json!(1)),
        (9, "event", json!("failed")),
    ] {
        let mut stream = records();
        stream[frame][field] = value;
        assert!(admit_shard_events(&inventory, &bytes(&stream)).is_err(), "{frame}:{field}");
    }
    let mut failed = records();
    failed[5]["event"] = json!("failed");
    failed[6]["event"] = json!("failed");
    failed[6]["passed"] = json!(1);
    failed[6]["failed"] = json!(1);
    let stream = bytes(&failed);
    let cases = admit_shard_events(&inventory, &stream)?;
    assert_eq!(cases.len(), 3);
    assert_eq!(cases.values().filter(|passed| !**passed).count(), 1);
    let plan = ShardDiscoveryPlan::from_inventory(&inventory, "hash:1/1", "0.9.100")?;
    let coverage =
        CoverageAttestation::reconcile_planned(&inventory, [("native", stream.as_slice(), &plan)]);
    assert!(!coverage.complete);
    assert!(!coverage.selected_shards_valid);
    let value = serde_json::to_value(coverage)?;
    assert_eq!(value["observed_count"], 3);
    assert_eq!(value["failed"].as_array().context("failed identities missing")?.len(), 1);
    assert_eq!(value["failed"][0]["test_name"], ORDINARY);
    Ok(())
}

#[test]
fn ignored_summary_counts_cannot_substitute_for_exact_planned_case_union() -> Result<()> {
    let inventory = expected_inventory(true)?;
    let mut omitted = records();
    omitted.remove(3);
    omitted.remove(2);
    omitted[4]["passed"] = json!(1);
    let stream = bytes(&omitted);
    let observed = admit_shard_events(&inventory, &stream)?;
    assert_eq!(observed.len(), 2);
    let plan = ShardDiscoveryPlan::from_inventory(&inventory, "hash:1/1", "0.9.100")?;
    let coverage =
        CoverageAttestation::reconcile_planned(&inventory, [("native", stream.as_slice(), &plan)]);
    assert!(!coverage.complete);
    assert!(!coverage.selected_shards_valid);
    let value = serde_json::to_value(coverage)?;
    assert_eq!(value["missing"].as_array().context("missing identities absent")?.len(), 1);
    assert_eq!(value["missing"][0]["test_name"], IGNORED);
    Ok(())
}

fn one_case_frames() -> Vec<Value> {
    let original = records();
    [ORDINARY, IGNORED, PENDING]
        .into_iter()
        .flat_map(|name| {
            let mut start = original[0].clone();
            start["test_count"] = json!(1);
            let mut close = original[9].clone();
            close["ignored"] = json!(0);
            vec![
                start,
                json!({"type":"test","event":"started","name":wire(name)}),
                json!({"type":"test","event":"ok","name":wire(name)}),
                close,
            ]
        })
        .collect()
}

#[test]
fn three_one_case_frames_preserve_lifetime_union_beyond_each_frame_declaration() -> Result<()> {
    let inventory = expected_inventory(true)?;
    let stream = bytes(&one_case_frames());
    let observed = admit_shard_events(&inventory, &stream)?;
    assert_eq!(observed.len(), 3);
    assert!(observed.values().all(|passed| *passed));
    let plan = ShardDiscoveryPlan::from_inventory(&inventory, "hash:1/1", "0.9.100")?;
    let coverage =
        CoverageAttestation::reconcile_planned(&inventory, [("native", stream.as_slice(), &plan)]);
    assert!(coverage.complete);
    let value = serde_json::to_value(coverage)?;
    assert_eq!(value["observed_count"], 3);
    assert_eq!(value["expected_count"], 3);
    Ok(())
}

#[test]
fn frame_start_and_completion_limits_do_not_reuse_the_lifetime_budget() -> Result<()> {
    let inventory = expected_inventory(true)?;
    let mut extra_start = one_case_frames();
    extra_start.insert(2, json!({"type":"test","event":"started","name":wire(PENDING)}));
    let error = admit_shard_events(&inventory, &bytes(&extra_start))
        .err()
        .context("two known cases cannot start in a frame declaring one")?;
    assert!(error.to_string().contains("frame starts exceed its declared count"));
    let template = one_case_frames();
    let mut empty_close = template[3].clone();
    empty_close["passed"] = json!(0);
    let mut over_complete = template[3].clone();
    over_complete["passed"] = json!(2);
    let carried = vec![
        template[0].clone(),
        template[1].clone(),
        empty_close.clone(),
        template[4].clone(),
        template[5].clone(),
        empty_close,
        template[8].clone(),
        template[2].clone(),
        template[6].clone(),
        over_complete,
    ];
    let error = admit_shard_events(&inventory, &bytes(&carried))
        .err()
        .context("two carried terminals cannot consume one frame slot")?;
    assert!(error.to_string().contains("frame completions exceed its declared count"));
    // A fourth lifetime result cannot be manufactured by replaying a known case in a frame
    // whose own one-case counts remain valid. The unchanged inventory is still the authority.
    let mut repeated_lifetime = one_case_frames();
    repeated_lifetime.extend(template[..4].iter().cloned());
    let error = admit_shard_events(&inventory, &bytes(&repeated_lifetime))
        .err()
        .context("a known case cannot occur twice across individually bounded frames")?;
    assert!(error.to_string().contains("test started more than once"));
    Ok(())
}
