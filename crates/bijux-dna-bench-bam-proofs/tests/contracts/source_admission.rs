#![cfg(test)]
use anyhow::{bail, Result};
use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups::BamContaminationSexHaplogroupsReadyReport;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyReport;
use bijux_dna_bench_bam_evidence::readiness::overlap_endogenous::BamOverlapEndogenousReadyReport;
use bijux_dna_bench_bam_evidence::readiness::recalibration_genotyping::BamRecalibrationGenotypingReadyReport;
use bijux_dna_bench_bam_proofs::ports::{NativeProofKind, NativeProofWriter, ReadinessEvidence};
use bijux_dna_bench_bam_proofs::ProofContext;
use std::cell::Cell;
use std::path::{Path, PathBuf};
pub(super) struct Refused;
impl ReadinessEvidence for Refused {
    fn damage_authenticity(&self, _root: &Path) -> Result<BamDamageAuthenticityReadyReport> {
        bail!("canonical readiness unavailable")
    }
    fn contamination_sex_haplogroups(
        &self,
        _root: &Path,
    ) -> Result<BamContaminationSexHaplogroupsReadyReport> {
        bail!("canonical readiness unavailable")
    }
    fn overlap_endogenous(&self, _root: &Path) -> Result<BamOverlapEndogenousReadyReport> {
        bail!("canonical readiness unavailable")
    }
    fn recalibration_genotyping(
        &self,
        _root: &Path,
    ) -> Result<BamRecalibrationGenotypingReadyReport> {
        bail!("canonical readiness unavailable")
    }
}
pub(super) struct NativeCalls(pub(super) Cell<usize>);
impl NativeProofWriter for NativeCalls {
    fn write(&self, _root: &Path, _kind: NativeProofKind) -> Result<PathBuf> {
        self.0.set(self.0.get() + 1);
        bail!("unexpected native execution")
    }
}
#[test]
fn authenticity_refuses_readiness_before_native_or_report_replacement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("assessment.json");
    std::fs::write(&path, b"prior evidence")?;
    let native = NativeCalls(Cell::new(0));
    let context = ProofContext { repo_root: dir.path(), readiness: &Refused, native: &native };
    let error =
        bijux_dna_bench_bam_proofs::authenticity::render_bam_authenticity_complete(&context, &path)
            .err()
            .ok_or_else(|| anyhow::anyhow!("invalid readiness accepted"))?;
    assert!(error.to_string().contains("canonical readiness unavailable"));
    assert_eq!(native.0.get(), 0);
    assert_eq!(std::fs::read(&path)?, b"prior evidence");
    Ok(())
}
#[test]
fn contamination_refuses_readiness_before_native_or_report_replacement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("assessment.json");
    std::fs::write(&path, b"prior evidence")?;
    let native = NativeCalls(Cell::new(0));
    let context = ProofContext { repo_root: dir.path(), readiness: &Refused, native: &native };
    let error = bijux_dna_bench_bam_proofs::contamination::render_bam_contamination_complete(
        &context, &path,
    )
    .err()
    .ok_or_else(|| anyhow::anyhow!("invalid readiness accepted"))?;
    assert!(error.to_string().contains("canonical readiness unavailable"));
    assert_eq!(native.0.get(), 0);
    assert_eq!(std::fs::read(&path)?, b"prior evidence");
    Ok(())
}
#[test]
fn damage_refuses_readiness_before_native_or_report_replacement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("assessment.json");
    std::fs::write(&path, b"prior evidence")?;
    let native = NativeCalls(Cell::new(0));
    let context = ProofContext { repo_root: dir.path(), readiness: &Refused, native: &native };
    let error = bijux_dna_bench_bam_proofs::damage::render_bam_damage_complete(&context, &path)
        .err()
        .ok_or_else(|| anyhow::anyhow!("invalid readiness accepted"))?;
    assert!(error.to_string().contains("canonical readiness unavailable"));
    assert_eq!(native.0.get(), 0);
    assert_eq!(std::fs::read(&path)?, b"prior evidence");
    Ok(())
}
#[test]
fn endogenous_content_refuses_readiness_before_native_or_report_replacement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("assessment.json");
    std::fs::write(&path, b"prior evidence")?;
    let native = NativeCalls(Cell::new(0));
    let context = ProofContext { repo_root: dir.path(), readiness: &Refused, native: &native };
    let error =
        bijux_dna_bench_bam_proofs::endogenous_content::render_bam_endogenous_content_complete(
            &context, &path,
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("invalid readiness accepted"))?;
    assert!(error.to_string().contains("canonical readiness unavailable"));
    assert_eq!(native.0.get(), 0);
    assert_eq!(std::fs::read(&path)?, b"prior evidence");
    Ok(())
}
#[test]
fn haplogroups_refuses_readiness_before_native_or_report_replacement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("assessment.json");
    std::fs::write(&path, b"prior evidence")?;
    let native = NativeCalls(Cell::new(0));
    let context = ProofContext { repo_root: dir.path(), readiness: &Refused, native: &native };
    let error =
        bijux_dna_bench_bam_proofs::haplogroups::render_bam_haplogroups_complete(&context, &path)
            .err()
            .ok_or_else(|| anyhow::anyhow!("invalid readiness accepted"))?;
    assert!(error.to_string().contains("canonical readiness unavailable"));
    assert_eq!(native.0.get(), 0);
    assert_eq!(std::fs::read(&path)?, b"prior evidence");
    Ok(())
}
#[test]
fn overlap_correction_refuses_readiness_before_native_or_report_replacement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("assessment.json");
    std::fs::write(&path, b"prior evidence")?;
    let native = NativeCalls(Cell::new(0));
    let context = ProofContext { repo_root: dir.path(), readiness: &Refused, native: &native };
    let error =
        bijux_dna_bench_bam_proofs::overlap_correction::render_bam_overlap_correction_complete(
            &context, &path,
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("invalid readiness accepted"))?;
    assert!(error.to_string().contains("canonical readiness unavailable"));
    assert_eq!(native.0.get(), 0);
    assert_eq!(std::fs::read(&path)?, b"prior evidence");
    Ok(())
}
#[test]
fn recalibration_refuses_readiness_before_native_or_report_replacement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("assessment.json");
    std::fs::write(&path, b"prior evidence")?;
    let native = NativeCalls(Cell::new(0));
    let context = ProofContext { repo_root: dir.path(), readiness: &Refused, native: &native };
    let error = bijux_dna_bench_bam_proofs::recalibration::render_bam_recalibration_complete(
        &context, &path,
    )
    .err()
    .ok_or_else(|| anyhow::anyhow!("invalid readiness accepted"))?;
    assert!(error.to_string().contains("canonical readiness unavailable"));
    assert_eq!(native.0.get(), 0);
    assert_eq!(std::fs::read(&path)?, b"prior evidence");
    Ok(())
}
#[test]
fn sex_refuses_readiness_before_native_or_report_replacement() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("assessment.json");
    std::fs::write(&path, b"prior evidence")?;
    let native = NativeCalls(Cell::new(0));
    let context = ProofContext { repo_root: dir.path(), readiness: &Refused, native: &native };
    let error = bijux_dna_bench_bam_proofs::sex::render_bam_sex_complete(&context, &path)
        .err()
        .ok_or_else(|| anyhow::anyhow!("invalid readiness accepted"))?;
    assert!(error.to_string().contains("canonical readiness unavailable"));
    assert_eq!(native.0.get(), 0);
    assert_eq!(std::fs::read(&path)?, b"prior evidence");
    Ok(())
}
