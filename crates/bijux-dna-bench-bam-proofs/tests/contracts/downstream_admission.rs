#![cfg(test)]
#[cfg(feature = "bam_downstream")]
mod enabled {
    use super::super::source_admission::{NativeCalls, Refused};
    use anyhow::{bail, Result};
    use bijux_dna_bench_bam_evidence::readiness::kinship::BamKinshipReadyReport;
    use bijux_dna_bench_bam_proofs::{
        downstream::{
            observations::{
                ParserObservation, PipelineObservation, PublishedVariantCounts,
                PublishedVariantMissingness,
            },
            ports::{KinshipReadiness, ParserEvidence, PipelineEvidence, VariantEvidence},
            DownstreamContext,
        },
        ProofContext,
    };
    use std::{cell::Cell, path::Path};
    impl ParserEvidence for Refused {
        fn genotyping(&self, _root: &Path) -> Result<ParserObservation> {
            bail!("unexpected parser acquisition")
        }
        fn all(&self, _root: &Path) -> Result<Vec<ParserObservation>> {
            bail!("unexpected parser acquisition")
        }
    }
    impl PipelineEvidence for Refused {
        fn validate(
            &self,
            _root: &Path,
            _pipeline_id: &str,
            _output_path: &Path,
        ) -> Result<PipelineObservation> {
            bail!("unexpected pipeline acquisition")
        }
    }
    impl VariantEvidence for Refused {
        fn counts(&self, _root: &Path) -> Result<PublishedVariantCounts> {
            bail!("unexpected count acquisition")
        }
        fn missingness(&self, _root: &Path) -> Result<PublishedVariantMissingness> {
            bail!("unexpected missingness acquisition")
        }
    }
    impl KinshipReadiness for Refused {
        fn kinship(&self, _root: &Path) -> Result<BamKinshipReadyReport> {
            bail!("canonical readiness unavailable")
        }
    }
    fn context<'a>(root: &'a Path, native: &'a NativeCalls) -> DownstreamContext<'a> {
        DownstreamContext {
            proofs: ProofContext { repo_root: root, readiness: &Refused, native },
            parsers: &Refused,
            pipelines: &Refused,
            variants: &Refused,
            kinship: &Refused,
        }
    }
    #[test]
    fn genotyping_source_failure_preserves_existing_report() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let output = dir.path().join("report.json");
        std::fs::write(&output, b"prior evidence")?;
        let native = NativeCalls(Cell::new(0));
        let error = bijux_dna_bench_bam_proofs::genotyping::render_bam_genotyping_complete(
            &context(dir.path(), &native),
            &output,
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("failed readiness accepted"))?;
        assert!(error.to_string().contains("canonical readiness unavailable"));
        assert_eq!(native.0.get(), 0);
        assert_eq!(std::fs::read(output)?, b"prior evidence");
        Ok(())
    }
    #[test]
    fn kinship_source_failure_preserves_existing_report() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let output = dir.path().join("report.json");
        std::fs::write(&output, b"prior evidence")?;
        let native = NativeCalls(Cell::new(0));
        let error = bijux_dna_bench_bam_proofs::kinship::render_bam_kinship_complete(
            &context(dir.path(), &native),
            &output,
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("failed readiness accepted"))?;
        assert!(error.to_string().contains("canonical readiness unavailable"));
        assert_eq!(native.0.get(), 0);
        assert_eq!(std::fs::read(output)?, b"prior evidence");
        Ok(())
    }
}
#[cfg(not(feature = "bam_downstream"))]
#[test]
fn genotyping_feature_requirement_is_explicit() {
    assert!(bijux_dna_bench_bam_proofs::genotyping::unavailable().is_err());
}
#[cfg(not(feature = "bam_downstream"))]
#[test]
fn kinship_feature_requirement_is_explicit() {
    assert!(bijux_dna_bench_bam_proofs::kinship::unavailable().is_err());
}
