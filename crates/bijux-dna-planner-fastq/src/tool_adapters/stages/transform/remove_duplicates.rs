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
use bijux_dna_domain_fastq::STAGE_REMOVE_DUPLICATES;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_REMOVE_DUPLICATES;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveDuplicatesPlanOptions {
    pub dedup_mode: DedupMode,
    pub keep_order: bool,
}

impl Default for RemoveDuplicatesPlanOptions {
    fn default() -> Self {
        Self {
            dedup_mode: DedupMode::Exact,
            keep_order: true,
        }
    }
}

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
    plan_deduplicate_with_options(
        tool,
        r1,
        r2,
        out_dir,
        &RemoveDuplicatesPlanOptions::default(),
    )
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
    let output_r1 = if paired_mode {
        out_dir.join(format!("{}.dedup.R1.fastq.gz", tool.tool_id))
    } else {
        let output_name = deduplicate_output_name(&tool.tool_id.0)
            .ok_or_else(|| anyhow!("unsupported deduplicate tool"))?;
        out_dir.join(output_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("{}.dedup.R2.fastq.gz", tool.tool_id)));
    let report = out_dir.join("deduplicate_report.json");
    let duplicate_classes_tsv = out_dir.join("duplicate_classes.tsv");
    let duplicate_provenance_json = out_dir.join("duplicate_provenance.json");
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
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("dedup_reads_r1"),
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("dedup_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("duplicate_classes_tsv"),
        duplicate_classes_tsv.clone(),
        ArtifactRole::SummaryTsv,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("duplicate_provenance_json"),
        duplicate_provenance_json.clone(),
        ArtifactRole::SummaryJson,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
    ));
    let effective_params = RemoveDuplicatesEffectiveParams {
        schema_version: REMOVE_DUPLICATES_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(paired_mode),
        dedup_mode: options.dedup_mode.clone(),
        keep_order: options.keep_order,
    };
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
                &output_r1,
                output_r2.as_deref(),
                &duplicate_classes_tsv,
                &duplicate_provenance_json,
                &report,
                out_dir,
                options,
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "duplicate_classes_tsv": duplicate_classes_tsv,
            "duplicate_provenance_json": duplicate_provenance_json,
            "report_json": report,
            "dedup_mode": options.dedup_mode,
            "keep_order": options.keep_order,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize deduplicate effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
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
    let backend_log = match tool_id {
        "fastuniq" => out_dir.join("fastuniq.log"),
        "clumpify" => out_dir.join("clumpify.log"),
        _ => return Err(anyhow!("unsupported deduplicate tool {tool_id}")),
    };
    if tool_id == "fastuniq" {
        r2.ok_or_else(|| anyhow!("fastuniq requires paired-end reads"))?;
        output_r2.ok_or_else(|| anyhow!("fastuniq requires paired deduplicated output"))?;
    }
    let paired_io_args = match (r2, output_r2) {
        (Some(r2), Some(output_r2)) => {
            format!("in2='{}' out2='{}'", r2.display(), output_r2.display())
        }
        (None, None) => String::new(),
        _ => return Err(anyhow!("paired remove-duplicates IO bindings are incomplete")),
    };
    let keep_order_args = if tool_id == "clumpify" {
        if options.keep_order {
            "reorder=t".to_string()
        } else {
            "reorder=f".to_string()
        }
    } else {
        String::new()
    };
    let rendered = crate::tool_adapters::template_render::render_command_template(
        &tool.command.template,
        &[
            ("reads", Some(r1.display().to_string())),
            ("reads_r1", Some(r1.display().to_string())),
            ("reads_r2", r2.map(|path| path.display().to_string())),
            ("dedup_reads_r1", Some(output_r1.display().to_string())),
            (
                "dedup_reads_r2",
                output_r2.map(|path| path.display().to_string()),
            ),
            ("report_json", Some(report.display().to_string())),
            ("out_dir", Some(out_dir.display().to_string())),
            ("paired_io_args", Some(paired_io_args)),
            ("keep_order_args", Some(keep_order_args)),
        ],
    )?;
    let mut script = format!(
        "set -euo pipefail\ncount_fastq_reads() {{ case \"$1\" in *.gz) gzip -dc -- \"$1\" ;; *) cat -- \"$1\" ;; esac | awk 'END {{ print NR / 4 }}'; }}\n{}\n",
        shell_join(&rendered),
    );
    script.push_str(&format!(
        "reads_in=$(count_fastq_reads {})\nreads_out=$(count_fastq_reads {})\n",
        shell_quote_path(r1),
        shell_quote_path(output_r1),
    ));
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
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
    script.push_str(&format!(
        "printf 'class\\treads_removed\\tpaired_mode\\n' > {}\nprintf 'duplicate\\t%s\\t%s\\n' \"$duplicates_removed\" {} >> {}\n",
        shell_quote_path(duplicate_classes_tsv),
        shell_quote_str(if r2.is_some() { "paired_end" } else { "single_end" }),
        shell_quote_path(duplicate_classes_tsv),
    ));
    let provenance_format = format!(
        "{{\"schema_version\":\"bijux.fastq.remove_duplicates.provenance.v1\",\"stage_id\":{},\"tool_id\":{},\"paired_mode\":{},\"dedup_mode\":{},\"keep_order\":{},\"duplicates_removed\":%s,\"dedup_rate\":%s,\"backend_log\":%s,\"input_r1\":%s,\"input_r2\":%s,\"output_r1\":%s,\"output_r2\":%s}}",
        json_string_literal(STAGE_ID.as_str())?,
        json_string_literal(tool_id)?,
        json_string_literal(if r2.is_some() { "paired_end" } else { "single_end" })?,
        serde_json::to_string(&options.dedup_mode)
            .map_err(|error| anyhow!("serialize dedup_mode for provenance: {error}"))?,
        if options.keep_order { "true" } else { "false" },
    );
    script.push_str(&format!(
        "printf '{}' \"$duplicates_removed\" \"$dedup_rate\" {} {} {} {} {} > {}\n",
        escape_printf_format(&provenance_format),
        shell_quote_str(&json_path_token(&backend_log)?),
        shell_quote_str(&json_path_token(r1)?),
        shell_quote_str(&json_optional_path_token(r2)?),
        shell_quote_str(&json_path_token(output_r1)?),
        shell_quote_str(&json_optional_path_token(output_r2)?),
        shell_quote_path(duplicate_provenance_json),
    ));
    let report_format = format!(
        "{{\"schema_version\":\"bijux.fastq.remove_duplicates.report.v1\",\"stage_id\":{},\"tool_id\":{},\"paired_mode\":{},\"dedup_mode\":{},\"keep_order\":{},\"input_r1\":%s,\"input_r2\":%s,\"output_r1\":%s,\"output_r2\":%s,\"backend_log\":%s,\"reads_in\":%s,\"reads_out\":%s,\"reads_in_r2\":%s,\"reads_out_r2\":%s,\"pairs_in\":%s,\"pairs_out\":%s,\"pair_count_match\":%s,\"duplicates_removed\":%s,\"dedup_rate\":%s}}",
        json_string_literal(STAGE_ID.as_str())?,
        json_string_literal(tool_id)?,
        json_string_literal(if r2.is_some() { "paired_end" } else { "single_end" })?,
        serde_json::to_string(&options.dedup_mode)
            .map_err(|error| anyhow!("serialize dedup_mode for report: {error}"))?,
        if options.keep_order { "true" } else { "false" },
    );
    script.push_str(&format!(
        "printf '{}' {} {} {} {} {} \"$reads_in\" \"$reads_out\" \"$reads_in_r2\" \"$reads_out_r2\" \"$pairs_in\" \"$pairs_out\" \"$pair_count_match\" \"$duplicates_removed\" \"$dedup_rate\" > {}\n",
        escape_printf_format(&report_format),
        shell_quote_str(&json_path_token(r1)?),
        shell_quote_str(&json_optional_path_token(r2)?),
        shell_quote_str(&json_path_token(output_r1)?),
        shell_quote_str(&json_optional_path_token(output_r2)?),
        shell_quote_str(&json_path_token(&backend_log)?),
        shell_quote_path(report),
    ));
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn validate_deduplicate_options(
    tool_id: &str,
    paired_mode: bool,
    options: &RemoveDuplicatesPlanOptions,
) -> Result<()> {
    if !deduplicate_tool_supports_paired_mode(tool_id, paired_mode) {
        return Err(anyhow!("fastuniq requires paired-end reads"));
    }
    if options.dedup_mode != DedupMode::Exact {
        return Err(anyhow!(
            "{tool_id} remove-duplicates adapter currently supports only dedup_mode=exact"
        ));
    }
    if !options.keep_order && tool_id != "clumpify" {
        return Err(anyhow!(
            "{tool_id} remove-duplicates adapter currently supports only keep_order=true"
        ));
    }
    Ok(())
}

pub fn parse_dedup_mode(value: &str) -> Result<DedupMode> {
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

fn escape_printf_format(value: &str) -> String {
    value.replace('%', "%%")
}

fn json_string_literal(value: &str) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|error| anyhow!("serialize deduplicate string literal: {error}"))
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn shell_join(command: &[String]) -> String {
    command
        .iter()
        .map(|part| shell_quote_str(part))
        .collect::<Vec<_>>()
        .join(" ")
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
                        "sh".to_string(),
                        "-lc".to_string(),
                        "set -euo pipefail\nprintf '%s\\n%s\\n' '{{reads_r1}}' '{{reads_r2}}' > '{{out_dir}}/fastuniq_inputs.txt'\nfastuniq -i '{{out_dir}}/fastuniq_inputs.txt' -t q -o '{{dedup_reads_r1}}' -p '{{dedup_reads_r2}}' > '{{out_dir}}/fastuniq.log' 2>&1\nprintf '%s\\n' '{\"schema_version\":\"bijux.fastq.remove_duplicates.report.v1\",\"tool_id\":\"fastuniq\"}' > '{{report_json}}'\n".to_string(),
                    ],
                    "clumpify" => vec![
                        "sh".to_string(),
                        "-lc".to_string(),
                        "set -euo pipefail\nclumpify.sh in='{{reads_r1}}' {{paired_io_args}} out='{{dedup_reads_r1}}' dedupe=t {{keep_order_args}} > '{{out_dir}}/clumpify.log' 2>&1\nprintf '%s\\n' '{\"schema_version\":\"bijux.fastq.remove_duplicates.report.v1\",\"tool_id\":\"clumpify\"}' > '{{report_json}}'\n".to_string(),
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
        assert_eq!(plan.command.template[0], "sh");
        assert_eq!(plan.command.template[1], "-lc");
        let script = &plan.command.template[2];
        assert!(script.contains("fastuniq_inputs.txt"));
        assert!(script.contains("fastuniq.log"));
        assert!(script.contains("\"tool_id\":\"fastuniq\""));
        assert!(script.contains("count_fastq_reads"));
        assert!(script.contains("\"reads_in\":%%s"));
        assert_eq!(
            plan.params["report_json"],
            serde_json::json!("out/deduplicate_report.json")
        );
        assert_eq!(
            plan.effective_params["schema_version"],
            serde_json::json!("bijux.fastq.params.remove_duplicates.v1")
        );
        assert_eq!(
            plan.effective_params["dedup_mode"],
            serde_json::json!("exact")
        );
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

        assert_eq!(plan.command.template[0], "sh");
        assert_eq!(plan.command.template[1], "-lc");
        let script = &plan.command.template[2];
        assert!(script.contains("clumpify.sh"));
        assert!(script.contains("clumpify.log"));
        assert!(script.contains("\"tool_id\":\"clumpify\""));
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
            },
        )
        .expect_err("non-exact dedup mode must fail until backend mapping exists");

        assert!(error.to_string().contains("dedup_mode=exact"));
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
            },
        )
        .expect("clumpify should map keep_order into execution");

        assert!(plan.command.template[2].contains("reorder=f"));
        assert_eq!(plan.effective_params["keep_order"], serde_json::json!(false));
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
}
