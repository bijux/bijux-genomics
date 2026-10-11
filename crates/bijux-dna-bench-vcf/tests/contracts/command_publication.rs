use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::commands::{
        collect_vcf_rendered_command_rows_with_observations, render_vcf_commands_with_observations,
        DEFAULT_VCF_RENDERED_COMMANDS_PATH,
    },
};
use std::path::PathBuf;
fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

fn collect_rows(
) -> anyhow::Result<Vec<bijux_dna_bench_vcf_evidence::model::commands::VcfRenderedCommandRow>> {
    let root = repo_root();
    let inputs = crate::support::inputs(&root);
    let retained = retained(&inputs);
    let observations = VcfAdapterObservations::new(&inputs, &root);
    collect_vcf_rendered_command_rows_with_observations(&observations, &retained)
}
fn render_commands(
    root: &std::path::Path,
) -> anyhow::Result<bijux_dna_bench_vcf_evidence::model::command_reports::VcfRenderedCommandsReport>
{
    let inputs = crate::support::inputs(root);
    let retained = retained(&inputs);
    let observations = VcfAdapterObservations::new(&inputs, root);
    render_vcf_commands_with_observations(
        root,
        std::path::Path::new(DEFAULT_VCF_RENDERED_COMMANDS_PATH),
        &observations,
        &retained,
    )
}
fn retained(
    inputs: &bijux_dna_bench_vcf::inputs::VcfAdapterInputs,
) -> Vec<bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow> {
    let schemas=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors().expect("schemas");
    inputs
        .bindings()
        .iter()
        .map(|row| {
            let schema = schemas
                .iter()
                .find(|schema| schema.stage_id == row.stage_id)
                .expect("stage schema");
            bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                corpus_id: row.corpus_id.clone(),
                asset_profile_id: row.asset_profile_id.clone(),
                adapter_id: row.adapter_id.clone(),
                parser_id: row.parser_id.clone(),
                schema_id: schema.schema_id.clone(),
                tool_status: "supported".into(),
                stage_support_status: "supported".into(),
                scope_state: "active".into(),
                scope_detail: "active".into(),
                scope_proof_path: "scope.tsv".into(),
                reason: "governed binding".into(),
            }
        })
        .collect()
}
#[test]
fn vcf_rendered_command_rows_follow_canonical_benchmark_ready_slice() {
    let rows = collect_rows().expect("collect VCF rendered command rows");

    let benchmark_ready_row_count =
        bijux_dna_bench_vcf::evidence::serving::collect_vcf_tool_serving_map_rows(
            &crate::support::inputs(&repo_root()),
        )
        .expect("collect VCF tool-serving rows")
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .count();
    assert_eq!(rows.len(), benchmark_ready_row_count);
    assert!(rows.iter().all(|row| row.benchmark_status == "benchmark_ready"));
    assert!(rows.iter().all(|row| !row.script_commands.is_empty()));
    assert!(rows.iter().any(|row| row.stage_id == "vcf.call" && row.tool_id == "bcftools"));
    assert!(rows.iter().any(|row| row.stage_id == "vcf.stats" && row.tool_id == "bcftools"));
    assert!(rows.iter().any(|row| row.stage_id == "vcf.impute" && row.tool_id == "beagle"));
    assert!(rows.iter().any(|row| row.stage_id == "vcf.qc" && row.tool_id == "plink2"));
    assert!(rows.iter().any(|row| row.stage_id == "vcf.phasing" && row.tool_id == "shapeit5"));
    assert!(rows.iter().any(|row| row.stage_id == "vcf.phasing" && row.tool_id == "shapeit"));
}
#[test]
fn vcf_rendered_commands_report_tracks_canonical_rows() {
    let root = repo_root();
    let report = render_commands(&root).expect("render VCF commands");

    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_rendered_commands.v1");
    assert_eq!(
        report.output_path,
        "artifacts/benchmark-smoke/readiness/vcf/vcf-rendered-commands.sh"
    );
    assert_eq!(
        report.argv_output_path,
        "artifacts/benchmark-smoke/readiness/vcf/vcf-rendered-commands.argv.jsonl"
    );
    let benchmark_ready_row_count =
        bijux_dna_bench_vcf::evidence::serving::collect_vcf_tool_serving_map_rows(
            &crate::support::inputs(&repo_root()),
        )
        .expect("collect VCF tool-serving rows")
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .count();
    assert_eq!(report.row_count, benchmark_ready_row_count);
    assert!(report
        .rows
        .iter()
        .any(|row| { row.stage_id == "vcf.prepare_reference_panel" && row.tool_id == "bcftools" }));
    assert!(report
        .rows
        .iter()
        .any(|row| { row.stage_id == "vcf.impute" && row.tool_id == "beagle" }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.admixture" && row.tool_id == "plink2" && row.command_steps.len() == 1
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.population_structure"
            && row.tool_id == "plink2"
            && !row.command_steps.is_empty()
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.pca" && row.tool_id == "plink2" && row.command_steps.len() == 1
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.pca" && row.tool_id == "eigensoft" && row.command_steps.len() == 4
    }));
    assert!(report.rows.iter().any(|row| row.stage_id == "vcf.qc" && row.tool_id == "plink"));
    assert!(report.rows.iter().any(|row| row.stage_id == "vcf.qc" && row.tool_id == "plink2"));
}
#[test]
fn malformed_retained_authority_refuses_before_acquisition() -> anyhow::Result<()> {
    let actual = repo_root();
    let inputs = crate::support::inputs(&actual);
    for field in ["corpus", "asset", "adapter", "parser", "schema", "duplicate", "state", "tool"] {
        let mut retained = retained(&inputs);
        match field {
            "corpus" => retained[0].corpus_id = "foreign".into(),
            "asset" => retained[0].asset_profile_id = "foreign".into(),
            "adapter" => retained[0].adapter_id = "foreign".into(),
            "parser" => retained[0].parser_id = "foreign".into(),
            "schema" => retained[0].schema_id = "foreign".into(),
            "duplicate" => retained.push(retained[0].clone()),
            "state" => retained[0].scope_state = "unknown".into(),
            _ => retained[0].tool_id = "unregistered".into(),
        }
        let root = tempfile::tempdir()?;
        let observations = VcfAdapterObservations::new(&inputs, root.path());
        let output = root.path().join("commands.sh");
        std::fs::write(&output, b"previous commands")?;
        assert!(
            render_vcf_commands_with_observations(root.path(), &output, &observations, &retained)
                .is_err(),
            "{field}"
        );
        assert_eq!(std::fs::read(&output)?, b"previous commands");
        assert_eq!(std::fs::read_dir(root.path())?.count(), 1);
    }
    Ok(())
}
#[test]
fn conflicting_request_roots_refuse_before_acquisition() -> anyhow::Result<()> {
    let inputs = crate::support::inputs(&repo_root());
    let retained = retained(&inputs);
    let root = tempfile::tempdir()?;
    let other = tempfile::tempdir()?;
    let observations = VcfAdapterObservations::new(&inputs, root.path());
    assert!(render_vcf_commands_with_observations(
        other.path(),
        std::path::Path::new("commands.sh"),
        &observations,
        &retained
    )
    .is_err());
    assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    assert_eq!(std::fs::read_dir(other.path())?.count(), 0);
    Ok(())
}
#[test]
fn published_commands_preserve_bash_syntax_and_structured_argv() -> anyhow::Result<()> {
    let root = repo_root();
    let report = render_commands(&root)?;
    let script = root.join(&report.output_path);
    assert!(std::process::Command::new("bash").arg("-n").arg(&script).status()?.success());
    let argv = std::fs::read_to_string(root.join(&report.argv_output_path))?;
    let expected =
        bijux_dna_bench_vcf_evidence::commands::projection::render_vcf_command_argv_jsonl(
            &report.rows,
        )?;
    assert_eq!(argv, expected);
    assert_eq!(argv.lines().count(), report.rows.len());
    let script = std::fs::read_to_string(script)?;
    for row in &report.rows {
        for command in &row.script_commands {
            assert!(script.contains(command));
        }
    }
    Ok(())
}
