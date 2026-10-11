use super::load_vcf_registry_snapshot;
use anyhow::Result;
use std::fs;
#[test]
fn shared_registry_rejects_conflicting_optional_smoke_metadata() -> Result<()> {
    let root = tempfile::tempdir()?;
    let directory = root.path().join("configs/ci/registry");
    fs::create_dir_all(&directory)?;
    let base = r#"[[tools]]
id = "bcftools"
expected_bin = "bcftools"
stage_ids = ["vcf.call"]
status = "production"
"#;
    for field in ["dockerfile", "apptainer_def", "smoke_minimal_cmd", "smoke_minimal_rationale"] {
        fs::write(
            directory.join("tool_registry_vcf.toml"),
            format!("{base}{field} = \"first\"\n"),
        )?;
        fs::write(
            directory.join("tool_registry_vcf_downstream.toml"),
            format!("{base}{field} = \"different\"\n"),
        )?;
        assert!(load_vcf_registry_snapshot(root.path()).is_err());
        fs::write(
            directory.join("tool_registry_vcf_downstream.toml"),
            format!("{base}{field} = \" first \"\n"),
        )?;
        let snapshot = load_vcf_registry_snapshot(root.path())?;
        assert_eq!(snapshot.smoke_records().len(), 1);
    }
    Ok(())
}
