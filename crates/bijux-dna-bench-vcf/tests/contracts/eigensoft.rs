use std::path::PathBuf;

use bijux_dna_bench_vcf::adapters::eigensoft::{
    render_vcf_eigensoft_adapter, DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_eigensoft_adapter_tracks_governed_rows() {
    let repo_root = repo_root();
    let report = render_vcf_eigensoft_adapter(
        &crate::support::inputs(&repo_root),
        &repo_root,
        &PathBuf::from(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH),
    )
    .expect("render VCF eigensoft adapter");

    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_eigensoft_adapter.v1");
    assert_eq!(report.tool_id, "eigensoft");
    assert_eq!(report.tool_status, "production");
    assert_eq!(report.row_count, 2);
    assert_eq!(report.benchmark_ready_row_count, 2);
    assert_eq!(report.parser_output_row_count, 2);
    assert_eq!(report.normalized_metrics_row_count, 2);
    assert_eq!(report.conversion_output_row_count, 2);
    assert_eq!(report.pca_output_row_count, 2);

    let pca = report.rows.iter().find(|row| row.stage_id == "vcf.pca").expect("eigensoft pca row");
    assert_eq!(pca.benchmark_status, "benchmark_ready");
    assert_eq!(pca.normalized_metrics_artifact_id, "pca_report");
    assert!(
        pca.command_steps
            .iter()
            .flat_map(|step| step.argv.iter())
            .any(|part| part.contains("convertf")),
        "eigensoft pca row must keep convertf conversion"
    );

    let population_structure = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.population_structure")
        .expect("eigensoft population structure row");
    assert_eq!(population_structure.normalized_metrics_artifact_id, "population_structure_report");
    assert!(
        population_structure
            .command_steps
            .iter()
            .flat_map(|step| step.argv.iter())
            .any(|part| part.contains("smartpca")),
        "eigensoft population structure row must keep smartpca execution"
    );
}

#[test]
fn malformed_family_stage_members_refuse_before_report_replacement() -> anyhow::Result<()> {
    let actual = repo_root();
    let inputs = crate::support::inputs(&actual);
    let mut outcomes = Vec::new();
    for stages in ["[7]", "[\"vcf.pca\",7]"] {
        let root = tempfile::tempdir()?;
        let registry = root.path().join("configs/ci/registry/tool_registry_vcf_downstream.toml");
        std::fs::create_dir_all(
            registry.parent().ok_or_else(|| anyhow::anyhow!("registry parent"))?,
        )?;
        std::fs::write(
            &registry,
            format!("[[tools]]\ntool_id = \"eigensoft\"\nstatus = \"production\"\nstage_ids = {stages}\n"),
        )?;
        let output = root.path().join("eigensoft.json");
        std::fs::write(&output, b"previous eigensoft evidence")?;
        let result = render_vcf_eigensoft_adapter(&inputs, root.path(), &output);
        outcomes.push((
            stages,
            result.err().map(|error| format!("{error:#}")),
            std::fs::read(&output)?,
            root.path().join("artifacts").exists(),
        ));
    }
    for (stages, error, previous, artifacts_created) in outcomes {
        assert!(
            error.is_some_and(|error| error.contains("stage_ids")),
            "malformed member must be admitted before fixture acquisition: {stages}"
        );
        assert_eq!(previous, b"previous eigensoft evidence");
        assert!(!artifacts_created, "malformed registry must not materialize fixtures");
    }
    Ok(())
}
