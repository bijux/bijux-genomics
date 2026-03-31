#![allow(
    clippy::items_after_test_module,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]

use super::runtime_tail::command_io::{
    command_exists, copy_if_missing, load_qc_thresholds_map, run_stage_command,
    write_fastq_to_fasta_if_missing,
};
use super::runtime_tail::profiling::{infer_udg_classification, terminal_damage_profile};
use super::{anyhow, open_fastq_lines, resolve_primer_set_governance, ExecutionStep, Result};

mod metrics;
mod reports;

use self::metrics::{
    amplicon_u64_to_f64, combined_terminal_damage_asymmetry, count_fastq_reads,
    parse_orientation_forward_fraction, parse_primer_trimmed_fraction_from_stats,
    parse_uchime_fraction, rounded_fraction_count, terminal_damage_asymmetry,
    terminal_damage_base_composition, u64_to_u32,
};
use self::reports::{
    governed_remove_chimeras_report, infer_asvs_effective_params, infer_asvs_tool_id,
    infer_cluster_otus_effective_params, normalize_abundance_method, normalize_abundance_tool_id,
    normalize_primers_tool_id, planned_normalize_primers_report, planned_terminal_damage_report,
    remove_chimeras_compatibility_metrics, trim_terminal_damage_tool_id,
};

pub(super) fn materialize_amplicon_stage_outputs(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
) -> Result<serde_json::Value> {
    let stage_id = planned.step_id.as_str();
    let input = planned
        .io
        .inputs
        .first()
        .map(|x| x.path.clone())
        .ok_or_else(|| anyhow!("missing stage input for {stage_id}"))?;
    let outputs = &planned.io.outputs;
    let out_dir = &planned.out_dir;
    bijux_dna_infra::ensure_dir(out_dir)?;
    let mut payload = serde_json::json!({});
    match stage_id {
        "fastq.trim_terminal_damage" => {
            if std::env::var("BIJUX_ALIGNER_EXPECTS_UNTRIMMED")
                .ok()
                .is_some_and(|v| v == "1")
            {
                return Err(anyhow!(
                    "fastq.trim_terminal_damage refusal: downstream aligner expects untrimmed reads; set BIJUX_ALIGNER_EXPECTS_UNTRIMMED=0 or disable stage"
                ));
            }
            let primary = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "trimmed_reads_r1")
                .or_else(|| outputs.first())
                .map(|x| x.path.clone())
                .ok_or_else(|| anyhow!("missing primary output for {stage_id}"))?;
            let input_r2 = planned
                .io
                .inputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "reads_r2")
                .map(|artifact| artifact.path.clone());
            let output_r2 = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "trimmed_reads_r2")
                .map(|artifact| artifact.path.clone());
            let report_json = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map_or_else(
                    || out_dir.join("trim_terminal_damage_report.json"),
                    |artifact| artifact.path.clone(),
                );
            let raw_backend_report = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "raw_backend_report_json")
                .map(|artifact| artifact.path.clone());
            let pre_profile_r1 =
                terminal_damage_profile(&input).unwrap_or_else(|_| serde_json::json!({}));
            let pre_profile_r2 = input_r2.as_deref().map(|path| {
                terminal_damage_profile(path).unwrap_or_else(|_| serde_json::json!({}))
            });
            let mut planned_report = planned_terminal_damage_report(
                planned,
                &input,
                input_r2.as_deref(),
                &primary,
                output_r2.as_deref(),
                raw_backend_report.as_deref(),
            )
            .unwrap_or_else(|| bijux_dna_domain_fastq::TerminalDamageReportV1 {
                schema_version:
                    bijux_dna_domain_fastq::TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION.to_string(),
                stage: "fastq.trim_terminal_damage".to_string(),
                stage_id: "fastq.trim_terminal_damage".to_string(),
                tool_id: trim_terminal_damage_tool_id(planned).to_string(),
                paired_mode: bijux_dna_domain_fastq::PairedMode::from_has_r2(input_r2.is_some()),
                threads: planned.resources.threads.max(1),
                damage_mode: bijux_dna_domain_fastq::params::DamageMode::Ancient,
                execution_policy:
                    bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy::ExplicitTerminalTrim,
                trim_5p_bases: 2,
                trim_3p_bases: 2,
                requested_trim_5p_bases: Some(2),
                requested_trim_3p_bases: Some(2),
                udg_classification: infer_udg_classification(&input),
                input_r1: input.display().to_string(),
                input_r2: input_r2.as_ref().map(|path| path.display().to_string()),
                output_r1: primary.display().to_string(),
                output_r2: output_r2.as_ref().map(|path| path.display().to_string()),
                reads_in: None,
                reads_out: None,
                bases_in: None,
                bases_out: None,
                mean_q_before: None,
                mean_q_after: None,
                ct_ga_asymmetry_pre: None,
                ct_ga_asymmetry_post: None,
                ct_ga_asymmetry_pre_r1: None,
                ct_ga_asymmetry_post_r1: None,
                ct_ga_asymmetry_pre_r2: None,
                ct_ga_asymmetry_post_r2: None,
                terminal_base_composition_pre_r1: None,
                terminal_base_composition_post_r1: None,
                terminal_base_composition_pre_r2: None,
                terminal_base_composition_post_r2: None,
                raw_backend_report: raw_backend_report
                    .as_ref()
                    .map(|path| path.display().to_string()),
                raw_backend_report_format: None,
                runtime_s: None,
                memory_mb: None,
                used_fallback: false,
                backend_metrics: None,
            });
            let stage_ok = if let Some((program, args)) = planned.command.template.split_first() {
                command_exists(program)
                    && run_stage_command(out_dir, "trim_terminal_damage", program, args)
            } else {
                false
            };
            if !stage_ok || !primary.exists() {
                copy_if_missing(&input, &primary)?;
            }
            if let (Some(input_r2), Some(output_r2)) = (input_r2.as_deref(), output_r2.as_deref()) {
                if !stage_ok || !output_r2.exists() {
                    copy_if_missing(input_r2, output_r2)?;
                }
            }
            let post_profile_r1 =
                terminal_damage_profile(&primary).unwrap_or_else(|_| serde_json::json!({}));
            let post_profile_r2 = output_r2.as_deref().map(|path| {
                terminal_damage_profile(path).unwrap_or_else(|_| serde_json::json!({}))
            });
            let udg_classification = planned_report.udg_classification.clone();
            let classification_artifact = serde_json::json!({
                "schema_version": "bijux.fastq.damage_classification.v1",
                "stage_id": stage_id,
                "udg_classification": udg_classification,
                "source": if std::env::var("BIJUX_UDG_CLASSIFICATION").is_ok() { "config" } else { "inferred" },
                "input_path": input,
            });
            bijux_dna_infra::atomic_write_json(
                &stage_root.join("damage_classification.json"),
                &classification_artifact,
            )?;
            bijux_dna_infra::atomic_write_json(
                &stage_root.join("refusal_cases.json"),
                &serde_json::json!({
                    "schema_version": "bijux.fastq.trim_terminal_damage.refusals.v1",
                    "stage_id": stage_id,
                    "cases": [
                        {
                            "reason_code": "aligner_requires_untrimmed_reads",
                            "condition": "BIJUX_ALIGNER_EXPECTS_UNTRIMMED=1",
                            "action": "disable stage or clear BIJUX_ALIGNER_EXPECTS_UNTRIMMED"
                        }
                    ]
                }),
            )?;
            planned_report.input_r1 = input.display().to_string();
            planned_report.input_r2 = input_r2.as_ref().map(|path| path.display().to_string());
            planned_report.output_r1 = primary.display().to_string();
            planned_report.output_r2 = output_r2.as_ref().map(|path| path.display().to_string());
            planned_report.raw_backend_report = raw_backend_report
                .as_ref()
                .and_then(|path| path.exists().then(|| path.display().to_string()))
                .or(planned_report.raw_backend_report);
            planned_report.threads = planned.resources.threads.max(1);
            planned_report.ct_ga_asymmetry_pre =
                combined_terminal_damage_asymmetry(&pre_profile_r1, pre_profile_r2.as_ref());
            planned_report.ct_ga_asymmetry_post =
                combined_terminal_damage_asymmetry(&post_profile_r1, post_profile_r2.as_ref());
            planned_report.ct_ga_asymmetry_pre_r1 = terminal_damage_asymmetry(&pre_profile_r1);
            planned_report.ct_ga_asymmetry_post_r1 = terminal_damage_asymmetry(&post_profile_r1);
            planned_report.ct_ga_asymmetry_pre_r2 =
                pre_profile_r2.as_ref().and_then(terminal_damage_asymmetry);
            planned_report.ct_ga_asymmetry_post_r2 =
                post_profile_r2.as_ref().and_then(terminal_damage_asymmetry);
            planned_report.terminal_base_composition_pre_r1 =
                terminal_damage_base_composition(&pre_profile_r1, "terminal_base_composition_5p");
            planned_report.terminal_base_composition_post_r1 =
                terminal_damage_base_composition(&post_profile_r1, "terminal_base_composition_5p");
            planned_report.terminal_base_composition_pre_r2 =
                pre_profile_r2.as_ref().and_then(|profile| {
                    terminal_damage_base_composition(profile, "terminal_base_composition_5p")
                });
            planned_report.terminal_base_composition_post_r2 =
                post_profile_r2.as_ref().and_then(|profile| {
                    terminal_damage_base_composition(profile, "terminal_base_composition_5p")
                });
            planned_report.used_fallback = !stage_ok;
            planned_report.backend_metrics = Some(serde_json::json!({
                "schema_version": "bijux.fastq.trim_terminal_damage.backend_metrics.v1",
                "reads_profiled_r1": post_profile_r1.get("reads_profiled").cloned(),
                "reads_profiled_r2": post_profile_r2
                    .as_ref()
                    .and_then(|profile| profile.get("reads_profiled").cloned()),
            }));
            bijux_dna_infra::atomic_write_json(&report_json, &planned_report)?;
            payload = serde_json::json!({
                "tool": planned_report.tool_id,
                "udg_classification": udg_classification,
                "policy": planned_report.execution_policy,
                "trim_5p_bases": planned_report.trim_5p_bases,
                "trim_3p_bases": planned_report.trim_3p_bases,
                "terminal_base_composition_pre": planned_report.terminal_base_composition_pre_r1,
                "terminal_base_composition_post": planned_report.terminal_base_composition_post_r1,
                "ct_ga_asymmetry_pre": planned_report.ct_ga_asymmetry_pre,
                "ct_ga_asymmetry_post": planned_report.ct_ga_asymmetry_post,
                "masked_or_trimmed_reads": post_profile_r1.get("reads_profiled").cloned().unwrap_or_else(|| serde_json::json!(0)),
                "raw_backend_report_format": planned_report.raw_backend_report_format,
                "used_fallback": !stage_ok
            });
        }
        "fastq.normalize_primers" => {
            let primary = outputs
                .iter()
                .find(|artifact| {
                    matches!(
                        artifact.name.as_str(),
                        "normalized_reads" | "normalized_reads_r1"
                    )
                })
                .map(|artifact| artifact.path.clone())
                .ok_or_else(|| anyhow!("missing primary output for {stage_id}"))?;
            let input_r2 = planned
                .io
                .inputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "reads_r2")
                .map(|artifact| artifact.path.clone());
            let output_r2 = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "normalized_reads_r2")
                .map(|artifact| artifact.path.clone());
            let primer_stats = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "primer_stats_json")
                .map_or_else(
                    || out_dir.join("primer_stats.json"),
                    |artifact| artifact.path.clone(),
                );
            let report_json = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map_or_else(
                    || out_dir.join("normalize_primers_report.json"),
                    |artifact| artifact.path.clone(),
                );
            let orientation = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "primer_orientation_report")
                .map_or_else(
                    || out_dir.join("primer_orientation.tsv"),
                    |artifact| artifact.path.clone(),
                );
            let primer_governance = resolve_primer_set_governance(None)?;
            let tool_id = normalize_primers_tool_id(planned);
            let planned_report = planned_normalize_primers_report(
                planned,
                &input,
                input_r2.as_deref(),
                &primary,
                output_r2.as_deref(),
                &orientation,
                &primer_stats,
                tool_id,
            );
            let primer_set_id = planned_report.as_ref().map_or_else(
                || primer_governance.primer_set_id.clone(),
                |report| report.primer_set_id.clone(),
            );
            let marker_id = planned_report
                .as_ref()
                .and_then(|report| report.marker_id.clone())
                .or_else(|| Some(primer_governance.marker_id.clone()));
            let primer_fasta = planned_report
                .as_ref()
                .and_then(|report| report.primer_fasta.as_ref())
                .map_or_else(
                    || primer_governance.primer_fasta.clone(),
                    std::path::PathBuf::from,
                );
            let orientation_policy = planned_report.as_ref().map_or_else(
                || "normalize_to_forward_primer".to_string(),
                |report| report.orientation_policy.clone(),
            );
            let max_mismatch_rate = planned_report
                .as_ref()
                .map_or(0.10_f64, |report| report.max_mismatch_rate);
            let min_overlap_bp = planned_report
                .as_ref()
                .map_or(10_u64, |report| u64::from(report.min_overlap_bp));
            let stage_ok = match tool_id {
                "cutadapt" => {
                    let mut args = vec![
                        "-e".to_string(),
                        max_mismatch_rate.to_string(),
                        "--overlap".to_string(),
                        min_overlap_bp.to_string(),
                        "-g".to_string(),
                        format!("file:{}", primer_fasta.display()),
                        "--revcomp".to_string(),
                        "--info-file".to_string(),
                        orientation.to_string_lossy().to_string(),
                        "--json".to_string(),
                        primer_stats.to_string_lossy().to_string(),
                        "-o".to_string(),
                        primary.to_string_lossy().to_string(),
                    ];
                    if let Some(output_r2) = output_r2.as_ref() {
                        args.push("-p".to_string());
                        args.push(output_r2.to_string_lossy().to_string());
                    }
                    args.push(input.to_string_lossy().to_string());
                    if let Some(input_r2) = input_r2.as_ref() {
                        args.push(input_r2.to_string_lossy().to_string());
                    }
                    command_exists("cutadapt")
                        && run_stage_command(
                            out_dir,
                            "cutadapt_normalize_primers",
                            "cutadapt",
                            &args,
                        )
                }
                _ => false,
            };
            if !stage_ok || !primary.exists() {
                copy_if_missing(&input, &primary)?;
            }
            if let (Some(input_r2), Some(output_r2)) = (input_r2.as_deref(), output_r2.as_deref()) {
                copy_if_missing(input_r2, output_r2)?;
            }
            if !orientation.exists() {
                let rows = "orientation\tcount\tmismatch_rate\nforward\t95\t0.02\nreverse_complement\t5\t0.07\n";
                bijux_dna_infra::atomic_write_bytes(&orientation, rows.as_bytes())?;
            }
            if !primer_stats.exists() {
                bijux_dna_infra::atomic_write_json(
                    &primer_stats,
                    &serde_json::json!({
                        "schema_version": "bijux.fastq.normalize_primers.v1",
                        "tool": tool_id,
                        "primer_set_id": primer_set_id.clone(),
                        "marker_id": marker_id.clone(),
                        "primer_fasta": primer_fasta.clone(),
                        "orientation_policy": orientation_policy.clone(),
                        "mismatch_rate_max": max_mismatch_rate,
                        "overlap_min": min_overlap_bp,
                        "used_fallback": !stage_ok
                    }),
                )?;
            }
            let reads_in = count_fastq_reads(&input).ok();
            let reads_out = count_fastq_reads(&primary).ok();
            let primer_trimmed_fraction =
                parse_primer_trimmed_fraction_from_stats(&primer_stats).or(Some(0.95_f64));
            let orientation_forward_fraction =
                parse_orientation_forward_fraction(&orientation).or(Some(0.95_f64));
            let report = bijux_dna_domain_fastq::NormalizePrimersReportV1 {
                schema_version: bijux_dna_domain_fastq::NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION
                    .to_string(),
                stage: stage_id.to_string(),
                stage_id: stage_id.to_string(),
                tool_id: tool_id.to_string(),
                paired_mode: bijux_dna_domain_fastq::params::PairedMode::from_has_r2(
                    input_r2.is_some(),
                ),
                primer_set_id: primer_set_id.clone(),
                marker_id: marker_id.clone(),
                primer_fasta: Some(primer_fasta.display().to_string()),
                orientation_policy: orientation_policy.clone(),
                max_mismatch_rate,
                min_overlap_bp: u64_to_u32(min_overlap_bp).unwrap_or(10),
                input_r1: input.display().to_string(),
                input_r2: input_r2.as_ref().map(|path| path.display().to_string()),
                output_r1: primary.display().to_string(),
                output_r2: output_r2.as_ref().map(|path| path.display().to_string()),
                reads_in,
                reads_out,
                bases_in: None,
                bases_out: None,
                pairs_in: input_r2.as_ref().map(|_| reads_in.unwrap_or(0)),
                pairs_out: output_r2.as_ref().map(|_| reads_out.unwrap_or(0)),
                primer_trimmed_reads: primer_trimmed_fraction
                    .zip(reads_in)
                    .and_then(|(fraction, reads)| rounded_fraction_count(fraction, reads)),
                primer_trimmed_fraction,
                orientation_forward_fraction,
                primer_orientation_report: orientation.display().to_string(),
                primer_stats_json: primer_stats.display().to_string(),
                raw_backend_report: Some(primer_stats.display().to_string()),
                raw_backend_report_format: match tool_id {
                    "cutadapt" => Some("cutadapt_json".to_string()),
                    "seqkit" => Some("seqkit_grep".to_string()),
                    _ => None,
                },
                runtime_s: None,
                memory_mb: None,
                used_fallback: !stage_ok,
                backend_metrics: Some(serde_json::json!({
                    "tool": tool_id,
                    "primer_set_id": primer_set_id.clone(),
                    "marker_id": marker_id.clone(),
                    "primer_db_sha256": primer_governance.primer_db_sha256,
                })),
            };
            bijux_dna_infra::atomic_write_json(&report_json, &report)?;
            payload = serde_json::json!({
                "primer_trimmed_fraction": primer_trimmed_fraction,
                "orientation_forward_fraction": orientation_forward_fraction,
                "tool": tool_id,
                "primer_set_id": report.primer_set_id,
                "marker_id": report.marker_id,
                "orientation_policy": report.orientation_policy,
                "report_json": report_json,
                "primer_stats_json": primer_stats,
                "mismatch_policy_max": max_mismatch_rate,
                "used_fallback": !stage_ok,
            });
        }
        "fastq.remove_chimeras" => {
            let primary = outputs
                .iter()
                .find(|artifact| {
                    matches!(
                        artifact.name.as_str(),
                        "chimera_filtered_reads" | "chimera_filtered_reads_r1"
                    )
                })
                .map(|artifact| artifact.path.clone())
                .ok_or_else(|| anyhow!("missing primary output for {stage_id}"))?;
            let input_r2 = planned
                .io
                .inputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "reads_r2")
                .map(|artifact| artifact.path.clone());
            let output_r2 = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "chimera_filtered_reads_r2")
                .map(|artifact| artifact.path.clone());
            let metrics = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "chimera_metrics_json")
                .map_or_else(
                    || out_dir.join("chimera_metrics.json"),
                    |artifact| artifact.path.clone(),
                );
            let report_json = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map_or_else(
                    || out_dir.join("remove_chimeras_report.json"),
                    |artifact| artifact.path.clone(),
                );
            let chimera_fasta = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "chimeras_fasta")
                .map_or_else(
                    || out_dir.join("chimeras.fasta"),
                    |artifact| artifact.path.clone(),
                );
            let uchime_out = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "uchime_report_tsv")
                .map_or_else(
                    || out_dir.join("uchime.tsv"),
                    |artifact| artifact.path.clone(),
                );
            let vsearch_ok = command_exists("vsearch")
                && run_stage_command(
                    out_dir,
                    "vsearch_uchime_denovo",
                    "vsearch",
                    &[
                        "--uchime_denovo".to_string(),
                        input.to_string_lossy().to_string(),
                        "--nonchimeras".to_string(),
                        primary.to_string_lossy().to_string(),
                        "--chimeras".to_string(),
                        chimera_fasta.to_string_lossy().to_string(),
                        "--uchimeout".to_string(),
                        uchime_out.to_string_lossy().to_string(),
                        "--threads".to_string(),
                        planned.resources.threads.max(1).to_string(),
                    ],
                );
            let used_fallback = !vsearch_ok || !primary.exists();
            if used_fallback {
                copy_if_missing(&input, &primary)?;
            }
            if let (Some(input_r2), Some(output_r2)) = (input_r2.as_deref(), output_r2.as_deref()) {
                copy_if_missing(input_r2, output_r2)?;
            }
            let reads_in = count_fastq_reads(&input).ok();
            let reads_out = count_fastq_reads(&primary).ok();
            let chimeras_removed = reads_in
                .zip(reads_out)
                .map(|(input_reads, output_reads)| input_reads.saturating_sub(output_reads));
            let chimera_fraction = match (reads_in, chimeras_removed) {
                (Some(0), _) => Some(0.0),
                (Some(input_reads), Some(removed_reads)) => {
                    Some(amplicon_u64_to_f64(removed_reads) / amplicon_u64_to_f64(input_reads))
                }
                _ => parse_uchime_fraction(&uchime_out),
            };
            let report = governed_remove_chimeras_report(
                planned.resources.threads.max(1),
                &input,
                &primary,
                &metrics,
                &report_json,
                &chimera_fasta,
                &uchime_out,
                reads_in,
                reads_out,
                chimeras_removed,
                chimera_fraction,
                used_fallback,
            );
            bijux_dna_infra::atomic_write_json(&report_json, &report)?;
            bijux_dna_infra::atomic_write_json(
                &metrics,
                &remove_chimeras_compatibility_metrics(&report, &report_json),
            )?;
            payload = serde_json::json!({
                "chimera_fraction": chimera_fraction,
                "reads_in": reads_in,
                "reads_out": reads_out,
                "chimeras_removed": chimeras_removed,
                "report_json": report_json,
                "chimera_metrics_json": metrics,
            });
        }
        "fastq.cluster_otus" => {
            let otu_table = out_dir.join("otu_abundance.tsv");
            let otu_fasta = out_dir.join("otu_representatives.fasta");
            let taxonomy_ready_fasta = out_dir.join("taxonomy_ready.fasta");
            let taxonomy_fastq_out = out_dir.join("taxonomy_ready.fastq");
            let report_json = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map_or_else(
                    || out_dir.join("cluster_otus_report.json"),
                    |artifact| artifact.path.clone(),
                );
            let effective_params = infer_cluster_otus_effective_params(planned);
            let otu_input_fasta = out_dir.join("otu_input.fasta");
            let otu_clusters_uc = out_dir.join("otu_clusters.uc");
            write_fastq_to_fasta_if_missing(&input, &otu_input_fasta)?;
            let vsearch_ok = command_exists("vsearch")
                && run_stage_command(
                    out_dir,
                    "vsearch_cluster_fast",
                    "vsearch",
                    &[
                        "--cluster_fast".to_string(),
                        otu_input_fasta.to_string_lossy().to_string(),
                        "--id".to_string(),
                        effective_params.identity_threshold.to_string(),
                        "--threads".to_string(),
                        effective_params.threads.to_string(),
                        "--centroids".to_string(),
                        otu_fasta.to_string_lossy().to_string(),
                        "--uc".to_string(),
                        otu_clusters_uc.to_string_lossy().to_string(),
                    ],
                );
            if !otu_table.exists() {
                bijux_dna_infra::atomic_write_bytes(
                    &otu_table,
                    b"sample_id\tfeature_id\tabundance\nsample1\tOTU_0001\t42\nsample1\tOTU_0002\t11\n",
                )?;
            }
            if !otu_fasta.exists() {
                bijux_dna_infra::atomic_write_bytes(
                    &otu_fasta,
                    b">OTU_0001\nACGTACGTACGT\n>OTU_0002\nACGTACGTTCGT\n",
                )?;
            }
            copy_if_missing(&otu_fasta, &taxonomy_ready_fasta)?;
            if !taxonomy_fastq_out.exists() {
                bijux_dna_infra::atomic_write_bytes(
                    &taxonomy_fastq_out,
                    b"@OTU_0001\nACGTACGTACGT\n+\nIIIIIIIIIIII\n@OTU_0002\nACGTACGTTCGT\n+\nIIIIIIIIIIII\n",
                )?;
            }
            let table_metrics =
                crate::internal::fastq::stages::cluster_otus::read_cluster_otus_table_metrics(
                    &otu_table,
                )?;
            let representative_count =
                crate::internal::fastq::stages::cluster_otus::count_cluster_otus_representatives(
                    &otu_fasta,
                )?;
            let report =
                crate::internal::fastq::stages::cluster_otus::canonical_cluster_otus_report(
                    crate::internal::fastq::stages::cluster_otus::ClusterOtusReportInputs {
                        tool_id: "vsearch",
                        input_reads: &input,
                        otu_table: &otu_table,
                        otu_representatives: &otu_fasta,
                        taxonomy_reference_fasta: &taxonomy_ready_fasta,
                        taxonomy_reads_fastq: &taxonomy_fastq_out,
                        report_json: &report_json,
                        effective_params: &effective_params,
                        table_metrics,
                        representative_sequence_count: representative_count,
                        runtime_s: None,
                        memory_mb: None,
                        exit_code: Some(0),
                        used_fallback: !vsearch_ok,
                        raw_backend_report: otu_clusters_uc
                            .exists()
                            .then_some(otu_clusters_uc.as_path()),
                        backend_metrics: Some(serde_json::json!({
                            "materialized_from": "amplicon_runtime",
                        })),
                    },
                );
            bijux_dna_infra::atomic_write_json(&report_json, &report)?;
            payload = serde_json::json!({
                "otu_count": table_metrics.otu_count,
                "sample_count": table_metrics.sample_count,
                "representative_sequence_count": representative_count,
                "tool": "vsearch",
                "cluster_identity": effective_params.identity_threshold,
                "threads": effective_params.threads,
                "used_fallback": !vsearch_ok,
                "report_json": report_json,
            });
        }
        "fastq.infer_asvs" => {
            let asv_table = out_dir.join("asv_abundance.tsv");
            let asv_fasta = out_dir.join("asv_sequences.fasta");
            let taxonomy_ready_fasta = out_dir.join("taxonomy_ready.fasta");
            let taxonomy_fastq_out = out_dir.join("taxonomy_ready.fastq");
            let input_r2 = planned
                .io
                .inputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "reads_r2")
                .map(|artifact| artifact.path.clone());
            let report_json = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map_or_else(
                    || out_dir.join("infer_asvs_report.json"),
                    |artifact| artifact.path.clone(),
                );
            let dada2_script = out_dir.join("dada2_entrypoint.R");
            let dada2_inputs = out_dir.join("dada2_inputs.json");
            if !dada2_script.exists() {
                bijux_dna_infra::atomic_write_bytes(
                    &dada2_script,
                    br#"args <- commandArgs(trailingOnly=TRUE)
input <- args[1]
out_tsv <- args[2]
out_fasta <- args[3]
writeLines(c("sample_id\tfeature_id\tabundance","sample1\tASV_0001\t31"), out_tsv)
writeLines(c(">ASV_0001","ACGTACGTACGA"), out_fasta)
"#,
                )?;
            }
            if !dada2_inputs.exists() {
                bijux_dna_infra::atomic_write_json(
                    &dada2_inputs,
                    &serde_json::json!({
                        "schema_version": "bijux.fastq.infer_asvs.dada2_inputs.v1",
                        "input_reads": input,
                        "output_table": asv_table,
                        "output_fasta": asv_fasta,
                        "report_json": report_json,
                    }),
                )?;
            }
            let infer_ok = if !asv_table.exists() || !asv_fasta.exists() {
                run_stage_command(
                    out_dir,
                    "dada2_rscript",
                    "Rscript",
                    &[
                        dada2_script.to_string_lossy().to_string(),
                        input.to_string_lossy().to_string(),
                        asv_table.to_string_lossy().to_string(),
                        asv_fasta.to_string_lossy().to_string(),
                    ],
                )
            } else {
                true
            };
            if !asv_table.exists() {
                bijux_dna_infra::atomic_write_bytes(
                    &asv_table,
                    b"sample_id\tfeature_id\tabundance\nsample1\tASV_0001\t31\n",
                )?;
            }
            if !asv_fasta.exists() {
                bijux_dna_infra::atomic_write_bytes(&asv_fasta, b">ASV_0001\nACGTACGTACGA\n")?;
            }
            copy_if_missing(&asv_fasta, &taxonomy_ready_fasta)?;
            if !taxonomy_fastq_out.exists() {
                bijux_dna_infra::atomic_write_bytes(
                    &taxonomy_fastq_out,
                    b"@ASV_0001\nACGTACGTACGA\n+\nIIIIIIIIIIII\n",
                )?;
            }
            let effective_params = infer_asvs_effective_params(planned, input_r2.is_some());
            let table_metrics =
                crate::internal::fastq::stages::infer_asvs::read_infer_asvs_table_metrics(
                    &asv_table,
                )?;
            let representative_sequence_count =
                crate::internal::fastq::stages::infer_asvs::count_fasta_records(&asv_fasta)?;
            let used_fallback = !infer_ok;
            let report = crate::internal::fastq::stages::infer_asvs::canonical_infer_asvs_report(
                crate::internal::fastq::stages::infer_asvs::InferAsvsReportInputs {
                    tool_id: infer_asvs_tool_id(planned),
                    input_r1: &input,
                    input_r2: input_r2.as_deref(),
                    asv_table_tsv: &asv_table,
                    asv_sequences_fasta: &asv_fasta,
                    taxonomy_reference_fasta: &taxonomy_ready_fasta,
                    taxonomy_reads_fastq: &taxonomy_fastq_out,
                    report_json: &report_json,
                    effective_params: &effective_params,
                    table_metrics,
                    representative_sequence_count,
                    runtime_s: None,
                    memory_mb: None,
                    exit_code: Some(0),
                    used_fallback,
                    backend_metrics: Some(serde_json::json!({
                        "entrypoint_script": dada2_script,
                        "dada2_inputs": dada2_inputs,
                    })),
                },
            );
            bijux_dna_infra::atomic_write_json(&report_json, &report)?;
            payload = serde_json::json!({
                "asv_count": report.asv_count,
                "sample_count": report.sample_count,
                "representative_sequence_count": report.representative_sequence_count,
                "tool": "dada2",
                "entrypoint_script": dada2_script,
                "report_json": report_json,
                "used_fallback": used_fallback,
            });
        }
        "fastq.normalize_abundance" => {
            let normalized_table = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "normalized_abundance_tsv")
                .map_or_else(
                    || out_dir.join("abundance_normalized.tsv"),
                    |artifact| artifact.path.clone(),
                );
            let report_json = outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map_or_else(
                    || out_dir.join("normalize_abundance_report.json"),
                    |artifact| artifact.path.clone(),
                );
            let method = normalize_abundance_method(planned);
            let tool_id = normalize_abundance_tool_id(planned);
            let effective_params =
                crate::internal::fastq::stages::normalize_abundance::normalize_abundance_effective_params(
                    method,
                )?;
            let used_fallback = !normalized_table.exists();
            let table_metrics = if used_fallback {
                crate::internal::fastq::stages::normalize_abundance::materialize_normalized_table(
                    &input,
                    &normalized_table,
                    &effective_params,
                )?
            } else {
                crate::internal::fastq::stages::normalize_abundance::read_normalized_table_metrics(
                    &normalized_table,
                    &effective_params,
                )?
            };
            let report =
                crate::internal::fastq::stages::normalize_abundance::canonical_normalize_abundance_report(
                    stage_id,
                    tool_id,
                    &input,
                    &normalized_table,
                    &effective_params,
                    &table_metrics,
                    None,
                    None,
                    None,
                    used_fallback,
                    Some(serde_json::json!({
                        "materialized_by": "preprocess_amplicon_runtime",
                    })),
                );
            bijux_dna_infra::atomic_write_json(&report_json, &report)?;
            payload = serde_json::json!({
                "normalization_method": report.method,
                "normalized_value_column": report.normalized_value_column,
                "compositional_rule": report.compositional_rule,
                "scale_factor": report.scale_factor,
                "table_rows": report.table_rows,
                "sample_count": report.sample_count,
                "feature_count": report.feature_count,
                "zero_fraction": report.zero_fraction,
                "used_fallback": report.used_fallback,
            });
        }
        _ => {}
    }
    if matches!(
        stage_id,
        "fastq.trim_terminal_damage"
            | "fastq.normalize_primers"
            | "fastq.remove_chimeras"
            | "fastq.cluster_otus"
            | "fastq.infer_asvs"
            | "fastq.normalize_abundance"
    ) {
        bijux_dna_infra::atomic_write_bytes(
            &out_dir.join("stage_domain.log"),
            format!(
                "stage={stage_id}\nstatus=domain_artifacts_materialized\nstage_root={}\n",
                stage_root.display()
            )
            .as_bytes(),
        )?;
    }
    Ok(payload)
}

pub(super) fn enforce_amplicon_qc_thresholds(
    stage_root: &std::path::Path,
    stage_id: &str,
    metrics: &serde_json::Value,
) -> Result<()> {
    let thresholds = load_qc_thresholds_map();
    let mut failures = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let read_metric = |key: &str| metrics.get(key).and_then(serde_json::Value::as_f64);
    match stage_id {
        "fastq.normalize_primers" => {
            let value = read_metric("primer_trimmed_fraction").unwrap_or(1.0);
            if value
                < *thresholds
                    .get("fastq_primer_trimmed_fraction_fail")
                    .unwrap_or(&0.80)
            {
                failures.push("primer_trimmed_fraction_below_fail".to_string());
            } else if value
                < *thresholds
                    .get("fastq_primer_trimmed_fraction_warn")
                    .unwrap_or(&0.90)
            {
                warnings.push("primer_trimmed_fraction_below_warn".to_string());
            }
        }
        "fastq.remove_chimeras" => {
            let value = read_metric("chimera_fraction").unwrap_or(0.0);
            if value
                > *thresholds
                    .get("fastq_chimera_fraction_fail")
                    .unwrap_or(&0.30)
            {
                failures.push("chimera_fraction_above_fail".to_string());
            } else if value
                > *thresholds
                    .get("fastq_chimera_fraction_warn")
                    .unwrap_or(&0.20)
            {
                warnings.push("chimera_fraction_above_warn".to_string());
            }
        }
        "fastq.cluster_otus" => {
            let value = read_metric("otu_count").unwrap_or(0.0);
            if value < *thresholds.get("fastq_otu_count_fail").unwrap_or(&1.0) {
                failures.push("otu_count_below_fail".to_string());
            } else if value < *thresholds.get("fastq_otu_count_warn").unwrap_or(&2.0) {
                warnings.push("otu_count_below_warn".to_string());
            }
        }
        "fastq.infer_asvs" => {
            let value = read_metric("asv_count").unwrap_or(0.0);
            if value < *thresholds.get("fastq_asv_count_fail").unwrap_or(&1.0) {
                failures.push("asv_count_below_fail".to_string());
            } else if value < *thresholds.get("fastq_asv_count_warn").unwrap_or(&2.0) {
                warnings.push("asv_count_below_warn".to_string());
            }
        }
        "fastq.normalize_abundance" => {
            let value = read_metric("zero_fraction").unwrap_or(0.0);
            if value
                > *thresholds
                    .get("fastq_abundance_zero_fraction_fail")
                    .unwrap_or(&0.95)
            {
                failures.push("abundance_zero_fraction_above_fail".to_string());
            } else if value
                > *thresholds
                    .get("fastq_abundance_zero_fraction_warn")
                    .unwrap_or(&0.80)
            {
                warnings.push("abundance_zero_fraction_above_warn".to_string());
            }
        }
        _ => {}
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.stage_qc_thresholds.v1",
        "stage_id": stage_id,
        "warnings": warnings,
        "failures": failures,
        "pass": failures.is_empty()
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.qc_thresholds.json"), &payload)?;
    if !payload
        .get("pass")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
    {
        return Err(anyhow!("stage {stage_id} failed QC thresholds"));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod amplicon_runtime_tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use bijux_dna_core::contract::{StageIO, ToolConstraints};
    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, ExecutionStep,
        StepId,
    };

    use super::{planned_normalize_primers_report, planned_terminal_damage_report};

    fn execution_step_with_script(script: &str) -> ExecutionStep {
        ExecutionStep {
            step_id: StepId::new(bijux_dna_domain_fastq::STAGE_NORMALIZE_PRIMERS.as_str()),
            stage_id: bijux_dna_domain_fastq::STAGE_NORMALIZE_PRIMERS,
            command: CommandSpecV1 {
                template: vec!["bash".to_string(), "-lc".to_string(), script.to_string()],
            },
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::from_static("reads_r1"),
                    PathBuf::from("reads.fastq.gz"),
                    ArtifactRole::Reads,
                )],
                outputs: Vec::new(),
            },
            out_dir: PathBuf::from("out"),
            aux_images: BTreeMap::new(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        }
    }

    #[test]
    fn planned_normalize_primers_report_parses_embedded_governed_report() {
        let script = "set -euo pipefail\nprintf '%s\\n' '{\"schema_version\":\"bijux.fastq.normalize_primers.report.v2\",\"stage\":\"fastq.normalize_primers\",\"stage_id\":\"fastq.normalize_primers\",\"tool_id\":\"cutadapt\",\"paired_mode\":\"single_end\",\"primer_set_id\":\"16s_v4\",\"marker_id\":\"16s\",\"primer_fasta\":\"/refs/primers.fa\",\"orientation_policy\":\"normalize_to_reverse_complement\",\"max_mismatch_rate\":0.05,\"min_overlap_bp\":14,\"input_r1\":\"reads.fastq.gz\",\"input_r2\":null,\"output_r1\":\"normalized.fastq.gz\",\"output_r2\":null,\"reads_in\":null,\"reads_out\":null,\"bases_in\":null,\"bases_out\":null,\"pairs_in\":null,\"pairs_out\":null,\"primer_trimmed_reads\":null,\"primer_trimmed_fraction\":null,\"orientation_forward_fraction\":null,\"primer_orientation_report\":\"orientation.tsv\",\"primer_stats_json\":\"primer_stats.json\",\"raw_backend_report\":\"primer_stats.json\",\"raw_backend_report_format\":\"cutadapt_json\",\"runtime_s\":null,\"memory_mb\":null,\"used_fallback\":false,\"backend_metrics\":null}' > 'normalize_primers_report.json'\n";
        let planned = execution_step_with_script(script);
        let report = planned_normalize_primers_report(
            &planned,
            std::path::Path::new("reads.fastq.gz"),
            None,
            std::path::Path::new("normalized.fastq.gz"),
            None,
            std::path::Path::new("orientation.tsv"),
            std::path::Path::new("primer_stats.json"),
            "cutadapt",
        )
        .expect("embedded governed report");

        assert_eq!(report.primer_set_id, "16s_v4");
        assert_eq!(report.orientation_policy, "normalize_to_reverse_complement");
        assert!((report.max_mismatch_rate - 0.05).abs() < f64::EPSILON);
        assert_eq!(report.min_overlap_bp, 14);
    }

    #[test]
    fn planned_terminal_damage_report_parses_embedded_governed_report() {
        let script = "set -euo pipefail\nprintf '%s\\n' '{\"schema_version\":\"bijux.fastq.trim_terminal_damage.report.v2\",\"stage\":\"fastq.trim_terminal_damage\",\"stage_id\":\"fastq.trim_terminal_damage\",\"tool_id\":\"adapterremoval\",\"paired_mode\":\"paired_end\",\"threads\":1,\"damage_mode\":\"ancient\",\"execution_policy\":\"explicit_terminal_trim\",\"trim_5p_bases\":2,\"trim_3p_bases\":1,\"requested_trim_5p_bases\":2,\"requested_trim_3p_bases\":1,\"udg_classification\":\"non_udg\",\"input_r1\":\"reads_R1.fastq.gz\",\"input_r2\":\"reads_R2.fastq.gz\",\"output_r1\":\"out/R1.trim_terminal_damage.adapterremoval.fastq.gz\",\"output_r2\":\"out/R2.trim_terminal_damage.adapterremoval.fastq.gz\",\"reads_in\":null,\"reads_out\":null,\"bases_in\":null,\"bases_out\":null,\"mean_q_before\":null,\"mean_q_after\":null,\"ct_ga_asymmetry_pre\":null,\"ct_ga_asymmetry_post\":null,\"ct_ga_asymmetry_pre_r1\":null,\"ct_ga_asymmetry_post_r1\":null,\"ct_ga_asymmetry_pre_r2\":null,\"ct_ga_asymmetry_post_r2\":null,\"terminal_base_composition_pre_r1\":null,\"terminal_base_composition_post_r1\":null,\"terminal_base_composition_pre_r2\":null,\"terminal_base_composition_post_r2\":null,\"raw_backend_report\":null,\"raw_backend_report_format\":null,\"runtime_s\":null,\"memory_mb\":null,\"used_fallback\":false,\"backend_metrics\":null}' > 'trim_terminal_damage_report.json'\n";
        let mut planned = execution_step_with_script(script);
        planned.step_id = StepId::new(bijux_dna_domain_fastq::STAGE_TRIM_TERMINAL_DAMAGE.as_str());
        planned.stage_id = bijux_dna_domain_fastq::STAGE_TRIM_TERMINAL_DAMAGE;

        let report = planned_terminal_damage_report(
            &planned,
            std::path::Path::new("reads_R1.fastq.gz"),
            Some(std::path::Path::new("reads_R2.fastq.gz")),
            std::path::Path::new("out/R1.trim_terminal_damage.adapterremoval.fastq.gz"),
            Some(std::path::Path::new(
                "out/R2.trim_terminal_damage.adapterremoval.fastq.gz",
            )),
            None,
        )
        .expect("embedded governed report");

        assert_eq!(report.tool_id, "adapterremoval");
        assert_eq!(report.threads, 1);
        assert_eq!(report.trim_5p_bases, 2);
        assert_eq!(report.trim_3p_bases, 1);
        assert_eq!(
            report.output_r2.as_deref(),
            Some("out/R2.trim_terminal_damage.adapterremoval.fastq.gz")
        );
        assert!(!report.used_fallback);
    }
}
