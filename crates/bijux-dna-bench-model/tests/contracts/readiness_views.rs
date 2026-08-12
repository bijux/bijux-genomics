use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    corpus_centric_report_schema, stage_centric_report_schema, tool_centric_report_schema,
    validate_corpus_centric_report, validate_stage_centric_report, validate_tool_centric_report,
    CorpusCentricCorpusReport, CorpusCentricReport, CorpusCentricStageRow,
    ReadinessComparisonContractStatus, StageCentricReport, StageCentricStageReport,
    StageCentricToolRow, ToolCentricReport, ToolCentricStageRow, ToolCentricToolReport,
    CORPUS_CENTRIC_REPORT_SCHEMA_V1, STAGE_CENTRIC_REPORT_SCHEMA_V1, TOOL_CENTRIC_REPORT_SCHEMA_V1,
};

fn counts(key: &str, value: usize) -> BTreeMap<String, usize> {
    BTreeMap::from([(key.to_string(), value)])
}

fn stage_tool(tool_id: &str, status: &str, gap: &str) -> StageCentricToolRow {
    StageCentricToolRow {
        tool_id: tool_id.to_string(),
        benchmark_status: status.to_string(),
        readiness_gap: gap.to_string(),
        support_status: "supported".to_string(),
        adapter_status: "implemented".to_string(),
        parser_status: "implemented".to_string(),
        corpus_status: "compatible".to_string(),
        asset_status: "available".to_string(),
        reason: "governed readiness state".to_string(),
    }
}

fn stage_report() -> StageCentricReport {
    let tools = vec![
        stage_tool("beagle", "benchmark_ready", "none"),
        stage_tool("impute5", "blocked", "external_artifact"),
    ];
    StageCentricReport {
        schema_version: STAGE_CENTRIC_REPORT_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/stage.md".to_string(),
        stage_count: 1,
        multi_tool_stage_count: 1,
        blocked_stage_count: 1,
        declared_shared_metric_stage_count: 1,
        not_declared_shared_metric_stage_count: 0,
        row_count: 2,
        benchmark_ready_row_count: 1,
        blocked_row_count: 1,
        domain_counts: counts("vcf", 1),
        section_counts: counts("imputation", 1),
        readiness_gap_counts: BTreeMap::from([
            ("external_artifact".to_string(), 1),
            ("none".to_string(), 1),
        ]),
        stages: vec![StageCentricStageReport {
            domain: "vcf".to_string(),
            stage_id: "vcf.impute".to_string(),
            canonical_stage_rank: 8,
            report_section_id: "imputation".to_string(),
            report_section_title: "Imputation".to_string(),
            summary_table_id: "imputation-summary".to_string(),
            summary_table_title: "Imputation summary".to_string(),
            anchor_tool_id: "beagle".to_string(),
            anchor_support_status: "supported".to_string(),
            comparison_contract_status: ReadinessComparisonContractStatus::Declared,
            shared_metric_field_count: 1,
            shared_metric_fields: vec!["concordance".to_string()],
            tool_count: 2,
            benchmark_ready_tool_count: 1,
            blocked_tool_count: 1,
            readiness_gap_counts: BTreeMap::from([
                ("external_artifact".to_string(), 1),
                ("none".to_string(), 1),
            ]),
            blocked_tool_ids: vec!["impute5 (external_artifact)".to_string()],
            tools,
        }],
    }
}

fn tool_stage(tool_id: &str) -> ToolCentricStageRow {
    ToolCentricStageRow {
        domain: "vcf".to_string(),
        stage_id: "vcf.impute".to_string(),
        report_section_id: "imputation".to_string(),
        report_section_title: "Imputation".to_string(),
        summary_table_id: "imputation-summary".to_string(),
        summary_table_title: "Imputation summary".to_string(),
        benchmark_status: if tool_id == "beagle" {
            "benchmark_ready".to_string()
        } else {
            "blocked".to_string()
        },
        readiness_gap: if tool_id == "beagle" {
            "none".to_string()
        } else {
            "external_artifact".to_string()
        },
        support_status: "supported".to_string(),
        adapter_status: "implemented".to_string(),
        parser_status: "implemented".to_string(),
        corpus_status: "compatible".to_string(),
        asset_status: "available".to_string(),
        reason: "governed readiness state".to_string(),
    }
}

fn tool_report() -> ToolCentricReport {
    ToolCentricReport {
        schema_version: TOOL_CENTRIC_REPORT_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/tool.md".to_string(),
        tool_count: 2,
        unique_stage_count: 1,
        row_count: 2,
        benchmark_ready_row_count: 1,
        blocked_row_count: 1,
        blocked_tool_count: 1,
        domain_counts: counts("vcf", 2),
        readiness_gap_counts: BTreeMap::from([
            ("external_artifact".to_string(), 1),
            ("none".to_string(), 1),
        ]),
        tools: vec![
            ToolCentricToolReport {
                tool_id: "beagle".to_string(),
                domains: vec!["vcf".to_string()],
                stage_count: 1,
                benchmark_ready_stage_count: 1,
                blocked_stage_count: 0,
                report_section_ids: vec!["imputation".to_string()],
                readiness_gap_counts: counts("none", 1),
                blocked_stage_ids: Vec::new(),
                stages: vec![tool_stage("beagle")],
            },
            ToolCentricToolReport {
                tool_id: "impute5".to_string(),
                domains: vec!["vcf".to_string()],
                stage_count: 1,
                benchmark_ready_stage_count: 0,
                blocked_stage_count: 1,
                report_section_ids: vec!["imputation".to_string()],
                readiness_gap_counts: counts("external_artifact", 1),
                blocked_stage_ids: vec!["vcf.impute (external_artifact)".to_string()],
                stages: vec![tool_stage("impute5")],
            },
        ],
    }
}

fn corpus_report() -> CorpusCentricReport {
    CorpusCentricReport {
        schema_version: CORPUS_CENTRIC_REPORT_SCHEMA_V1.to_string(),
        output_path: "artifacts/readiness/corpus.md".to_string(),
        corpus_count: 1,
        stage_count: 1,
        tool_row_count: 2,
        benchmark_ready_tool_row_count: 1,
        blocked_tool_row_count: 1,
        blocked_corpus_count: 1,
        domain_counts: counts("vcf", 1),
        corpus_stage_counts: counts("masked-truth-vcf", 1),
        corpora: vec![CorpusCentricCorpusReport {
            corpus_family_id: "masked-truth-vcf".to_string(),
            domains: vec!["vcf".to_string()],
            fixture_ids: vec!["masked-truth-vcf-01".to_string()],
            stage_count: 1,
            tool_row_count: 2,
            benchmark_ready_tool_row_count: 1,
            blocked_stage_count: 1,
            blocked_stage_ids: vec!["vcf.impute".to_string()],
            stages: vec![CorpusCentricStageRow {
                domain: "vcf".to_string(),
                stage_id: "vcf.impute".to_string(),
                canonical_stage_rank: 8,
                fixture_ids: vec!["masked-truth-vcf-01".to_string()],
                report_section_id: "imputation".to_string(),
                report_section_title: "Imputation".to_string(),
                summary_table_id: "imputation-summary".to_string(),
                summary_table_title: "Imputation summary".to_string(),
                comparison_contract_status: ReadinessComparisonContractStatus::Declared,
                shared_metric_field_count: 1,
                shared_metric_fields: vec!["concordance".to_string()],
                tool_count: 2,
                benchmark_ready_tool_count: 1,
                blocked_tool_count: 1,
                tool_ids: vec!["beagle".to_string(), "impute5".to_string()],
                blocked_tool_ids: vec!["impute5 (external_artifact)".to_string()],
            }],
        }],
    }
}

#[test]
fn readiness_views_round_trip_and_validate() -> anyhow::Result<()> {
    let stage = stage_report();
    validate_stage_centric_report(&stage)?;
    assert_eq!(serde_json::from_slice::<StageCentricReport>(&serde_json::to_vec(&stage)?)?, stage);
    let tool = tool_report();
    validate_tool_centric_report(&tool)?;
    assert_eq!(serde_json::from_slice::<ToolCentricReport>(&serde_json::to_vec(&tool)?)?, tool);
    let corpus = corpus_report();
    validate_corpus_centric_report(&corpus)?;
    assert_eq!(
        serde_json::from_slice::<CorpusCentricReport>(&serde_json::to_vec(&corpus)?)?,
        corpus
    );
    Ok(())
}

#[test]
fn readiness_views_reject_nested_and_aggregate_drift() {
    let mut stage = stage_report();
    stage.stages[0].blocked_tool_ids.clear();
    assert!(validate_stage_centric_report(&stage).is_err());

    let mut tool = tool_report();
    tool.tools[1].readiness_gap_counts = counts("none", 1);
    assert!(validate_tool_centric_report(&tool).is_err());

    let mut tool = tool_report();
    tool.tools.swap(0, 1);
    assert!(validate_tool_centric_report(&tool).is_err());

    let mut corpus = corpus_report();
    corpus.corpora[0].fixture_ids.push("unbound-fixture".to_string());
    assert!(validate_corpus_centric_report(&corpus).is_err());

    let mut corpus = corpus_report();
    corpus.corpora[0].stages[0].tool_ids.swap(0, 1);
    assert!(validate_corpus_centric_report(&corpus).is_err());
}

#[test]
fn readiness_views_preserve_declared_metric_order() {
    let mut stage = stage_report();
    stage.stages[0].shared_metric_fields = vec!["precision".to_string(), "accuracy".to_string()];
    stage.stages[0].shared_metric_field_count = 2;
    assert!(validate_stage_centric_report(&stage).is_ok());
}

#[test]
fn readiness_view_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    for (file_name, generated) in [
        ("stage-centric-report.schema.json", serde_json::to_value(stage_centric_report_schema())?),
        ("tool-centric-report.schema.json", serde_json::to_value(tool_centric_report_schema())?),
        (
            "corpus-centric-report.schema.json",
            serde_json::to_value(corpus_centric_report_schema())?,
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
fn readiness_views_reject_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(stage_report())?;
    value["stages"][0]["scheduler_job_id"] = serde_json::json!(42);
    assert!(serde_json::from_value::<StageCentricReport>(value).is_err());
    Ok(())
}
