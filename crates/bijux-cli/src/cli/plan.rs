use anyhow::Result;
use bijux_core::{StageId, ToolId};
use bijux_stages_fastq::args as engine_args;

use crate::cli::parse::{
    BenchFastqCorrectArgs, BenchFastqFilterArgs, BenchFastqMergeArgs, BenchFastqPreprocessArgs,
    BenchFastqQcPostArgs, BenchFastqScreenArgs, BenchFastqStatsArgs, BenchFastqTrimArgs,
    BenchFastqUmiArgs, BenchFastqValidateArgs, Commands, CommonArgs, FastqCommand,
    FastqPreprocessArgs, FastqTrimArgs, FastqValidateArgs,
};

#[must_use]
pub fn resolve_stage_tool(command: &Commands) -> (StageId, ToolId, CommonArgs) {
    match command {
        Commands::Fastq { command } => match command {
            FastqCommand::ListStages
            | FastqCommand::Stages
            | FastqCommand::Doctor
            | FastqCommand::ListTools { .. }
            | FastqCommand::Explain { .. }
            | FastqCommand::Benchmark(_)
            | FastqCommand::Analyze(_)
            | FastqCommand::Compare(_)
            | FastqCommand::Run(_) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                CommonArgs::default(),
            ),
            FastqCommand::Trim(args) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                args.common.clone(),
            ),
            FastqCommand::ValidatePre(args) => (
                StageId("fastq.validate_pre".to_string()),
                ToolId("fastqvalidator".to_string()),
                args.common.clone(),
            ),
            FastqCommand::StatsNeutral(common) => (
                StageId("fastq.stats_neutral".to_string()),
                ToolId("seqkit_stats".to_string()),
                common.clone(),
            ),
            FastqCommand::Filter(args) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                args.common.clone(),
            ),
            FastqCommand::Merge(common)
            | FastqCommand::Contam(common)
            | FastqCommand::Umi(common)
            | FastqCommand::ErrorCorrect(common)
            | FastqCommand::Qc(common)
            | FastqCommand::Align(common) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                common.clone(),
            ),
            FastqCommand::Preprocess(args) => (
                StageId("fastq.preprocess".to_string()),
                ToolId("fastp".to_string()),
                args.common.clone(),
            ),
        },
        _ => (
            StageId("fastq.trim".to_string()),
            ToolId("fastp".to_string()),
            CommonArgs::default(),
        ),
    }
}

#[must_use]
pub fn bench_args_trim(args: &BenchFastqTrimArgs) -> engine_args::BenchFastqTrimArgs {
    engine_args::BenchFastqTrimArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
        adapter_bank_preset: args.adapter_bank_preset.clone(),
        adapter_bank: args.adapter_bank.clone(),
        adapter_bank_file: args.adapter_bank_file.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
        polyx_preset: args.polyx_preset.clone(),
        contaminant_preset: args.contaminant_preset.clone(),
    }
}

#[must_use]
pub fn bench_args_validate(args: &BenchFastqValidateArgs) -> engine_args::BenchFastqValidateArgs {
    engine_args::BenchFastqValidateArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        strict: args.strict,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
    }
}

#[must_use]
pub fn bench_args_filter(args: &BenchFastqFilterArgs) -> engine_args::BenchFastqFilterArgs {
    engine_args::BenchFastqFilterArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
        max_n: args.max_n,
        low_complexity_threshold: args.low_complexity_threshold,
        kmer_ref: args.kmer_ref.clone(),
    }
}

#[must_use]
pub fn bench_args_merge(args: &BenchFastqMergeArgs) -> engine_args::BenchFastqMergeArgs {
    engine_args::BenchFastqMergeArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
    }
}

#[must_use]
pub fn bench_args_correct(args: &BenchFastqCorrectArgs) -> engine_args::BenchFastqCorrectArgs {
    engine_args::BenchFastqCorrectArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
    }
}

#[must_use]
pub fn bench_args_qc_post(args: &BenchFastqQcPostArgs) -> engine_args::BenchFastqQcPostArgs {
    engine_args::BenchFastqQcPostArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
    }
}

#[must_use]
pub fn bench_args_umi(args: &BenchFastqUmiArgs) -> engine_args::BenchFastqUmiArgs {
    engine_args::BenchFastqUmiArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
    }
}

#[must_use]
pub fn bench_args_screen(args: &BenchFastqScreenArgs) -> engine_args::BenchFastqScreenArgs {
    engine_args::BenchFastqScreenArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
    }
}

#[must_use]
pub fn bench_args_stats(args: &BenchFastqStatsArgs) -> engine_args::BenchFastqStatsArgs {
    engine_args::BenchFastqStatsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        replicates: args.replicates,
        ci_bootstrap: args.ci_bootstrap,
    }
}

#[must_use]
pub fn bench_args_preprocess(
    args: &BenchFastqPreprocessArgs,
) -> engine_args::BenchFastqPreprocessArgs {
    engine_args::BenchFastqPreprocessArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        strict: args.strict,
        auto: false,
        objective: bijux_core::selection::Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        replicates: args.replicates,
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
        r1,
        r2: args.r2.clone(),
        out,
        strict: args.strict,
        auto: args.auto,
        objective: args.objective.into(),
        bench_corpus: args.bench_corpus.map(Into::into),
        allow_partial: args.allow_partial,
        replicates: 1,
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
    };
    if let Some(preset) = args.scientific_preset {
        apply_scientific_preset(preset, &mut out_args);
    }
    Ok(out_args)
}

fn apply_scientific_preset(
    preset: crate::cli::parse::ScientificPresetArg,
    args: &mut engine_args::BenchFastqPreprocessArgs,
) {
    match preset {
        crate::cli::parse::ScientificPresetArg::AncientDna => {
            if args.adapter_bank_preset.is_none() {
                args.adapter_bank_preset = Some("ssdna".to_string());
            }
            args.enable_contaminant_removal = true;
            args.force_merge = false;
        }
        crate::cli::parse::ScientificPresetArg::Amplicon => {
            if args.adapter_bank_preset.is_none() {
                args.adapter_bank_preset = Some("illumina-default".to_string());
            }
            args.force_merge = true;
        }
        crate::cli::parse::ScientificPresetArg::Metagenomic => {
            args.enable_contaminant_removal = true;
            args.force_merge = false;
        }
        crate::cli::parse::ScientificPresetArg::WgsStandard => {}
    }
}
