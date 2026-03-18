//! Power-user/internal exports behind the `api_internal` feature.

#![allow(unused_imports)]

pub(crate) use crate::request_args;
pub(crate) use crate::run;

pub(crate) mod handlers {
    pub(crate) use crate::internal::handlers::bam;
    pub(crate) use crate::internal::handlers::cross;
    pub(crate) use crate::internal::handlers::fastq;
}

pub(crate) mod fastq {
    pub(crate) use crate::internal::fastq::stats_neutral;
}
