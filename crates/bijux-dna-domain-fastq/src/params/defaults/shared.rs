use crate::params::PairedMode;

pub(super) fn paired_mode(paired: bool) -> PairedMode {
    if paired {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    }
}
