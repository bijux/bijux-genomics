//! Pure command/default projection and declared artifact metadata.
mod arguments;
mod command;
mod manifest;
pub(crate) mod values;
pub(crate) use arguments::merge_stage_args_from_config;
pub(crate) use command::{build_stage_command_args, StageCommandRequest};
pub(crate) use manifest::collect_stage_manifest_fields;
