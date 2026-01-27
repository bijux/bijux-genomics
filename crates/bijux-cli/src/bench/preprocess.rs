use std::fs;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use crate::utils::{bench_base_dir, hash_file_sha256, input_fastq_stats, output_fastq_stats};

use super::correct::bench_fastq_correct;
use super::filter::bench_fastq_filter;
use super::helpers::{
    compute_run_id, delta_metrics, find_first_fastq, params_hash, prepare_tool_run_dirs,
    tool_run_artifacts_dir, DeltaMetrics, ExecutionManifest,
};
use super::merge::bench_fastq_merge;
use super::stats::bench_fastq_stats;
use super::trim::bench_fastq_trim;
use super::validate::bench_fastq_validate;

#[allow(clippy::too_many_lines)]
pub fn bench_fastq_preprocess(
    catalog: &std::collections::HashMap<String, ToolImageSpec>,
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
    args: &crate::cli::BenchFastqPreprocessArgs,
) -> Result<()> {
    let out_dir = bench_base_dir(&args.out, "preprocess", &args.sample_id);
    fs::create_dir_all(&out_dir).context("create preprocess output dir")?;

    let r1_canon = args.r1.canonicalize().context("resolve r1 path")?;
    let validate_args = crate::cli::BenchFastqValidateArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec![
            "seqtk".to_string(),
            "fastqc".to_string(),
            "fastqvalidator".to_string(),
            "fqtools".to_string(),
        ],
        explain: false,
        strict: args.strict,
    };
    bench_fastq_validate(catalog, platform, runner_override, &validate_args)?;

    let trim_args = crate::cli::BenchFastqTrimArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: vec!["fastp".to_string()],
        explain: false,
    };
    bench_fastq_trim(catalog, platform, runner_override, &trim_args)?;
    let trim_params = serde_json::json!({
        "sample_id": args.sample_id,
        "r1": args.r1,
    });
    let trim_hash = params_hash(&trim_params).unwrap_or_else(|_| "unknown".to_string());
    let trim_run_id = {
        let image_digest = catalog
            .get("fastp")
            .and_then(|spec| spec.digest.as_ref())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let input_hash = hash_file_sha256(&r1_canon)?;
        compute_run_id(
            "fastq.trim",
            "fastp",
            &image_digest,
            &input_hash,
            &trim_hash,
        )
    };
    let trim_out_dir =
        tool_run_artifacts_dir(&args.out, "trim", &args.sample_id, "fastp", &trim_run_id);
    let trimmed_r1 = if trim_out_dir.exists() {
        find_first_fastq(&trim_out_dir).unwrap_or(args.r1.clone())
    } else {
        args.r1.clone()
    };

    if let Some(r2) = &args.r2 {
        let merge_args = crate::cli::BenchFastqMergeArgs {
            sample_id: args.sample_id.clone(),
            r1: trimmed_r1.clone(),
            r2: r2.clone(),
            out: args.out.clone(),
            tools: vec!["pear".to_string()],
            explain: false,
        };
        bench_fastq_merge(catalog, platform, runner_override, &merge_args)?;
    }

    let correct_args = crate::cli::BenchFastqCorrectArgs {
        sample_id: args.sample_id.clone(),
        r1: trimmed_r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: vec!["rcorrector".to_string()],
        explain: false,
    };
    bench_fastq_correct(catalog, platform, runner_override, &correct_args)?;
    let correct_params = serde_json::json!({
        "sample_id": args.sample_id,
        "r1": trimmed_r1,
    });
    let correct_hash = params_hash(&correct_params).unwrap_or_else(|_| "unknown".to_string());
    let correct_run_id = {
        let image_digest = catalog
            .get("rcorrector")
            .and_then(|spec| spec.digest.as_ref())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let input_hash = hash_file_sha256(&trimmed_r1)?;
        compute_run_id(
            "fastq.correct",
            "rcorrector",
            &image_digest,
            &input_hash,
            &correct_hash,
        )
    };
    let correct_out_dir = tool_run_artifacts_dir(
        &args.out,
        "correct",
        &args.sample_id,
        "rcorrector",
        &correct_run_id,
    );
    let corrected_r1 = if correct_out_dir.exists() {
        find_first_fastq(&correct_out_dir).unwrap_or(trimmed_r1.clone())
    } else {
        trimmed_r1.clone()
    };

    let filter_args = crate::cli::BenchFastqFilterArgs {
        sample_id: args.sample_id.clone(),
        r1: corrected_r1.clone(),
        out: args.out.clone(),
        tools: vec!["fastp".to_string()],
        explain: false,
    };
    bench_fastq_filter(catalog, platform, runner_override, &filter_args)?;
    let filter_params = serde_json::json!({
        "sample_id": args.sample_id,
        "r1": corrected_r1,
    });
    let filter_hash = params_hash(&filter_params).unwrap_or_else(|_| "unknown".to_string());
    let filter_run_id = {
        let image_digest = catalog
            .get("fastp")
            .and_then(|spec| spec.digest.as_ref())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let input_hash = hash_file_sha256(&corrected_r1)?;
        compute_run_id(
            "fastq.filter",
            "fastp",
            &image_digest,
            &input_hash,
            &filter_hash,
        )
    };
    let filter_out_dir = tool_run_artifacts_dir(
        &args.out,
        "filter",
        &args.sample_id,
        "fastp",
        &filter_run_id,
    );
    let final_r1 = if filter_out_dir.exists() {
        find_first_fastq(&filter_out_dir).unwrap_or(corrected_r1.clone())
    } else {
        corrected_r1.clone()
    };

    let stats_args = crate::cli::BenchFastqStatsArgs {
        sample_id: args.sample_id.clone(),
        r1: final_r1.clone(),
        out: args.out.clone(),
        tools: vec!["seqkit_stats".to_string()],
        explain: false,
    };
    bench_fastq_stats(catalog, platform, runner_override, &stats_args)?;

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = super::helpers::resolve_image_for_run(seqkit_spec, platform)?;
    let r1 = r1_canon;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("r1 has no parent"))?
        .to_path_buf();
    let before = input_fastq_stats(&seqkit_image, &r1_dir, &r1)?;
    let final_r1 = final_r1.canonicalize().context("resolve final r1 path")?;
    let final_dir = final_r1
        .parent()
        .ok_or_else(|| anyhow!("final r1 has no parent"))?
        .to_path_buf();
    let after = output_fastq_stats(&seqkit_image, &final_dir, &final_r1)?;
    let deltas: DeltaMetrics = delta_metrics(before, after);
    let run_id = {
        let input_hash = hash_file_sha256(&r1)?;
        compute_run_id("fastq.preprocess", "pipeline", "n/a", &input_hash, "v1")
    };
    let run_dirs = prepare_tool_run_dirs(&out_dir, "pipeline", &run_id)?;
    let delta_path = run_dirs.metrics_path.clone();
    fs::write(
        &delta_path,
        serde_json::to_vec_pretty(&serde_json::json!({ "delta_metrics": deltas }))?,
    )
    .context("write delta metrics")?;
    fs::write(
        run_dirs.logs_dir.join("pipeline.log"),
        "bijux preprocess pipeline completed\n",
    )
    .context("write preprocess log")?;

    let manifest = ExecutionManifest {
        run_id: run_id.clone(),
        stage: "fastq.preprocess".to_string(),
        tool: "pipeline".to_string(),
        tool_version: "v1".to_string(),
        image_digest: "n/a".to_string(),
        command: "preprocess".to_string(),
        input_hashes: vec![hash_file_sha256(&r1)?],
        input_files: vec![r1.display().to_string()],
        output_dir: run_dirs.artifacts_dir.display().to_string(),
        runner: platform.runner.to_string(),
        platform: platform.name.clone(),
        arch: platform.arch.clone(),
    };
    fs::write(
        &run_dirs.manifest_path,
        serde_json::to_vec_pretty(&manifest)?,
    )
    .context("write preprocess manifest")?;

    deltas.validate()?;
    Ok(())
}
