#![allow(clippy::format_push_string, clippy::too_many_arguments, clippy::uninlined_format_args)]

use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    remove_duplicates::{
        DedupMode, RemoveDuplicatesEffectiveParams, REMOVE_DUPLICATES_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::{
    REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION, REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION,
    STAGE_REMOVE_DUPLICATES,
};
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_REMOVE_DUPLICATES;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveDuplicatesPlanOptions {
    pub dedup_mode: DedupMode,
    pub keep_order: bool,
    pub threads_override: Option<u32>,
}

struct DeduplicatePlanPaths {
    output_r1: std::path::PathBuf,
    output_r2: Option<std::path::PathBuf>,
    report: std::path::PathBuf,
    duplicate_classes_tsv: std::path::PathBuf,
    duplicate_provenance_json: std::path::PathBuf,
}

impl Default for RemoveDuplicatesPlanOptions {
    fn default() -> Self {
        Self { dedup_mode: DedupMode::Exact, keep_order: true, threads_override: None }
    }
}

/// # Errors
/// Returns an error if any requested deduplication tool is not admitted for
/// `fastq.remove_duplicates`.
pub fn normalize_deduplicate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}

#[must_use]
pub fn deduplicate_tool_supports_paired_mode(tool_id: &str, paired_mode: bool) -> bool {
    match tool_id {
        "fastuniq" => paired_mode,
        "clumpify" => true,
        _ => false,
    }
}

fn deduplicate_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastuniq" => Some("fastuniq.fastq.gz"),
        "clumpify" => Some("clumpify.fastq.gz"),
        _ => None,
    }
}

/// Build a deduplicate plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_deduplicate(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_deduplicate_with_options(tool, r1, r2, out_dir, &RemoveDuplicatesPlanOptions::default())
}

/// Build a deduplicate plan with governed stage options.
///
/// # Errors
/// Returns an error if the tool is unsupported or the requested options are not yet supported
/// by the backend-specific adapter.
pub fn plan_deduplicate_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &RemoveDuplicatesPlanOptions,
) -> Result<StagePlanV1> {
    let paired_mode = r2.is_some();
    validate_deduplicate_options(&tool.tool_id.0, paired_mode, options)?;
    let threads = options.threads_override.unwrap_or(tool.resources.threads).max(1);
    let paths = deduplicate_plan_paths(&tool.tool_id.0, paired_mode, out_dir)?;
    let inputs = deduplicate_inputs(r1, r2);
    let outputs = deduplicate_outputs(&paths);
    let effective_params = deduplicate_effective_params(paired_mode, threads, options);
    let mut resources = tool.resources.clone();
    resources.threads = threads;
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: deduplicate_command(
                tool,
                r1,
                r2,
                &paths.output_r1,
                paths.output_r2.as_deref(),
                &paths.duplicate_classes_tsv,
                &paths.duplicate_provenance_json,
                &paths.report,
                out_dir,
                options,
            )?,
        },
        resources,
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: deduplicate_plan_params(&tool.tool_id.0, r1, r2, &paths, threads, options),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize deduplicate effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn deduplicate_plan_paths(
    tool_id: &str,
    paired_mode: bool,
    out_dir: &Path,
) -> Result<DeduplicatePlanPaths> {
    let output_r1 = if paired_mode {
        out_dir.join(format!("{tool_id}.dedup.R1.fastq.gz"))
    } else {
        out_dir.join(
            deduplicate_output_name(tool_id)
                .ok_or_else(|| anyhow!("unsupported deduplicate tool"))?,
        )
    };
    Ok(DeduplicatePlanPaths {
        output_r1,
        output_r2: paired_mode.then(|| out_dir.join(format!("{tool_id}.dedup.R2.fastq.gz"))),
        report: out_dir.join("deduplicate_report.json"),
        duplicate_classes_tsv: out_dir.join("duplicate_classes.tsv"),
        duplicate_provenance_json: out_dir.join("duplicate_provenance.json"),
    })
}

fn deduplicate_inputs(r1: &Path, r2: Option<&Path>) -> Vec<ArtifactRef> {
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    inputs
}

fn deduplicate_outputs(paths: &DeduplicatePlanPaths) -> Vec<ArtifactRef> {
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("dedup_reads_r1"),
        paths.output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &paths.output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("dedup_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("duplicate_classes_tsv"),
        paths.duplicate_classes_tsv.clone(),
        ArtifactRole::SummaryTsv,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("duplicate_provenance_json"),
        paths.duplicate_provenance_json.clone(),
        ArtifactRole::SummaryJson,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        paths.report.clone(),
        ArtifactRole::ReportJson,
    ));
    outputs
}

fn deduplicate_effective_params(
    paired_mode: bool,
    threads: u32,
    options: &RemoveDuplicatesPlanOptions,
) -> RemoveDuplicatesEffectiveParams {
    RemoveDuplicatesEffectiveParams {
        schema_version: REMOVE_DUPLICATES_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(paired_mode),
        threads,
        dedup_mode: options.dedup_mode.clone(),
        keep_order: options.keep_order,
    }
}

fn deduplicate_plan_params(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    paths: &DeduplicatePlanPaths,
    threads: u32,
    options: &RemoveDuplicatesPlanOptions,
) -> serde_json::Value {
    serde_json::json!({
        "tool": tool_id,
        "input_r1": r1,
        "input_r2": r2,
        "output_r1": paths.output_r1,
        "output_r2": paths.output_r2,
        "duplicate_classes_tsv": paths.duplicate_classes_tsv,
        "duplicate_provenance_json": paths.duplicate_provenance_json,
        "report_json": paths.report,
        "threads": threads,
        "dedup_mode": options.dedup_mode,
        "keep_order": options.keep_order,
    })
}

fn deduplicate_command(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    duplicate_classes_tsv: &Path,
    duplicate_provenance_json: &Path,
    report: &Path,
    out_dir: &Path,
    options: &RemoveDuplicatesPlanOptions,
) -> Result<Vec<String>> {
    let tool_id = tool.tool_id.as_str();
    if tool_id == "fastuniq" {
        r2.ok_or_else(|| anyhow!("fastuniq requires paired-end reads"))?;
        output_r2.ok_or_else(|| anyhow!("fastuniq requires paired deduplicated output"))?;
    }
    let context = DeduplicateCommandContext {
        tool_id,
        r1,
        r2,
        output_r1,
        output_r2,
        duplicate_classes_tsv,
        duplicate_provenance_json,
        report,
        out_dir,
        backend_log: deduplicate_backend_log(tool_id, out_dir)?,
        threads: options.threads_override.unwrap_or(tool.resources.threads).max(1),
        options,
    };
    let rendered = render_deduplicate_command(tool, &context)?;
    let mut script = deduplicate_script_prelude(&rendered);
    script.push_str(&deduplicate_read_count_script(&context));
    script.push_str(&deduplicate_classes_script(&context));
    script.push_str(&deduplicate_provenance_script(&context)?);
    script.push_str(&deduplicate_report_script(&context)?);
    Ok(vec!["bash".to_string(), "-lc".to_string(), script])
}

struct DeduplicateCommandContext<'a> {
    tool_id: &'a str,
    r1: &'a Path,
    r2: Option<&'a Path>,
    output_r1: &'a Path,
    output_r2: Option<&'a Path>,
    duplicate_classes_tsv: &'a Path,
    duplicate_provenance_json: &'a Path,
    report: &'a Path,
    out_dir: &'a Path,
    backend_log: std::path::PathBuf,
    threads: u32,
    options: &'a RemoveDuplicatesPlanOptions,
}

fn deduplicate_backend_log(tool_id: &str, out_dir: &Path) -> Result<std::path::PathBuf> {
    match tool_id {
        "fastuniq" => Ok(out_dir.join("fastuniq.log")),
        "clumpify" => Ok(out_dir.join("clumpify.log")),
        _ => Err(anyhow!("unsupported deduplicate tool {tool_id}")),
    }
}

fn render_deduplicate_command(
    tool: &ToolExecutionSpecV1,
    context: &DeduplicateCommandContext<'_>,
) -> Result<Vec<String>> {
    let paired_io_args = match (context.r2, context.output_r2) {
        (Some(r2), Some(output_r2)) => {
            format!("in2='{}' out2='{}'", r2.display(), output_r2.display())
        }
        (None, None) => String::new(),
        _ => return Err(anyhow!("paired remove-duplicates IO bindings are incomplete")),
    };
    let keep_order_args = if context.tool_id == "clumpify" {
        if context.options.keep_order {
            "reorder=t".to_string()
        } else {
            "reorder=f".to_string()
        }
    } else {
        String::new()
    };
    let dedup_mode_args = match (context.tool_id, &context.options.dedup_mode) {
        ("clumpify", DedupMode::Exact) => "dedupe=t".to_string(),
        ("clumpify", DedupMode::OpticalAware) => "dedupe=t optical dupedist=40".to_string(),
        _ => String::new(),
    };
    let threads_args = if context.tool_id == "clumpify" {
        format!("threads={}", context.threads)
    } else {
        String::new()
    };
    crate::tool_adapters::template_render::render_command_template(
        &tool.command.template,
        &[
            ("reads", Some(context.r1.display().to_string())),
            ("reads_r1", Some(context.r1.display().to_string())),
            ("reads_r2", context.r2.map(|path| path.display().to_string())),
            ("dedup_reads_r1", Some(context.output_r1.display().to_string())),
            ("dedup_reads_r2", context.output_r2.map(|path| path.display().to_string())),
            ("report_json", Some(context.report.display().to_string())),
            ("out_dir", Some(context.out_dir.display().to_string())),
            ("paired_io_args", Some(paired_io_args)),
            ("keep_order_args", Some(keep_order_args)),
            ("dedup_mode_args", Some(dedup_mode_args)),
            ("threads_args", Some(threads_args)),
        ],
    )
}

fn deduplicate_script_prelude(rendered: &[String]) -> String {
    format!(
        "set -euo pipefail\ncount_fastq_reads() {{ case \"$1\" in *.gz) gzip -dc -- \"$1\" ;; *) cat -- \"$1\" ;; esac | awk 'END {{ print NR / 4 }}'; }}\n{}\n",
        shell_join(rendered),
    )
}

fn deduplicate_read_count_script(context: &DeduplicateCommandContext<'_>) -> String {
    let mut script = String::new();
    script.push_str(&format!(
        "reads_in=$(count_fastq_reads {})\nreads_out=$(count_fastq_reads {})\n",
        shell_quote_path(context.r1),
        shell_quote_path(context.output_r1),
    ));
    if let (Some(r2), Some(output_r2)) = (context.r2, context.output_r2) {
        script.push_str(&format!(
            "reads_in_r2=$(count_fastq_reads {})\nreads_out_r2=$(count_fastq_reads {})\n",
            shell_quote_path(r2),
            shell_quote_path(output_r2),
        ));
        script.push_str(
            "pairs_in=$reads_in\npairs_out=$reads_out\npair_count_match=true\nif [ \"$reads_in\" -ne \"$reads_in_r2\" ] || [ \"$reads_out\" -ne \"$reads_out_r2\" ]; then pair_count_match=false; fi\n",
        );
    } else {
        script.push_str(
            "reads_in_r2=null\nreads_out_r2=null\npairs_in=null\npairs_out=null\npair_count_match=null\n",
        );
    }
    script.push_str(
        "duplicates_removed=$((reads_in - reads_out))\nif [ \"$reads_in\" -gt 0 ]; then dedup_rate=$(awk -v removed=\"$duplicates_removed\" -v total=\"$reads_in\" 'BEGIN { printf \"%.12f\", removed / total }'); else dedup_rate=0; fi\n",
    );
    script
}

fn deduplicate_classes_script(context: &DeduplicateCommandContext<'_>) -> String {
    let paired_mode = if context.r2.is_some() { "paired_end" } else { "single_end" };
    let mut script = String::new();
    script.push_str(&format!(
        "printf 'class\\treads_removed\\tpaired_mode\\n' > {}\nprintf 'duplicate\\t%s\\t%s\\n' \"$duplicates_removed\" {} >> {}\n",
        shell_quote_path(context.duplicate_classes_tsv),
        shell_quote_str(paired_mode),
        shell_quote_path(context.duplicate_classes_tsv),
    ));
    script
}

fn deduplicate_backend_report_format(tool_id: &str) -> &'static str {
    match tool_id {
        "fastuniq" => "fastuniq_log",
        "clumpify" => "clumpify_log",
        _ => "backend_log",
    }
}

fn deduplicate_paired_mode_label(context: &DeduplicateCommandContext<'_>) -> &'static str {
    if context.r2.is_some() {
        "paired_end"
    } else {
        "single_end"
    }
}

fn deduplicate_provenance_script(context: &DeduplicateCommandContext<'_>) -> Result<String> {
    let backend_report_format = deduplicate_backend_report_format(context.tool_id);
    let provenance_format = format!(
        "{{\"schema_version\":{},\"stage_id\":{},\"tool_id\":{},\"paired_mode\":{},\"threads\":{},\"dedup_mode\":{},\"keep_order\":{},\"duplicates_removed\":%s,\"dedup_rate\":%s,\"backend_log\":%s,\"input_r1\":%s,\"input_r2\":%s,\"output_r1\":%s,\"output_r2\":%s,\"raw_backend_report\":%s,\"raw_backend_report_format\":{}}}",
        json_string_literal(REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION)?,
        json_string_literal(STAGE_ID.as_str())?,
        json_string_literal(context.tool_id)?,
        json_string_literal(deduplicate_paired_mode_label(context))?,
        context.threads,
        serde_json::to_string(&context.options.dedup_mode)
            .map_err(|error| anyhow!("serialize dedup_mode for provenance: {error}"))?,
        if context.options.keep_order { "true" } else { "false" },
        json_string_literal(backend_report_format)?,
    );
    Ok(format!(
        "printf '{}' \"$duplicates_removed\" \"$dedup_rate\" {} {} {} {} {} {} > {}\n",
        provenance_format,
        shell_quote_str(&json_path_token(&context.backend_log)?),
        shell_quote_str(&json_path_token(context.r1)?),
        shell_quote_str(&json_optional_path_token(context.r2)?),
        shell_quote_str(&json_path_token(context.output_r1)?),
        shell_quote_str(&json_optional_path_token(context.output_r2)?),
        shell_quote_str(&json_path_token(&context.backend_log)?),
        shell_quote_path(context.duplicate_provenance_json),
    ))
}

fn deduplicate_report_script(context: &DeduplicateCommandContext<'_>) -> Result<String> {
    let backend_report_format = deduplicate_backend_report_format(context.tool_id);
    let report_format = format!(
        "{{\"schema_version\":{},\"stage\":{},\"stage_id\":{},\"tool_id\":{},\"paired_mode\":{},\"threads\":{},\"dedup_mode\":{},\"keep_order\":{},\"input_r1\":%s,\"input_r2\":%s,\"output_r1\":%s,\"output_r2\":%s,\"reads_in\":%s,\"reads_out\":%s,\"reads_in_r2\":%s,\"reads_out_r2\":%s,\"pairs_in\":%s,\"pairs_out\":%s,\"pair_count_match\":%s,\"duplicates_removed\":%s,\"dedup_rate\":%s,\"duplicate_classes_tsv\":%s,\"duplicate_provenance_json\":%s,\"duplicate_classes\":[{{\"class\":\"duplicate\",\"reads_removed\":%s,\"paired_mode\":{}}}],\"raw_backend_report\":%s,\"raw_backend_report_format\":{},\"runtime_s\":null,\"memory_mb\":null}}",
        json_string_literal(REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION)?,
        json_string_literal(STAGE_ID.as_str())?,
        json_string_literal(STAGE_ID.as_str())?,
        json_string_literal(context.tool_id)?,
        json_string_literal(deduplicate_paired_mode_label(context))?,
        context.threads,
        serde_json::to_string(&context.options.dedup_mode)
            .map_err(|error| anyhow!("serialize dedup_mode for report: {error}"))?,
        if context.options.keep_order { "true" } else { "false" },
        json_string_literal(deduplicate_paired_mode_label(context))?,
        json_string_literal(backend_report_format)?,
    );
    Ok(format!(
        "printf '{}' {} {} {} {} \"$reads_in\" \"$reads_out\" \"$reads_in_r2\" \"$reads_out_r2\" \"$pairs_in\" \"$pairs_out\" \"$pair_count_match\" \"$duplicates_removed\" \"$dedup_rate\" {} {} \"$duplicates_removed\" {} > {}\n",
        report_format,
        shell_quote_str(&json_path_token(context.r1)?),
        shell_quote_str(&json_optional_path_token(context.r2)?),
        shell_quote_str(&json_path_token(context.output_r1)?),
        shell_quote_str(&json_optional_path_token(context.output_r2)?),
        shell_quote_str(&json_path_token(context.duplicate_classes_tsv)?),
        shell_quote_str(&json_path_token(context.duplicate_provenance_json)?),
        shell_quote_str(&json_path_token(&context.backend_log)?),
        shell_quote_path(context.report),
    ))
}

fn validate_deduplicate_options(
    tool_id: &str,
    paired_mode: bool,
    options: &RemoveDuplicatesPlanOptions,
) -> Result<()> {
    if !deduplicate_tool_supports_paired_mode(tool_id, paired_mode) {
        return Err(anyhow!("fastuniq requires paired-end reads"));
    }
    if options.dedup_mode != DedupMode::Exact
        && !(tool_id == "clumpify" && options.dedup_mode == DedupMode::OpticalAware)
    {
        return Err(anyhow!(
            "{tool_id} remove-duplicates adapter currently supports dedup_mode=exact{}",
            if tool_id == "clumpify" { " or dedup_mode=optical_aware" } else { "" }
        ));
    }
    if !options.keep_order && tool_id != "clumpify" {
        return Err(anyhow!(
            "{tool_id} remove-duplicates adapter currently supports only keep_order=true"
        ));
    }
    if options.threads_override.is_some() && tool_id != "clumpify" {
        return Err(anyhow!(
            "{tool_id} remove-duplicates adapter currently supports explicit thread overrides only for clumpify"
        ));
    }
    Ok(())
}

/// # Errors
/// Returns an error if the deduplication mode literal is not supported.
pub fn dedup_mode_from_literal(value: &str) -> Result<DedupMode> {
    match value {
        "exact" => Ok(DedupMode::Exact),
        "sequence_identity" => Ok(DedupMode::SequenceIdentity),
        "optical_aware" => Ok(DedupMode::OpticalAware),
        _ => Err(anyhow!("unsupported dedup_mode: {value}")),
    }
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn json_path_token(path: &Path) -> Result<String> {
    serde_json::to_string(&path.display().to_string())
        .map_err(|error| anyhow!("serialize path token for deduplicate report: {error}"))
}

fn json_optional_path_token(path: Option<&Path>) -> Result<String> {
    serde_json::to_string(&path.map(|value| value.display().to_string()))
        .map_err(|error| anyhow!("serialize optional path token for deduplicate report: {error}"))
}

fn json_string_literal(value: &str) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|error| anyhow!("serialize deduplicate string literal: {error}"))
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn shell_join(command: &[String]) -> String {
    command.iter().map(|part| shell_quote_str(part)).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::ids::ToolId;
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolConstraints};

    fn dummy_tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id.to_string()),
            tool_version: "fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: match tool_id {
                    "fastuniq" => vec![
                        "bash".to_string(),
                        "-lc".to_string(),
                        "set -euo pipefail\nwork_dir='{{out_dir}}/fastuniq_workspace'\nmkdir -p \"$work_dir\"\ngzip -dc -- '{{reads_r1}}' > \"$work_dir/input_r1.fastq\"\ngzip -dc -- '{{reads_r2}}' > \"$work_dir/input_r2.fastq\"\nprintf '%s\\n%s\\n' \"$work_dir/input_r1.fastq\" \"$work_dir/input_r2.fastq\" > '{{out_dir}}/fastuniq_inputs.txt'\nfastuniq -i '{{out_dir}}/fastuniq_inputs.txt' -t q -o \"$work_dir/output_r1.fastq\" -p \"$work_dir/output_r2.fastq\" > '{{out_dir}}/fastuniq.log' 2>&1\ngzip -c \"$work_dir/output_r1.fastq\" > '{{dedup_reads_r1}}'\ngzip -c \"$work_dir/output_r2.fastq\" > '{{dedup_reads_r2}}'\nrm -rf \"$work_dir\"\n".to_string(),
                    ],
                    "clumpify" => vec![
                        "bash".to_string(),
                        "-lc".to_string(),
                        "set -euo pipefail\nclumpify in='{{reads_r1}}' {{paired_io_args}} out='{{dedup_reads_r1}}' {{dedup_mode_args}} {{keep_order_args}} {{threads_args}} > '{{out_dir}}/clumpify.log' 2>&1\n".to_string(),
                    ],
                    _ => vec!["unused".to_string()],
                },
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        }
    }

    #[test]
    fn deduplicate_output_name_rejects_unadmitted_tools() {
        assert!(deduplicate_output_name("prinseq").is_none());
    }

    #[test]
    fn deduplicate_tool_supports_paired_mode_reflects_backend_capabilities() {
        assert!(!deduplicate_tool_supports_paired_mode("fastuniq", false));
        assert!(deduplicate_tool_supports_paired_mode("fastuniq", true));
        assert!(deduplicate_tool_supports_paired_mode("clumpify", false));
        assert!(deduplicate_tool_supports_paired_mode("clumpify", true));
    }

    #[test]
    fn plan_deduplicate_fastuniq_builds_paired_command_and_report() {
        let plan = plan_deduplicate(
            &dummy_tool("fastuniq"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
        )
        .expect("deduplicate planner should build fastuniq command");
        assert_eq!(plan.command.template[0], "bash");
        assert_eq!(plan.command.template[1], "-lc");
        let script = &plan.command.template[2];
        assert!(script.contains("fastuniq_inputs.txt"));
        assert!(script.contains("fastuniq_workspace"));
        assert!(script.contains("fastuniq.log"));
        assert!(script.contains("gzip -dc"));
        assert!(script.contains("gzip -c"));
        assert!(script.contains("\"tool_id\":\"fastuniq\""));
        assert!(script.contains("bijux.fastq.remove_duplicates.report.v2"));
        assert!(script.contains("bijux.fastq.remove_duplicates.provenance.v2"));
        assert!(!script.contains("bijux.fastq.remove_duplicates.report.v1"));
        assert!(script.contains("count_fastq_reads"));
        assert!(script.contains("\"reads_in\":%s"));
        assert_eq!(plan.params["report_json"], serde_json::json!("out/deduplicate_report.json"));
        assert_eq!(
            plan.effective_params["schema_version"],
            serde_json::json!(REMOVE_DUPLICATES_SCHEMA_VERSION)
        );
        assert_eq!(plan.effective_params["dedup_mode"], serde_json::json!("exact"));
        assert_eq!(plan.effective_params["keep_order"], serde_json::json!(true));
        assert!(script.contains("reads_R1.fastq.gz"));
        assert!(script.contains("reads_R2.fastq.gz"));
        assert!(script.contains("deduplicate_report.json"));
    }

    #[test]
    fn plan_deduplicate_fastuniq_rejects_single_end_inputs() {
        let error = plan_deduplicate(
            &dummy_tool("fastuniq"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("out"),
        )
        .expect_err("fastuniq must reject single-end dedup planning");
        assert!(error.to_string().contains("paired-end"));
    }

    #[test]
    fn plan_deduplicate_clumpify_emits_governed_report() {
        let plan = plan_deduplicate(
            &dummy_tool("clumpify"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
        )
        .expect("clumpify single-end dedup planning should succeed");

        assert_eq!(plan.command.template[0], "bash");
        assert_eq!(plan.command.template[1], "-lc");
        let script = &plan.command.template[2];
        assert!(script.contains("clumpify "));
        assert!(script.contains("clumpify.log"));
        assert!(script.contains("\"tool_id\":\"clumpify\""));
        assert!(script.contains("\"raw_backend_report_format\":\"clumpify_log\""));
        assert!(!script.contains("bijux.fastq.remove_duplicates.report.v1"));
        assert!(script.contains("duplicate_classes.tsv"));
        assert!(script.contains("duplicate_provenance.json"));
        assert!(!script.contains("{{paired_io_args}}"));
    }

    #[test]
    fn plan_deduplicate_rejects_non_exact_mode_until_backend_support_exists() {
        let error = plan_deduplicate_with_options(
            &dummy_tool("clumpify"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &RemoveDuplicatesPlanOptions {
                dedup_mode: DedupMode::SequenceIdentity,
                keep_order: true,
                threads_override: None,
            },
        )
        .expect_err("non-exact dedup mode must fail until backend mapping exists");

        assert!(error.to_string().contains("optical_aware"));
    }

    #[test]
    fn plan_deduplicate_clumpify_maps_keep_order_policy() {
        let plan = plan_deduplicate_with_options(
            &dummy_tool("clumpify"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &RemoveDuplicatesPlanOptions {
                dedup_mode: DedupMode::Exact,
                keep_order: false,
                threads_override: None,
            },
        )
        .expect("clumpify should map keep_order into execution");

        assert!(plan.command.template[2].contains("reorder=f"));
        assert_eq!(plan.effective_params["keep_order"], serde_json::json!(false));
    }

    #[test]
    fn plan_deduplicate_clumpify_maps_optical_aware_mode() {
        let plan = plan_deduplicate_with_options(
            &dummy_tool("clumpify"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &RemoveDuplicatesPlanOptions {
                dedup_mode: DedupMode::OpticalAware,
                keep_order: true,
                threads_override: None,
            },
        )
        .expect("clumpify should map optical-aware dedup mode");

        assert!(plan.command.template[2].contains("optical"));
        assert!(plan.command.template[2].contains("dupedist=40"));
        assert_eq!(plan.effective_params["dedup_mode"], serde_json::json!("optical_aware"));
    }

    #[test]
    fn plan_deduplicate_emits_duplicate_provenance_outputs() {
        let plan = plan_deduplicate(
            &dummy_tool("clumpify"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
        )
        .expect("deduplicate planner should emit provenance artifacts");

        let output_ids = plan
            .io
            .outputs
            .iter()
            .map(|artifact| artifact.name.as_str().to_string())
            .collect::<Vec<_>>();
        assert!(output_ids.contains(&"duplicate_classes_tsv".to_string()));
        assert!(output_ids.contains(&"duplicate_provenance_json".to_string()));
    }

    #[test]
    fn plan_deduplicate_clumpify_maps_thread_override() {
        let plan = plan_deduplicate_with_options(
            &dummy_tool("clumpify"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("out"),
            &RemoveDuplicatesPlanOptions {
                dedup_mode: DedupMode::Exact,
                keep_order: true,
                threads_override: Some(8),
            },
        )
        .expect("clumpify should map explicit threads");

        let script = &plan.command.template[2];
        assert_eq!(plan.resources.threads, 8);
        assert_eq!(plan.effective_params["threads"], serde_json::json!(8));
        assert!(script.contains("threads=8"));
    }

    #[test]
    fn plan_deduplicate_fastuniq_rejects_explicit_thread_override() {
        let error = plan_deduplicate_with_options(
            &dummy_tool("fastuniq"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            &RemoveDuplicatesPlanOptions {
                dedup_mode: DedupMode::Exact,
                keep_order: true,
                threads_override: Some(8),
            },
        )
        .expect_err("fastuniq should reject explicit thread overrides");

        assert!(error.to_string().contains("supports explicit thread overrides only for clumpify"));
    }
}
