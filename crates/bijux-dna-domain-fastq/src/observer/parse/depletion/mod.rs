mod host;
mod reference_contaminants;
mod rrna;

pub use host::parse_deplete_host_report;
pub use reference_contaminants::parse_deplete_reference_contaminants_report;
pub use rrna::parse_deplete_rrna_report;
