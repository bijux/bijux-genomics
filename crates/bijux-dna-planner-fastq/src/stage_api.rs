pub use crate::selection::args;
pub use crate::selection::{allowed_tools_for_stage, default_tool_for_stage};
pub use crate::tool_adapters::fastq;
pub use crate::tool_adapters::fastq::StageInfo;
pub use crate::STAGE_REPORT_AGGREGATE;
pub use crate::TOOL_SEQKIT;
pub use bijux_dna_core::prelude::RawFailure;
pub use bijux_dna_domain_fastq::banks;
pub use bijux_dna_domain_fastq::banks::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
    resolve_adapter_selection, resolve_contaminant_selection, resolve_effective_adapters,
    resolve_effective_contaminants, resolve_effective_polyx, resolve_polyx_selection,
    AdapterSelection, DEFAULT_ADAPTER_PRESET, DEFAULT_CONTAMINANT_PRESET, DEFAULT_POLYX_PRESET,
};
pub use bijux_dna_domain_fastq::{
    ensure_umi_headers, inspect_headers, log_header_warnings, preflight_stage, FastqArtifact,
    FastqArtifactKind,
};
pub use bijux_dna_stages_fastq::stage_specs::*;

pub type StagePlanJson = bijux_dna_stage_contract::StagePlanJsonV1;

pub fn adapter_bank_path() -> std::path::PathBuf {
    bijux_dna_domain_fastq::adapter_bank_path()
}
