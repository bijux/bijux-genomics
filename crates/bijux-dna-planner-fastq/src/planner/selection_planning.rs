#![allow(clippy::uninlined_format_args, clippy::wildcard_imports)]

use super::*;

/// # Errors
/// Returns an error if node-aware tool selection fails.
pub fn select_preprocess_stage_tools(
    registry: &bijux_dna_core::contract::ToolRegistry,
    pipeline: &PipelineSpec,
    args: &crate::selection::args::BenchFastqPreprocessArgs,
    bench_repo: Option<&dyn BenchResultsRepository>,
) -> Result<Vec<StageToolSelection>> {
    let executable_nodes = pipeline
        .ordered_nodes()
        .into_iter()
        .filter(|node| !planner_owned_graph_stage(&node.stage_id))
        .collect::<Vec<_>>();
    let paired_end = args.r2.is_some();
    let mut selected_tools: Vec<StageToolSelection> = executable_nodes
        .iter()
        .map(|node| {
            let stage_id = StageId::new(node.stage_id.clone());
            let compatible_tools = crate::stage_api::filter_tools_for_input_layout(
                &stage_id,
                registry
                    .tools_for_stage(&stage_id)
                    .iter()
                    .map(|tool| tool.tool_id.clone())
                    .collect(),
                paired_end,
            );
            let tool_id = crate::selection::default_tool_for_stage(&stage_id)
                .filter(|tool_id| {
                    crate::stage_api::tool_supports_input_layout(&stage_id, tool_id, paired_end)
                })
                .or_else(|| compatible_tools.first().cloned())
                .map(|tool| tool.to_string())
                .ok_or_else(|| anyhow!("no layout-compatible tool for stage {}", node.stage_id))?;
            Ok(StageToolSelection {
                stage_id: node.stage_id.clone(),
                stage_instance_id: node.stage_instance_id.clone(),
                tool_id,
                reason: PlanDecisionReason::new(
                    PlanReasonKind::Default,
                    "default tool from pipeline catalog",
                ),
            })
        })
        .collect::<Result<_>>()?;

    if args.auto {
        let corpus_id =
            args.bench_corpus.ok_or_else(|| anyhow!("--bench-corpus is required with --auto"))?;
        let corpus = bijux_dna_domain_fastq::bench_corpus(corpus_id);
        let objective = bijux_dna_core::contract::objective_spec(args.objective);
        let repo = bench_repo.ok_or_else(|| {
            anyhow!("bench results repository required for --auto tool selection")
        })?;
        let mut selections = Vec::new();
        for (idx, node) in executable_nodes.iter().enumerate() {
            let stage_id = bijux_dna_core::ids::StageId::new(node.stage_id.clone());
            let prior_stage_ids = selected_tools[..idx]
                .iter()
                .map(|selection| selection.stage_id.clone())
                .collect::<Vec<_>>();
            let query_context = bench_query_context_for_preprocess_stage(
                &stage_id,
                args,
                &prior_stage_ids,
                &selected_tools[..idx],
            )?;
            let tool_ids = registry
                .tools_for_stage(&stage_id)
                .iter()
                .map(|tool| tool.tool_id.clone())
                .collect::<Vec<_>>();
            let tool_ids =
                crate::stage_api::filter_tools_for_input_layout(&stage_id, tool_ids, paired_end)
                    .into_iter()
                    .map(|tool| tool.to_string())
                    .collect::<Vec<_>>();
            let mut tool_records = Vec::new();
            for tool in &tool_ids {
                let records = repo.bench_results(&stage_id, tool, &corpus, &query_context)?;
                tool_records.push((tool.clone(), records));
            }
            let selection = bijux_dna_core::contract::select_stage(
                &stage_id,
                &tool_records,
                &objective,
                args.allow_partial,
            );
            if let Some(selected) = selection.selected.as_ref() {
                selected_tools[idx] = StageToolSelection {
                    stage_id: node.stage_id.clone(),
                    stage_instance_id: node.stage_instance_id.clone(),
                    tool_id: selected.clone(),
                    reason: PlanDecisionReason::new(
                        PlanReasonKind::InputAssessed,
                        "auto-selected from benchmark corpus",
                    ),
                };
            }
            selections.push(selection);
        }
    }

    Ok(selected_tools)
}

pub(crate) fn bench_query_context_for_stage(
    stage_id: &bijux_dna_core::ids::StageId,
) -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(stage_id.as_str())
        .map_err(|err| anyhow!("compute benchmark query context for {}: {err}", stage_id.as_str()))
}

pub(crate) fn bench_query_context_for_preprocess_stage(
    stage_id: &bijux_dna_core::ids::StageId,
    args: &crate::selection::args::BenchFastqPreprocessArgs,
    prior_stages: &[String],
    prior_tools: &[StageToolSelection],
) -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    let mut context = bench_query_context_for_stage(stage_id)?;
    if let Some(reference_fasta) = args.reference_fasta.as_ref() {
        context = context.with_reference_hash(
            bijux_dna_infra::hash_file_sha256(reference_fasta).map_err(|err| {
                anyhow!(
                    "hash reference FASTA for benchmark query context {}: {err}",
                    reference_fasta.display()
                )
            })?,
        );
    }
    for (bank_id, bank_hash) in bank_hashes_for_preprocess_args(args)? {
        context = context.with_bank_hash(bank_id, bank_hash);
    }
    let lineage_hash = prior_stages
        .iter()
        .zip(prior_tools.iter())
        .map(|(stage_id, tool)| format!("{stage_id}={}", tool.tool_id))
        .collect::<Vec<_>>()
        .join("|");
    if !lineage_hash.is_empty() {
        context = context.with_lineage_hash(lineage_hash);
    }
    Ok(context)
}

fn bank_hashes_for_preprocess_args(
    args: &crate::selection::args::BenchFastqPreprocessArgs,
) -> Result<Vec<(String, String)>> {
    let mut hashes = Vec::new();
    if args.adapter_bank_preset.is_some()
        || args.adapter_bank.is_some()
        || args.adapter_bank_file.is_some()
        || !args.enable_adapters.is_empty()
        || !args.disable_adapters.is_empty()
    {
        if let Some(context) = bijux_dna_domain_fastq::banks::adapter_bank_context(
            args.adapter_bank_preset.as_deref(),
            args.adapter_bank.as_deref(),
            args.adapter_bank_file.as_deref(),
            &args.enable_adapters,
            &args.disable_adapters,
        )? {
            if let Some(bank_hash) = context.get("bank_hash").and_then(serde_json::Value::as_str) {
                hashes.push(("adapter_bank".to_string(), bank_hash.to_string()));
            }
        }
    }
    if args.polyx_preset.is_some() {
        if let Some(context) =
            bijux_dna_domain_fastq::banks::polyx_bank_context(args.polyx_preset.as_deref())?
        {
            if let Some(bank_hash) = context.get("bank_hash").and_then(serde_json::Value::as_str) {
                hashes.push(("polyx_bank".to_string(), bank_hash.to_string()));
            }
        }
    }
    if args.contaminant_preset.is_some() {
        if let Some(context) = bijux_dna_domain_fastq::banks::contaminant_bank_context(
            args.contaminant_preset.as_deref(),
        )? {
            if let Some(bank_hash) = context.get("bank_hash").and_then(serde_json::Value::as_str) {
                hashes.push(("contaminant_bank".to_string(), bank_hash.to_string()));
            }
        }
    }
    hashes.sort();
    Ok(hashes)
}
