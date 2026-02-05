//! Internal request handlers (non-public API surface).

pub(crate) mod bam;
pub(crate) mod fastq;
pub(crate) mod cross {
    #[allow(unused_imports)]
    pub(crate) use crate::cross_router::*;
}
