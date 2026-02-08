#![allow(non_snake_case)]
#![allow(non_snake_case)]
use std::path::PathBuf;

use bijux_policies::GuardrailConfig;

#[test]
fn policy__contracts__policy_snapshot__guardrail_default_policy_snapshot() {
    let config = GuardrailConfig::default();
    let serialized = serde_json::to_string_pretty(&config).expect("serialize config");
    let snapshot = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join("bijux-policies__contracts__guardrail_default.json");
    let expected = std::fs::read_to_string(&snapshot).expect("read snapshot");
    bijux_policies::policy_assert_eq!(serialized.trim_end(), expected.trim_end());
}
