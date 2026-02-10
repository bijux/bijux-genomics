use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::bench::fastq_args as engine_args;
use bijux_dna_api::v1::api::run::{StageId, ToolId};

use crate::commands::cli::env::registry_tools_for_stage;
use crate::commands::cli::parse::{
    BamCommand, BenchFastqCorrectArgs, BenchFastqFilterArgs, BenchFastqMergeArgs,
    BenchFastqPreprocessArgs, BenchFastqQcPostArgs, BenchFastqScreenArgs, BenchFastqStatsArgs,
    BenchFastqTrimArgs, BenchFastqUmiArgs, BenchFastqValidateArgs, CommonArgs, DnaCommand,
    FastqCommand, FastqPreprocessArgs, FastqTrimArgs, FastqValidateArgs,
};

#[must_use]
pub fn resolve_stage_tool(command: &DnaCommand) -> (StageId, ToolId, CommonArgs) {
    match command {
        DnaCommand::Fastq { command } => match command {
            FastqCommand::ListStages
            | FastqCommand::Stages
            | FastqCommand::Doctor
            | FastqCommand::ListTools { .. }
            | FastqCommand::Explain { .. }
            | FastqCommand::Benchmark(_)
            | FastqCommand::Analyze(_)
            | FastqCommand::Compare(_)
            | FastqCommand::Run(_) => (
                StageId::from_static("fastq.trim"),
                ToolId::from_static("fastp"),
                CommonArgs::default(),
            ),
            FastqCommand::Trim(args) => (
                StageId::from_static("fastq.trim"),
                ToolId::from_static("fastp"),
                args.common.clone(),
            ),
            FastqCommand::ValidatePre(args) => (
                StageId::from_static("fastq.validate_pre"),
                ToolId::from_static("fastqvalidator"),
                args.common.clone(),
            ),
            FastqCommand::StatsNeutral(common) => (
                StageId::from_static("fastq.stats_neutral"),
                ToolId::from_static("seqkit_stats"),
                common.clone(),
            ),
            FastqCommand::Filter(args) => (
                StageId::from_static("fastq.trim"),
                ToolId::from_static("fastp"),
                args.common.clone(),
            ),
            FastqCommand::Merge(common)
            | FastqCommand::Contam(common)
            | FastqCommand::Umi(common)
            | FastqCommand::ErrorCorrect(common)
            | FastqCommand::Qc(common)
            | FastqCommand::Align(common) => (
                StageId::from_static("fastq.trim"),
                ToolId::from_static("fastp"),
                common.clone(),
            ),
            FastqCommand::Preprocess(args) => (
                StageId::from_static("fastq.preprocess"),
                ToolId::from_static("fastp"),
                args.common.clone(),
            ),
        },
        DnaCommand::Bam { command } => match command {
            BamCommand::ListStages | BamCommand::Explain { .. } => (
                StageId::from_static("bam.validate"),
                ToolId::from_static("samtools"),
                CommonArgs::default(),
            ),
            BamCommand::Run(args) => (
                StageId::new(args.stage.stage().as_str()),
                ToolId::new(args.tool.clone().unwrap_or_else(|| "samtools".to_string())),
                CommonArgs::default(),
            ),
        },
        _ => (
            StageId::from_static("fastq.trim"),
            ToolId::from_static("fastp"),
            CommonArgs::default(),
        ),
    }
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_trim(args: &BenchFastqTrimArgs) -> Result<engine_args::BenchFastqTrimArgs> {
    Ok(engine_args::BenchFastqTrimArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.trim", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_validate(
    args: &BenchFastqValidateArgs,
) -> Result<engine_args::BenchFastqValidateArgs> {
    Ok(engine_args::BenchFastqValidateArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.validate_pre", &args.tools)?,
        explain: args.explain,
        strict: args.strict,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_filter(args: &BenchFastqFilterArgs) -> Result<engine_args::BenchFastqFilterArgs> {
    Ok(engine_args::BenchFastqFilterArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.filter", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        max_n: args.max_n,
        low_complexity_threshold: args.low_complexity_threshold,
        kmer_ref: args.kmer_ref.clone(),
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_merge(args: &BenchFastqMergeArgs) -> Result<engine_args::BenchFastqMergeArgs> {
    Ok(engine_args::BenchFastqMergeArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.merge", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_correct(
    args: &BenchFastqCorrectArgs,
) -> Result<engine_args::BenchFastqCorrectArgs> {
    Ok(engine_args::BenchFastqCorrectArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.correct", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_qc_post(
    args: &BenchFastqQcPostArgs,
) -> Result<engine_args::BenchFastqQcPostArgs> {
    Ok(engine_args::BenchFastqQcPostArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.qc_post", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_umi(args: &BenchFastqUmiArgs) -> Result<engine_args::BenchFastqUmiArgs> {
    Ok(engine_args::BenchFastqUmiArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.umi", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_screen(args: &BenchFastqScreenArgs) -> Result<engine_args::BenchFastqScreenArgs> {
    Ok(engine_args::BenchFastqScreenArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.screen", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

/// # Errors
/// Returns an error if tool mode cannot be resolved for this stage.
pub fn bench_args_stats(args: &BenchFastqStatsArgs) -> Result<engine_args::BenchFastqStatsArgs> {
    Ok(engine_args::BenchFastqStatsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: resolve_bench_tools("fastq.stats_neutral", &args.tools)?,
        explain: args.explain,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
    })
}

fn resolve_bench_tools(stage: &str, raw_tools: &[String]) -> Result<Vec<String>> {
    let mut normalized = raw_tools
        .iter()
        .map(|tool| tool.trim().to_lowercase())
        .filter(|tool| !tool.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    let mode = if normalized.is_empty() {
        "auto"
    } else if normalized.len() == 1 && normalized[0] == "all" {
        "all"
    } else if normalized.len() == 1 && normalized[0] == "auto" {
        "auto"
    } else {
        if normalized.iter().any(|v| v == "auto" || v == "all") {
            return Err(anyhow!(
                "--tools accepts either `auto`, `all`, or an explicit CSV list"
            ));
        }
        "csv"
    };

    let registry_path = std::env::current_dir()
        .map_err(|err| anyhow!("resolve cwd: {err}"))?
        .join("configs")
        .join("tool_registry.toml");
    let all_tools = registry_tools_for_stage(&registry_path, stage, "all")?;
    if all_tools.is_empty() {
        return Err(anyhow!("no compatible tools found for stage `{stage}`"));
    }
    let mut selected = match mode {
        "auto" => registry_tools_for_stage(&registry_path, stage, "primary")?,
        "all" => all_tools.clone(),
        _ => normalized,
    };
    selected.sort();
    selected.dedup();

    if selected.is_empty() {
        return Err(anyhow!("resolved empty tool set for stage `{stage}`"));
    }
    for tool in &selected {
        if !all_tools.contains(tool) {
            return Err(anyhow!(
                "tool `{tool}` is not compatible with stage `{stage}`"
            ));
        }
    }
    Ok(selected)
}

#[must_use]
pub fn bench_args_preprocess(
    args: &BenchFastqPreprocessArgs,
) -> engine_args::BenchFastqPreprocessArgs {
    engine_args::BenchFastqPreprocessArgs {
        sample_id: args.sample_id.clone(),
        profile: args.pipeline_profile.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        strict: args.strict,
        auto: false,
        objective: bijux_dna_api::v1::api::bench::Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        dry_run: false,
        replicates: args.replicates,
        jobs: args.jobs,
        ci_bootstrap: args.ci_bootstrap,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
        enable_contaminant_removal: args.enable_contaminant_removal,
        no_qc_post: args.no_qc_post,
        force_merge: args.force_merge,
        enable_correct: args.enable_correct,
    }
}

/// # Errors
/// Returns an error if CLI arguments are invalid for benchmarking.
pub fn bench_args_from_trim(args: &FastqTrimArgs) -> Result<engine_args::BenchFastqTrimArgs> {
    Ok(engine_args::BenchFastqTrimArgs {
        sample_id: args
            .sample_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("sample_id required for benchmark"))?,
        r1: args
            .r1
            .clone()
            .ok_or_else(|| anyhow::anyhow!("r1 required for benchmark"))?,
        out: args
            .out
            .clone()
            .ok_or_else(|| anyhow::anyhow!("out required for benchmark"))?,
        tools: args.tools.clone(),
        explain: false,
        replicates: 1,
        jobs: 1,
        ci_bootstrap: None,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
    })
}

/// # Errors
/// Returns an error if CLI arguments are invalid for benchmarking.
pub fn bench_args_from_validate(
    args: &FastqValidateArgs,
) -> Result<engine_args::BenchFastqValidateArgs> {
    Ok(engine_args::BenchFastqValidateArgs {
        sample_id: args
            .sample_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("sample_id required for benchmark"))?,
        r1: args
            .r1
            .clone()
            .ok_or_else(|| anyhow::anyhow!("r1 required for benchmark"))?,
        out: args
            .out
            .clone()
            .ok_or_else(|| anyhow::anyhow!("out required for benchmark"))?,
        tools: args.tools.clone(),
        explain: false,
        strict: args.strict,
        replicates: 1,
        jobs: 1,
        ci_bootstrap: None,
    })
}

/// # Errors
/// Returns an error if CLI arguments are invalid for preprocessing.
pub fn preprocess_args_from_cli(
    args: &FastqPreprocessArgs,
) -> Result<engine_args::BenchFastqPreprocessArgs> {
    let sample_id = args
        .sample_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--sample-id is required"))?;
    let r1 = args
        .r1
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--r1 is required"))?;
    let out = args
        .out
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--out is required"))?;
    let mut out_args = engine_args::BenchFastqPreprocessArgs {
        sample_id,
        profile: args.pipeline_profile.clone(),
        r1,
        r2: args.r2.clone(),
        out,
        strict: args.strict,
        auto: args.auto,
        objective: args.objective.into(),
        bench_corpus: args.bench_corpus.map(Into::into),
        allow_partial: args.allow_partial,
        dry_run: args.common.dry_run,
        replicates: 1,
        jobs: args.jobs,
        ci_bootstrap: None,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
        enable_contaminant_removal: args.enable_contaminant_removal,
        no_qc_post: args.no_qc_post,
        force_merge: args.force_merge,
        enable_correct: args.enable_correct,
    };
    if let Some(preset) = args.scientific_preset {
        apply_scientific_preset(preset, &mut out_args);
    }
    Ok(out_args)
}

#[must_use]
pub fn fastq_cross_args_from_cli(
    args: &FastqPreprocessArgs,
) -> bijux_dna_api::v1::api::plan::FastqCrossArgs {
    bijux_dna_api::v1::api::plan::FastqCrossArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        alignment_bam: args.alignment_bam.clone(),
        alignment_bai: args.alignment_bai.clone(),
        alignment_reference: args.alignment_reference.clone(),
        alignment_rg_policy: args.alignment_rg_policy.clone(),
        alignment_meta: args.alignment_meta.clone(),
    }
}

fn apply_scientific_preset(
    preset: crate::commands::cli::parse::ScientificPresetArg,
    args: &mut engine_args::BenchFastqPreprocessArgs,
) {
    match preset {
        crate::commands::cli::parse::ScientificPresetArg::AncientDna => {
            if args.adapter_bank_preset.is_none() {
                args.adapter_bank_preset = Some("ssdna".to_string());
            }
            args.enable_contaminant_removal = true;
            args.force_merge = false;
        }
        crate::commands::cli::parse::ScientificPresetArg::Amplicon => {
            if args.adapter_bank_preset.is_none() {
                args.adapter_bank_preset = Some("illumina-default".to_string());
            }
            args.force_merge = true;
        }
        crate::commands::cli::parse::ScientificPresetArg::Metagenomic => {
            args.enable_contaminant_removal = true;
            args.force_merge = false;
        }
        crate::commands::cli::parse::ScientificPresetArg::WgsStandard => {}
    }
}
