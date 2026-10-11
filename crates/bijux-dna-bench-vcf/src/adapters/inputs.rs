//! Shared required-file admission for adapter execution and missing-input audits.
use anyhow::{bail, ensure, Result};
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Copy)]
pub(crate) enum InputContext {
    Adapter { family: &'static str, descent: bool },
    Audit,
}

pub(crate) fn validate_required_files<'a>(
    repo: &Path,
    stage: &str,
    context: InputContext,
    inputs: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<()> {
    ensure!(
        !stage.is_empty() && stage.trim() == stage,
        "required-input stage identity must be nonempty and unpadded"
    );
    let mut identities = BTreeSet::new();
    for (artifact, path) in inputs {
        ensure!(
            !artifact.is_empty() && artifact.trim() == artifact && identities.insert(artifact),
            "required-input artifact identities must be nonempty, unpadded and unique"
        );
        if !repo.join(path).is_file() {
            match context {
                InputContext::Adapter { family, descent: true } => bail!("VCF {family} adapter row `{stage}` is missing required input `{artifact}` at `{path}`"),
                InputContext::Adapter { family, descent: false } => bail!("VCF {family} adapter for `{stage}` is missing required input `{artifact}` at `{path}`"),
                InputContext::Audit => bail!("required input `{artifact}` for VCF stage `{stage}` does not exist at {path}"),
            }
        }
    }
    ensure!(!identities.is_empty(), "required-input declaration must retain at least one artifact");
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/contracts/required_file_admission.rs"]
mod required_file_admission;
