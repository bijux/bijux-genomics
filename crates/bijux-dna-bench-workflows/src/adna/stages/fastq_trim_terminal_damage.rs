//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages fastq trim terminal damage.
use super::super::{
    bail, first_base_histogram, last_base_histogram, mean_quality, path_relative_to_repo,
    read_fastq_records, required_output, succeeded_row, terminal_transition_asymmetry,
    terminal_transition_asymmetry_one_end, total_fastq_bases, trim_terminal_damage_record,
    write_fastq_records, AdnaMicroPipelineRow, BTreeMap, Context, DamageMode, PairedMode, Path,
    Result, TerminalDamageExecutionPolicy, TerminalDamageReportV1, Value,
    FASTQ_TRIM_TERMINAL_DAMAGE_TOOL_ID, TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_fastq_trim_terminal_damage_stage(
    repo_root: &Path,
    output_root: &Path,
    input_r1: &Path,
    input_r2: &Path,
    validate_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.trim_terminal_damage");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let input_left = read_fastq_records(input_r1)?;
    let input_right = read_fastq_records(input_r2)?;
    if input_left.len() != input_right.len() {
        bail!("aDNA trim stage requires synchronized paired FASTQ inputs");
    }

    let output_r1 = stage_root.join("trimmed_R1.fastq");
    let output_r2 = stage_root.join("trimmed_R2.fastq");
    let report_path = stage_root.join("trim_terminal_damage_report.json");
    let backend_path = stage_root.join("trim_terminal_damage.backend.json");
    let trimmed_left =
        input_left.iter().map(trim_terminal_damage_record).collect::<Result<Vec<_>>>()?;
    let trimmed_right =
        input_right.iter().map(trim_terminal_damage_record).collect::<Result<Vec<_>>>()?;
    write_fastq_records(&output_r1, &trimmed_left)?;
    write_fastq_records(&output_r2, &trimmed_right)?;

    let reads_in = u64::try_from(input_left.len() + input_right.len()).unwrap_or(u64::MAX);
    let reads_out = u64::try_from(trimmed_left.len() + trimmed_right.len()).unwrap_or(u64::MAX);
    let bases_in = total_fastq_bases(&input_left) + total_fastq_bases(&input_right);
    let bases_out = total_fastq_bases(&trimmed_left) + total_fastq_bases(&trimmed_right);
    let terminal_base_composition_pre_r1 = Some(first_base_histogram(&input_left));
    let terminal_base_composition_post_r1 = Some(first_base_histogram(&trimmed_left));
    let terminal_base_composition_pre_r2 = Some(last_base_histogram(&input_right));
    let terminal_base_composition_post_r2 = Some(last_base_histogram(&trimmed_right));
    let report = TerminalDamageReportV1 {
        schema_version: TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION.to_string(),
        stage: stage::FASTQ_TRIM_TERMINAL_DAMAGE.to_string(),
        stage_id: stage::FASTQ_TRIM_TERMINAL_DAMAGE.to_string(),
        tool_id: FASTQ_TRIM_TERMINAL_DAMAGE_TOOL_ID.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 1,
        damage_mode: DamageMode::Ancient,
        execution_policy: TerminalDamageExecutionPolicy::ExplicitTerminalTrim,
        trim_5p_bases: 1,
        trim_3p_bases: 1,
        requested_trim_5p_bases: Some(1),
        requested_trim_3p_bases: Some(1),
        udg_classification: "non_udg".to_string(),
        input_r1: path_relative_to_repo(repo_root, input_r1),
        input_r2: Some(path_relative_to_repo(repo_root, input_r2)),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: Some(path_relative_to_repo(repo_root, &output_r2)),
        reads_in: Some(reads_in),
        reads_out: Some(reads_out),
        bases_in: Some(bases_in),
        bases_out: Some(bases_out),
        mean_q_before: Some(mean_quality(&input_left, &input_right)),
        mean_q_after: Some(mean_quality(&trimmed_left, &trimmed_right)),
        ct_ga_asymmetry_pre: Some(terminal_transition_asymmetry(&input_left, &input_right)),
        ct_ga_asymmetry_post: Some(terminal_transition_asymmetry(&trimmed_left, &trimmed_right)),
        ct_ga_asymmetry_pre_r1: Some(terminal_transition_asymmetry_one_end(&input_left, true)),
        ct_ga_asymmetry_post_r1: Some(terminal_transition_asymmetry_one_end(&trimmed_left, true)),
        ct_ga_asymmetry_pre_r2: Some(terminal_transition_asymmetry_one_end(&input_right, false)),
        ct_ga_asymmetry_post_r2: Some(terminal_transition_asymmetry_one_end(&trimmed_right, false)),
        terminal_base_composition_pre_r1,
        terminal_base_composition_post_r1,
        terminal_base_composition_pre_r2,
        terminal_base_composition_post_r2,
        raw_backend_report: Some(path_relative_to_repo(repo_root, &backend_path)),
        raw_backend_report_format: Some("bijux_terminal_damage_trace".to_string()),
        runtime_s: None,
        memory_mb: None,
        used_fallback: false,
        backend_metrics: Some(serde_json::json!({
            "reads_profiled_r1": input_left.len(),
            "reads_profiled_r2": input_right.len(),
            "bases_removed": bases_in.saturating_sub(bases_out),
        })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    bijux_dna_infra::atomic_write_json(
        &backend_path,
        &serde_json::json!({
            "schema_version": "bijux.fastq.trim_terminal_damage.backend_metrics.v1",
            "trim_5p_bases": 1,
            "trim_3p_bases": 1,
            "bases_removed": bases_in.saturating_sub(bases_out),
        }),
    )?;

    publish_row(repo_root, validate_row, &report_path, &backend_path, &report)
}

fn publish_row(
    repo_root: &Path,
    validate_row: &AdnaMicroPipelineRow,
    report_path: &Path,
    backend_path: &Path,
    report: &TerminalDamageReportV1,
) -> Result<AdnaMicroPipelineRow> {
    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::FASTQ_TRIM_TERMINAL_DAMAGE,
            domain: "fastq",
            tool_id: FASTQ_TRIM_TERMINAL_DAMAGE_TOOL_ID,
            execution_mode: "governed_runtime",
        },
        path_relative_to_repo(repo_root, report_path),
        TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION,
        BTreeMap::from([
            (
                "validated_reads_r1_path".to_string(),
                required_output(validate_row, "validated_reads_r1_path")?,
            ),
            (
                "validated_reads_r2_path".to_string(),
                required_output(validate_row, "validated_reads_r2_path")?,
            ),
        ]),
        BTreeMap::from([
            ("terminal_damage_trimmed_reads_r1_path".to_string(), report.output_r1.clone()),
            (
                "terminal_damage_trimmed_reads_r2_path".to_string(),
                report.output_r2.clone().ok_or_else(|| {
                    anyhow::anyhow!("paired trim report is missing its mate output")
                })?,
            ),
            (
                "terminal_damage_trim_report_json".to_string(),
                path_relative_to_repo(repo_root, report_path),
            ),
            (
                "trim_terminal_damage_backend_json".to_string(),
                path_relative_to_repo(repo_root, backend_path),
            ),
        ]),
        BTreeMap::from([
            ("reads_in".to_string(), Value::from(report.reads_in)),
            ("reads_out".to_string(), Value::from(report.reads_out)),
            (
                "bases_removed".to_string(),
                report
                    .backend_metrics
                    .as_ref()
                    .and_then(|metrics| metrics.get("bases_removed"))
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("trim report is missing bases removed"))?,
            ),
        ]),
        "trimmed one base from each read end while preserving residual terminal damage signal"
            .to_string(),
    ))
}
