//! Owner: evidence-bound container result movement; admission precedes mutation.
mod admission;
mod archive;
mod layout;
mod observations;
mod paths;
mod repository;
pub use archive::{
    apply_superseded_result_archive, derive_superseded_result_archive_plan,
    rollback_superseded_result_archive, verify_superseded_result_archive,
};
pub use layout::{
    apply_result_layout, derive_result_layout_plan, rollback_result_layout, verify_result_layout,
};
