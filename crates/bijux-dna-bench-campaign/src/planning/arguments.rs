use super::values::{append_missing_arg, parse_cli_arg_pairs};
use crate::request::StageArgumentDefaults;
use anyhow::Result;
/// Preserve configured flag order and caller precedence without loading configuration.
pub(crate) fn merge_stage_args_from_config(
    defaults: &StageArgumentDefaults,
    stage_args: &[String],
    manifest_args: &[String],
) -> Result<(Vec<String>, Vec<String>)> {
    let stage_values = parse_cli_arg_pairs("stage-arg", stage_args)?;
    let manifest_values = parse_cli_arg_pairs("manifest-arg", manifest_args)?;
    let mut merged_stage_args = stage_args.to_vec();
    let mut merged_manifest_args = manifest_args.to_vec();
    for (key, value) in &defaults.stage {
        append_missing_arg(&mut merged_stage_args, &stage_values, key, Some(value));
    }
    for (key, value) in &defaults.manifest {
        append_missing_arg(&mut merged_manifest_args, &manifest_values, key, Some(value));
    }
    Ok((merged_stage_args, merged_manifest_args))
}
