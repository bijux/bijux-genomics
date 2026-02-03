use crate::stage_registry::FastqStageKind;

use super::metrics::{CORE_STAGES, META_STAGES, OPTIONAL_STAGES};

#[allow(dead_code)]
pub fn stage_kind(stage_id: &str) -> Option<FastqStageKind> {
    if CORE_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Core);
    }
    if OPTIONAL_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Optional);
    }
    if META_STAGES.contains(&stage_id) {
        return Some(FastqStageKind::Meta);
    }
    None
}
