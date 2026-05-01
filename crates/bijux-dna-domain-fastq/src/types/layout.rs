use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FastqReadLayout {
    SingleEnd,
    PairedEnd,
    Interleaved,
    Deinterleaved,
    Merged,
    Singleton,
    Rejected,
}

pub const FASTQ_DECLARED_READ_LAYOUTS: [FastqReadLayout; 7] = [
    FastqReadLayout::SingleEnd,
    FastqReadLayout::PairedEnd,
    FastqReadLayout::Interleaved,
    FastqReadLayout::Deinterleaved,
    FastqReadLayout::Merged,
    FastqReadLayout::Singleton,
    FastqReadLayout::Rejected,
];
