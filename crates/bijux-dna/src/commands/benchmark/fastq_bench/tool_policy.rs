use crate::commands::support::prelude::{cli, FastqCommand};

pub(super) fn tool_tier_label(role: bijux_dna_api::v1::api::run::ToolRole) -> &'static str {
    match role {
        bijux_dna_api::v1::api::run::ToolRole::Authoritative => "gold",
        bijux_dna_api::v1::api::run::ToolRole::Diagnostic => "silver",
        bijux_dna_api::v1::api::run::ToolRole::Experimental => "experimental",
    }
}

pub(super) fn set_scientific_preset(preset: Option<cli::parse::ScientificPresetArg>) {
    if let Some(preset) = preset {
        std::env::set_var("BIJUX_SCIENTIFIC_PRESET", format!("{preset:?}").to_lowercase());
    } else {
        std::env::remove_var("BIJUX_SCIENTIFIC_PRESET");
    }
}

pub(crate) fn set_tool_tier_policy(allow_silver: bool, allow_experimental: bool) {
    if allow_silver || allow_experimental {
        std::env::set_var("BIJUX_ALLOW_SILVER", "1");
    } else {
        std::env::remove_var("BIJUX_ALLOW_SILVER");
    }
    if allow_experimental {
        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
    } else {
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");
    }
}

pub(super) fn tool_tier_policy_for_fastq(command: &FastqCommand) -> (bool, bool) {
    match command {
        FastqCommand::Trim(args) => (args.common.allow_silver, args.common.allow_experimental),
        FastqCommand::ValidateReads(args) => {
            (args.common.allow_silver, args.common.allow_experimental)
        }
        FastqCommand::Filter(args) => (args.common.allow_silver, args.common.allow_experimental),
        FastqCommand::Preprocess(args) => {
            (args.common.allow_silver, args.common.allow_experimental)
        }
        FastqCommand::Run(args) => {
            (args.args.common.allow_silver, args.args.common.allow_experimental)
        }
        FastqCommand::Merge(args)
        | FastqCommand::ErrorCorrect(args)
        | FastqCommand::Qc(args)
        | FastqCommand::Umi(args)
        | FastqCommand::Contam(args)
        | FastqCommand::ProfileReads(args)
        | FastqCommand::Align(args) => (args.allow_silver, args.allow_experimental),
        _ => (false, false),
    }
}
