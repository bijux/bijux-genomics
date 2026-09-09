//! Scope membership, coverage evidence and command audit output contracts.
mod removed;
pub use removed::{RemovedFromScopeReport, RemovedFromScopeRow, REMOVED_FROM_SCOPE_SCHEMA_VERSION};
mod parser_fixtures;
pub use parser_fixtures::{
    AllDomainParserFixtureCoverageReport, AllDomainParserFixtureCoverageRow,
    ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
};
mod outputs;
pub use outputs::{
    AllDomainOutputContractCoverageReport, AllDomainOutputContractCoverageRow,
    ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_SCHEMA_VERSION,
};
mod expected_results;
pub use expected_results::{
    AllDomainExpectedResultCoverageReport, AllDomainExpectedResultCoverageRow,
    ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_SCHEMA_VERSION,
};
mod report_map;
pub use report_map::{
    AllDomainReportMapCoverageReport, AllDomainReportMapCoverageRow,
    ALL_DOMAIN_REPORT_MAP_COVERAGE_SCHEMA_VERSION,
};
mod local_jobs;
pub use local_jobs::{
    AllDomainLocalJobCoverageReport, AllDomainLocalJobCoverageRow,
    ALL_DOMAIN_LOCAL_JOB_COVERAGE_SCHEMA_VERSION,
};
mod commands;
pub use commands::{
    AllDomainNoPlaceholderCommandCheckReport, AllDomainNoPlaceholderCommandRow,
    AllDomainNoPlaceholderCommandStepAudit, ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_SCHEMA_VERSION,
};
