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

pub(crate) fn read_text_if_exists(path: &Path) -> Option<String> {
    if path.exists() {
        std::fs::read_to_string(path).ok()
    } else {
        None
    }
}

fn parse_git_checkout_pin(recipe: &str) -> Option<String> {
    for line in recipe.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("git checkout ") {
            continue;
        }
        let Some((_, rhs)) = trimmed.split_once("git checkout ") else {
            continue;
        };
        let commit = rhs.chars().take_while(char::is_ascii_hexdigit).collect::<String>();
        if commit.len() == 40 {
            return Some(format!("git:{commit}"));
        }
    }
    None
}

fn parse_upstream_from_recipe(recipe: &str) -> Option<String> {
    for line in recipe.lines() {
        let trimmed = line.trim();
        if let Some((_, rhs)) = trimmed.split_once("git clone ") {
            let url = rhs.split_whitespace().next().unwrap_or_default();
            if url.starts_with("http://") || url.starts_with("https://") {
                return Some(url.to_string());
            }
        }
        if let Some((_, rhs)) = trimmed.split_once("wget -q ") {
            let url = rhs.split_whitespace().next().unwrap_or_default();
            if url.starts_with("http://") || url.starts_with("https://") {
                return Some(url.to_string());
            }
        }
    }
    None
}

pub(crate) fn parse_version_from_recipe(recipe: &str) -> Option<String> {
    for line in recipe.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("ARG VERSION_") || !trimmed.contains('=') {
            continue;
        }
        let Some((_, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        let value = rhs.trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn tool_upstream_override(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "adapterremoval" => Some("https://github.com/MikkelSchubert/adapterremoval"),
        "bbduk" | "bbmerge" => Some("https://sourceforge.net/projects/bbmap/"),
        "bayeshammer" | "spades" => Some("https://github.com/ablab/spades"),
        "atropos" => Some("https://github.com/jdidion/atropos"),
        "centrifuge" => Some("https://github.com/DaehwanKimLab/centrifuge"),
        "flash2" => Some("https://github.com/dstreett/FLASH2"),
        "fqtools" => Some("https://github.com/alastair-droop/fqtools"),
        "kaiju" => Some("https://github.com/bioinformatics-centre/kaiju"),
        "lighter" => Some("https://github.com/mourisl/Lighter"),
        "metaphlan" => Some("https://github.com/biobakery/MetaPhlAn"),
        "musket" => Some("https://github.com/alexdobin/musket"),
        "pear" => Some("https://github.com/xflouris/PEAR"),
        "prinseq" => Some("https://github.com/uwb-linux/prinseq"),
        "qualimap" => Some("http://qualimap.conesalab.org/"),
        "rcorrector" => Some("https://github.com/mourisl/Rcorrector"),
        "rxy" => Some("https://github.com/pontussk/rxy"),
        "sortmerna" => Some("https://github.com/sortmerna/sortmerna"),
        "trim_galore" => Some("https://github.com/FelixKrueger/TrimGalore"),
        _ => None,
    }
}

pub(crate) fn tool_version_override(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "authenticct" | "rxy" => Some("1.0.0"),
        "schmutzi" => Some("1.5.4"),
        "seqkit_stats" => Some("2.7.0"),
        "yleaf" => Some("3.0.3"),
        _ => None,
    }
}

pub(crate) fn tool_pin_override(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "rxy" => Some("release:1.0.0"),
        _ => None,
    }
}

pub(crate) fn resolve_tool_upstream(
    raw_upstream: &str,
    tool_id: &str,
    dockerfile: &Path,
) -> String {
    if !raw_upstream.eq_ignore_ascii_case("unknown") {
        return raw_upstream.to_string();
    }
    if let Some(override_url) = tool_upstream_override(tool_id) {
        return override_url.to_string();
    }
    if let Some(content) = read_text_if_exists(dockerfile) {
        if let Some(url) = parse_upstream_from_recipe(&content) {
            return url;
        }
    }
    format!("https://github.com/{tool_id}/{tool_id}")
}

pub(crate) fn resolve_tool_citation(raw_citation: &str, upstream: &str) -> String {
    if !raw_citation.starts_with("pending:") {
        return raw_citation.to_string();
    }
    format!("upstream:{upstream}")
}

pub(crate) fn resolve_upstream_pin(
    container_digest: &str,
    dockerfile: &Path,
    apptainer_def: &Path,
    default_version: &str,
) -> String {
    if container_digest.starts_with("sha256:") {
        return container_digest.to_string();
    }
    if let Some(content) = read_text_if_exists(dockerfile) {
        if let Some(pin) = parse_git_checkout_pin(&content) {
            return pin;
        }
    }
    if let Some(content) = read_text_if_exists(apptainer_def) {
        if let Some(pin) = parse_git_checkout_pin(&content) {
            return pin;
        }
    }
    if default_version != "latest-pinned" {
        return format!("release:{default_version}");
    }
    "unresolved".to_string()
}

pub(crate) fn parse_container_ref(
    image: &str,
    digest: &str,
    tool_id: &str,
    version: &str,
) -> String {
    if !image.is_empty() && digest.starts_with("sha256:") {
        return format!("{image}@{digest}");
    }
    if !image.is_empty() && version != "latest-pinned" {
        return format!("{image}:{version}");
    }
    if digest.starts_with("sha256:") {
        return format!("bijuxdna/{tool_id}@{digest}");
    }
    format!("bijuxdna/{tool_id}:{version}")
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
        "fastq.deplete_reference_contaminants" => vec!["screen", "transform", "aligner"],
        "fastq.merge_pairs" => vec!["merger", "transform"],
        "fastq.normalize_abundance" => vec!["transform", "filter"],
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
