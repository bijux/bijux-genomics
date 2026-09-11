mod declarations;
mod models;
mod projection;
mod publication;
mod service;
mod tsv;
pub use declarations::BAM_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION;
pub use models::{
    BamParserFixtureCoverageReport, BamParserFixtureCoverageRow, BamParserFixtureCoverageStatus,
};
pub use publication::render_bam_parser_fixture_coverage;
pub use service::collect_bam_parser_fixture_coverage_rows;
pub use service::ParserCoverageRows;
