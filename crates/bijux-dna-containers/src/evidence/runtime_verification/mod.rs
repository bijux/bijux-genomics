//! Catalog-wide runtime verification for canonical Apptainer images.

mod report;

pub use report::{render_runtime_verification_markdown, verify_runtime_verification_report};
