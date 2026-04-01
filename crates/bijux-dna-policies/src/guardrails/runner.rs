use anyhow::Result;
use std::path::Path;

use crate::checks::{check_empty_modules, check_mod_only_dirs, check_mod_reexports_only};
use crate::checks::{
    check_panic_expect, check_pub_items, check_pub_use_spam, check_stage_id_strings,
};

use super::source_inventory::collect_sources;
use super::GuardrailConfig;

/// Run guardrail checks for a crate source tree.
///
/// # Errors
/// Returns an error when any configured guardrail is violated or file traversal fails.
pub fn check(crate_root: &Path, config: &GuardrailConfig) -> Result<()> {
    let sources = collect_sources(crate_root)?;
    let src_dir = &sources.src_dir;
    let files = &sources.files;
    check_mod_only_dirs(&src_dir, config)?;
    check_empty_modules(&files)?;
    check_mod_reexports_only(&files)?;
    check_pub_items(&files, config)?;
    if config.forbid_pub_use_spam {
        check_pub_use_spam(&files, config)?;
    }
    if config.forbid_panic_expect {
        check_panic_expect(&files, config)?;
    }
    if config.forbid_stage_id_strings {
        check_stage_id_strings(&files, config)?;
    }
    Ok(())
}
