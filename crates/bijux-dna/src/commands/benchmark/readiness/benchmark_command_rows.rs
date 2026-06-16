use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ArtifactRole;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::params::correct::QualityEncoding;
use bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy;
use bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy;
use bijux_dna_domain_fastq::params::DamageMode;
use bijux_dna_stage_contract::StagePlanV1;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use crate::commands::benchmark::local_stage_commands::rendered_stage_materialize_argv;
use crate::commands::benchmark::local_stage_commands::{
    collect_local_stage_plan_bundles, local_stage_plans,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BenchmarkCommandRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) argv: Vec<String>,
}

#[derive(Debug, Clone)]
struct CanonicalStagePlan {
    readiness_kind: LocalStageReadinessKind,
    plan: StagePlanV1,
}

pub(crate) fn collect_benchmark_command_rows(repo_root: &Path) -> Result<Vec<BenchmarkCommandRow>> {
    let fastq_base_plans = canonical_stage_plan_map(repo_root, BenchLocalDomain::Fastq)?;
    let (_, _, fastq_rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;

    let mut rows = Vec::new();

    for row in fastq_rows
        .into_iter()
        .filter(|row| row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady)
    {
        let base = fastq_base_plans.get(&row.stage_id).ok_or_else(|| {
            anyhow!(
                "missing canonical FASTQ local stage plan for benchmark-ready row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            )
        })?;
        let argv =
            render_fastq_stage_tool_argv(repo_root, &row.stage_id, &row.tool_id, &base.plan)?;
        rows.push(BenchmarkCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: readiness_kind_label(base.readiness_kind).to_string(),
            argv,
        });
    }

    rows.extend(collect_bam_benchmark_command_rows(repo_root)?);

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_unique_rows(&rows)?;
    Ok(rows)
}

pub(crate) fn collect_bam_benchmark_command_rows(
    repo_root: &Path,
) -> Result<Vec<BenchmarkCommandRow>> {
    let bam_readiness_by_stage = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?
        .stages
        .into_iter()
        .map(|stage| (stage.stage_id, stage.readiness_kind))
        .collect::<BTreeMap<_, _>>();
    let (_, _, bam_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let mut rows = Vec::new();

    for row in bam_rows
        .into_iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
    {
        let (readiness_kind, argv) = if stage_uses_materialize_stage_fallback(&row.stage_id) {
            (
                bam_readiness_by_stage
                    .get(&row.stage_id)
                    .copied()
                    .unwrap_or(LocalStageReadinessKind::Smoke),
                rendered_stage_materialize_argv(&row.stage_id),
            )
        } else {
            let base = local_stage_plans(repo_root, &row.stage_id)?.into_iter().next().ok_or_else(
                || {
                    anyhow!(
                        "local benchmark BAM stage `{}` did not yield any governed plans",
                        row.stage_id
                    )
                },
            )?;
            let readiness_kind = bam_readiness_by_stage.get(&row.stage_id).copied().ok_or_else(
                || {
                    anyhow!(
                        "missing canonical BAM local stage inventory row for benchmark-ready row `{}` / `{}`",
                        row.stage_id,
                        row.tool_id
                    )
                },
            )?;
            (
                readiness_kind,
                render_bam_stage_tool_argv(repo_root, &row.stage_id, &row.tool_id, &base)?,
            )
        };
        rows.push(BenchmarkCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: readiness_kind_label(readiness_kind).to_string(),
            argv,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_unique_rows(&rows)?;
    Ok(rows)
}

pub(crate) fn collect_selected_fastq_command_rows(
    repo_root: &Path,
    stage_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BenchmarkCommandRow>> {
    collect_selected_domain_command_rows(repo_root, BenchLocalDomain::Fastq, stage_ids)
}

pub(crate) fn collect_selected_bam_command_rows(
    repo_root: &Path,
    stage_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BenchmarkCommandRow>> {
    collect_selected_domain_command_rows(repo_root, BenchLocalDomain::Bam, stage_ids)
}

pub(crate) fn render_shell_command(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            if arg
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':'))
            {
                arg.clone()
            } else {
                let escaped = arg.replace('\'', "'\"'\"'");
                format!("'{escaped}'")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn canonical_stage_plan_map(
    repo_root: &Path,
    domain: BenchLocalDomain,
) -> Result<BTreeMap<String, CanonicalStagePlan>> {
    collect_local_stage_plan_bundles(repo_root, Some(domain))?
        .into_iter()
        .map(|bundle| {
            let plan = bundle.plans.into_iter().next().ok_or_else(|| {
                anyhow!(
                    "local stage bundle `{}` did not include a canonical governed plan",
                    bundle.stage_id
                )
            })?;
            Ok((
                bundle.stage_id,
                CanonicalStagePlan { readiness_kind: bundle.readiness_kind, plan },
            ))
        })
        .collect()
}

fn collect_selected_domain_command_rows(
    repo_root: &Path,
    domain: BenchLocalDomain,
    stage_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BenchmarkCommandRow>> {
    let readiness_by_stage = load_local_stage_inventory(repo_root, domain)?
        .stages
        .into_iter()
        .map(|stage| (stage.stage_id, stage.readiness_kind))
        .collect::<BTreeMap<_, _>>();
    let mut rows = BTreeMap::new();

    for stage_id in stage_ids {
        let Some(readiness_kind) = readiness_by_stage.get(stage_id).copied() else {
            return Err(anyhow!(
                "missing canonical {} local stage inventory row for `{stage_id}`",
                domain_label(domain)
            ));
        };
        let plan = local_stage_plans(repo_root, stage_id)?.into_iter().next().ok_or_else(|| {
            anyhow!(
                "local benchmark stage `{stage_id}` did not yield any governed {} plans",
                domain_label(domain)
            )
        })?;
        let tool_id = plan.tool_id.as_str().to_string();
        let argv = match domain {
            BenchLocalDomain::Fastq => {
                render_fastq_stage_tool_argv(repo_root, stage_id, &tool_id, &plan)?
            }
            BenchLocalDomain::Bam => {
                render_bam_stage_tool_argv(repo_root, stage_id, &tool_id, &plan)?
            }
            BenchLocalDomain::Vcf => {
                return Err(anyhow!(
                    "benchmark command rows do not render VCF plans through the FASTQ/BAM command adapter path"
                ));
            }
        };
        rows.insert(
            stage_id.clone(),
            BenchmarkCommandRow {
                stage_id: stage_id.clone(),
                tool_id,
                readiness_kind: readiness_kind_label(readiness_kind).to_string(),
                argv,
            },
        );
    }

    Ok(rows)
}

fn stage_uses_materialize_stage_fallback(stage_id: &str) -> bool {
    matches!(stage_id, "bam.bias_mitigation" | "bam.genotyping" | "bam.haplogroups" | "bam.kinship")
}

fn render_fastq_stage_tool_argv(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
    base_plan: &StagePlanV1,
) -> Result<Vec<String>> {
    let stage_id_value = StageId::new(stage_id.to_string());
    let tool_id_value = ToolId::new(tool_id.to_string());
    let tool = bijux_dna_planner_fastq::stage_api::load_fastq_domain_tool_execution_spec(
        repo_root,
        &stage_id_value,
        &tool_id_value,
    )
    .with_context(|| format!("load FASTQ execution spec for `{stage_id}` / `{tool_id}`"))?;
    let params = project_fastq_benchmark_params_for_tool(
        stage_id,
        tool_id,
        fastq_stage_params_from_plan(stage_id, base_plan)?,
    );
    let explicit_inputs = base_plan
        .io
        .inputs
        .iter()
        .map(|artifact| {
            let mut artifact = artifact.clone();
            artifact.path = resolve_repo_input_path(repo_root, &artifact.path);
            bijux_dna_planner_fastq::FastqStageExplicitInput {
                input_id: artifact.name.as_str().to_string(),
                source_tool_id: explicit_input_source_tool_id(stage_id, tool_id, &artifact),
                artifact,
            }
        })
        .collect::<Vec<_>>();
    let fallback_input = base_plan.io.inputs.first().ok_or_else(|| {
        anyhow!("FASTQ benchmark-ready row `{stage_id}` / `{tool_id}` has no canonical inputs")
    })?;
    let fallback_r1 = resolve_repo_input_path(repo_root, &fallback_input.path);
    let fallback_r2 = find_fastq_input(base_plan, "reads_r2")
        .map(|path| resolve_repo_input_path(repo_root, path));
    let reference_fasta =
        find_reference_fasta(base_plan).map(|path| resolve_repo_input_path(repo_root, path));
    let adapter_bank = load_default_fastq_adapter_bank_context(repo_root)
        .context("load default FASTQ adapter bank context")?;
    let polyx_bank = load_default_fastq_polyx_bank_context(repo_root)
        .context("load default FASTQ polyx bank context")?;
    let contaminant_bank = load_default_fastq_contaminant_bank_context(repo_root)
        .context("load default FASTQ contaminant bank context")?;
    let out_dir = benchmark_command_out_dir("fastq", stage_id, tool_id).with_context(|| {
        format!("build benchmark command output dir for `{stage_id}` / `{tool_id}`")
    })?;
    let plan = bijux_dna_planner_fastq::plan_fastq_stage_binding_with_explicit_inputs(
        bijux_dna_planner_fastq::FastqStageBinding {
            stage_id: stage_id.to_string(),
            stage_instance_id: None,
            tool,
            reason: None,
            params,
        },
        &std::collections::BTreeMap::new(),
        adapter_bank.as_ref(),
        polyx_bank.as_ref(),
        contaminant_bank.as_ref(),
        false,
        &fallback_r1,
        fallback_r2.as_deref(),
        reference_fasta.as_deref(),
        &explicit_inputs,
        &out_dir,
    )
    .with_context(|| format!("plan FASTQ benchmark command row `{stage_id}` / `{tool_id}`"))?;
    Ok(plan.command.template)
}

fn render_bam_stage_tool_argv(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
    base_plan: &StagePlanV1,
) -> Result<Vec<String>> {
    let stage_id_value = StageId::new(stage_id.to_string());
    let tool_id_value = ToolId::new(tool_id.to_string());
    let tool = bijux_dna_planner_bam::stage_api::load_bam_domain_tool_execution_spec(
        repo_root,
        &stage_id_value,
        &tool_id_value,
    )
    .with_context(|| format!("load BAM execution spec for `{stage_id}` / `{tool_id}`"))?;
    let (r1, r2) = if stage_id == "bam.align" {
        (
            Some(
                find_first_input(base_plan, &["reads_r1", "fastq_r1"])
                    .map(|path| resolve_repo_input_path(repo_root, path))
                    .context("resolve align reads_r1")?,
            ),
            find_first_input(base_plan, &["reads_r2", "fastq_r2"])
                .map(|path| resolve_repo_input_path(repo_root, path)),
        )
    } else {
        (None, None)
    };
    let bam = if stage_id == "bam.align" {
        None
    } else {
        find_bam_input(base_plan).map(|path| resolve_repo_input_path(repo_root, path))
    };
    let bam_index = if stage_id == "bam.align" {
        None
    } else {
        find_bam_index_input(base_plan).map(|path| resolve_repo_input_path(repo_root, path))
    };
    let derived_reference = find_bam_corpus_reference(repo_root, base_plan);
    let reference = find_reference_fasta(base_plan)
        .map(|path| resolve_repo_input_path(repo_root, path))
        .or(derived_reference);
    let params = project_bam_benchmark_params_for_tool(stage_id, tool_id, base_plan);
    let params_ref = params.as_ref();
    let sample_id = params_ref
        .and_then(|value| value.get("sample_id"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| base_plan.params.get("sample_id").and_then(serde_json::Value::as_str));
    let out_dir = benchmark_command_out_dir("bam", stage_id, tool_id).with_context(|| {
        format!("build benchmark command output dir for `{stage_id}` / `{tool_id}`")
    })?;
    let plan = bijux_dna_planner_bam::plan_stage(bijux_dna_planner_bam::StagePlanRequest {
        stage_id,
        tool: &tool,
        out_dir: &out_dir,
        bam: bam.as_deref(),
        bam_index: bam_index.as_deref(),
        r1: r1.as_deref(),
        r2: r2.as_deref(),
        reference: reference.as_deref(),
        sample_id,
        params: params_ref,
    })
    .with_context(|| format!("plan BAM benchmark command row `{stage_id}` / `{tool_id}`"))?;
    Ok(plan.command.template)
}

fn benchmark_command_out_dir(domain: &str, stage_id: &str, tool_id: &str) -> Result<PathBuf> {
    let stage_path = stage_id.replace('.', "/");
    Ok(PathBuf::from("benchmarks/readiness/stage-tool-commands")
        .join(domain)
        .join(stage_path)
        .join(tool_id))
}

fn resolve_repo_input_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn domain_label(domain: BenchLocalDomain) -> &'static str {
    match domain {
        BenchLocalDomain::Fastq => "FASTQ",
        BenchLocalDomain::Bam => "BAM",
        BenchLocalDomain::Vcf => "VCF",
    }
}

fn fastq_stage_params_from_plan(
    stage_id: &str,
    plan: &StagePlanV1,
) -> Result<Option<bijux_dna_planner_fastq::FastqStageParameters>> {
    let params = &plan.params;
    let effective_params = &plan.effective_params;
    Ok(match stage_id {
        "fastq.cluster_otus" => Some(bijux_dna_planner_fastq::FastqStageParameters::ClusterOtus(
            bijux_dna_planner_fastq::ClusterOtusStageParams {
                otu_identity: json_f64(params, "otu_identity").unwrap_or(
                    bijux_dna_planner_fastq::ClusterOtusStageParams::baseline().otu_identity,
                ),
                threads: json_u32(params, "threads"),
            },
        )),
        "fastq.correct_errors" => {
            let baseline = bijux_dna_planner_fastq::CorrectErrorsStageParams::baseline();
            Some(bijux_dna_planner_fastq::FastqStageParameters::CorrectErrors(
                bijux_dna_planner_fastq::CorrectErrorsStageParams {
                    threads: json_u32(params, "threads"),
                    quality_encoding: parse_quality_encoding(
                        json_string(params, "quality_encoding")
                            .as_deref()
                            .unwrap_or("phred33"),
                    )?,
                    kmer_size: json_u32(params, "kmer_size"),
                    musket_kmer_budget: json_u64(params, "musket_kmer_budget"),
                    genome_size: json_u64(params, "genome_size"),
                    max_memory_gb: json_u32(params, "max_memory_gb"),
                    trusted_kmer_artifact: json_path(params, "trusted_kmer_artifact"),
                    conservative_mode: json_bool(params, "conservative_mode")
                        .unwrap_or(baseline.conservative_mode),
                },
            ))
        }
        "fastq.deplete_host" => Some(bijux_dna_planner_fastq::FastqStageParameters::DepleteHost(
            bijux_dna_planner_fastq::DepleteHostStageParams {
                host_identity_threshold: json_f64(params, "host_identity_threshold").unwrap_or(
                    bijux_dna_planner_fastq::DepleteHostStageParams::baseline()
                        .host_identity_threshold,
                ),
                retain_unmapped_only: json_bool(params, "retain_unmapped_only").unwrap_or(
                    bijux_dna_planner_fastq::DepleteHostStageParams::baseline()
                        .retain_unmapped_only,
                ),
                threads: json_u32(params, "threads"),
            },
        )),
        "fastq.deplete_reference_contaminants" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::DepleteReferenceContaminants(
                bijux_dna_planner_fastq::DepleteReferenceContaminantsStageParams {
                    decoy_mode: json_string(params, "decoy_mode").unwrap_or_else(|| {
                        bijux_dna_planner_fastq::DepleteReferenceContaminantsStageParams::baseline(
                        )
                        .decoy_mode
                    }),
                    threads: json_u32(params, "threads"),
                },
            ),
        ),
        "fastq.deplete_rrna" => Some(bijux_dna_planner_fastq::FastqStageParameters::DepleteRrna(
            bijux_dna_planner_fastq::DepleteRrnaStageParams {
                rrna_db: json_string(params, "rrna_db").unwrap_or_else(|| {
                    bijux_dna_planner_fastq::DepleteRrnaStageParams::baseline().rrna_db
                }),
                min_identity: json_f64(params, "min_identity").unwrap_or(
                    bijux_dna_planner_fastq::DepleteRrnaStageParams::baseline().min_identity,
                ),
                threads: json_u32(params, "threads"),
            },
        )),
        "fastq.detect_adapters" => {
            Some(bijux_dna_planner_fastq::FastqStageParameters::DetectAdapters(
                bijux_dna_planner_fastq::DetectAdaptersStageParams {
                    threads: json_u32(params, "threads"),
                },
            ))
        }
        "fastq.detect_duplicates_premerge" | "fastq.estimate_library_complexity_prealign" => None,
        "fastq.extract_umis" => Some(bijux_dna_planner_fastq::FastqStageParameters::ExtractUmis(
            bijux_dna_planner_fastq::ExtractUmisStageParams {
                threads: json_u32(params, "threads"),
                umi_pattern: json_string(params, "umi_pattern"),
                extraction_location: json_string(params, "extraction_location"),
                read_name_transform: json_string(params, "read_name_transform"),
                failed_extraction_policy: json_string(params, "failed_extraction_policy"),
                grouping_policy: json_string(params, "grouping_policy"),
                downstream_dedup_policy: json_string(params, "downstream_dedup_policy"),
                downstream_propagation: json_string(params, "downstream_propagation"),
            },
        )),
        "fastq.filter_low_complexity" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::FilterLowComplexity(
                bijux_dna_planner_fastq::FilterLowComplexityStageParams {
                    entropy_threshold: json_f64(params, "entropy_threshold"),
                    polyx_threshold: json_u32(params, "polyx_threshold"),
                },
            ),
        ),
        "fastq.filter_reads" => Some(bijux_dna_planner_fastq::FastqStageParameters::FilterReads(
            bijux_dna_planner_fastq::FilterReadsStageParams {
                threads: json_u32(params, "threads"),
                max_n: json_u32(params, "max_n"),
                max_n_fraction: json_f64(params, "max_n_fraction"),
                max_n_count: json_u32(params, "max_n_count"),
                low_complexity_threshold: json_f64(params, "low_complexity_threshold"),
                entropy_threshold: json_f64(params, "entropy_threshold"),
                kmer_ref: json_path(params, "kmer_ref"),
                polyx_policy: json_string(params, "polyx_policy"),
            },
        )),
        "fastq.infer_asvs" => Some(bijux_dna_planner_fastq::FastqStageParameters::InferAsvs(
            bijux_dna_planner_fastq::InferAsvsStageParams {
                denoising_method: json_string(params, "denoising_method").unwrap_or_else(|| {
                    bijux_dna_planner_fastq::InferAsvsStageParams::baseline().denoising_method
                }),
                pooling_mode: json_string(params, "pooling_mode").unwrap_or_else(|| {
                    bijux_dna_planner_fastq::InferAsvsStageParams::baseline().pooling_mode
                }),
                chimera_policy: json_string(params, "chimera_policy").unwrap_or_else(|| {
                    bijux_dna_planner_fastq::InferAsvsStageParams::baseline().chimera_policy
                }),
                threads: json_u32(params, "threads"),
            },
        )),
        "fastq.merge_pairs" => Some(bijux_dna_planner_fastq::FastqStageParameters::MergePairs(
            bijux_dna_planner_fastq::MergePairsStageParams {
                threads: json_u32(params, "threads"),
                merge_overlap: json_u32(params, "merge_overlap"),
                min_len: json_u32(params, "min_len"),
                unmerged_read_policy: parse_unmerged_read_policy(
                    json_string(params, "unmerged_read_policy")
                        .as_deref()
                        .unwrap_or("emit_unmerged_pairs"),
                )?,
            },
        )),
        "fastq.normalize_abundance" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::NormalizeAbundance(
                bijux_dna_planner_fastq::NormalizeAbundanceStageParams {
                    method: json_string(params, "method").unwrap_or_else(|| {
                        bijux_dna_planner_fastq::NormalizeAbundanceStageParams::baseline().method
                    }),
                },
            ),
        ),
        "fastq.normalize_primers" => {
            let baseline = bijux_dna_planner_fastq::NormalizePrimersStageParams::baseline();
            Some(bijux_dna_planner_fastq::FastqStageParameters::NormalizePrimers(
                bijux_dna_planner_fastq::NormalizePrimersStageParams {
                    primer_set_id: json_string(params, "primer_set_id")
                        .unwrap_or(baseline.primer_set_id),
                    marker_id: json_string(params, "marker_id"),
                    primer_fasta: json_path(params, "primer_fasta"),
                    orientation_policy: json_string(params, "orientation_policy")
                        .unwrap_or(baseline.orientation_policy),
                    max_mismatch_rate: json_f64(params, "max_mismatch_rate")
                        .unwrap_or(baseline.max_mismatch_rate),
                    min_overlap_bp: json_u32(params, "min_overlap_bp")
                        .unwrap_or(baseline.min_overlap_bp),
                    strict_5p_anchor: json_bool(params, "strict_5p_anchor")
                        .unwrap_or(baseline.strict_5p_anchor),
                    allow_iupac_codes: json_bool(params, "allow_iupac_codes")
                        .unwrap_or(baseline.allow_iupac_codes),
                },
            ))
        }
        "fastq.profile_read_lengths" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::ProfileReadLengths(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!("decode effective params for `{stage_id}` as FastqReadLengthProfileParams")
                })?,
            ),
        ),
        "fastq.profile_overrepresented_sequences" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::ProfileOverrepresented(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!(
                        "decode effective params for `{stage_id}` as FastqOverrepresentedProfileParams"
                    )
                })?,
            ),
        ),
        "fastq.profile_reads" => Some(bijux_dna_planner_fastq::FastqStageParameters::ProfileReads(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as FastqStatsParams")
            })?,
        )),
        "fastq.index_reference" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::IndexReference(
                bijux_dna_planner_fastq::IndexReferenceStageParams {
                    threads: Some(
                        serde_json::from_value::<
                            bijux_dna_domain_fastq::params::reference_index::ReferenceIndexEffectiveParams,
                        >(effective_params.clone())
                        .with_context(|| {
                            format!(
                                "decode effective params for `{stage_id}` as ReferenceIndexEffectiveParams"
                            )
                        })?
                        .threads,
                    ),
                },
            ),
        ),
        "fastq.remove_chimeras" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::RemoveChimeras(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!(
                        "decode effective params for `{stage_id}` as ChimeraDetectionEffectiveParams"
                    )
                })?,
            ),
        ),
        "fastq.remove_duplicates" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::RemoveDuplicates(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!(
                        "decode effective params for `{stage_id}` as RemoveDuplicatesEffectiveParams"
                    )
                })?,
            ),
        ),
        "fastq.screen_taxonomy" => Some(bijux_dna_planner_fastq::FastqStageParameters::Screen(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as ScreenEffectiveParams")
            })?,
        )),
        "fastq.trim_polyg_tails" => Some(
            bijux_dna_planner_fastq::FastqStageParameters::TrimPolygTails(
                serde_json::from_value(effective_params.clone()).with_context(|| {
                    format!("decode effective params for `{stage_id}` as TrimPolygTailsParams")
                })?,
            ),
        ),
        "fastq.trim_reads" => Some(bijux_dna_planner_fastq::FastqStageParameters::Trim(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as TrimEffectiveParams")
            })?,
        )),
        "fastq.trim_terminal_damage" => {
            let baseline = bijux_dna_planner_fastq::TrimTerminalDamageStageParams::baseline();
            Some(
                bijux_dna_planner_fastq::FastqStageParameters::TrimTerminalDamage(
                    bijux_dna_planner_fastq::TrimTerminalDamageStageParams {
                        threads: json_u32(params, "threads"),
                        damage_mode: parse_damage_mode(
                            json_string(params, "damage_mode")
                                .as_deref()
                                .unwrap_or("ancient"),
                        )?,
                        execution_policy: json_string(params, "execution_policy")
                            .map(|value| parse_terminal_damage_execution_policy(&value))
                            .transpose()?,
                        trim_5p_bases: json_u32(params, "trim_5p_bases")
                            .unwrap_or(baseline.trim_5p_bases),
                        trim_3p_bases: json_u32(params, "trim_3p_bases")
                            .unwrap_or(baseline.trim_3p_bases),
                    },
                ),
            )
        }
        "fastq.validate_reads" => Some(bijux_dna_planner_fastq::FastqStageParameters::Validate(
            serde_json::from_value(effective_params.clone()).with_context(|| {
                format!("decode effective params for `{stage_id}` as ValidateEffectiveParams")
            })?,
        )),
        other => {
            return Err(anyhow!(
                "FASTQ benchmark command renderer does not support stage `{other}`"
            ));
        }
    })
}

fn project_fastq_benchmark_params_for_tool(
    stage_id: &str,
    tool_id: &str,
    params: Option<bijux_dna_planner_fastq::FastqStageParameters>,
) -> Option<bijux_dna_planner_fastq::FastqStageParameters> {
    match params {
        Some(bijux_dna_planner_fastq::FastqStageParameters::Trim(params))
            if stage_id == "fastq.trim_reads" =>
        {
            project_trim_benchmark_params_for_tool(tool_id, params)
        }
        Some(bijux_dna_planner_fastq::FastqStageParameters::Screen(params))
            if stage_id == "fastq.screen_taxonomy" =>
        {
            Some(project_screen_benchmark_params_for_tool(tool_id, params))
        }
        Some(bijux_dna_planner_fastq::FastqStageParameters::RemoveDuplicates(params))
            if stage_id == "fastq.remove_duplicates" =>
        {
            project_remove_duplicates_benchmark_params_for_tool(tool_id, params)
        }
        Some(bijux_dna_planner_fastq::FastqStageParameters::MergePairs(params))
            if stage_id == "fastq.merge_pairs" =>
        {
            Some(project_merge_pairs_benchmark_params_for_tool(tool_id, params))
        }
        Some(bijux_dna_planner_fastq::FastqStageParameters::CorrectErrors(params))
            if stage_id == "fastq.correct_errors" =>
        {
            Some(project_correct_errors_benchmark_params_for_tool(tool_id, params))
        }
        Some(bijux_dna_planner_fastq::FastqStageParameters::FilterLowComplexity(params))
            if stage_id == "fastq.filter_low_complexity" =>
        {
            Some(project_filter_low_complexity_benchmark_params_for_tool(tool_id, params))
        }
        other => other,
    }
}

fn project_trim_benchmark_params_for_tool(
    tool_id: &str,
    mut params: bijux_dna_domain_fastq::params::trim::TrimEffectiveParams,
) -> Option<bijux_dna_planner_fastq::FastqStageParameters> {
    let supports_adapter_bank = matches!(
        tool_id,
        "fastp"
            | "cutadapt"
            | "atropos"
            | "adapterremoval"
            | "alientrimmer"
            | "trim_galore"
            | "fastx_clipper"
            | "skewer"
            | "leehom"
    );
    let supports_polyx_bank = tool_id == "fastp";
    let supports_contaminant_bank = tool_id == "bbduk";
    let supports_length_and_quality = matches!(
        tool_id,
        "fastp"
            | "cutadapt"
            | "atropos"
            | "bbduk"
            | "adapterremoval"
            | "trimmomatic"
            | "alientrimmer"
            | "trim_galore"
            | "skewer"
            | "prinseq"
    );
    let supports_length_without_quality = matches!(tool_id, "seqkit" | "seqpurge");

    if matches!(params.adapter_policy.as_str(), "bank" | "ancient_strict") && !supports_adapter_bank
    {
        params.adapter_policy = "none".to_string();
    }
    if params.polyx_policy.as_deref().is_some_and(|policy| policy != "none") && !supports_polyx_bank
    {
        params.polyx_policy = Some("none".to_string());
    }
    if params.contaminant_policy.as_deref() == Some("bank") && !supports_contaminant_bank {
        params.contaminant_policy = Some("none".to_string());
    }
    if !supports_length_and_quality {
        params.q_cutoff = None;
        if !supports_length_without_quality {
            return None;
        }
    }

    Some(bijux_dna_planner_fastq::FastqStageParameters::Trim(params))
}

fn project_screen_benchmark_params_for_tool(
    tool_id: &str,
    mut params: bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams,
) -> bijux_dna_planner_fastq::FastqStageParameters {
    use bijux_dna_domain_fastq::params::screen::{
        TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyReportFormat,
    };

    let (classifier, report_format, assignment_format) = match tool_id {
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
        _ => (params.classifier, params.report_format, params.assignment_format),
    };
    params.classifier = classifier;
    params.report_format = report_format;
    params.assignment_format = assignment_format;
    bijux_dna_planner_fastq::FastqStageParameters::Screen(params)
}

fn project_remove_duplicates_benchmark_params_for_tool(
    tool_id: &str,
    params: bijux_dna_domain_fastq::params::remove_duplicates::RemoveDuplicatesEffectiveParams,
) -> Option<bijux_dna_planner_fastq::FastqStageParameters> {
    if tool_id == "clumpify" {
        Some(bijux_dna_planner_fastq::FastqStageParameters::RemoveDuplicates(params))
    } else {
        None
    }
}

fn project_merge_pairs_benchmark_params_for_tool(
    tool_id: &str,
    mut params: bijux_dna_planner_fastq::MergePairsStageParams,
) -> bijux_dna_planner_fastq::FastqStageParameters {
    if !matches!(tool_id, "adapterremoval" | "pear" | "vsearch" | "bbmerge") {
        params.min_len = None;
    }
    if tool_id == "leehom" {
        params.merge_overlap = None;
    }
    bijux_dna_planner_fastq::FastqStageParameters::MergePairs(params)
}

fn project_correct_errors_benchmark_params_for_tool(
    tool_id: &str,
    mut params: bijux_dna_planner_fastq::CorrectErrorsStageParams,
) -> bijux_dna_planner_fastq::FastqStageParameters {
    if tool_id == "lighter" && params.genome_size.is_none() {
        // Matches the governed `fastq_correct_errors_surface` benchmark suite binding.
        params.genome_size = Some(2_500_000);
    }
    if tool_id == "musket" && params.musket_kmer_budget.is_none() {
        // Matches the admitted musket benchmark fanout contract for command rendering.
        params.musket_kmer_budget = Some(536_870_912);
    }
    bijux_dna_planner_fastq::FastqStageParameters::CorrectErrors(
        bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::project_correct_options_for_tool(
            tool_id,
            &params,
        ),
    )
}

fn project_filter_low_complexity_benchmark_params_for_tool(
    tool_id: &str,
    mut params: bijux_dna_planner_fastq::FilterLowComplexityStageParams,
) -> bijux_dna_planner_fastq::FastqStageParameters {
    if tool_id != "bbduk" {
        params.polyx_threshold = None;
    }
    bijux_dna_planner_fastq::FastqStageParameters::FilterLowComplexity(params)
}

fn project_bam_benchmark_params_for_tool(
    stage_id: &str,
    tool_id: &str,
    plan: &StagePlanV1,
) -> Option<serde_json::Value> {
    let params = if !plan.effective_params.is_null() {
        Some(plan.effective_params.clone())
    } else if !plan.params.is_null() {
        Some(plan.params.clone())
    } else {
        None
    }?;

    match stage_id {
        "bam.contamination" => project_bam_contamination_params_for_tool(tool_id, params),
        _ => Some(params),
    }
}

fn project_bam_contamination_params_for_tool(
    tool_id: &str,
    mut params: serde_json::Value,
) -> Option<serde_json::Value> {
    let object = params.as_object_mut()?;
    match tool_id {
        "schmutzi" => {
            object.insert("scope".to_string(), serde_json::Value::String("mito".to_string()));
        }
        "verifybamid2" | "contammix" => {
            object.insert("scope".to_string(), serde_json::Value::String("nuclear".to_string()));
        }
        _ => {}
    }
    Some(params)
}

fn find_fastq_input<'a>(plan: &'a StagePlanV1, artifact_name: &str) -> Option<&'a Path> {
    find_first_input(plan, &[artifact_name])
}

fn find_first_input<'a>(plan: &'a StagePlanV1, artifact_names: &[&str]) -> Option<&'a Path> {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact_names.iter().any(|name| artifact.name.as_str() == *name))
        .map(|artifact| artifact.path.as_path())
}

fn find_bam_input(plan: &StagePlanV1) -> Option<&Path> {
    plan.io.inputs.iter().find_map(|artifact| {
        if matches!(artifact.role, ArtifactRole::Bam | ArtifactRole::DedupBam) {
            Some(artifact.path.as_path())
        } else {
            None
        }
    })
}

fn find_bam_index_input(plan: &StagePlanV1) -> Option<&Path> {
    plan.io.inputs.iter().find_map(|artifact| {
        if artifact.role == ArtifactRole::Index
            && (artifact.name.as_str().contains("bam") || artifact.name.as_str().contains("bai"))
        {
            Some(artifact.path.as_path())
        } else {
            None
        }
    })
}

fn find_reference_fasta(plan: &StagePlanV1) -> Option<&Path> {
    plan.io.inputs.iter().find_map(|artifact| {
        if artifact.role == ArtifactRole::Reference {
            Some(artifact.path.as_path())
        } else {
            None
        }
    })
}

fn find_bam_corpus_reference(repo_root: &Path, plan: &StagePlanV1) -> Option<PathBuf> {
    #[derive(serde::Deserialize)]
    struct BamCorpusManifestReference {
        reference_fasta: String,
    }

    let bam_path = find_bam_input(plan)?;
    let bam_abs =
        if bam_path.is_absolute() { bam_path.to_path_buf() } else { repo_root.join(bam_path) };
    let corpus_root =
        bam_abs.ancestors().find(|ancestor| ancestor.join("manifest.toml").is_file())?;
    let manifest_path = corpus_root.join("manifest.toml");
    let raw = std::fs::read_to_string(&manifest_path).ok()?;
    let manifest: BamCorpusManifestReference = toml::from_str(&raw).ok()?;
    Some(corpus_root.join(manifest.reference_fasta))
}

fn explicit_input_source_tool_id(
    stage_id: &str,
    tool_id: &str,
    artifact: &bijux_dna_stage_contract::ArtifactRef,
) -> Option<String> {
    if artifact.role == ArtifactRole::Index && artifact.name.as_str() == "reference_index" {
        let compatible_backends = bijux_dna_domain_fastq::reference_index_backends_for_tool(
            &ToolId::new(tool_id.to_string()),
        );
        if compatible_backends.len() == 1 {
            return Some(compatible_backends[0].as_str().to_string());
        }
        if matches!(stage_id, "fastq.deplete_host" | "fastq.deplete_reference_contaminants") {
            return compatible_backends.first().map(|tool_id| tool_id.as_str().to_string());
        }
    }
    None
}

fn readiness_kind_label(readiness_kind: LocalStageReadinessKind) -> &'static str {
    match readiness_kind {
        LocalStageReadinessKind::DryRun => "dry_run",
        LocalStageReadinessKind::Smoke => "smoke",
        LocalStageReadinessKind::DryOrSmoke => "dry_or_smoke",
    }
}

fn ensure_unique_rows(rows: &[BenchmarkCommandRow]) -> Result<()> {
    let mut seen = BTreeSet::<(String, String)>::new();
    for row in rows {
        let pair = (row.stage_id.clone(), row.tool_id.clone());
        if !seen.insert(pair.clone()) {
            return Err(anyhow!(
                "benchmark command rows repeat stage/tool pair `{}` / `{}`",
                pair.0,
                pair.1
            ));
        }
    }
    Ok(())
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(serde_json::Value::as_str).map(ToString::to_string)
}

fn json_u32(value: &serde_json::Value, key: &str) -> Option<u32> {
    value.get(key).and_then(serde_json::Value::as_u64).and_then(|raw| u32::try_from(raw).ok())
}

fn json_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    value.get(key).and_then(serde_json::Value::as_u64)
}

fn json_f64(value: &serde_json::Value, key: &str) -> Option<f64> {
    value.get(key).and_then(serde_json::Value::as_f64)
}

fn json_bool(value: &serde_json::Value, key: &str) -> Option<bool> {
    value.get(key).and_then(serde_json::Value::as_bool)
}

fn json_path(value: &serde_json::Value, key: &str) -> Option<PathBuf> {
    json_string(value, key).map(PathBuf::from)
}

fn parse_quality_encoding(raw: &str) -> Result<QualityEncoding> {
    match raw {
        "phred33" => Ok(QualityEncoding::Phred33),
        "phred64" => Ok(QualityEncoding::Phred64),
        other => Err(anyhow!("unsupported FASTQ quality_encoding `{other}`")),
    }
}

fn parse_unmerged_read_policy(raw: &str) -> Result<UnmergedReadPolicy> {
    match raw {
        "emit_unmerged_pairs" => Ok(UnmergedReadPolicy::EmitUnmergedPairs),
        "omit_unmerged_pairs" => Ok(UnmergedReadPolicy::OmitUnmergedPairs),
        other => Err(anyhow!("unsupported fastq.merge_pairs unmerged_read_policy `{other}`")),
    }
}

fn parse_damage_mode(raw: &str) -> Result<DamageMode> {
    match raw {
        "ancient" => Ok(DamageMode::Ancient),
        "udg_trimmed" => Ok(DamageMode::UdgTrimmed),
        other => Err(anyhow!("unsupported fastq.trim_terminal_damage damage_mode `{other}`")),
    }
}

fn parse_terminal_damage_execution_policy(raw: &str) -> Result<TerminalDamageExecutionPolicy> {
    match raw {
        "explicit_terminal_trim" => Ok(TerminalDamageExecutionPolicy::ExplicitTerminalTrim),
        "preserve_udg_trimmed_ends" => Ok(TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds),
        other => Err(anyhow!("unsupported fastq.trim_terminal_damage execution_policy `{other}`")),
    }
}

fn load_default_fastq_adapter_bank_context(repo_root: &Path) -> Result<Option<serde_json::Value>> {
    let bank_path = repo_root.join(bijux_dna_domain_fastq::adapter_bank_path());
    let presets_path = repo_root.join(bijux_dna_domain_fastq::adapter_presets_path());
    let bank = bijux_dna_domain_fastq::load_adapter_bank(&bank_path)
        .with_context(|| format!("load {}", bank_path.display()))?;
    let presets = bijux_dna_domain_fastq::load_adapter_presets(&presets_path, &bank)
        .with_context(|| format!("load {}", presets_path.display()))?;
    let selection = bijux_dna_domain_fastq::banks::AdapterSelection {
        bank,
        presets,
        preset_name: bijux_dna_domain_fastq::banks::DEFAULT_ADAPTER_PRESET.to_string(),
        bank_checksum: bijux_dna_infra::hash_file_sha256(&bank_path)?,
        presets_checksum: bijux_dna_infra::hash_file_sha256(&presets_path)?,
    };
    let effective =
        bijux_dna_domain_fastq::banks::resolve_effective_adapters(&selection, &[], &[])?;
    Ok(Some(bijux_dna_domain_fastq::banks::adapter_bank_provenance_json(
        &selection,
        &effective,
        &[],
        &[],
    )))
}

fn load_default_fastq_polyx_bank_context(repo_root: &Path) -> Result<Option<serde_json::Value>> {
    let bank_path = repo_root.join(bijux_dna_domain_fastq::polyx_bank_path());
    let presets_path = repo_root.join(bijux_dna_domain_fastq::polyx_presets_path());
    let bank = bijux_dna_domain_fastq::load_polyx_bank(&bank_path)
        .with_context(|| format!("load {}", bank_path.display()))?;
    let presets = bijux_dna_domain_fastq::load_polyx_presets(&presets_path, &bank)
        .with_context(|| format!("load {}", presets_path.display()))?;
    let selection = bijux_dna_domain_fastq::banks::PolyxSelection {
        bank,
        presets,
        preset_name: bijux_dna_domain_fastq::banks::DEFAULT_POLYX_PRESET.to_string(),
        bank_checksum: bijux_dna_infra::hash_file_sha256(&bank_path)?,
        presets_checksum: bijux_dna_infra::hash_file_sha256(&presets_path)?,
    };
    let effective = bijux_dna_domain_fastq::banks::resolve_effective_polyx(&selection)?;
    Ok(Some(bijux_dna_domain_fastq::banks::polyx_bank_provenance_json(&selection, &effective)))
}

fn load_default_fastq_contaminant_bank_context(
    repo_root: &Path,
) -> Result<Option<serde_json::Value>> {
    let motifs_path = repo_root.join(bijux_dna_domain_fastq::contaminant_motifs_path());
    let presets_path = repo_root.join(bijux_dna_domain_fastq::contaminant_presets_path());
    let references_dir = repo_root.join(bijux_dna_domain_fastq::contaminant_references_dir());
    let motifs = bijux_dna_domain_fastq::load_contaminant_motifs(&motifs_path)
        .with_context(|| format!("load {}", motifs_path.display()))?;
    let presets =
        bijux_dna_domain_fastq::load_contaminant_presets(&presets_path, &motifs, &references_dir)
            .with_context(|| format!("load {}", presets_path.display()))?;
    let selection = bijux_dna_domain_fastq::banks::ContaminantSelection {
        motifs,
        presets,
        preset_name: bijux_dna_domain_fastq::banks::DEFAULT_CONTAMINANT_PRESET.to_string(),
        motifs_checksum: bijux_dna_infra::hash_file_sha256(&motifs_path)?,
        presets_checksum: bijux_dna_infra::hash_file_sha256(&presets_path)?,
    };
    let effective = bijux_dna_domain_fastq::resolve_contaminant_preset(
        &selection.motifs,
        &selection.presets,
        &selection.preset_name,
        &references_dir,
    )?;
    let enabled_entries: Vec<serde_json::Value> = effective
        .motifs
        .iter()
        .map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "sequence": entry.sequence,
                "rationale": entry.rationale,
                "source": entry.source,
            })
        })
        .collect();
    let references = effective
        .references
        .iter()
        .map(|reference| {
            let path = references_dir.join(&reference.file);
            let fasta = std::fs::read_to_string(&path)
                .with_context(|| format!("read {}", path.display()))?;
            let sha256 = bijux_dna_infra::hash_file_sha256(&path)
                .with_context(|| format!("hash {}", path.display()))?;
            Ok(serde_json::json!({
                "id": reference.id,
                "file": reference.file,
                "sha256": sha256,
                "rationale": reference.rationale,
                "source": reference.source,
                "fasta": fasta,
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(serde_json::json!({
        "bank_id": selection.motifs.bank_id,
        "bank_version": selection.motifs.version,
        "bank_hash": selection.motifs_checksum,
        "presets_hash": selection.presets_checksum,
        "preset": selection.preset_name,
        "preset_hash": effective.preset_hash,
        "enabled_entries": enabled_entries,
        "references": references,
    })))
}
