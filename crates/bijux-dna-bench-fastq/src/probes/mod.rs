//! Small synthetic parser fixtures; these declarations do not prove backend execution.
mod inputs;
mod model;
mod parameters;
mod paths;
mod publication;
mod reference;
pub mod reports;
mod streams;
pub use reference::generate_index_reference_probe;
mod correction;
pub use correction::generate_correct_errors_probe;
mod host;
pub use host::generate_deplete_host_probe;
mod rrna;
pub use rrna::generate_deplete_rrna_probe;
mod taxonomy;
pub use model::{CorrectErrorsProbeInput, MaterializedFastqProof};
pub use taxonomy::generate_screen_taxonomy_probe;
