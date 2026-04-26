use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::checks::path_match::path_has_allowed_suffix;
use crate::GuardrailConfig;

pub(crate) fn check_panic_expect(files: &[PathBuf], config: &GuardrailConfig) -> Result<()> {
    let panic_re = Regex::new(r"\bpanic!\(")?;
    let expect_re = Regex::new(r"\.expect\(")?;
    for path in files {
        if path_has_allowed_suffix(path, &config.allow_panic_expect_paths) {
            continue;
        }
        let content = fs::read_to_string(path)?;
        for (idx, line) in content.lines().enumerate() {
            if panic_re.is_match(line) || expect_re.is_match(line) {
                anyhow::bail!("panic/expect found in {}:{}", path.display(), idx + 1);
            }
        }
    }
    Ok(())
}
