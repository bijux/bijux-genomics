use std::io::BufRead;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use flate2::read::MultiGzDecoder;

fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|value| value.eq_ignore_ascii_case(ext))
}

fn file_name_ends_with(path: &Path, suffix: &str) -> bool {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(suffix))
}

/// # Errors
/// Returns an error when a VCF/BCF input lacks an index.
pub fn validate_bgzip_tabix(input: &Path) -> Result<()> {
    if file_name_ends_with(input, ".vcf.gz") || has_extension(input, "vcf") {
        let tbi = Path::new(&format!("{}.tbi", input.display())).to_path_buf();
        let csi = Path::new(&format!("{}.csi", input.display())).to_path_buf();
        if !tbi.exists() && !csi.exists() {
            bail!(
                "input contract violation: missing VCF index (.tbi/.csi) for {}",
                input.display()
            );
        }
    }
    if has_extension(input, "bcf") {
        let index_candidates = [input.with_extension("bcf.csi"), input.with_extension("csi")];
        if !index_candidates.iter().any(|path| path.exists()) {
            bail!("input contract violation: missing BCF index (.csi) for {}", input.display());
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error when BAM/CRAM input lacks an index.
pub fn validate_bam_index(input: &Path) -> Result<()> {
    if has_extension(input, "bam") {
        let index_candidates = [
            input.with_extension("bam.bai"),
            input.with_extension("bai"),
            input.with_extension("bam.csi"),
            input.with_extension("csi"),
        ];
        if !index_candidates.iter().any(|path| path.exists()) {
            bail!(
                "input contract violation: missing BAM index (.bai/.csi) for {}",
                input.display()
            );
        }
    }
    if has_extension(input, "cram") {
        let index_candidates = [input.with_extension("cram.crai"), input.with_extension("crai")];
        if !index_candidates.iter().any(|path| path.exists()) {
            bail!("input contract violation: missing CRAM index (.crai) for {}", input.display());
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error when FASTQ content does not start with a header marker.
pub fn validate_fastq_format(input: &Path) -> Result<()> {
    let is_gzip_fastq =
        file_name_ends_with(input, ".fastq.gz") || file_name_ends_with(input, ".fq.gz");
    if !(has_extension(input, "fastq") || has_extension(input, "fq") || is_gzip_fastq) {
        return Ok(());
    }
    let file = std::fs::File::open(input).with_context(|| format!("open {}", input.display()))?;
    let mut reader: Box<dyn BufRead> = if is_gzip_fastq {
        Box::new(std::io::BufReader::new(MultiGzDecoder::new(file)))
    } else {
        Box::new(std::io::BufReader::new(file))
    };
    let mut first = String::new();
    let _ = reader.read_line(&mut first).with_context(|| format!("read {}", input.display()))?;
    if !first.starts_with('@') {
        bail!("input contract violation: FASTQ header must start with '@' ({})", input.display());
    }
    Ok(())
}

/// # Errors
/// Returns an error if any known input contract fails.
pub fn validate_stage_inputs(step: &ExecutionStep) -> Result<()> {
    for artifact in &step.io.inputs {
        let path = &artifact.path;
        if !path.exists() {
            if !artifact.optional {
                bail!(
                    "input contract violation: missing required input {} ({})",
                    artifact.name,
                    path.display()
                );
            }
            continue;
        }
        validate_bgzip_tabix(path)?;
        validate_bam_index(path)?;
        validate_fastq_format(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Write;
    use std::path::PathBuf;

    use bijux_dna_core::contract::ExecutionStep;
    use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
    use bijux_dna_core::prelude::{
        ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
    };
    use flate2::write::GzEncoder;
    use flate2::Compression;

    use super::{
        validate_bam_index, validate_bgzip_tabix, validate_fastq_format, validate_stage_inputs,
    };

    fn validation_step(inputs: Vec<ArtifactSpec>) -> ExecutionStep {
        ExecutionStep {
            step_id: StepId::new("test.validate"),
            stage_id: StageId::new("test.validate"),
            command: CommandSpecV1 { template: vec!["echo".to_string(), "ok".to_string()] },
            image: ContainerImageRefV1 { image: "example/validator:1".to_string(), digest: None },
            resources: ToolConstraints::default(),
            io: StageIO { inputs, outputs: Vec::new() },
            out_dir: PathBuf::from("out"),
            aux_images: BTreeMap::default(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        }
    }

    #[test]
    fn validate_fastq_format_accepts_gzipped_fastq() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("reads.fastq.gz");
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"@read-1\nACGT\n+\n!!!!\n")?;
        bijux_dna_infra::write_bytes(&path, encoder.finish()?)?;

        validate_fastq_format(&path)
    }

    #[test]
    fn validate_bam_index_accepts_sibling_bai() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let bam = temp.path().join("sample.bam");
        bijux_dna_infra::write_bytes(&bam, b"bam")?;
        bijux_dna_infra::write_bytes(temp.path().join("sample.bai"), b"index")?;

        validate_bam_index(&bam)
    }

    #[test]
    fn validate_bam_index_accepts_sibling_crai() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let cram = temp.path().join("sample.cram");
        bijux_dna_infra::write_bytes(&cram, b"cram")?;
        bijux_dna_infra::write_bytes(temp.path().join("sample.crai"), b"index")?;

        validate_bam_index(&cram)
    }

    #[test]
    fn validate_bgzip_tabix_accepts_sibling_bcf_csi() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let bcf = temp.path().join("sample.bcf");
        bijux_dna_infra::write_bytes(&bcf, b"bcf")?;
        bijux_dna_infra::write_bytes(temp.path().join("sample.csi"), b"index")?;

        validate_bgzip_tabix(&bcf)
    }

    #[test]
    fn validate_stage_inputs_rejects_missing_required_artifact() {
        let step = validation_step(vec![ArtifactSpec::required(
            ArtifactId::new("reads"),
            PathBuf::from("missing.fastq"),
            ArtifactRole::Reads,
        )]);

        let err = match validate_stage_inputs(&step) {
            Ok(()) => panic!("missing input should fail"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("missing required input reads"), "{err}");
    }

    #[test]
    fn validate_stage_inputs_allows_missing_optional_artifact() -> anyhow::Result<()> {
        let step = validation_step(vec![ArtifactSpec::optional(
            ArtifactId::new("reads"),
            PathBuf::from("missing.fastq"),
            ArtifactRole::Reads,
        )]);

        validate_stage_inputs(&step)
    }
}
