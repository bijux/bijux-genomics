//! Internal request handlers (non-public API surface).

pub(crate) mod fastq {
    #[allow(unused_imports)]
    pub(crate) use crate::fastq_router::*;
}

pub(crate) mod bam {
    #[allow(unused_imports)]
    pub(crate) use crate::bam_router::*;
}

pub(crate) mod cross {
    #[allow(unused_imports)]
    pub(crate) use crate::cross_router::*;
}
