#![allow(clippy::too_many_arguments)]

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    defaults::screen_defaults,
    screen::{
        ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyReportFormat,
        SCREEN_TAXONOMY_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::{
    ScreenTaxonomyReportV1, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION, STAGE_SCREEN_TAXONOMY,
};
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_SCREEN_TAXONOMY;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScreenPlanOptions {
    pub database_root: Option<std::path::PathBuf>,
    pub threads: Option<u32>,
}

/// # Errors
/// Returns an error if any requested taxonomy-screening tool is not admitted for
/// `fastq.screen_taxonomy`.
pub fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a screen plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_screen(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_screen_with_options(tool, r1, r2, out_dir, &ScreenPlanOptions::default())
}

/// Build a screen plan with explicit governed stage options.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_screen_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &ScreenPlanOptions,
) -> Result<StagePlanV1> {
    let mut effective_params = screen_defaults(r2.is_some());
    let (classifier, report_format, assignment_format) = classifier_contract(&tool.tool_id.0)?;
    effective_params.classifier = classifier;
    effective_params.report_format = report_format;
    effective_params.assignment_format = assignment_format;
    if let Some(database_root) = options.database_root.as_ref() {
        let resolved_database_root = if database_root.is_absolute() {
            std::fs::canonicalize(database_root).unwrap_or_else(|_| database_root.clone())
        } else {
            database_root.clone()
        };
        effective_params.contaminant_db = Some(resolved_database_root.display().to_string());
    }
    if let Some(threads) = options.threads {
        effective_params.threads = threads.max(1);
    }
    plan_screen_with_effective_params(tool, r1, r2, out_dir, &effective_params)
}

/// Build a screen plan with explicit governed effective params.
///
/// # Errors
/// Returns an error if the tool is unsupported or the effective params do not
/// match the selected classifier contract.
pub fn plan_screen_with_effective_params(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    effective_params: &ScreenEffectiveParams,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_screen_tool_list(std::slice::from_ref(&tool_id))?;
    let outputs = taxonomy_outputs(&tool.tool_id.0, out_dir, r2.is_some())?;
    let effective_params =
        normalized_screen_effective_params(tool.tool_id.as_str(), r2.is_some(), effective_params)?;
    let inputs = screen_inputs(r1, r2, &effective_params);
    let io_outputs = screen_outputs(&outputs);
    let mut resources = tool.resources.clone();
    resources.threads = effective_params.threads;
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
        command: CommandSpecV1 {
            template: screen_command_template(
                tool,
                r1,
                r2,
                &outputs.report,
                &outputs.assignments,
                outputs.unclassified_output_pattern.as_deref(),
                &effective_params,
            )?,
        },
        resources,
        io: StageIO { inputs, outputs: io_outputs },
        out_dir: out_dir.to_path_buf(),
        params: screen_plan_params(&tool.tool_id.0, r1, r2, out_dir, &outputs, &effective_params),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize screen effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn normalized_screen_effective_params(
    tool_id: &str,
    paired: bool,
    effective_params: &ScreenEffectiveParams,
) -> Result<ScreenEffectiveParams> {
    let (classifier, report_format, assignment_format) = classifier_contract(tool_id)?;
    let paired_mode = if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
    let mut effective_params = effective_params.clone();
    if effective_params.schema_version.trim().is_empty() {
        effective_params.schema_version = SCREEN_TAXONOMY_SCHEMA_VERSION.to_string();
    }
    effective_params.threads = effective_params.threads.max(1);
    ensure_screen_contract(
        tool_id,
        &effective_params,
        paired_mode,
        &classifier,
        &report_format,
        &assignment_format,
    )?;
    Ok(effective_params)
}

fn ensure_screen_contract(
    tool_id: &str,
    effective_params: &ScreenEffectiveParams,
    paired_mode: PairedMode,
    classifier: &TaxonomyClassifier,
    report_format: &TaxonomyReportFormat,
    assignment_format: &TaxonomyAssignmentFormat,
) -> Result<()> {
    if effective_params.paired_mode != paired_mode {
        return Err(anyhow!(
            "screen taxonomy paired_mode {:?} does not match input layout {:?}",
            effective_params.paired_mode,
            paired_mode
        ));
    }
    if effective_params.classifier != *classifier {
        return Err(anyhow!(
            "screen taxonomy classifier {:?} is incompatible with tool {}",
            effective_params.classifier,
            tool_id
        ));
    }
    if effective_params.report_format != *report_format {
        return Err(anyhow!(
            "screen taxonomy report format {:?} is incompatible with tool {}",
            effective_params.report_format,
            tool_id
        ));
    }
    if effective_params.assignment_format != *assignment_format {
        return Err(anyhow!(
            "screen taxonomy assignment format {:?} is incompatible with tool {}",
            effective_params.assignment_format,
            tool_id
        ));
    }
    Ok(())
}

fn screen_inputs(
    r1: &Path,
    r2: Option<&Path>,
    effective_params: &ScreenEffectiveParams,
) -> Vec<ArtifactRef> {
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(database_root) = effective_params.contaminant_db.as_ref() {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("taxonomy_database_root"),
            std::path::PathBuf::from(database_root),
            ArtifactRole::Reference,
        ));
    }
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    inputs
}

fn screen_outputs(outputs: &TaxonomyOutputs) -> Vec<ArtifactRef> {
    let mut artifacts = vec![
        ArtifactRef::required(
            ArtifactId::from_static("screen_report_tsv"),
            outputs.report.clone(),
            ArtifactRole::SummaryTsv,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("classification_report_json"),
            outputs.assignments.clone(),
            ArtifactRole::ReportJson,
        ),
    ];
    if let Some(path) = outputs.unclassified_r1.as_ref() {
        artifacts.push(ArtifactRef::optional(
            ArtifactId::from_static("unclassified_reads_r1"),
            path.clone(),
            ArtifactRole::Reads,
        ));
    }
    if let Some(path) = outputs.unclassified_r2.as_ref() {
        artifacts.push(ArtifactRef::optional(
            ArtifactId::from_static("unclassified_reads_r2"),
            path.clone(),
            ArtifactRole::Reads,
        ));
    }
    artifacts
}

fn screen_plan_params(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    outputs: &TaxonomyOutputs,
    effective_params: &ScreenEffectiveParams,
) -> serde_json::Value {
    serde_json::json!({
        "tool": tool_id,
        "input_r1": r1,
        "input_r2": r2,
        "out_dir": out_dir,
        "report": outputs.report,
        "assignments": outputs.assignments,
        "unclassified_reads_r1": outputs.unclassified_r1,
        "unclassified_reads_r2": outputs.unclassified_r2,
        "threads": effective_params.threads,
        "contaminant_db": effective_params.contaminant_db,
        "database_root": effective_params.contaminant_db,
        "database_catalog_id": effective_params.database_catalog_id,
        "database_artifact_id": effective_params.database_artifact_id,
        "database_build_id": effective_params.database_build_id,
        "database_digest": effective_params.database_digest,
        "database_namespace": effective_params.database_namespace,
        "database_scope": effective_params.database_scope,
        "classifier": effective_params.classifier,
        "report_format": effective_params.report_format,
        "assignment_format": effective_params.assignment_format,
        "minimum_confidence": effective_params.minimum_confidence,
        "emit_unclassified": effective_params.emit_unclassified,
    })
}

fn screen_command_template(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    report_path: &Path,
    classification_report_path: &Path,
    unclassified_output_pattern: Option<&Path>,
    effective_params: &ScreenEffectiveParams,
) -> Result<Vec<String>> {
    let governed_report = ScreenTaxonomyReportV1 {
        schema_version: SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.as_str().to_string(),
        stage_id: STAGE_ID.as_str().to_string(),
        tool_id: tool.tool_id.to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        classifier: effective_params.classifier.clone(),
        report_format: effective_params.report_format.clone(),
        assignment_format: effective_params.assignment_format.clone(),
        database_catalog_id: effective_params.database_catalog_id.clone(),
        database_artifact_id: effective_params.database_artifact_id.clone(),
        database_build_id: effective_params.database_build_id.clone(),
        database_digest: effective_params.database_digest.clone(),
        database_namespace: effective_params.database_namespace.clone(),
        database_scope: effective_params.database_scope.clone(),
        minimum_confidence: effective_params.minimum_confidence,
        emit_unclassified: effective_params.emit_unclassified,
        interpretation_boundary: effective_params.interpretation_boundary.clone(),
        truth_conditions: effective_params.truth_conditions.clone(),
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        screen_report_tsv: report_path.display().to_string(),
        classification_report_json: classification_report_path.display().to_string(),
        unclassified_reads_r1: outputs_unclassified_r1(unclassified_output_pattern, r2.is_some())
            .map(|path| path.display().to_string()),
        unclassified_reads_r2: outputs_unclassified_r2(unclassified_output_pattern, r2.is_some())
            .map(|path| path.display().to_string()),
        reads_in: None,
        reads_out: None,
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        contamination_rate: None,
        classified_fraction: None,
        unclassified_fraction: None,
        summary_entries: Vec::new(),
        top_taxa: Vec::new(),
        runtime_s: None,
        memory_mb: None,
    };
    let command_body = if let Some(database_root) = effective_params.contaminant_db.as_deref() {
        let native_assignments_path = classification_report_path.with_extension("native.tsv");
        let native_report_path = report_path.with_extension("native.tsv");
        screen_native_command(
            &tool.tool_id.0,
            Path::new(database_root),
            r1,
            r2,
            &native_report_path,
            &native_assignments_path,
            report_path,
            unclassified_output_pattern,
            effective_params.threads,
        )?
    } else {
        let rendered = crate::tool_adapters::template_render::render_command_template(
            &tool.command.template,
            &[
                ("reads", Some(r1.display().to_string())),
                ("reads_r1", Some(r1.display().to_string())),
                ("reads_r2", r2.map(|path| path.display().to_string())),
                ("screen_report_tsv", Some(report_path.display().to_string())),
                (
                    "classification_report_json",
                    Some(classification_report_path.display().to_string()),
                ),
                ("threads", Some(effective_params.threads.to_string())),
            ],
        )?;
        let command = rendered.into_iter().filter(|token| !token.is_empty()).collect::<Vec<_>>();
        if command.is_empty() {
            return Err(anyhow!("screen taxonomy command template resolved to an empty command"));
        }
        shell_join(&command)
    };
    let script = format!(
        "set -eu\n{}\nprintf '%s\\n' {} > {}\n",
        command_body,
        shell_quote_str(
            &serde_json::to_string(&governed_report)
                .map_err(|error| anyhow!("serialize governed taxonomy screen report: {error}"))?,
        ),
        shell_quote_path(classification_report_path),
    );
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn screen_native_command(
    tool_id: &str,
    database_root: &Path,
    r1: &Path,
    r2: Option<&Path>,
    native_report_path: &Path,
    native_assignments_path: &Path,
    normalized_report_path: &Path,
    unclassified_output_pattern: Option<&Path>,
    threads: u32,
) -> Result<String> {
    let script = match tool_id {
        "kraken2" => kraken2_screen_script(
            database_root,
            r1,
            r2,
            native_report_path,
            native_assignments_path,
            normalized_report_path,
            unclassified_output_pattern,
            threads,
        ),
        "krakenuniq" => krakenuniq_screen_script(
            database_root,
            r1,
            r2,
            native_report_path,
            native_assignments_path,
            normalized_report_path,
            threads,
        ),
        "centrifuge" => centrifuge_screen_script(
            database_root,
            r1,
            r2,
            native_report_path,
            native_assignments_path,
            normalized_report_path,
            threads,
        ),
        "kaiju" => kaiju_screen_script(
            database_root,
            r1,
            r2,
            native_report_path,
            native_assignments_path,
            normalized_report_path,
            threads,
        ),
        _ => return Err(anyhow!("unsupported taxonomy screening tool: {tool_id}")),
    };
    Ok(script)
}

fn kraken2_screen_script(
    database_root: &Path,
    r1: &Path,
    r2: Option<&Path>,
    native_report_path: &Path,
    native_assignments_path: &Path,
    normalized_report_path: &Path,
    unclassified_output_pattern: Option<&Path>,
    threads: u32,
) -> String {
    let reads_clause = if let Some(r2) = r2 {
        format!("--paired {} {}", shell_quote_path(r1), shell_quote_path(r2),)
    } else {
        shell_quote_path(r1)
    };
    let unclassified_clause = unclassified_output_pattern
        .map_or_else(String::new, |path| format!(" --unclassified-out {}", shell_quote_path(path)));
    format!(
        "mkdir -p {db}\n\
         kraken2 --db {db} --threads {threads} --report {native_report} --output {native_assignments}{unclassified_clause} {reads}\n\
         awk -F '\\t' 'NF >= 6 {{ label=$6; sub(/^[[:space:]]+/, \"\", label); pct=$1; sub(/%$/, \"\", pct); printf \"%s\\t0\\t%s%%\\n\", label, pct }}' {native_report} > {normalized_report}\n",
        db = shell_quote_path(&database_root.join("kraken2")),
        threads = threads,
        native_report = shell_quote_path(native_report_path),
        native_assignments = shell_quote_path(native_assignments_path),
        unclassified_clause = unclassified_clause,
        reads = reads_clause,
        normalized_report = shell_quote_path(normalized_report_path),
    )
}

fn krakenuniq_screen_script(
    database_root: &Path,
    r1: &Path,
    r2: Option<&Path>,
    native_report_path: &Path,
    native_assignments_path: &Path,
    normalized_report_path: &Path,
    threads: u32,
) -> String {
    let reads_clause = if let Some(r2) = r2 {
        format!("--paired {} {}", shell_quote_path(r1), shell_quote_path(r2),)
    } else {
        shell_quote_path(r1)
    };
    format!(
        "mkdir -p {db}\n\
         krakenuniq --db {db} --threads {threads} --fastq-input --report-file {native_report} --output {native_assignments} {reads}\n\
         awk -F '\\t' 'NF >= 9 {{ if ($1 == \"%\") next; label=$9; sub(/^[[:space:]]+/, \"\", label); pct=$1; sub(/%$/, \"\", pct); printf \"%s\\t0\\t%s%%\\n\", label, pct }}' {native_report} > {normalized_report}\n",
        db = shell_quote_path(&database_root.join("krakenuniq")),
        threads = threads,
        native_report = shell_quote_path(native_report_path),
        native_assignments = shell_quote_path(native_assignments_path),
        reads = reads_clause,
        normalized_report = shell_quote_path(normalized_report_path),
    )
}

fn centrifuge_screen_script(
    database_root: &Path,
    r1: &Path,
    r2: Option<&Path>,
    native_report_path: &Path,
    native_assignments_path: &Path,
    normalized_report_path: &Path,
    threads: u32,
) -> String {
    let mut prelude = String::new();
    let mut cleanup = Vec::new();
    let r1_input = if is_gzip_input(r1) {
        let inflated = native_assignments_path.with_file_name("centrifuge.reads_r1.fastq");
        prelude
            .write_fmt(format_args!(
                "gzip -cd {} > {}\n",
                shell_quote_path(r1),
                shell_quote_path(&inflated),
            ))
            .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
        cleanup.push(shell_quote_path(&inflated));
        inflated
    } else {
        r1.to_path_buf()
    };
    let r2_input = if let Some(r2) = r2 {
        if is_gzip_input(r2) {
            let inflated = native_assignments_path.with_file_name("centrifuge.reads_r2.fastq");
            prelude
                .write_fmt(format_args!(
                    "gzip -cd {} > {}\n",
                    shell_quote_path(r2),
                    shell_quote_path(&inflated),
                ))
                .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
            cleanup.push(shell_quote_path(&inflated));
            Some(inflated)
        } else {
            Some(r2.to_path_buf())
        }
    } else {
        None
    };
    let reads_clause = if r2.is_some() {
        let Some(r2_input) = r2_input else {
            unreachable!("paired centrifuge input must be present");
        };
        format!("-1 {} -2 {}", shell_quote_path(&r1_input), shell_quote_path(&r2_input),)
    } else {
        format!("-U {}", shell_quote_path(&r1_input))
    };
    let cleanup_clause =
        if cleanup.is_empty() { String::new() } else { format!("rm -f {}\n", cleanup.join(" ")) };
    format!(
        "mkdir -p {db_root}\n\
         {prelude}\
         centrifuge -x {db_prefix} -q {reads} -S {native_assignments} --report-file {native_report} -p {threads}\n\
         awk -F '\\t' 'NF >= 7 {{ if ($1 == \"name\") next; pct=$7; sub(/%$/, \"\", pct); printf \"%s\\t0\\t%s%%\\n\", $1, pct }}' {native_report} > {normalized_report}\n\
         {cleanup_clause}",
        db_root = shell_quote_path(&database_root.join("centrifuge")),
        prelude = prelude,
        db_prefix = shell_quote_path(&database_root.join("centrifuge").join("reference")),
        reads = reads_clause,
        native_assignments = shell_quote_path(native_assignments_path),
        native_report = shell_quote_path(native_report_path),
        threads = threads,
        normalized_report = shell_quote_path(normalized_report_path),
        cleanup_clause = cleanup_clause,
    )
}

fn kaiju_screen_script(
    database_root: &Path,
    r1: &Path,
    r2: Option<&Path>,
    native_report_path: &Path,
    native_assignments_path: &Path,
    normalized_report_path: &Path,
    threads: u32,
) -> String {
    let reads_clause = if let Some(r2) = r2 {
        format!("-i {} -j {}", shell_quote_path(r1), shell_quote_path(r2),)
    } else {
        format!("-i {}", shell_quote_path(r1))
    };
    let kaiju_root = database_root.join("kaiju");
    let taxonomy_root = database_root.join("taxonomy");
    format!(
        "mkdir -p {kaiju_root}\n\
         kaiju -t {nodes} -f {db_fmi} {reads} -o {native_assignments} -z {threads}\n\
         kaiju2table -t {nodes} -n {names} -r genus -o {native_report} {native_assignments}\n\
         awk -F '\\t' 'NF >= 6 {{ if ($1 == \"percent\") next; label=$6; sub(/^[[:space:]]+/, \"\", label); pct=$1; sub(/%$/, \"\", pct); printf \"%s\\t0\\t%s%%\\n\", label, pct }}' {native_report} > {normalized_report}\n",
        kaiju_root = shell_quote_path(&kaiju_root),
        nodes = shell_quote_path(&taxonomy_root.join("nodes.dmp")),
        names = shell_quote_path(&taxonomy_root.join("names.dmp")),
        db_fmi = shell_quote_path(&kaiju_root.join("kaiju_db.fmi")),
        reads = reads_clause,
        native_assignments = shell_quote_path(native_assignments_path),
        native_report = shell_quote_path(native_report_path),
        threads = threads,
        normalized_report = shell_quote_path(normalized_report_path),
    )
}

fn normalize_tools_with_allowlist(
    tools: &[String],
    allowlist: &[bijux_dna_core::ids::ToolId],
) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}

struct TaxonomyOutputs {
    report: std::path::PathBuf,
    assignments: std::path::PathBuf,
    unclassified_r1: Option<std::path::PathBuf>,
    unclassified_r2: Option<std::path::PathBuf>,
    unclassified_output_pattern: Option<std::path::PathBuf>,
}

fn taxonomy_outputs(tool_id: &str, out_dir: &Path, paired: bool) -> Result<TaxonomyOutputs> {
    let outputs = match tool_id {
        "kraken2" => {
            let unclassified_output_pattern = kraken2_unclassified_output_pattern(out_dir, paired);
            TaxonomyOutputs {
                report: out_dir.join("kraken2.report.tsv"),
                assignments: out_dir.join("kraken2.classifications.json"),
                unclassified_r1: outputs_unclassified_r1(
                    Some(unclassified_output_pattern.as_path()),
                    paired,
                ),
                unclassified_r2: outputs_unclassified_r2(
                    Some(unclassified_output_pattern.as_path()),
                    paired,
                ),
                unclassified_output_pattern: Some(unclassified_output_pattern),
            }
        }
        "krakenuniq" => TaxonomyOutputs {
            report: out_dir.join("krakenuniq.report.tsv"),
            assignments: out_dir.join("krakenuniq.classifications.json"),
            unclassified_r1: None,
            unclassified_r2: None,
            unclassified_output_pattern: None,
        },
        "centrifuge" => TaxonomyOutputs {
            report: out_dir.join("centrifuge.report.tsv"),
            assignments: out_dir.join("centrifuge.classifications.json"),
            unclassified_r1: None,
            unclassified_r2: None,
            unclassified_output_pattern: None,
        },
        "kaiju" => TaxonomyOutputs {
            report: out_dir.join("kaiju.summary.tsv"),
            assignments: out_dir.join("kaiju.classifications.json"),
            unclassified_r1: None,
            unclassified_r2: None,
            unclassified_output_pattern: None,
        },
        _ => return Err(anyhow!("unsupported taxonomy screening tool: {tool_id}")),
    };
    Ok(outputs)
}

fn kraken2_unclassified_output_pattern(out_dir: &Path, paired: bool) -> std::path::PathBuf {
    if paired {
        out_dir.join("kraken2.unclassified_reads_#.fastq")
    } else {
        out_dir.join("kraken2.unclassified_reads.fastq")
    }
}

fn outputs_unclassified_r1(
    unclassified_output_pattern: Option<&Path>,
    paired: bool,
) -> Option<std::path::PathBuf> {
    let path = unclassified_output_pattern?;
    if paired {
        Some(path.with_file_name("kraken2.unclassified_reads_1.fastq"))
    } else {
        Some(path.to_path_buf())
    }
}

fn outputs_unclassified_r2(
    unclassified_output_pattern: Option<&Path>,
    paired: bool,
) -> Option<std::path::PathBuf> {
    if paired {
        Some(unclassified_output_pattern?.with_file_name("kraken2.unclassified_reads_2.fastq"))
    } else {
        None
    }
}

fn classifier_contract(
    tool_id: &str,
) -> Result<(TaxonomyClassifier, TaxonomyReportFormat, TaxonomyAssignmentFormat)> {
    let contract = match tool_id {
        "kraken2" => (
            TaxonomyClassifier::Kraken2,
            TaxonomyReportFormat::KrakenReport,
            TaxonomyAssignmentFormat::KrakenAssignments,
        ),
        "krakenuniq" => (
            TaxonomyClassifier::KrakenUniq,
            TaxonomyReportFormat::KrakenUniqReport,
            TaxonomyAssignmentFormat::KrakenUniqAssignments,
        ),
        "centrifuge" => (
            TaxonomyClassifier::Centrifuge,
            TaxonomyReportFormat::CentrifugeReport,
            TaxonomyAssignmentFormat::CentrifugeAssignments,
        ),
        "kaiju" => (
            TaxonomyClassifier::Kaiju,
            TaxonomyReportFormat::KaijuSummary,
            TaxonomyAssignmentFormat::KaijuAssignments,
        ),
        _ => return Err(anyhow!("unsupported taxonomy screening tool: {tool_id}")),
    };
    Ok(contract)
}

fn shell_join(command: &[String]) -> String {
    command.iter().map(|part| shell_quote_str(part)).collect::<Vec<_>>().join(" ")
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn is_gzip_input(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("gz"))
}

#[cfg(test)]
mod tests {
    use super::{
        plan_screen_with_effective_params, plan_screen_with_options, ScreenPlanOptions, STAGE_ID,
    };
    use anyhow::Result;
    use bijux_dna_core::prelude::{
        ArtifactRole, CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1,
        ToolId, ToolVersion,
    };
    use bijux_dna_domain_fastq::params::{
        defaults::screen_defaults,
        screen::{TaxonomyClassifier, TaxonomyInterpretationBoundary, TaxonomyTruthCondition},
    };
    use std::path::Path;

    fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id),
            tool_version: ToolVersion::from("1.0.0"),
            image: ContainerImageRefV1 {
                image: format!("ghcr.io/bijux/{tool_id}:latest"),
                digest: Some("sha256:test".to_string()),
            },
            command: CommandSpecV1 { template: vec![tool_id.to_string()] },
            resources: ToolConstraints {
                runtime: "local".to_string(),
                mem_gb: 4,
                tmp_gb: 1,
                threads: 4,
            },
        }
    }

    #[test]
    fn screen_plan_thread_override_updates_resources_and_effective_params() -> Result<()> {
        let plan = plan_screen_with_options(
            &tool("kraken2"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("out"),
            &ScreenPlanOptions {
                database_root: Some(std::path::PathBuf::from("taxonomy_db")),
                threads: Some(12),
            },
        )?;
        assert_eq!(plan.stage_id, STAGE_ID);
        assert_eq!(plan.resources.threads, 12);
        assert_eq!(plan.params["threads"], serde_json::json!(12));
        assert_eq!(plan.effective_params["threads"], serde_json::json!(12));
        assert_eq!(plan.command.template[0], "sh");
        assert_eq!(plan.command.template[1], "-lc");
        assert!(plan.command.template[2].contains("kraken2"));
        assert!(plan.command.template[2].contains("taxonomy_db/kraken2"));
        assert!(plan.command.template[2].contains("out/kraken2.report.tsv"));
        assert!(plan.command.template[2].contains("out/kraken2.classifications.json"));
        assert!(plan.command.template[2]
            .contains("\"schema_version\":\"bijux.fastq.screen_taxonomy.report.v2\""));
        assert!(plan.command.template[2].contains("\"tool_id\":\"kraken2\""));
        assert!(plan.command.template[2].contains("\"interpretation_boundary\":\"screening_only\""));
        assert_eq!(plan.io.inputs.len(), 2);
        assert_eq!(plan.io.inputs[0].role, ArtifactRole::Reads);
        assert_eq!(plan.io.inputs[1].role, ArtifactRole::Reference);
        assert!(plan
            .command
            .template
            .iter()
            .all(|part| !part.contains("{{") && !part.contains("}}")));
        assert_eq!(plan.io.outputs[1].role, ArtifactRole::ReportJson);
        Ok(())
    }

    #[test]
    fn screen_plan_rejects_classifier_mismatch_from_effective_params() {
        let mut effective_params = screen_defaults(false);
        effective_params.classifier = TaxonomyClassifier::Kaiju;

        let error = plan_screen_with_effective_params(
            &tool("kraken2"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("out"),
            &effective_params,
        )
        .expect_err("mismatched classifier must fail");

        assert!(error.to_string().contains("classifier"));
    }

    #[test]
    fn screen_plan_threads_truth_boundaries_into_the_governed_report() -> Result<()> {
        let mut effective_params = screen_defaults(false);
        effective_params.interpretation_boundary =
            TaxonomyInterpretationBoundary::DefinitiveWithGovernedTruth;
        effective_params.truth_conditions = vec![
            TaxonomyTruthCondition::LockedReferenceDatabase,
            TaxonomyTruthCondition::OrthogonalValidationRequired,
        ];

        let plan = plan_screen_with_effective_params(
            &tool("kraken2"),
            Path::new("reads_R1.fastq.gz"),
            None,
            Path::new("out"),
            &effective_params,
        )?;

        assert!(plan.command.template[2]
            .contains("\"interpretation_boundary\":\"definitive_with_governed_truth\""));
        assert!(plan.command.template[2].contains("locked_reference_database"));
        assert!(plan.command.template[2].contains("orthogonal_validation_required"));
        Ok(())
    }

    #[test]
    fn krakenuniq_screen_plan_skips_native_header_and_uses_tax_name_column() -> Result<()> {
        let plan = plan_screen_with_options(
            &tool("krakenuniq"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            &ScreenPlanOptions {
                database_root: Some(std::path::PathBuf::from("taxonomy_db")),
                threads: Some(8),
            },
        )?;

        assert!(plan.command.template[2].contains("krakenuniq --db 'taxonomy_db/krakenuniq'"));
        assert!(plan.command.template[2].contains("NF >= 9"));
        assert!(plan.command.template[2].contains("if ($1 == \"%\") next"));
        assert!(plan.command.template[2].contains("label=$9"));
        Ok(())
    }
}
