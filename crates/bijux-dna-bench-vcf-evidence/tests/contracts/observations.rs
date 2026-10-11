use bijux_dna_bench_vcf_evidence::{
    commands::projection::render_shell_command,
    model::{
        commands::{VcfRenderedCommandRow, VcfRenderedCommandStep},
        expectations::VcfExpectedBenchmarkResultRow,
        outputs::{VcfAdapterOutputCoverageRow, VcfAdapterOutputCoverageStatus},
        parsers::{VcfParserFixtureCoverageRow, VcfParserFixtureCoverageStatus},
        reports::VcfReportMapRow,
        scope::VcfActiveStageToolMatrixRow,
    },
    snapshot::{ActiveVcfBinding, VcfEvidenceRows, VcfStageReadinessBindingKey},
};

pub fn evidence() -> VcfEvidenceRows {
    VcfEvidenceRows {
        retained: vec![retained()],
        active: vec![active()],
        commands: vec![commands()],
        outputs: vec![outputs()],
        parsers: vec![parsers()],
        expected: vec![expected()],
        reports: vec![reports()],
    }
}

fn retained() -> VcfActiveStageToolMatrixRow {
    let stage = "vcf.call";
    let tool = "bcftools";
    let corpus = "bam_mini";
    let asset = "reference";
    VcfActiveStageToolMatrixRow {
        stage_id: stage.into(),
        tool_id: tool.into(),
        corpus_id: corpus.into(),
        asset_profile_id: asset.into(),
        adapter_id: "call.adapter".into(),
        parser_id: "call.parser".into(),
        schema_id: "call.schema".into(),
        tool_status: "active".into(),
        stage_support_status: "supported".into(),
        scope_state: "active".into(),
        scope_detail: "active".into(),
        scope_proof_path: "evidence/scope.tsv".into(),
        reason: "declared".into(),
    }
}

fn active() -> ActiveVcfBinding {
    let stage = "vcf.call";
    let tool = "bcftools";
    let corpus = "bam_mini";
    let asset = "reference";
    ActiveVcfBinding {
        key: VcfStageReadinessBindingKey {
            stage_id: stage.into(),
            tool_id: tool.into(),
            corpus_id: corpus.into(),
            asset_profile_id: asset.into(),
        },
        adapter_id: "call.adapter".into(),
        parser_id: "call.parser".into(),
        schema_id: "call.schema".into(),
    }
}

fn commands() -> VcfRenderedCommandRow {
    let stage = "vcf.call";
    let tool = "bcftools";
    let argv = vec![tool.to_owned(), "call".into(), "input with spaces.bcf".into()];
    let command = render_shell_command(&argv);
    VcfRenderedCommandRow {
        stage_id: stage.into(),
        tool_id: tool.into(),
        readiness_kind: "benchmark_ready".into(),
        benchmark_status: "benchmark_ready".into(),
        command_source: "bcftools".into(),
        command_steps: vec![VcfRenderedCommandStep {
            step_id: "call".into(),
            step_kind: "execute".into(),
            consumes_previous_stdout: false,
            argv,
            command: command.clone(),
        }],
        script_commands: vec![command],
        reason: "projection".into(),
    }
}

fn outputs() -> VcfAdapterOutputCoverageRow {
    let stage = "vcf.call";
    let tool = "bcftools";
    VcfAdapterOutputCoverageRow {
        stage_id: stage.into(),
        tool_id: tool.into(),
        raw_outputs: vec!["output.vcf".into()],
        normalized_metrics: vec!["metrics.json".into()],
        stdout: "stdout.txt".into(),
        stderr: "stderr.txt".into(),
        manifest: "manifest.json".into(),
        index_outputs: vec![],
        status: VcfAdapterOutputCoverageStatus::Complete,
        benchmark_status: "benchmark_ready".into(),
        missing_declarations: vec![],
        reason: "declared".into(),
    }
}

fn parsers() -> VcfParserFixtureCoverageRow {
    let stage = "vcf.call";
    let tool = "bcftools";
    let corpus = "bam_mini";
    let asset = "reference";
    VcfParserFixtureCoverageRow {
        stage_id: stage.into(),
        tool_id: tool.into(),
        corpus_id: corpus.into(),
        asset_profile_id: asset.into(),
        adapter_id: "call.adapter".into(),
        parser_id: "call.parser".into(),
        schema_id: "call.schema".into(),
        parser_fixture_parser_id: "call.parser".into(),
        parser_fixture_schema_id: "call.schema".into(),
        parser_fixture_root_path: "fixtures/call".into(),
        expected_normalized_path: "fixtures/call/expected.json".into(),
        raw_fixture_count: 1,
        raw_fixture_paths: vec!["fixtures/call/raw.vcf".into()],
        coverage_status: VcfParserFixtureCoverageStatus::Covered,
        reason: "fixture observed".into(),
    }
}

fn expected() -> VcfExpectedBenchmarkResultRow {
    let stage = "vcf.call";
    let tool = "bcftools";
    let corpus = "bam_mini";
    let asset = "reference";
    VcfExpectedBenchmarkResultRow {
        domain: "vcf".into(),
        stage_id: stage.into(),
        tool_id: tool.into(),
        corpus_id: corpus.into(),
        asset_profile_id: asset.into(),
        expected_outputs: vec!["output.vcf".into()],
        expected_metrics: vec!["sites".into()],
        report_section: "calling".into(),
    }
}

fn reports() -> VcfReportMapRow {
    let stage = "vcf.call";
    let tool = "bcftools";
    VcfReportMapRow {
        stage_id: stage.into(),
        tool_id: tool.into(),
        section_id: "calling".into(),
        summary_table: "calls".into(),
        metric_columns: vec!["sites".into()],
        failure_columns: vec!["error".into()],
    }
}
