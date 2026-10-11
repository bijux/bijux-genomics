use super::*;

pub(crate) fn validate_tool_output_subset(
    tool_raw: &str,
    stage_specs: &[(&str, String)],
    tool_path: &Path,
) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct NamedOutput {
        name: String,
    }
    #[derive(serde::Deserialize)]
    struct ToolOutputsDoc {
        #[serde(default)]
        outputs: Vec<NamedOutput>,
    }
    #[derive(serde::Deserialize)]
    struct StageOutputsDoc {
        #[serde(default)]
        outputs: Vec<NamedOutput>,
    }

    let parsed_tool: ToolOutputsDoc = bijux_dna_infra::formats::parse_yaml(tool_raw)
        .with_context(|| format!("parse {}", tool_path.display()))?;
    if parsed_tool.outputs.is_empty() {
        return Ok(());
    }
    let output_names =
        parsed_tool.outputs.iter().map(|entry| entry.name.as_str()).collect::<BTreeSet<_>>();
    if output_names.is_empty() {
        bail!("{} outputs section must include named outputs", tool_path.display());
    }
    let mut stage_outputs = BTreeSet::new();
    for (stage_id, stage_raw) in stage_specs {
        let stage_yaml: StageOutputsDoc = bijux_dna_infra::formats::parse_yaml(stage_raw)
            .with_context(|| format!("parse stage {stage_id}"))?;
        stage_outputs.extend(stage_yaml.outputs.into_iter().map(|entry| entry.name));
    }
    for output in &output_names {
        if !stage_outputs.contains(*output) {
            bail!(
                "{} output `{}` is not declared by any bound stage outputs",
                tool_path.display(),
                output
            );
        }
    }
    Ok(())
}

pub(crate) fn resolve_tool_citation(raw_citation: &str, upstream: &str) -> String {
    if !raw_citation.starts_with("pending:") {
        return raw_citation.to_string();
    }
    format!("upstream:{upstream}")
}

pub(crate) fn default_version_regex(tool_id: &str) -> &'static str {
    match tool_id {
        "authenticct" => "authentic|v?[0-9]+[.][0-9]+",
        "fastqvalidator" => "fastqvalidator|v?[0-9]+[.][0-9]+",
        _ => "v?[0-9]+[.][0-9]+([.-][0-9A-Za-z]+)?",
    }
}

pub(crate) fn default_healthcheck_cmd(tool_id: &str, help_cmd: &str) -> String {
    if help_cmd.trim().is_empty() {
        return format!("{tool_id} --help");
    }
    help_cmd.to_string()
}

fn tool_role_from_stage_id(stage_id: &str) -> &'static str {
    if stage_id.contains(".align") || stage_id.contains("host_depletion") {
        "aligner"
    } else if stage_id.contains("screen") || stage_id.contains("contaminant") {
        "screen"
    } else if stage_id.contains("trim") || stage_id.contains("adapter") {
        "trimmer"
    } else if stage_id.contains("qc") || stage_id.contains("stats") || stage_id.contains("report") {
        "qc"
    } else if stage_id.contains("filter") {
        "filter"
    } else if stage_id.contains("validate") {
        "validator"
    } else if stage_id.contains("merge") {
        "merger"
    } else if stage_id.contains("correct") {
        "corrector"
    } else {
        "transform"
    }
}

pub(crate) fn infer_tool_role(stage_ids: &[String]) -> String {
    stage_ids.first().map_or_else(
        || "transform".to_string(),
        |stage_id| tool_role_from_stage_id(stage_id).to_string(),
    )
}

pub(crate) fn required_tool_roles_for_stage(stage_id: &str) -> Vec<String> {
    let roles = match stage_id {
        "bam.filter" | "bam.length_filter" | "bam.mapq_filter" => {
            vec!["filter", "transform"]
        }
        "bam.qc_pre" => vec!["qc", "transform"],
        "bam.validate" => vec!["validator", "filter", "transform"],
        "fastq.deplete_host" => vec!["aligner", "transform"],
        "fastq.deplete_reference_contaminants" => vec!["screen", "transform", "aligner"],
        "fastq.estimate_library_complexity_prealign" | "fastq.merge_pairs" => {
            vec!["merger", "transform"]
        }
        "fastq.normalize_abundance" | "fastq.profile_read_lengths" | "fastq.profile_reads" => {
            vec!["transform", "filter"]
        }
        "fastq.profile_overrepresented_sequences" => vec!["transform", "filter", "trimmer"],
        "fastq.trim_polyg_tails" => vec!["trimmer", "filter"],
        "fastq.trim_reads" | "fastq.trim_terminal_damage" => {
            vec!["trimmer", "filter", "transform", "merger"]
        }
        "fastq.validate_reads" => vec!["validator", "transform", "trimmer"],
        _ => vec![tool_role_from_stage_id(stage_id)],
    };

    roles.into_iter().map(str::to_string).collect()
}
