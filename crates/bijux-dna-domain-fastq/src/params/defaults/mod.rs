mod processing;
mod profiling;
mod quality;
mod shared;

pub use processing::{
    correct_defaults, merge_defaults, preprocess_defaults, remove_duplicates_defaults, umi_defaults,
};
pub use profiling::{
    overrepresented_profile_defaults, read_length_profile_defaults, stats_defaults,
};
pub use quality::{
    detect_adapters_defaults, filter_defaults, qc_post_defaults, screen_defaults, trim_defaults,
    trim_polyg_tails_defaults, trim_terminal_damage_defaults, validate_defaults,
};
