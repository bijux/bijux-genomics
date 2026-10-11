use super::ExpectedBenchmarkResultRow;
use crate::ExpectedContext;
use std::collections::BTreeMap;
pub(super) struct ProjectionContext<'a, 'b> {
    pub request: &'a ExpectedContext<'b>,
    pub fixture_sample_scopes: &'a BTreeMap<String, String>,
}
mod bam;
mod fastq;
mod identity;
pub(super) use bam::build_bam_expected_result_row;
pub(super) use fastq::{
    build_fastq_asset_backed_expected_result_row, build_fastq_expected_result_row,
};
