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
        let typed = classify_input_artifact(step, artifact, &path, &lower_name, &lower_path)?;

        if domain == "fastq" {
            if let Some(kind) = typed.as_ref().map(Self::kind) {
                if matches!(
                    kind,
                    TypedArtifactKind::Bam
                        | TypedArtifactKind::BamIndex
                        | TypedArtifactKind::Vcf
                        | TypedArtifactKind::VcfIndex
                        | TypedArtifactKind::Panel
                ) {
                    return Err(BijuxError::validation(format!(
                        "typed artifact handoff rejected {} for {}: fastq stages cannot consume {}",
                        artifact.path.display(),
                        step.stage_id,
                        kind.as_str()
                    )));
                }
            }
        }

        Ok(typed)
    }
}

fn classify_input_artifact(
    step: &ExecutionStep,
    artifact: &ArtifactSpec,
    path: &Path,
    lower_name: &str,
    lower_path: &str,
) -> Result<Option<TypedArtifactRef>> {
    match artifact.role {
        ArtifactRole::Reads | ArtifactRole::TrimmedReads => classify_fastq_artifact(path, artifact),
        ArtifactRole::Bam | ArtifactRole::DedupBam | ArtifactRole::Alignment => {
            classify_bam_artifact(path, artifact)
        }
        ArtifactRole::Variant => classify_variant_artifact(path, artifact, lower_name, lower_path),
        ArtifactRole::Index => {
            classify_index_artifact(step, artifact, path, lower_name, lower_path)
        }
        ArtifactRole::Reference => {
            classify_reference_artifact(path, artifact, lower_name, lower_path)
        }
        ArtifactRole::SummaryTsv => classify_table_artifact(path, artifact),
        ArtifactRole::MetricsJson
        | ArtifactRole::MetricsEnvelope
        | ArtifactRole::Metrics
        | ArtifactRole::SummaryJson => classify_metrics_artifact(path, artifact),
        ArtifactRole::ReportJson
        | ArtifactRole::Report
        | ArtifactRole::Log
        | ArtifactRole::StageReport
        | ArtifactRole::Provenance
        | ArtifactRole::Evidence
        | ArtifactRole::ReportHtml
        | ArtifactRole::Unknown => Ok(None),
    }
}

fn classify_fastq_artifact(
    path: &Path,
    artifact: &ArtifactSpec,
) -> Result<Option<TypedArtifactRef>> {
    ensure_suffix(path, &[".fastq", ".fq", ".fastq.gz", ".fq.gz"], artifact, "FASTQ")?;
    Ok(Some(TypedArtifactRef::Fastq(FastqArtifact::new(path.to_path_buf()))))
}

fn classify_bam_artifact(path: &Path, artifact: &ArtifactSpec) -> Result<Option<TypedArtifactRef>> {
    ensure_suffix(path, &[".bam", ".cram", ".sam"], artifact, "BAM/SAM/CRAM")?;
    Ok(Some(TypedArtifactRef::Bam(BamArtifact::new(path.to_path_buf()))))
}

fn classify_variant_artifact(
    path: &Path,
    artifact: &ArtifactSpec,
    lower_name: &str,
    lower_path: &str,
) -> Result<Option<TypedArtifactRef>> {
    ensure_suffix(path, &[".vcf", ".vcf.gz", ".bcf", ".m3vcf", ".m3vcf.gz"], artifact, "VCF/BCF")?;
    let typed = if lower_name.contains("panel")
        || lower_path.contains("/panel")
        || lower_path.contains("panel.")
        || path_has_suffix_ignore_ascii_case(lower_path, ".m3vcf")
        || path_has_suffix_ignore_ascii_case(lower_path, ".m3vcf.gz")
    {
        TypedArtifactRef::Panel(PanelArtifact::new(path.to_path_buf()))
    } else {
        TypedArtifactRef::Vcf(VcfArtifact::new(path.to_path_buf()))
    };
    Ok(Some(typed))
}

fn classify_index_artifact(
    step: &ExecutionStep,
    artifact: &ArtifactSpec,
    path: &Path,
    lower_name: &str,
    lower_path: &str,
) -> Result<Option<TypedArtifactRef>> {
    let typed = match infer_index_kind(step, artifact, lower_name, lower_path)? {
        TypedArtifactKind::BamIndex => {
            TypedArtifactRef::BamIndex(BamIndexArtifact::new(path.to_path_buf()))
        }
        TypedArtifactKind::VcfIndex => {
            TypedArtifactRef::VcfIndex(VcfIndexArtifact::new(path.to_path_buf()))
        }
        other => {
            return Err(BijuxError::validation(format!(
                "unsupported typed index artifact kind {} for {} ({})",
                other.as_str(),
                artifact.name,
                artifact.path.display()
            )))
        }
    };
    Ok(Some(typed))
}

fn classify_reference_artifact(
    path: &Path,
    artifact: &ArtifactSpec,
    lower_name: &str,
    lower_path: &str,
) -> Result<Option<TypedArtifactRef>> {
    let typed = match infer_reference_adjacent_kind(lower_name, lower_path) {
        TypedArtifactKind::Table => {
            ensure_table_suffix(path, artifact)?;
            TypedArtifactRef::Table(TableArtifact::new(path.to_path_buf()))
        }
        TypedArtifactKind::Database => {
            TypedArtifactRef::Database(DatabaseArtifact::new(path.to_path_buf()))
        }
        _ => {
            ensure_suffix(
                path,
                &[".fa", ".fasta", ".fna", ".fa.gz", ".fasta.gz"],
                artifact,
                "reference FASTA",
            )?;
            TypedArtifactRef::Reference(ReferenceArtifact::new(path.to_path_buf()))
        }
    };
    Ok(Some(typed))
}

fn classify_table_artifact(
    path: &Path,
    artifact: &ArtifactSpec,
) -> Result<Option<TypedArtifactRef>> {
    ensure_table_suffix(path, artifact)?;
    Ok(Some(TypedArtifactRef::Table(TableArtifact::new(path.to_path_buf()))))
}

fn classify_metrics_artifact(
    path: &Path,
    artifact: &ArtifactSpec,
) -> Result<Option<TypedArtifactRef>> {
    ensure_suffix(path, &[".json"], artifact, "JSON metrics")?;
    Ok(Some(TypedArtifactRef::JsonMetrics(JsonMetricsArtifact::new(path.to_path_buf()))))
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
    if path_has_extension_ignore_ascii_case(lower_path, "bai")
        || path_has_extension_ignore_ascii_case(lower_path, "crai")
    {
        return Ok(TypedArtifactKind::BamIndex);
    }
    if path_has_extension_ignore_ascii_case(lower_path, "tbi") {
        return Ok(TypedArtifactKind::VcfIndex);
    }
    if path_has_extension_ignore_ascii_case(lower_path, "csi") {
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
        || path_has_extension_ignore_ascii_case(lower_path, "db")
        || path_has_extension_ignore_ascii_case(lower_path, "sqlite")
        || path_has_extension_ignore_ascii_case(lower_path, "sqlite3")
    {
        return TypedArtifactKind::Database;
    }
    if lower_name.contains("table")
        || lower_name.contains("map")
        || lower_name.contains("bed")
        || lower_name.contains("segments")
        || lower_name.contains("tsv")
        || path_has_extension_ignore_ascii_case(lower_path, "tsv")
        || path_has_extension_ignore_ascii_case(lower_path, "csv")
        || path_has_extension_ignore_ascii_case(lower_path, "bed")
        || path_has_extension_ignore_ascii_case(lower_path, "map")
        || path_has_extension_ignore_ascii_case(lower_path, "txt")
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

fn path_has_extension_ignore_ascii_case(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn path_has_suffix_ignore_ascii_case(path: &str, suffix: &str) -> bool {
    path.len() >= suffix.len() && path[path.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
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
    use crate::ids::{ArtifactId, DomainKind, StageId, StepId};

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

    fn domain_stage_id(domain: DomainKind, stage_name: &str) -> String {
        format!("{}.{}", domain.as_str(), stage_name)
    }

    fn required_kind(step: &ExecutionStep, artifact: &ArtifactSpec) -> Result<TypedArtifactKind> {
        TypedArtifactRef::from_input(step, artifact)?.map_or_else(
            || {
                Err(BijuxError::validation(format!(
                    "typed artifact inference returned None for {} ({})",
                    artifact.name,
                    artifact.path.display()
                )))
            },
            |typed| Ok(typed.kind()),
        )
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
        let bam_index = ArtifactSpec::required(
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
                bam_index.clone(),
                vcf.clone(),
                tbi.clone(),
                reference.clone(),
                panel.clone(),
                table.clone(),
                metrics.clone(),
                database.clone(),
            ],
        );

        assert_eq!(required_kind(&step, &fastq)?, TypedArtifactKind::Fastq);
        assert_eq!(required_kind(&step, &bam)?, TypedArtifactKind::Bam);
        assert_eq!(required_kind(&step, &bam_index)?, TypedArtifactKind::BamIndex);
        assert_eq!(required_kind(&step, &vcf)?, TypedArtifactKind::Vcf);
        assert_eq!(required_kind(&step, &tbi)?, TypedArtifactKind::VcfIndex);
        assert_eq!(required_kind(&step, &reference)?, TypedArtifactKind::Reference);
        assert_eq!(required_kind(&step, &panel)?, TypedArtifactKind::Panel);
        assert_eq!(required_kind(&step, &table)?, TypedArtifactKind::Table);
        assert_eq!(required_kind(&step, &metrics)?, TypedArtifactKind::JsonMetrics);
        assert_eq!(required_kind(&step, &database)?, TypedArtifactKind::Database);
        Ok(())
    }

    #[test]
    fn typed_artifact_handoffs_reject_reads_declared_with_bam_paths() {
        let step = step(
            &domain_stage_id(DomainKind::Fastq, "validate_reads"),
            vec![ArtifactSpec::required(
                ArtifactId::new("reads"),
                PathBuf::from("aligned.bam"),
                ArtifactRole::Reads,
            )],
        );

        let err = match validate_typed_input_handoffs(&step) {
            Ok(()) => panic!("reads role must reject BAM path"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("expected FASTQ"), "{err}");
    }
}
