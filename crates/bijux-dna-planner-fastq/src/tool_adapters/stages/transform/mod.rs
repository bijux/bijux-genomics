pub mod correct_errors;
pub mod deplete_host;
pub mod deplete_reference_contaminants;
pub mod extract_umis;
pub mod filter_low_complexity;
pub mod filter_reads;
pub mod merge_pairs;
pub mod remove_duplicates;
pub mod trim_polyg_tails;
pub mod trim_reads;
pub mod trim_terminal_damage;

pub const MODULE: &str = "stages/transform";
