#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn read_yaml(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()));
    serde_yaml::from_str(&raw).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

fn yaml_sequence<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_sequence)
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("missing sequence `{key}`"))
}

fn yaml_string<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string `{key}`"))
}

#[test]
fn policy__contracts__backlog_metadata_policy__backlog_dir_is_declared_from_ci_index() {
    let root = workspace_root();
    let index = std::fs::read_to_string(root.join("configs/ci/index.md")).expect("read ci index");
    bijux_dna_policies::policy_assert!(
        index.contains("artifacts/planning/"),
        "configs/ci/index.md must document artifacts/planning/"
    );
}

#[test]
fn policy__contracts__backlog_metadata_policy__issue_cards_cover_remaining_level1_goals() {
    let root = workspace_root();
    let cards = read_yaml(&root.join("artifacts/planning/cards.yaml"));
    let issue_labels = read_yaml(&root.join("artifacts/planning/issue_labels.yaml"));
    let scoreboard = read_yaml(&root.join("artifacts/planning/scoreboard.yaml"));

    let known_labels = issue_labels
        .get("labels")
        .and_then(Value::as_mapping)
        .expect("labels mapping")
        .keys()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();

    let scoreboard_ids = yaml_sequence(&scoreboard, "goals")
        .iter()
        .map(|goal| yaml_string(goal, "id").to_string())
        .collect::<BTreeSet<_>>();

    let mut seen_ids = BTreeSet::new();
    let mut offenders = Vec::new();
    for card in yaml_sequence(&cards, "cards") {
        let id = yaml_string(card, "id");
        if !seen_ids.insert(id.to_string()) {
            offenders.push(format!("duplicate card id `{id}`"));
        }
        if !scoreboard_ids.contains(id) {
            offenders.push(format!("card `{id}` missing from scoreboard"));
        }
        for key in ["target_paths", "acceptance", "close_with"] {
            if card.get(key).and_then(Value::as_sequence).is_none() {
                offenders.push(format!("card `{id}` missing sequence `{key}`"));
            }
        }
        let Some(labels) = card.get("labels").and_then(Value::as_sequence) else {
            offenders.push(format!("card `{id}` missing labels"));
            continue;
        };
        for label in labels.iter().filter_map(Value::as_str) {
            if !known_labels.contains(label) {
                offenders.push(format!("card `{id}` references unknown label `{label}`"));
            }
        }
    }

    let expected = (91..=100).map(|n| format!("G{n:03}")).collect::<BTreeSet<_>>();
    if seen_ids != expected {
        offenders.push(format!(
            "cards.yaml must cover G091-G100 exactly (observed {:?})",
            seen_ids
        ));
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "backlog card policy violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__backlog_metadata_policy__scoreboard_tracks_all_level1_goals() {
    let root = workspace_root();
    let scoreboard = read_yaml(&root.join("artifacts/planning/scoreboard.yaml"));
    let goals = yaml_sequence(&scoreboard, "goals");

    let allowed_flags = [
        "not_started",
        "in_progress",
        "blocked",
        "implemented",
        "tested",
        "documented",
        "released",
        "deferred",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    let mut seen_ids = BTreeSet::new();
    let mut offenders = Vec::new();
    for goal in goals {
        let id = yaml_string(goal, "id");
        if !seen_ids.insert(id.to_string()) {
            offenders.push(format!("duplicate scoreboard id `{id}`"));
        }
        let Some(flags) = goal.get("status_flags").and_then(Value::as_sequence) else {
            offenders.push(format!("scoreboard goal `{id}` missing status_flags"));
            continue;
        };
        if flags.is_empty() {
            offenders.push(format!("scoreboard goal `{id}` has empty status_flags"));
        }
        for flag in flags.iter().filter_map(Value::as_str) {
            if !allowed_flags.contains(flag) {
                offenders.push(format!("scoreboard goal `{id}` uses unknown status flag `{flag}`"));
            }
        }
    }

    let expected = (1..=100).map(|n| format!("G{n:03}")).collect::<BTreeSet<_>>();
    if seen_ids != expected {
        offenders.push(format!(
            "scoreboard must cover G001-G100 exactly (observed {:?})",
            seen_ids
        ));
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "backlog scoreboard policy violations:\n{}",
        offenders.join("\n")
    );
}
