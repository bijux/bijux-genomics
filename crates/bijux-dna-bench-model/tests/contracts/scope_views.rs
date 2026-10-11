use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    all_domain_active_stage_tool_matrix_schema, all_domain_retained_tools_schema,
    stage_registry_extra_pairs_schema, validate_all_domain_active_stage_tool_matrix,
    validate_all_domain_retained_tools, validate_stage_registry_extra_pairs,
    AllDomainActiveStageToolMatrixReport, AllDomainActiveStageToolMatrixRow,
    AllDomainRetainedToolRow, AllDomainRetainedToolsReport, StageRegistryExtraPairRow,
    StageRegistryExtraPairsReport, ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1,
    ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1, STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1,
};

fn active_row(domain: &str, stage_id: &str, tool_id: &str) -> AllDomainActiveStageToolMatrixRow {
    AllDomainActiveStageToolMatrixRow {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        corpus_id: format!("{domain}-truth"),
        asset_profile_id: format!("{domain}-assets"),
        adapter_id: format!("{tool_id}-adapter"),
        parser_id: format!("{stage_id}-parser"),
        schema_id: format!("{stage_id}-metrics"),
        status: "benchmark_ready".to_string(),
    }
}

fn active_report() -> AllDomainActiveStageToolMatrixReport {
    let rows = vec![
        active_row("bam", "bam.damage", "ngsbriggs"),
        active_row("bam", "bam.kinship", "angsd"),
        active_row("bam", "bam.sex", "angsd"),
        active_row("fastq", "fastq.remove_duplicates", "fastuniq"),
        active_row("vcf", "vcf.call_gl", "angsd"),
        active_row("vcf", "vcf.call_pseudohaploid", "angsd"),
        active_row("vcf", "vcf.imputation_metrics", "impute5"),
        active_row("vcf", "vcf.impute", "impute5"),
    ];
    AllDomainActiveStageToolMatrixReport {
        schema_version: ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/active.tsv".to_string(),
        row_count: rows.len(),
        stage_count: 8,
        tool_count: 4,
        domain_counts: BTreeMap::from([
            ("bam".to_string(), 3),
            ("fastq".to_string(), 1),
            ("vcf".to_string(), 4),
        ]),
        status_counts: BTreeMap::from([("benchmark_ready".to_string(), 8)]),
        rows,
    }
}

fn retained_row(tool_id: &str, domains: &[&str], stage_ids: &[&str]) -> AllDomainRetainedToolRow {
    AllDomainRetainedToolRow {
        tool_id: tool_id.to_string(),
        domains: domains.iter().map(ToString::to_string).collect(),
        active_stage_count: stage_ids.len(),
        benchmark_ready_stage_count: stage_ids.len(),
        active_binding_count: stage_ids.len(),
        benchmark_ready_binding_count: stage_ids.len(),
        benchmark_statuses: vec!["benchmark_ready".to_string(), "not_benchmark_ready".to_string()],
        active_stage_ids: stage_ids.iter().map(ToString::to_string).collect(),
        benchmark_ready_stage_ids: stage_ids.iter().map(ToString::to_string).collect(),
    }
}

fn retained_report() -> AllDomainRetainedToolsReport {
    let rows = vec![
        retained_row(
            "angsd",
            &["bam", "vcf"],
            &["bam.kinship", "bam.sex", "vcf.call_gl", "vcf.call_pseudohaploid"],
        ),
        retained_row("fastuniq", &["fastq"], &["fastq.remove_duplicates"]),
        retained_row("impute5", &["vcf"], &["vcf.imputation_metrics", "vcf.impute"]),
        retained_row("ngsbriggs", &["bam"], &["bam.damage"]),
    ];
    AllDomainRetainedToolsReport {
        schema_version: ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/retained.tsv".to_string(),
        row_count: rows.len(),
        active_matrix_tool_count: rows.len(),
        benchmark_ready_tool_count: 4,
        mixed_status_tool_count: 4,
        not_benchmark_ready_only_tool_count: 0,
        domain_tool_counts: BTreeMap::from([
            ("bam".to_string(), 2),
            ("fastq".to_string(), 1),
            ("vcf".to_string(), 2),
        ]),
        rows,
    }
}

fn extra_pairs_report() -> StageRegistryExtraPairsReport {
    StageRegistryExtraPairsReport {
        schema_version: STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/extra.tsv".to_string(),
        extra_pair_count: 0,
        domain_counts: BTreeMap::new(),
        ok: true,
        rows: Vec::new(),
    }
}

fn documented_extra_pairs_report() -> StageRegistryExtraPairsReport {
    StageRegistryExtraPairsReport {
        schema_version: STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/extra.tsv".to_string(),
        extra_pair_count: 1,
        domain_counts: BTreeMap::from([("bam".to_string(), 1)]),
        ok: true,
        rows: vec![StageRegistryExtraPairRow {
            domain: "bam".to_string(),
            stage_id: "bam.coverage".to_string(),
            tool_id: "coverage-tool".to_string(),
            contract_status: "pair_missing_from_contract".to_string(),
            registry_sources: vec!["catalog".to_string()],
            registered_stage_ids: vec!["bam.depth".to_string()],
            documented_exception_status: "explicit_intent".to_string(),
            documented_exception_reason: "non-benchmark registry binding".to_string(),
            reason: "registry pair is explicitly outside this benchmark contract".to_string(),
        }],
    }
}

#[test]
fn scope_views_round_trip_and_validate() -> anyhow::Result<()> {
    let active = active_report();
    validate_all_domain_active_stage_tool_matrix(&active)?;
    assert_eq!(
        serde_json::from_slice::<AllDomainActiveStageToolMatrixReport>(&serde_json::to_vec(
            &active
        )?)?,
        active
    );
    let retained = retained_report();
    validate_all_domain_retained_tools(&retained)?;
    assert_eq!(
        serde_json::from_slice::<AllDomainRetainedToolsReport>(&serde_json::to_vec(&retained)?)?,
        retained
    );
    let extra = extra_pairs_report();
    validate_stage_registry_extra_pairs(&extra)?;
    validate_stage_registry_extra_pairs(&documented_extra_pairs_report())?;
    Ok(())
}

#[test]
fn scope_views_reject_licensed_pair_and_aggregate_drift() {
    let mut active = active_report();
    active.rows.remove(0);
    active.row_count -= 1;
    active.stage_count -= 1;
    active.domain_counts.insert("bam".to_string(), 2);
    active.status_counts.insert("benchmark_ready".to_string(), 7);
    assert!(validate_all_domain_active_stage_tool_matrix(&active).is_err());

    let mut retained = retained_report();
    retained.rows[0].active_stage_ids.pop();
    retained.rows[0].benchmark_ready_stage_ids.pop();
    retained.rows[0].active_stage_count -= 1;
    retained.rows[0].benchmark_ready_stage_count -= 1;
    retained.rows[0].active_binding_count -= 1;
    retained.rows[0].benchmark_ready_binding_count -= 1;
    assert!(validate_all_domain_retained_tools(&retained).is_err());

    let mut extra = extra_pairs_report();
    extra.ok = false;
    assert!(validate_stage_registry_extra_pairs(&extra).is_err());
}

#[test]
fn scope_view_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    for (file_name, generated) in [
        (
            "all-domain-active-stage-tool-matrix.schema.json",
            serde_json::to_value(all_domain_active_stage_tool_matrix_schema())?,
        ),
        (
            "all-domain-retained-tools.schema.json",
            serde_json::to_value(all_domain_retained_tools_schema())?,
        ),
        (
            "stage-registry-extra-pairs.schema.json",
            serde_json::to_value(stage_registry_extra_pairs_schema())?,
        ),
    ] {
        let path = repository_root.join("benchmarks/contracts/schemas").join(file_name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "{file_name} drifted from model authority");
    }
    Ok(())
}

#[test]
fn scope_views_reject_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(retained_report())?;
    value["rows"][0]["redistribution_permitted"] = serde_json::json!(false);
    assert!(serde_json::from_value::<AllDomainRetainedToolsReport>(value).is_err());
    Ok(())
}
