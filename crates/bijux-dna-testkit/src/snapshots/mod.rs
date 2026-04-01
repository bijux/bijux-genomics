mod environment;
mod json_normalization;
mod naming;
mod text_normalization;

pub use environment::install_snapshot_env;
pub use json_normalization::{
    sanitize_snapshot_json, snapshot_normalize, snapshot_normalize_json, stable_json,
};
pub use naming::snapshot_name;

#[must_use]
pub fn snapshot_normalize_text(input: &str) -> String {
    text_normalization::sanitize_snapshot_text(input)
}

pub use text_normalization::sanitize_snapshot_text;
