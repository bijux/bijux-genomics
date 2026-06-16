use std::path::{Path, PathBuf};

use crate::foundation::{BijuxError, Result};

use super::graph::ExecutionStep;
use super::io::{ArtifactRole, ArtifactSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypedArtifactKind {
    Fastq,
    Bam,
    BamIndex,
    Vcf,
    VcfIndex,
    Reference,
    Panel,
    Table,
    JsonMetrics,
    Database,
}

impl TypedArtifactKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
            Self::BamIndex => "bam_index",
            Self::Vcf => "vcf",
            Self::VcfIndex => "vcf_index",
            Self::Reference => "reference",
            Self::Panel => "panel",
            Self::Table => "table",
            Self::JsonMetrics => "json_metrics",
            Self::Database => "database",
        }
    }
}

macro_rules! typed_artifact_path {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            path: PathBuf,
        }

        impl $name {
            #[must_use]
            pub fn new(path: PathBuf) -> Self {
                Self { path }
            }

            #[must_use]
            pub fn path(&self) -> &Path {
                &self.path
            }
        }
    };
}

typed_artifact_path!(FastqArtifact);
typed_artifact_path!(BamArtifact);
typed_artifact_path!(BamIndexArtifact);
typed_artifact_path!(VcfArtifact);
typed_artifact_path!(VcfIndexArtifact);
typed_artifact_path!(ReferenceArtifact);
typed_artifact_path!(PanelArtifact);
typed_artifact_path!(TableArtifact);
typed_artifact_path!(JsonMetricsArtifact);
typed_artifact_path!(DatabaseArtifact);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedArtifactRef {
    Fastq(FastqArtifact),
    Bam(BamArtifact),
    BamIndex(BamIndexArtifact),
    Vcf(VcfArtifact),
    VcfIndex(VcfIndexArtifact),
    Reference(ReferenceArtifact),
    Panel(PanelArtifact),
    Table(TableArtifact),
    JsonMetrics(JsonMetricsArtifact),
    Database(DatabaseArtifact),
}

impl TypedArtifactRef {
    #[must_use]
    pub const fn kind(&self) -> TypedArtifactKind {
        match self {
            Self::Fastq(_) => TypedArtifactKind::Fastq,
            Self::Bam(_) => TypedArtifactKind::Bam,
            Self::BamIndex(_) => TypedArtifactKind::BamIndex,
            Self::Vcf(_) => TypedArtifactKind::Vcf,
            Self::VcfIndex(_) => TypedArtifactKind::VcfIndex,
            Self::Reference(_) => TypedArtifactKind::Reference,
            Self::Panel(_) => TypedArtifactKind::Panel,
            Self::Table(_) => TypedArtifactKind::Table,
            Self::JsonMetrics(_) => TypedArtifactKind::JsonMetrics,
            Self::Database(_) => TypedArtifactKind::Database,
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Self::Fastq(artifact) => artifact.path(),
            Self::Bam(artifact) => artifact.path(),
            Self::BamIndex(artifact) => artifact.path(),
            Self::Vcf(artifact) => artifact.path(),
            Self::VcfIndex(artifact) => artifact.path(),
            Self::Reference(artifact) => artifact.path(),
            Self::Panel(artifact) => artifact.path(),
            Self::Table(artifact) => artifact.path(),
            Self::JsonMetrics(artifact) => artifact.path(),
            Self::Database(artifact) => artifact.path(),
        }
    }

    /// # Errors
    /// Returns an error if the artifact cannot be classified into a typed handoff.
    pub fn from_input(step: &ExecutionStep, artifact: &ArtifactSpec) -> Result<Option<Self>> {
        let path = artifact.path.clone();
        let lower_name = artifact.name.as_str().to_ascii_lowercase();
        let lower_path = path.to_string_lossy().to_ascii_lowercase();
        let domain = stage_domain(step.stage_id.as_str());

        let typed = match artifact.role {
            ArtifactRole::Reads | ArtifactRole::TrimmedReads => {
                ensure_suffix(&path, &[".fastq", ".fq", ".fastq.gz", ".fq.gz"], artifact, "FASTQ")?;
                Some(Self::Fastq(FastqArtifact::new(path)))
            }
            ArtifactRole::Bam | ArtifactRole::DedupBam | ArtifactRole::Alignment => {
                ensure_suffix(&path, &[".bam", ".cram", ".sam"], artifact, "BAM/SAM/CRAM")?;
                Some(Self::Bam(BamArtifact::new(path)))
            }
            ArtifactRole::Variant => {
                ensure_suffix(
                    &path,
                    &[".vcf", ".vcf.gz", ".bcf", ".m3vcf", ".m3vcf.gz"],
                    artifact,
                    "VCF/BCF",
                )?;
                if lower_name.contains("panel")
                    || lower_path.contains("/panel")
                    || lower_path.contains("panel.")
                    || lower_path.ends_with(".m3vcf")
                    || lower_path.ends_with(".m3vcf.gz")
                {
                    Some(Self::Panel(PanelArtifact::new(path)))
                } else {
                    Some(Self::Vcf(VcfArtifact::new(path)))
                }
            }
            ArtifactRole::Index => {
                Some(match infer_index_kind(step, artifact, &lower_name, &lower_path)? {
                    TypedArtifactKind::BamIndex => Self::BamIndex(BamIndexArtifact::new(path)),
                    TypedArtifactKind::VcfIndex => Self::VcfIndex(VcfIndexArtifact::new(path)),
                    other => {
                        return Err(BijuxError::validation(format!(
                            "unsupported typed index artifact kind {} for {} ({})",
                            other.as_str(),
                            artifact.name,
                            artifact.path.display()
                        )))
                    }
                })
            }
            ArtifactRole::Reference => {
                Some(match infer_reference_adjacent_kind(&lower_name, &lower_path) {
                    TypedArtifactKind::Table => {
                        ensure_table_suffix(&path, artifact)?;
                        Self::Table(TableArtifact::new(path))
                    }
                    TypedArtifactKind::Database => Self::Database(DatabaseArtifact::new(path)),
                    _ => {
                        ensure_suffix(
                            &path,
                            &[".fa", ".fasta", ".fna", ".fa.gz", ".fasta.gz"],
                            artifact,
                            "reference FASTA",
                        )?;
                        Self::Reference(ReferenceArtifact::new(path))
                    }
                })
            }
            ArtifactRole::SummaryTsv => {
                ensure_table_suffix(&path, artifact)?;
                Some(Self::Table(TableArtifact::new(path)))
            }
            ArtifactRole::MetricsJson
            | ArtifactRole::MetricsEnvelope
            | ArtifactRole::Metrics
            | ArtifactRole::SummaryJson => {
                ensure_suffix(&path, &[".json"], artifact, "JSON metrics")?;
                Some(Self::JsonMetrics(JsonMetricsArtifact::new(path)))
            }
            ArtifactRole::ReportJson
            | ArtifactRole::Report
            | ArtifactRole::Log
            | ArtifactRole::StageReport
            | ArtifactRole::Provenance
            | ArtifactRole::Evidence
            | ArtifactRole::ReportHtml
            | ArtifactRole::Unknown => None,
        };

        if domain == "fastq"
            && matches!(
                typed.as_ref().map(Self::kind),
                Some(TypedArtifactKind::Bam)
                    | Some(TypedArtifactKind::BamIndex)
                    | Some(TypedArtifactKind::Vcf)
                    | Some(TypedArtifactKind::VcfIndex)
                    | Some(TypedArtifactKind::Panel)
            )
        {
            return Err(BijuxError::validation(format!(
                "typed artifact handoff rejected {} for {}: fastq stages cannot consume {}",
                artifact.path.display(),
                step.stage_id,
                typed.expect("typed artifact").kind().as_str()
            )));
        }

        Ok(typed)
    }
}

/// # Errors
/// Returns an error if any typed artifact handoff is incompatible with the declared stage input contract.
pub fn validate_typed_input_handoffs(step: &ExecutionStep) -> Result<()> {
    for artifact in &step.io.inputs {
        let _ = TypedArtifactRef::from_input(step, artifact)?;
    }
    Ok(())
}

fn infer_index_kind(
    step: &ExecutionStep,
    artifact: &ArtifactSpec,
    lower_name: &str,
    lower_path: &str,
) -> Result<TypedArtifactKind> {
    if lower_path.ends_with(".bai") || lower_path.ends_with(".crai") {
        return Ok(TypedArtifactKind::BamIndex);
    }
    if lower_path.ends_with(".tbi") {
        return Ok(TypedArtifactKind::VcfIndex);
    }
    if lower_path.ends_with(".csi") {
        if lower_name.contains("bam")
            || lower_path.contains(".bam.")
            || step.io.inputs.iter().any(|input| {
                matches!(
                    input.role,
                    ArtifactRole::Bam | ArtifactRole::DedupBam | ArtifactRole::Alignment
                )
            })
        {
            return Ok(TypedArtifactKind::BamIndex);
        }
        if lower_name.contains("vcf")
            || lower_name.contains("panel")
            || lower_path.contains(".vcf")
            || lower_path.contains(".bcf")
            || lower_path.contains(".m3vcf")
            || step.io.inputs.iter().any(|input| input.role == ArtifactRole::Variant)
        {
            return Ok(TypedArtifactKind::VcfIndex);
        }
    }
    Err(BijuxError::validation(format!(
        "typed artifact handoff could not resolve index kind for {} ({})",
        artifact.name,
        artifact.path.display()
    )))
}

fn infer_reference_adjacent_kind(lower_name: &str, lower_path: &str) -> TypedArtifactKind {
    if lower_name.contains("database")
        || lower_name.ends_with("_db")
        || lower_path.contains("/database")
        || lower_path.contains("/db/")
        || lower_path.ends_with(".db")
        || lower_path.ends_with(".sqlite")
        || lower_path.ends_with(".sqlite3")
    {
        return TypedArtifactKind::Database;
    }
    if lower_name.contains("table")
        || lower_name.contains("map")
        || lower_name.contains("bed")
        || lower_name.contains("segments")
        || lower_name.contains("tsv")
        || lower_path.ends_with(".tsv")
        || lower_path.ends_with(".csv")
        || lower_path.ends_with(".bed")
        || lower_path.ends_with(".map")
        || lower_path.ends_with(".txt")
    {
        return TypedArtifactKind::Table;
    }
    TypedArtifactKind::Reference
}

fn ensure_suffix(
    path: &Path,
    suffixes: &[&str],
    artifact: &ArtifactSpec,
    label: &str,
) -> Result<()> {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    if suffixes.iter().any(|suffix| lower.ends_with(suffix)) {
        return Ok(());
    }
    Err(BijuxError::validation(format!(
        "typed artifact handoff expected {label} for {} ({})",
        artifact.name,
        path.display()
    )))
}

fn ensure_table_suffix(path: &Path, artifact: &ArtifactSpec) -> Result<()> {
    ensure_suffix(path, &[".tsv", ".csv", ".bed", ".map", ".txt"], artifact, "table")
}

fn stage_domain(stage_id: &str) -> &str {
    stage_id.split('.').next().unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::contract::execution::graph::ExecutionStep;
    use crate::contract::execution::io::StageIO;
    use crate::contract::tooling::ToolConstraints;
    use crate::foundation::{CommandSpecV1, ContainerImageRefV1};
    use crate::ids::{ArtifactId, StageId, StepId};

    use super::*;

    fn step(stage_id: &str, inputs: Vec<ArtifactSpec>) -> ExecutionStep {
        ExecutionStep {
            step_id: StepId::new("step"),
            stage_id: StageId::new(stage_id),
            command: CommandSpecV1 { template: vec!["echo".to_string(), "ok".to_string()] },
            image: ContainerImageRefV1 { image: "example/tool".to_string(), digest: None },
            resources: ToolConstraints::default(),
            io: StageIO { inputs, outputs: Vec::new() },
            out_dir: PathBuf::from("out"),
            aux_images: BTreeMap::new(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        }
    }

    #[test]
    fn typed_artifact_inference_covers_primary_handoff_kinds() -> Result<()> {
        let fastq = ArtifactSpec::required(
            ArtifactId::new("reads"),
            PathBuf::from("reads.fastq.gz"),
            ArtifactRole::Reads,
        );
        let bam = ArtifactSpec::required(
            ArtifactId::new("input_bam"),
            PathBuf::from("aligned.bam"),
            ArtifactRole::Bam,
        );
        let bai = ArtifactSpec::required(
            ArtifactId::new("input_bam_index"),
            PathBuf::from("aligned.bam.bai"),
            ArtifactRole::Index,
        );
        let vcf = ArtifactSpec::required(
            ArtifactId::new("vcf"),
            PathBuf::from("calls.vcf.gz"),
            ArtifactRole::Variant,
        );
        let tbi = ArtifactSpec::required(
            ArtifactId::new("vcf_index"),
            PathBuf::from("calls.vcf.gz.tbi"),
            ArtifactRole::Index,
        );
        let reference = ArtifactSpec::required(
            ArtifactId::new("reference_fasta"),
            PathBuf::from("reference.fa"),
            ArtifactRole::Reference,
        );
        let panel = ArtifactSpec::required(
            ArtifactId::new("reference_panel_vcf"),
            PathBuf::from("panels/panel.vcf.gz"),
            ArtifactRole::Variant,
        );
        let table = ArtifactSpec::required(
            ArtifactId::new("genetic_map_tsv"),
            PathBuf::from("maps/grch38.tsv"),
            ArtifactRole::Reference,
        );
        let metrics = ArtifactSpec::required(
            ArtifactId::new("metrics_json"),
            PathBuf::from("metrics.json"),
            ArtifactRole::MetricsJson,
        );
        let database = ArtifactSpec::required(
            ArtifactId::new("taxonomy_database"),
            PathBuf::from("taxonomy/database.sqlite"),
            ArtifactRole::Reference,
        );
        let step = step(
            "vcf.call",
            vec![
                fastq.clone(),
                bam.clone(),
                bai.clone(),
                vcf.clone(),
                tbi.clone(),
                reference.clone(),
                panel.clone(),
                table.clone(),
                metrics.clone(),
                database.clone(),
            ],
        );

        assert_eq!(
            TypedArtifactRef::from_input(&step, &fastq)?.expect("fastq").kind(),
            TypedArtifactKind::Fastq
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &bam)?.expect("bam").kind(),
            TypedArtifactKind::Bam
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &bai)?.expect("bai").kind(),
            TypedArtifactKind::BamIndex
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &vcf)?.expect("vcf").kind(),
            TypedArtifactKind::Vcf
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &tbi)?.expect("tbi").kind(),
            TypedArtifactKind::VcfIndex
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &reference)?.expect("reference").kind(),
            TypedArtifactKind::Reference
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &panel)?.expect("panel").kind(),
            TypedArtifactKind::Panel
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &table)?.expect("table").kind(),
            TypedArtifactKind::Table
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &metrics)?.expect("metrics").kind(),
            TypedArtifactKind::JsonMetrics
        );
        assert_eq!(
            TypedArtifactRef::from_input(&step, &database)?.expect("database").kind(),
            TypedArtifactKind::Database
        );
        Ok(())
    }

    #[test]
    fn typed_artifact_handoffs_reject_reads_declared_with_bam_paths() {
        let step = step(
            "fastq.validate_reads",
            vec![ArtifactSpec::required(
                ArtifactId::new("reads"),
                PathBuf::from("aligned.bam"),
                ArtifactRole::Reads,
            )],
        );

        let err =
            validate_typed_input_handoffs(&step).expect_err("reads role must reject BAM path");
        assert!(err.to_string().contains("expected FASTQ"), "{err}");
    }
}
