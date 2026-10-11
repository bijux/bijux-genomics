use anyhow::Result;
use bijux_dna_bench_vcf::evidence::registry::{
    matrix::render_vcf_matrix_registry_consistency, orphans::render_vcf_orphan_tools,
};
use std::path::Path;
const PRIMARY: &str = "configs/ci/registry/tool_registry_vcf.toml";
const SECONDARY: &str = "configs/ci/registry/tool_registry_vcf_downstream.toml";
fn registry(stage: &str) -> String {
    format!("[[tools]]\nid = \"bcftools\"\nexpected_bin = \"bcftools\"\nstatus = \"production\"\nstage_ids = [\"{stage}\"]\n")
}
fn write(root: &Path, relative: &str, body: &str) -> Result<()> {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent"))?;
    std::fs::write(path, body)?;
    Ok(())
}
#[test]
fn malformed_registry_declarations_refuse_before_report_replacement() -> Result<()> {
    let actual = crate::support::repo_root();
    let inputs = crate::support::inputs(&actual);
    let valid = registry("vcf.qc");
    let invalid = [
        valid.replace("[\"vcf.qc\"]", "[1]"),
        valid.replace("id = \"bcftools\"", "id = \" padded \""),
        valid.replace("[\"vcf.qc\"]", "[\"vcf.qc\",\"vcf.qc\"]"),
        format!("{valid}{valid}"),
        valid.replace("status = \"production\"\n", ""),
    ];
    for document in invalid {
        let root = tempfile::tempdir()?;
        write(root.path(), PRIMARY, &document)?;
        write(root.path(), SECONDARY, "tools = []\n")?;
        let output = root.path().join("readiness.json");
        std::fs::write(&output, b"previous readiness")?;
        assert!(render_vcf_matrix_registry_consistency(&inputs, root.path(), &output).is_err());
        assert_eq!(std::fs::read(&output)?, b"previous readiness");
        assert!(!root.path().join("artifacts").exists());
    }
    Ok(())
}
#[test]
fn registry_union_retains_distinct_stages_and_refuses_conflicting_binaries() -> Result<()> {
    let actual = crate::support::repo_root();
    let inputs = crate::support::inputs(&actual);
    let root = tempfile::tempdir()?;
    write(root.path(), PRIMARY, &registry("vcf.qc"))?;
    write(root.path(), SECONDARY, &registry("vcf.stats"))?;
    let output = root.path().join("readiness.json");
    let report = render_vcf_matrix_registry_consistency(&inputs, root.path(), &output)?;
    assert_eq!(report.registry_pair_count, 2);
    assert_eq!(report.benchmark_ready_registry_pair_count, 2);
    assert!(!report.passes_gate);
    let previous = std::fs::read(&output)?;
    write(
        root.path(),
        SECONDARY,
        &registry("vcf.stats").replace("expected_bin = \"bcftools\"", "expected_bin = \"foreign\""),
    )?;
    assert!(render_vcf_matrix_registry_consistency(&inputs, root.path(), &output).is_err());
    assert_eq!(std::fs::read(&output)?, previous);
    Ok(())
}
#[test]
fn malformed_required_tools_refuse_before_serving_publication() -> Result<()> {
    let actual = crate::support::repo_root();
    let inputs = crate::support::inputs(&actual);
    for required in ["[1]", r#"[" padded "]"#, r#"["bcftools","bcftools"]"#] {
        let root = tempfile::tempdir()?;
        write(root.path(), PRIMARY, &registry("vcf.qc"))?;
        write(root.path(), SECONDARY, "tools = []\n")?;
        write(
            root.path(),
            "configs/ci/tools/required_tools_vcf.toml",
            &format!("required_tools = {required}\n"),
        )?;
        write(
            root.path(),
            "configs/ci/tools/required_tools_vcf_downstream.toml",
            "required_tools = []\n",
        )?;
        let output = root.path().join("orphans.tsv");
        std::fs::write(&output, b"previous orphans")?;
        assert!(render_vcf_orphan_tools(&inputs, root.path(), &output).is_err());
        assert_eq!(std::fs::read(&output)?, b"previous orphans");
        assert!(!root.path().join("artifacts").exists());
    }
    Ok(())
}
