#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__public_api_stability_tiers_policy__runtime_facing_crates_declare_stability_tiers(
) {
    let root = repo_root();
    let raw = std::fs::read_to_string(root.join("configs/ci/public_api_tiers.toml"))
        .unwrap_or_else(|err| panic!("read public_api_tiers.toml: {err}"));
    let value: toml::Value = toml::from_str(&raw).unwrap_or_else(|err| panic!("parse public_api_tiers.toml: {err}"));
    let crates = value["runtime_facing_crates"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();

    let mut offenders = Vec::new();
    for crate_name in crates {
        let path = root.join("crates").join(&crate_name).join("docs/PUBLIC_API.md");
        let raw =
            std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        for needle in [
            "## Stability Tiers",
            "- Stable:",
            "- Experimental:",
            "- Internal:",
        ] {
            if !raw.contains(needle) {
                offenders.push(format!("{} missing `{needle}`", path.display()));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "runtime-facing PUBLIC_API docs must declare stability tiers:\n{}",
        offenders.join("\n")
    );
}
