use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::checks::{check_empty_modules, check_mod_only_dirs, check_mod_reexports_only};
use crate::checks::{
    check_panic_expect, check_pub_items, check_pub_use_spam, check_stage_id_strings,
};
use crate::source_scan::collect_rs_files;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    pub max_pub_items_per_file: usize,
    pub max_pub_use_per_file: usize,
    pub forbid_pub_use_spam: bool,
    pub forbid_panic_expect: bool,
    pub forbid_stage_id_strings: bool,
    pub allow_panic_expect_paths: Vec<String>,
    pub allow_stage_id_paths: Vec<String>,
    pub allow_mod_only_dirs: Vec<String>,
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            max_pub_items_per_file: 50,
            max_pub_use_per_file: 25,
            forbid_pub_use_spam: false,
            forbid_panic_expect: false,
            forbid_stage_id_strings: false,
            allow_panic_expect_paths: Vec::new(),
            allow_stage_id_paths: Vec::new(),
            allow_mod_only_dirs: Vec::new(),
        }
    }
}

impl GuardrailConfig {
    #[must_use]
    pub fn for_crate(name: &str) -> Self {
        let mut config = Self::default();
        if name == "bijux-dna-domain-bam" {
            config.allow_stage_id_paths = vec!["/src/stage_specs/mod.rs".to_string()];
        }
        if name == "bijux-dna-domain-fastq" {
            config.max_pub_items_per_file = 80;
            config.allow_stage_id_paths = vec!["/src/id_catalog.rs".to_string()];
        }
        if name == "bijux-dna-pipelines" {
            config.allow_mod_only_dirs = vec!["/src/vcf".to_string()];
        }
        config
    }
}

/// Run guardrail checks for a crate source tree.
///
/// # Errors
/// Returns an error when any configured guardrail is violated or file traversal fails.
pub fn check(crate_root: &Path, config: &GuardrailConfig) -> Result<()> {
    let src_dir = crate_root.join("src");
    let files = collect_rs_files(&src_dir)?;
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
