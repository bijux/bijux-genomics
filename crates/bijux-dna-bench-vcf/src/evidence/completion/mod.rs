//! Complete retained VCF readiness with explicit execution ports and owned evidence admission.
mod checks;
mod contracts;
mod keys;
mod model;
mod paths;
pub mod ports;
mod projection;
mod service;
mod state;
pub use model::{
    VcfAllRetainedToolsCompleteGoalCheck, VcfAllRetainedToolsCompleteReport,
    DEFAULT_VCF_ALL_RETAINED_TOOLS_COMPLETE_PATH,
};
pub use service::render_vcf_all_retained_tools_complete;

#[cfg(test)]
#[path = "../../../tests/contracts/retained_completion.rs"]
mod contract_tests;
