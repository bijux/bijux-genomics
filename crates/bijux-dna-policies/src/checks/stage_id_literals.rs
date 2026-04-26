use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::checks::path_match::path_has_allowed_suffix;
use crate::GuardrailConfig;

pub(crate) fn check_stage_id_strings(files: &[PathBuf], config: &GuardrailConfig) -> Result<()> {
    let stage_re = Regex::new(r#"\"(fastq|bam|vcf)\.[^\"]+\""#)?;
    let raw_stage_re = Regex::new(r##"r#*"(fastq|bam|vcf)\.[^"]+"#*"##)?;
    for path in files {
        if path_has_allowed_suffix(path, &config.allow_stage_id_paths) {
            continue;
        }
        let content = fs::read_to_string(path)?;
        let mut in_block_comment = false;
        for (idx, line) in content.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            let code = strip_block_comments(line, &mut in_block_comment);
            if stage_re.is_match(&code) || raw_stage_re.is_match(&code) {
                anyhow::bail!("stage id literal found in {}:{}", path.display(), idx + 1);
            }
        }
    }
    Ok(())
}

fn strip_block_comments(line: &str, in_block_comment: &mut bool) -> String {
    let mut remaining = line;
    let mut code = String::new();
    loop {
        if *in_block_comment {
            if let Some(end) = remaining.find("*/") {
                remaining = &remaining[end + 2..];
                *in_block_comment = false;
                continue;
            }
            return code;
        }
        if let Some(start) = remaining.find("/*") {
            code.push_str(&remaining[..start]);
            remaining = &remaining[start + 2..];
            *in_block_comment = true;
            continue;
        }
        code.push_str(remaining);
        return code;
    }
}
