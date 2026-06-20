//! Power-user exports behind the `api_internal` feature flag.

#![allow(unused_imports)]

pub(crate) use crate::request_args;
pub(crate) use crate::run;

pub(crate) mod handlers {
    pub(crate) use crate::internal::handlers::bam;
    pub(crate) use crate::internal::handlers::cross;
    pub(crate) use crate::internal::handlers::fastq;
}

pub(crate) mod bam_stages {
    pub(crate) use crate::internal::bam::stages::{
        authenticity, bias_mitigation, complexity, contamination, coverage, damage,
        duplication_metrics, endogenous_content, filter, gc_bias, insert_size, kinship,
        length_filter, mapping_summary, mapq_filter, markdup, overlap_correction, qc_pre,
        recalibration, sex, validate,
    };
    #[cfg(feature = "bam_downstream")]
    pub(crate) use crate::internal::bam::stages::haplogroups;
}

pub(crate) mod fastq {
    pub(crate) use crate::internal::fastq::stages::profile_reads;
}
