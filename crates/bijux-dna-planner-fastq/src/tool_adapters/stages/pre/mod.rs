pub mod detect_adapters;
pub mod detect_duplicates_premerge;
pub mod estimate_library_complexity_prealign;
pub mod index_reference;
pub mod plan_preprocess;
pub mod preprocess;
pub mod profile_overrepresented_sequences;
pub mod profile_read_lengths;
pub mod validate_reads;

pub const MODULE: &str = "stages/pre";
