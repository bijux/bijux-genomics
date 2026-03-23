use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    correct::{CorrectionEngine, FastqCorrectParams, QualityEncoding, CORRECT_SCHEMA_VERSION},
    PairedMode,
};
use bijux_dna_domain_fastq::{
    CorrectErrorsReportV1, CORRECT_ERRORS_REPORT_SCHEMA_VERSION, STAGE_CORRECT_ERRORS,
};
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_CORRECT_ERRORS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub type CorrectPlanOptions = crate::CorrectErrorsStageParams;
const DEFAULT_CORRECT_ERRORS_THREADS: u32 = 1;

pub fn normalize_correct_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a correct plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_correct(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_correct_with_options(tool, r1, r2, out_dir, &CorrectPlanOptions::default())
}

/// Build a correct plan with governed stage options.
///
/// # Errors
/// Returns an error if the tool is unsupported or the requested explicit options are not
/// supported by the current backend adapter.
pub fn plan_correct_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &CorrectPlanOptions,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_correct_tool_list(std::slice::from_ref(&tool_id))?;
    validate_correct_options(&tool_id, options)?;
    let effective_threads = options
        .threads
        .unwrap_or(DEFAULT_CORRECT_ERRORS_THREADS)
        .max(1);
    let output_r1 = out_dir.join("reads_r1.fastq.gz");
    let output_r2 = r2.map(|_| out_dir.join("reads_r2.fastq.gz"));
    let report_json = out_dir.join("correct_report.json");
    let correction_engine = correction_engine_for_tool(&tool.tool_id.0)?;
    let effective_params = FastqCorrectParams {
        schema_version: CORRECT_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        threads: effective_threads,
        correction_engine: correction_engine.clone(),
        quality_encoding: options.quality_encoding.clone(),
        kmer_size: options.kmer_size,
        genome_size: options.genome_size,
        max_memory_gb: options.max_memory_gb,
        trusted_kmer_artifact: options.trusted_kmer_artifact.clone(),
        conservative_mode: options.conservative_mode,
    };
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
    if let Some(trusted_kmer_artifact) = options.trusted_kmer_artifact.as_ref() {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("trusted_kmer_artifact"),
            trusted_kmer_artifact.clone(),
            ArtifactRole::Index,
        ));
    }
    let mut outputs = vec![
        ArtifactRef::required(
            ArtifactId::from_static("corrected_reads_r1"),
            output_r1.clone(),
            ArtifactRole::Reads,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("report_json"),
            report_json.clone(),
            ArtifactRole::ReportJson,
        ),
    ];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("corrected_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    let mut resources = tool.resources.clone();
    resources.threads = effective_threads;
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
            template: correct_command_template(
                &tool.tool_id.0,
                r1,
                r2,
                &output_r1,
                output_r2.as_deref(),
                &report_json,
                effective_threads,
                options,
                &correction_engine,
            )?,
        },
        resources,
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "out_dir": out_dir,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report_json,
            "threads": effective_threads,
            "quality_encoding": options.quality_encoding,
            "kmer_size": options.kmer_size,
            "genome_size": options.genome_size,
            "max_memory_gb": options.max_memory_gb,
            "trusted_kmer_artifact": options.trusted_kmer_artifact,
            "conservative_mode": options.conservative_mode,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize correct effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn validate_correct_options(tool_id: &str, options: &CorrectPlanOptions) -> Result<()> {
    if options.quality_encoding != QualityEncoding::Phred33 && tool_id != "bayeshammer" {
        return Err(anyhow!(
            "{tool_id} error-correction planning currently supports only quality_encoding=phred33"
        ));
    }
    if options.kmer_size.is_some() && !matches!(tool_id, "musket" | "lighter") {
        return Err(anyhow!(
            "{tool_id} error-correction planning does not yet map kmer_size into backend execution"
        ));
    }
    if options.genome_size.is_some() && tool_id != "lighter" {
        return Err(anyhow!(
            "{tool_id} error-correction planning does not yet map genome_size into backend execution"
        ));
    }
    if tool_id == "lighter" && options.genome_size.is_none() {
        return Err(anyhow!(
            "lighter error-correction planning requires genome_size to build the governed command"
        ));
    }
    if options.max_memory_gb.is_some() && tool_id != "bayeshammer" {
        return Err(anyhow!(
            "{tool_id} error-correction planning does not yet map max_memory_gb into backend execution"
        ));
    }
    if options.trusted_kmer_artifact.is_some() && tool_id != "lighter" {
        return Err(anyhow!(
            "{tool_id} error-correction planning does not yet map trusted_kmer_artifact into backend execution"
        ));
    }
    if options.conservative_mode {
        return Err(anyhow!(
            "{tool_id} error-correction planning does not yet map conservative_mode into backend execution"
        ));
    }
    Ok(())
}

fn correct_command_template(
    tool_id: &str,
    input_r1: &Path,
    input_r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    options: &CorrectPlanOptions,
    correction_engine: &CorrectionEngine,
) -> Result<Vec<String>> {
    let work_dir = report_json
        .parent()
        .ok_or_else(|| anyhow!("correction report path must have a parent directory"))?
        .join(format!("{tool_id}_work"));
    let mut script = format!(
        "set -euo pipefail\nmkdir -p {}\nnormalize_fastq_output() {{ src=\"$1\"; dest=\"$2\"; case \"$src\" in *.gz) mv -- \"$src\" \"$dest\" ;; *) gzip -c -- \"$src\" > \"$dest\" ;; esac; }}\n",
        shell_quote_path(&work_dir),
    );
    match tool_id {
        "rcorrector" => {
            script.push_str("run_rcorrector.pl");
            script.push_str(&format!(
                " -t {threads} -od {}",
                shell_quote_path(&work_dir)
            ));
            if let Some(input_r2) = input_r2 {
                script.push_str(&format!(
                    " -1 {} -2 {}",
                    shell_quote_path(input_r1),
                    shell_quote_path(input_r2),
                ));
            } else {
                script.push_str(&format!(" -s {}", shell_quote_path(input_r1)));
            }
            script.push('\n');
            script.push_str(&move_corrected_outputs_script(
                &work_dir, output_r1, output_r2, true,
            ));
        }
        "musket" => {
            let kmer_size = options.kmer_size.unwrap_or(21);
            let prefix = work_dir.join("corrected");
            script.push_str(&format!("musket -p {threads} -k {kmer_size}"));
            if let Some(input_r2) = input_r2 {
                script.push_str(&format!(
                    " -omulti {} -inorder {} {}",
                    shell_quote_path(&prefix),
                    shell_quote_path(input_r1),
                    shell_quote_path(input_r2),
                ));
            } else {
                script.push_str(&format!(
                    " -o {} {}",
                    shell_quote_path(&prefix),
                    shell_quote_path(input_r1),
                ));
            }
            script.push('\n');
            if let Some(output_r2) = output_r2 {
                script.push_str(&format!(
                    "normalize_fastq_output {} {}\nnormalize_fastq_output {} {}\n",
                    shell_quote_path(&prefix.with_extension("0")),
                    shell_quote_path(output_r1),
                    shell_quote_path(&prefix.with_extension("1")),
                    shell_quote_path(output_r2),
                ));
            } else {
                script.push_str(&format!(
                    "normalize_fastq_output {} {}\n",
                    shell_quote_path(&prefix),
                    shell_quote_path(output_r1),
                ));
            }
        }
        "lighter" => {
            let kmer_size = options.kmer_size.unwrap_or(21);
            let genome_size = options
                .genome_size
                .ok_or_else(|| anyhow!("lighter requires genome_size"))?;
            script.push_str(&format!(
                "lighter -K {kmer_size} {genome_size} -t {threads} -od {} -r {}",
                shell_quote_path(&work_dir),
                shell_quote_path(input_r1),
            ));
            if let Some(input_r2) = input_r2 {
                script.push_str(&format!(" -r {}", shell_quote_path(input_r2)));
            }
            if let Some(trusted_kmer_artifact) = options.trusted_kmer_artifact.as_ref() {
                script.push_str(&format!(
                    " -loadTrustedKmers {}",
                    shell_quote_path(trusted_kmer_artifact),
                ));
            }
            script.push('\n');
            script.push_str(&move_corrected_outputs_script(
                &work_dir, output_r1, output_r2, false,
            ));
        }
        "bayeshammer" => {
            script.push_str("spades.py --only-error-correction");
            script.push_str(&format!(" --threads {threads}"));
            let phred_offset = match options.quality_encoding {
                QualityEncoding::Phred33 => 33,
                QualityEncoding::Phred64 => 64,
            };
            script.push_str(&format!(" --phred-offset {phred_offset}"));
            if let Some(max_memory_gb) = options.max_memory_gb {
                script.push_str(&format!(" -m {max_memory_gb}"));
            }
            if let Some(input_r2) = input_r2 {
                script.push_str(&format!(
                    " -1 {} -2 {}",
                    shell_quote_path(input_r1),
                    shell_quote_path(input_r2),
                ));
            } else {
                script.push_str(&format!(" -s {}", shell_quote_path(input_r1)));
            }
            script.push_str(&format!(" -o {}\n", shell_quote_path(&work_dir)));
            script.push_str(&move_corrected_outputs_script(
                &work_dir.join("corrected"),
                output_r1,
                output_r2,
                false,
            ));
        }
        _ => return Err(anyhow!("unsupported tool: {tool_id}")),
    }
    script.push_str(&write_correction_report_script(
        tool_id,
        report_json,
        input_r1,
        input_r2,
        output_r1,
        output_r2,
        threads,
        correction_engine,
        options,
    )?);
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn move_corrected_outputs_script(
    search_dir: &Path,
    output_r1: &Path,
    output_r2: Option<&Path>,
    cor_suffix_only: bool,
) -> String {
    let patterns = if cor_suffix_only {
        "\\( -name '*.cor.fq' -o -name '*.cor.fastq' -o -name '*.cor.fq.gz' -o -name '*.cor.fastq.gz' \\)"
    } else {
        "\\( -name '*.cor.fq' -o -name '*.cor.fastq' -o -name '*.cor.fq.gz' -o -name '*.cor.fastq.gz' -o -name '*.fq' -o -name '*.fastq' -o -name '*.fq.gz' -o -name '*.fastq.gz' \\)"
    };
    let expected_count = if output_r2.is_some() { 2 } else { 1 };
    let list_path = search_dir.join("corrected_outputs.list");
    let mut script = format!(
        "find {} -type f {} | LC_ALL=C sort > {}\nactual_outputs=$(wc -l < {} | tr -d '[:space:]')\nif [ \"$actual_outputs\" -ne {} ]; then echo \"expected {} corrected outputs in {} but found $actual_outputs\" >&2; exit 64; fi\nnormalize_fastq_output \"$(sed -n '1p' {})\" {}\n",
        shell_quote_path(search_dir),
        patterns,
        shell_quote_path(&list_path),
        shell_quote_path(&list_path),
        expected_count,
        expected_count,
        shell_quote_path(search_dir),
        shell_quote_path(&list_path),
        shell_quote_path(output_r1),
    );
    if let Some(output_r2) = output_r2 {
        script.push_str(&format!(
            "normalize_fastq_output \"$(sed -n '2p' {})\" {}\n",
            shell_quote_path(&list_path),
            shell_quote_path(output_r2),
        ));
    }
    script
}

fn write_correction_report_script(
    tool_id: &str,
    report_json: &Path,
    input_r1: &Path,
    input_r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    threads: u32,
    correction_engine: &CorrectionEngine,
    options: &CorrectPlanOptions,
) -> Result<String> {
    let report_payload = CorrectErrorsReportV1 {
        schema_version: CORRECT_ERRORS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.as_str().to_string(),
        stage_id: STAGE_ID.as_str().to_string(),
        tool_id: tool_id.to_string(),
        paired_mode: PairedMode::from_has_r2(input_r2.is_some()),
        threads,
        correction_engine: correction_engine.clone(),
        quality_encoding: options.quality_encoding.clone(),
        kmer_size: options.kmer_size,
        genome_size: options.genome_size,
        max_memory_gb: options.max_memory_gb,
        trusted_kmer_artifact: options.trusted_kmer_artifact.clone(),
        conservative_mode: options.conservative_mode,
        input_r1: input_r1.display().to_string(),
        input_r2: input_r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        corrected_reads: None,
        reads_in: None,
        reads_out: None,
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: None,
        mean_q_after: None,
        kmer_fix_rate: None,
        correction_effect: None,
        runtime_s: None,
        memory_mb: None,
        exit_code: None,
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    };
    let report_payload = serde_json::to_string(&report_payload)
        .map_err(|error| anyhow!("serialize correction report: {error}"))?;
    Ok(format!(
        "printf '%s\\n' {} > {}\n",
        shell_quote_str(&report_payload),
        shell_quote_path(report_json),
    ))
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn correction_engine_for_tool(tool_id: &str) -> Result<CorrectionEngine> {
    match tool_id {
        "rcorrector" => Ok(CorrectionEngine::Rcorrector),
        "musket" => Ok(CorrectionEngine::Musket),
        "lighter" => Ok(CorrectionEngine::Lighter),
        "bayeshammer" => Ok(CorrectionEngine::Bayeshammer),
        _ => Err(anyhow!("unsupported tool: {tool_id}")),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::ids::ToolId;
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolConstraints};

    fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::new(tool_id.to_string()),
            tool_version: "fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec![tool_id.to_string(), "{{reads_r1}}".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 2,
            },
        }
    }

    #[test]
    fn plan_correct_uses_typed_default_effective_params() {
        let plan = plan_correct(
            &tool("rcorrector"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
        )
        .expect("default correct plan should build");

        assert_eq!(
            plan.effective_params["correction_engine"],
            serde_json::json!("rcorrector")
        );
        assert_eq!(
            plan.effective_params["quality_encoding"],
            serde_json::json!("phred33")
        );
        assert!(plan.command.template[2].contains(CORRECT_ERRORS_REPORT_SCHEMA_VERSION));
    }

    #[test]
    fn plan_correct_rejects_non_phred33_quality_encoding_for_unsupported_tools() {
        let error = plan_correct_with_options(
            &tool("rcorrector"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            &CorrectPlanOptions {
                quality_encoding: QualityEncoding::Phred64,
                ..CorrectPlanOptions::default()
            },
        )
        .expect_err("unsupported quality encoding must fail");

        assert!(error.to_string().contains("quality_encoding=phred33"));
    }

    #[test]
    fn plan_correct_maps_phred64_for_bayeshammer() {
        let plan = plan_correct_with_options(
            &tool("bayeshammer"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &CorrectPlanOptions {
                threads: Some(7),
                quality_encoding: QualityEncoding::Phred64,
                ..CorrectPlanOptions::default()
            },
        )
        .expect("bayeshammer should accept explicit phred64 encoding");

        assert_eq!(plan.effective_params["threads"], serde_json::json!(7));
        assert_eq!(
            plan.effective_params["quality_encoding"],
            serde_json::json!("phred64")
        );
        assert!(plan.command.template[2].contains("--threads 7"));
        assert!(plan.command.template[2].contains("--phred-offset 64"));
    }

    #[test]
    fn plan_correct_supports_single_end_rcorrector() {
        let plan = plan_correct(
            &tool("rcorrector"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
        )
        .expect("single-end correction plan should build");

        assert_eq!(plan.io.inputs.len(), 1);
        assert_eq!(plan.io.outputs.len(), 2);
        assert_eq!(plan.effective_params["paired_mode"], "single_end");
        let script = &plan.command.template[2];
        assert!(script.contains("run_rcorrector.pl"));
        assert!(script.contains(" -s "));
    }

    #[test]
    fn plan_correct_requires_genome_size_for_lighter() {
        let error = plan_correct_with_options(
            &tool("lighter"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &CorrectPlanOptions::default(),
        )
        .expect_err("lighter must require genome_size");

        assert!(error.to_string().contains("genome_size"));
    }

    #[test]
    fn plan_correct_maps_explicit_kmer_size_for_musket() {
        let plan = plan_correct_with_options(
            &tool("musket"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &CorrectPlanOptions {
                kmer_size: Some(31),
                ..CorrectPlanOptions::default()
            },
        )
        .expect("musket plan should accept explicit kmer size");

        assert_eq!(plan.effective_params["kmer_size"], serde_json::json!(31));
        assert!(plan.command.template[2].contains("musket -p 2 -k 31"));
    }

    #[test]
    fn plan_correct_maps_explicit_memory_limit_for_bayeshammer() {
        let plan = plan_correct_with_options(
            &tool("bayeshammer"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &CorrectPlanOptions {
                max_memory_gb: Some(24),
                ..CorrectPlanOptions::default()
            },
        )
        .expect("bayeshammer plan should accept explicit memory limit");

        assert_eq!(
            plan.effective_params["max_memory_gb"],
            serde_json::json!(24)
        );
        assert!(plan.command.template[2].contains("spades.py --only-error-correction"));
        assert!(plan.command.template[2].contains(" -m 24"));
    }

    #[test]
    fn plan_correct_maps_trusted_kmers_for_lighter() {
        let plan = plan_correct_with_options(
            &tool("lighter"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &CorrectPlanOptions {
                genome_size: Some(3_200_000),
                trusted_kmer_artifact: Some(Path::new("trusted.kmers").to_path_buf()),
                ..CorrectPlanOptions::default()
            },
        )
        .expect("lighter should accept trusted kmer artifacts");

        assert_eq!(
            plan.effective_params["trusted_kmer_artifact"],
            serde_json::json!("trusted.kmers")
        );
        assert!(plan
            .io
            .inputs
            .iter()
            .any(|artifact| artifact.name.as_str() == "trusted_kmer_artifact"
                && artifact.role == ArtifactRole::Index));
        assert!(plan.command.template[2].contains(" -loadTrustedKmers 'trusted.kmers'"));
        assert!(plan.command.template[2].contains("\"trusted_kmer_artifact\":\"trusted.kmers\""));
    }

    #[test]
    fn correction_report_payload_tracks_executable_correction_settings() {
        let plan = plan_correct_with_options(
            &tool("bayeshammer"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &CorrectPlanOptions {
                quality_encoding: QualityEncoding::Phred64,
                max_memory_gb: Some(24),
                ..CorrectPlanOptions::default()
            },
        )
        .expect("bayeshammer plan should carry executable correction settings");

        let script = &plan.command.template[2];
        assert!(script.contains(CORRECT_ERRORS_REPORT_SCHEMA_VERSION));
        assert!(script.contains("\"stage\":\"fastq.correct_errors\""));
        assert!(script.contains("\"threads\":2"));
        assert!(script.contains("\"quality_encoding\":\"phred64\""));
        assert!(script.contains("\"conservative_mode\":false"));
        assert!(script.contains("\"max_memory_gb\":24"));
    }

    #[test]
    fn plan_correct_rejects_trusted_kmers_for_unsupported_tools() {
        let error = plan_correct_with_options(
            &tool("musket"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &CorrectPlanOptions {
                trusted_kmer_artifact: Some(Path::new("trusted.kmers").to_path_buf()),
                ..CorrectPlanOptions::default()
            },
        )
        .expect_err("unsupported trusted kmer mappings must fail");

        assert!(error.to_string().contains("trusted_kmer_artifact"));
    }
}
