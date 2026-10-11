use bijux_dna_bench::{build_vcf_stage_catalog_rows, build_vcf_stage_matrix_rows};

#[test]
fn vcf_stage_catalog_rows_preserve_domain_contract_details() {
    let rows = build_vcf_stage_catalog_rows().expect("build VCF stage catalog rows");
    assert_eq!(rows.len(), 20);
    assert_eq!(rows.first().map(|row| row.stage_id.as_str()), Some("vcf.prepare_reference_panel"));
    assert_eq!(rows.last().map(|row| row.stage_id.as_str()), Some("vcf.stats"));

    let panel = rows
        .iter()
        .find(|row| row.stage_id == "vcf.prepare_reference_panel")
        .expect("reference-panel row");
    assert_eq!(panel.stage_name, "Reference Panel Preparation");
    assert_eq!(panel.input_types, ["vcf"]);
    assert_eq!(panel.output_types, ["vcf"]);
    assert!(panel.required_assets.iter().any(|asset| asset == "reference_panel_lock"));
    assert_eq!(panel.local_smoke_mode, "vcf_reference_panel");

    let call_gl = rows.iter().find(|row| row.stage_id == "vcf.call_gl").expect("call_gl row");
    assert_eq!(call_gl.input_types, ["bam"]);
    assert_eq!(call_gl.output_types, ["vcf"]);
    assert!(call_gl.required_assets.iter().any(|asset| asset == "bam_index"));

    let population = rows
        .iter()
        .find(|row| row.stage_id == "vcf.population_structure")
        .expect("population-structure row");
    assert_eq!(population.output_types, ["json"]);
    assert!(population.required_assets.iter().any(|asset| asset == "sample_metadata_manifest"));
}

#[test]
fn vcf_stage_matrix_rows_preserve_binding_contract_details() {
    let rows = build_vcf_stage_matrix_rows().expect("build VCF stage matrix rows");
    assert_eq!(rows.len(), 20);

    let panel = rows
        .iter()
        .find(|row| row.stage_id == "vcf.prepare_reference_panel")
        .expect("reference-panel row");
    assert_eq!(panel.tool_id, "bcftools");
    assert_eq!(panel.corpus_id, "vcf_production_regression");
    assert_eq!(panel.adapter_id, "vcf.adapter.reference_panel");
    assert_eq!(panel.expected_outputs, ["prepared_panel", "chunks_json"]);

    let phasing = rows.iter().find(|row| row.stage_id == "vcf.phasing").expect("phasing row");
    assert_eq!(phasing.tool_id, "shapeit5");
    assert_eq!(phasing.asset_profile_id, "vcf_cohort_with_panel");
    assert_eq!(phasing.adapter_id, "vcf.adapter.panel_workflow");
    assert_eq!(phasing.expected_outputs, ["phased_vcf"]);

    let metrics = rows
        .iter()
        .find(|row| row.stage_id == "vcf.imputation_metrics")
        .expect("imputation-metrics row");
    assert_eq!(metrics.tool_id, "beagle");
    assert_eq!(metrics.parser_id, "vcf.parser.report_json");
    assert_eq!(metrics.expected_outputs, ["imputation_metrics_json"]);
}
