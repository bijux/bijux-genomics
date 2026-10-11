use super::{load_historical_active_scope_snapshot, load_historical_operational_snapshot};
use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path};

#[test]
fn historical_green_snapshots_are_diagnostic_after_current_observation_failure() -> Result<()> {
    let root = tempfile::tempdir()?;
    let active = Path::new("active.json");
    let operational = Path::new("operational.json");
    fs::write(
        root.path().join(active),
        serde_json::to_vec(
            &json!({"ok":true,"output_path":"old-active.json","active_row_count":116,"failed_surface_count":0}),
        )?,
    )?;
    fs::write(
        root.path().join(operational),
        serde_json::to_vec(
            &json!({"ok":true,"benchmark_ready_row_count":116,"blocker_count":0,"blockers":[]}),
        )?,
    )?;
    let observed_active = load_historical_active_scope_snapshot(root.path(), active)?;
    let observed_operational = load_historical_operational_snapshot(root.path(), operational)?;
    assert!(!observed_active.ok, "old active-scope success cannot mask a failed current render");
    assert!(
        !observed_operational.ok,
        "old operational success cannot mask a failed current render"
    );
    assert_eq!(observed_active.active_row_count, 116);
    assert_eq!(observed_operational.benchmark_ready_row_count, 116);
    assert_eq!(observed_active.output_path, "old-active.json");
    fs::write(root.path().join(active), b"truncated")?;
    fs::write(root.path().join(operational), b"{")?;
    assert!(load_historical_active_scope_snapshot(root.path(), active).is_err());
    assert!(load_historical_operational_snapshot(root.path(), operational).is_err());
    Ok(())
}
