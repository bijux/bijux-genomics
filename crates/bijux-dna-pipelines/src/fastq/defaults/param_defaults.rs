use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;

use super::analysis_params::fastq_analysis_params;
use super::preprocess_params::fastq_preprocess_params;

use crate::DefaultParams;

pub(super) fn fastq_default_params(paired: bool) -> BTreeMap<StageId, DefaultParams> {
    let mut params = fastq_preprocess_params(paired);
    params.extend(fastq_analysis_params(paired));
    params
}
