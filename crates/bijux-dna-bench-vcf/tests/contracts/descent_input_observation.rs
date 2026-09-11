use super::{
    admission::{run_missing_input_probe, validate_required_inputs},
    VcfDescentFamilyAdapterArtifact,
};
use anyhow::Result;
use std::fs;
#[test]
fn descent_missing_input_probe_is_sensitive_to_actual_file_presence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let inputs = vec![VcfDescentFamilyAdapterArtifact {
        artifact_id: "vcf".to_owned(),
        role: "variants".to_owned(),
        path: "input.vcf".to_owned(),
    }];
    fs::write(root.path().join("input.vcf"), b"fixture")?;
    validate_required_inputs(root.path(), "vcf.roh", &inputs)?;
    assert!(run_missing_input_probe(root.path(), "vcf.roh", &inputs)?.3);
    let missing =
        root.path().join("artifacts/benchmark-smoke/readiness/adapters/probes/vcf.roh/vcf.missing");
    fs::create_dir_all(missing.parent().ok_or_else(|| anyhow::anyhow!("probe parent"))?)?;
    fs::write(&missing, b"unexpected existing file")?;
    let observed = run_missing_input_probe(root.path(), "vcf.roh", &inputs)?;
    assert!(!observed.3);
    assert!(observed.2.contains("unexpectedly accepted"));
    fs::remove_file(&missing)?;
    assert!(run_missing_input_probe(root.path(), "vcf.roh", &inputs)?.3);
    fs::remove_file(root.path().join("input.vcf"))?;
    fs::create_dir(root.path().join("input.vcf"))?;
    assert!(validate_required_inputs(root.path(), "vcf.roh", &inputs).is_err());
    Ok(())
}
