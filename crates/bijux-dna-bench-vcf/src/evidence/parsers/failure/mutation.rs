use super::cases::{VcfParserFailureCase, VcfParserFailureReason};
use anyhow::Context;
use anyhow::Result;
use std::path::{Path, PathBuf};
pub(super) fn materialize_failure_probe(
    probe_root: &Path,
    case: &VcfParserFailureCase,
) -> Result<PathBuf> {
    let path = probe_root.join(case.probe_artifact);
    match case.failure_reason {
        VcfParserFailureReason::EmptyOutput => {
            bijux_dna_infra::write_string(&path, "")
                .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MalformedVcf => {
            bijux_dna_infra::write_string(
                &path,
                "##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample_a\nchr1\n",
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MissingIndex => {
            if path.exists() {
                bijux_dna_infra::remove_file(&path)
                    .with_context(|| format!("remove {}", path.display()))?;
            }
        }
        VcfParserFailureReason::MissingSampleColumn => {
            bijux_dna_infra::write_string(
                &path,
                "##fileformat=VCFv4.3\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample_a\nchr1\t10\t.\tA\tG\t60\tPASS\t.\tGT\n",
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MalformedPcaTable => {
            bijux_dna_infra::write_string(&path, "sample_a cohort_alpha\n")
                .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MalformedImputationQualityFile => {
            bijux_dna_infra::write_string(&path, "{ malformed json")
                .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MalformedSegmentFile => {
            bijux_dna_infra::write_string(
                &path,
                "sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count\nsample_a\tsample_b\tchr1\t100\t200\t3.0\n",
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
    }
    Ok(path)
}
