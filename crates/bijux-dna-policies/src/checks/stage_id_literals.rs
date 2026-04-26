use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::checks::path_match::path_has_allowed_suffix;
use crate::GuardrailConfig;

pub(crate) fn check_stage_id_strings(files: &[PathBuf], config: &GuardrailConfig) -> Result<()> {
    let stage_re = Regex::new(r#"\"(fastq|bam|vcf)\.[^\"]+\""#)?;
    for path in files {
        if path_has_allowed_suffix(path, &config.allow_stage_id_paths) {
            continue;
        }
        let content = fs::read_to_string(path)?;
        for (idx, line) in content.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if stage_re.is_match(line) {
                anyhow::bail!("stage id literal found in {}:{}", path.display(), idx + 1);
            }
        }
    }
    Ok(())
}
