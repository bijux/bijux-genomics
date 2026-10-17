use std::path::Path;

use anyhow::{anyhow, Result};

use super::RepoSandbox;

#[allow(dead_code)]
impl RepoSandbox {
    pub fn materialize_vcf_score_evidence(&self, home: &Path) -> Result<()> {
        const EVIDENCE_COMMANDS: &[&[&str]] = &[
            &["bench", "local", "run-vcf-admixture-smoke"],
            &["bench", "local", "run-vcf-call-smoke"],
            &["bench", "local", "run-vcf-call-diploid-smoke"],
            &["bench", "local", "run-vcf-call-gl-smoke"],
            &["bench", "local", "run-vcf-call-pseudohaploid-smoke"],
            &["bench", "local", "run-vcf-damage-filter-smoke"],
            &["bench", "local", "run-vcf-demography-smoke"],
            &["bench", "local", "run-vcf-filter-smoke"],
            &["bench", "local", "run-vcf-gl-propagation-smoke"],
            &["bench", "local", "run-vcf-ibd-smoke"],
            &["bench", "local", "run-vcf-ibd-smoke", "--tool-id", "ibdseq"],
            &["bench", "local", "run-vcf-impute-smoke"],
            &["bench", "local", "run-vcf-imputation-metrics-smoke"],
            &["bench", "local", "run-vcf-pca-smoke"],
            &["bench", "local", "run-vcf-pca-smoke", "--tool-id", "eigensoft"],
            &["bench", "local", "run-vcf-phasing-smoke"],
            &["bench", "local", "run-vcf-population-structure-smoke"],
            &["bench", "local", "run-vcf-postprocess-smoke"],
            &["bench", "local", "run-vcf-prepare-reference-panel-smoke"],
            &["bench", "local", "run-vcf-qc-smoke", "--tool-id", "bcftools"],
            &["bench", "local", "run-vcf-qc-smoke", "--tool-id", "plink"],
            &["bench", "local", "run-vcf-qc-smoke"],
            &["bench", "local", "run-vcf-roh-smoke"],
            &["bench", "local", "run-vcf-stats-smoke"],
            &["bench", "run-micro"],
        ];

        for args in EVIDENCE_COMMANDS {
            let output = self
                .bijux_dna_command()
                .env("HOME", home)
                .env("BIJUX_SKIP_QA", "1")
                .env("BIJUX_ALLOW_SILVER", "1")
                .env("BIJUX_SKIP_IMAGE_CHECK", "1")
                .args(*args)
                .output()
                .map_err(|error| anyhow!("run VCF score evidence command: {error}"))?;
            if !output.status.success() {
                return Err(anyhow!(
                    "VCF score evidence command failed: {}\ncommand: {}\nstdout:\n{}\nstderr:\n{}",
                    output.status,
                    args.join(" "),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        Ok(())
    }

    pub fn materialize_bam_score_evidence(&self, home: &Path) -> Result<()> {
        const EVIDENCE_COMMANDS: &[&[&str]] = &[
            &["bench", "local", "run-bam-stage-smoke", "--stage-id", "bam.coverage"],
            &["bench", "readiness", "render-bam-contamination-complete"],
            &["bench", "local", "run-bam-stage-smoke", "--stage-id", "bam.gc_bias"],
        ];

        for args in EVIDENCE_COMMANDS {
            let output = self
                .bijux_dna_command()
                .env("HOME", home)
                .env("BIJUX_SKIP_QA", "1")
                .env("BIJUX_ALLOW_SILVER", "1")
                .env("BIJUX_SKIP_IMAGE_CHECK", "1")
                .args(*args)
                .output()?;
            if !output.status.success() {
                return Err(anyhow!(
                    "BAM score evidence command `{}` failed: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
        }
        Ok(())
    }
}
