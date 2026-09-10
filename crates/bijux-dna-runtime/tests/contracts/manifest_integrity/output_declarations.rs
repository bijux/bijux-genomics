use crate::support;
use anyhow::Context as _;

#[test]
fn registry_stages_declare_semantics_and_artifacts() -> anyhow::Result<()> {
    let root = support::repo_root()?;
    let registry = bijux_dna_runtime::manifests::load_manifests(
        &root.join("configs").join("tool_registry.toml"),
    )
    .context("load tool registry manifests")?;

    let mut offenders = Vec::new();
    let stable_name = regex::Regex::new(r"^[a-z0-9_]+$").context("compile stable name regex")?;
    let semver_re =
        regex::Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+$").context("compile semver regex")?;
    for (stage_id, stage) in registry.stages() {
        if stage.produced_artifacts.is_empty() {
            offenders.push(format!("stage={stage_id}: missing produced_artifacts"));
        }
        for artifact in &stage.produced_artifacts {
            if !stable_name.is_match(artifact) {
                offenders.push(format!(
                    "stage={stage_id}: produced artifact `{artifact}` must be stable snake_case"
                ));
            }
        }
        if !semver_re.is_match(&stage.stage_semver) {
            offenders
                .push(format!("stage={stage_id}: invalid stage_semver `{}`", stage.stage_semver));
        }
    }

    assert!(
        offenders.is_empty(),
        "stage semantic declaration violations:\n{}",
        offenders.join("\n")
    );
    Ok(())
}
