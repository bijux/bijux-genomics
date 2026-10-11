//! Execute admitted BAM calling modes without acquiring benchmark catalogs.
use anyhow::{Context, Result};
use bijux_dna_domain_vcf::params::VcfCallParams;
use bijux_dna_stages_vcf::pipeline::{
    run_call_diploid_stage, run_call_gl_stage, run_call_pseudohaploid_stage, CallStageOutputs,
};
use std::path::{Path, PathBuf};

pub(crate) struct PreparedBam {
    pub(crate) alignment: PathBuf,
    pub(crate) reference: PathBuf,
    pub(crate) artifacts: PathBuf,
}
impl PreparedBam {
    pub(crate) fn prepare(
        repository: &Path,
        output: &Path,
        alignment: &str,
        reference: &str,
    ) -> Result<Self> {
        let artifacts = output.join("artifacts");
        bijux_dna_infra::ensure_dir(&artifacts)?;
        let reference =
            super::reference::materialize_reference_fasta(&repository.join(reference), &artifacts)?;
        Ok(Self { alignment: repository.join(alignment), reference, artifacts })
    }
    pub(crate) fn execute(
        &self,
        mode: CallingMode,
        tool: &str,
        sample: &str,
        replay: bool,
    ) -> Result<CallStageOutputs> {
        let parameters = VcfCallParams {
            caller: tool.to_owned(),
            sample_name: sample.to_owned(),
            reference_fasta: Some(self.reference.display().to_string()),
            ..VcfCallParams::default()
        };
        let output = if replay { self.artifacts.join("replay") } else { self.artifacts.clone() };
        let execute = match mode {
            CallingMode::Gl => run_call_gl_stage,
            CallingMode::Diploid => run_call_diploid_stage,
            CallingMode::Pseudohaploid => run_call_pseudohaploid_stage,
        };
        execute(&self.alignment, &output, &parameters).with_context(|| {
            format!("execute {mode:?} calling with `{tool}` from {}", self.alignment.display())
        })
    }
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum CallingMode {
    Gl,
    Diploid,
    Pseudohaploid,
}

pub(crate) struct CalledArtifacts {
    pub(crate) vcf: PathBuf,
    pub(crate) index: PathBuf,
}
impl CalledArtifacts {
    pub(crate) fn materialize(
        native: &CallStageOutputs,
        output: &Path,
        name: &str,
    ) -> Result<Self> {
        let vcf = output.join(name);
        let index = PathBuf::from(format!("{}.tbi", vcf.display()));
        for (source, destination) in [(&native.called_vcf, &vcf), (&native.called_tbi, &index)] {
            std::fs::copy(source, destination).with_context(|| {
                format!("copy {} to {}", source.display(), destination.display())
            })?;
        }
        Ok(Self { vcf, index })
    }
}
