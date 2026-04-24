use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::GuardrailConfig;

pub(crate) fn check_stage_id_strings(files: &[PathBuf], config: &GuardrailConfig) -> Result<()> {
    let stage_re = Regex::new(r#"\"(fastq|bam|vcf)\.[^\"]+\""#)?;
    for path in files {
        let path_str = path.to_string_lossy();
        if config.allow_stage_id_paths.iter().any(|allowed| path_str.contains(allowed)) {
            continue;
        }
        let content = fs::read_to_string(path)?;
        if stage_re.is_match(&content) {
            anyhow::bail!("stage id literal found in {}", path.display());
        }
    }
    Ok(())
}
