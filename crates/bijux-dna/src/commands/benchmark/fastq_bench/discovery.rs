use crate::commands::support::prelude::{
    normalize_fastq_stage_id, resolve_effective_adapters, FastqCommand, Result, StageId,
};

use super::{
    explain_fastq_stage, list_adapter_presets, list_adapters, load_adapter_selection,
    print_bank_presets, tool_tier_label,
};

pub(super) fn handle_fastq_discovery(
    command: &FastqCommand,
    registry: &bijux_dna_api::v1::api::run::ToolRegistry,
) -> Result<Option<bool>> {
    match command {
        FastqCommand::ListStages => {
            list_fastq_stages();
            Ok(Some(true))
        }
        FastqCommand::Stages => {
            list_fastq_stage_registry();
            Ok(Some(true))
        }
        FastqCommand::ListTools { stage } => {
            let stage_id = normalize_fastq_stage_id(stage);
            list_fastq_tools(registry, &stage_id);
            Ok(Some(true))
        }
        FastqCommand::Explain { stage } => {
            let stage_id = normalize_fastq_stage_id(stage);
            explain_fastq_stage(registry, &stage_id)?;
            Ok(Some(true))
        }
        FastqCommand::Trim(args) => {
            if args.list_adapter_presets {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                list_adapter_presets(&selection.presets);
                return Ok(Some(true));
            }
            if args.list_adapters {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                let effective = resolve_effective_adapters(
                    &selection,
                    &args.enable_adapter,
                    &args.disable_adapter,
                )?;
                list_adapters(&effective);
                return Ok(Some(true));
            }
            Ok(None)
        }
        FastqCommand::Preprocess(args) => {
            if args.list_adapter_presets {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                list_adapter_presets(&selection.presets);
                return Ok(Some(true));
            }
            if args.list_adapters {
                let selection = load_adapter_selection(
                    args.adapter_bank_preset.as_deref(),
                    args.adapter_bank.as_deref(),
                    args.adapter_bank_file.as_deref(),
                )?;
                let effective = resolve_effective_adapters(
                    &selection,
                    &args.enable_adapter,
                    &args.disable_adapter,
                )?;
                list_adapters(&effective);
                return Ok(Some(true));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn list_fastq_stages() {
    for stage in &bijux_dna_api::v1::api::bench::STAGES {
        println!("{}", stage.as_str());
    }
    print_bank_presets();
}

fn list_fastq_stage_registry() {
    for stage in &bijux_dna_api::v1::api::bench::STAGES {
        println!("{}", stage.as_str());
    }
    print_bank_presets();
}

pub(super) fn list_fastq_tools(
    registry: &bijux_dna_api::v1::api::run::ToolRegistry,
    stage_id: &str,
) {
    let Ok(stage_id) = StageId::try_from(stage_id) else {
        eprintln!("invalid stage id: {stage_id}");
        return;
    };
    let mut tools: Vec<_> = registry
        .tools_for_stage(&stage_id)
        .iter()
        .map(|tool| (tool.tool_id.to_string(), tool.role))
        .collect();
    tools.sort_by(|a, b| a.0.cmp(&b.0));
    for (tool_id, role) in tools {
        println!("{tool_id}\t{}", tool_tier_label(role));
    }
}
