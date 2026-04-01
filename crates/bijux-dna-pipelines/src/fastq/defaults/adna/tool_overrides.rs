use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::id_catalog;

use crate::EffectiveDefaults;

pub(super) fn apply(defaults: &mut EffectiveDefaults) {
    defaults.tools.insert(
        StageId::from_static("fastq.trim_reads"),
        ToolId::from_static(id_catalog::TOOL_ADAPTERREMOVAL),
    );
    defaults.tools.insert(
        StageId::from_static("fastq.merge_pairs"),
        ToolId::from_static(id_catalog::TOOL_LEEHOM),
    );
}
