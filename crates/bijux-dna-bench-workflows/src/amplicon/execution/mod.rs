//! Owner: bijux-dna-bench-workflows
//! amplicon execution responsibilities.
mod artifacts;
pub(crate) use artifacts::{
    copy_absolute_file, copy_repo_relative_file, copy_repo_relative_file_with_absolute_fallback,
    is_nonempty_file, load_json, stage_root, write_json_file, write_text_file,
    AmpliconStageArtifacts,
};
mod timing;
pub(crate) use timing::timestamp_marker;
