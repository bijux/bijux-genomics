use anyhow::{bail, Context};
use bijux_dna_policies::{
    admit_release_evidence, derive_release_evidence, qualification_sources_sha256,
    ContainerReleaseEvidencePublication, QualificationReport, ReleaseEvidenceDerivationRequest,
};

use super::{path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result, Workspace};

const SCHEMA_PATH: &str = "containers/contracts/schemas/release-evidence.v1.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run release-evidence -- compose <qualification-report> <released-at-utc> <output>\n  bijux-dna-dev containers run release-evidence -- admit <qualification-report> <release-publication>\n  bijux-dna-dev containers run release-evidence -- generate-schema [<output>]\n  bijux-dna-dev containers run release-evidence -- check-schema";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "compose" => compose(workspace, options),
        "admit" => admit(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        _ => bail!(USAGE),
    }
}

fn compose(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [qualification_path, released_at_utc, output_path] = args else {
        bail!(USAGE);
    };
    let qualification = load_qualification(workspace, qualification_path)?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&workspace.root)?;
    let catalog_sha256 = catalog_sha256(&qualification)?;
    let qualification_sources_sha256 = qualification_sources_sha256(&qualification.sources)?;
    let publication = derive_release_evidence(&ReleaseEvidenceDerivationRequest {
        released_at_utc,
        repository_commit: &repository_commit,
        catalog_sha256,
        qualification_sources_sha256: &qualification_sources_sha256,
        assessments: &qualification.assessments,
    })?;
    let output = path_from_arg(workspace, output_path);
    bijux_dna_infra::atomic_write_json(&output, &publication)?;
    success_line(format!(
        "release evidence: wrote {} tools to {}",
        publication.tools.len(),
        output.display()
    ))
}

fn admit(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [qualification_path, publication_path] = args else {
        bail!(USAGE);
    };
    let qualification = load_qualification(workspace, qualification_path)?;
    let publication_path = path_from_arg(workspace, publication_path);
    let publication: ContainerReleaseEvidencePublication =
        serde_json::from_str(&read_utf8(&publication_path)?)
            .with_context(|| format!("parse release publication {}", publication_path.display()))?;
    let publication_sha256 = bijux_dna_infra::hash_file_sha256(&publication_path)?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&workspace.root)?;
    let catalog_sha256 = catalog_sha256(&qualification)?;
    let qualification_sources_sha256 = qualification_sources_sha256(&qualification.sources)?;
    let admitted = admit_release_evidence(
        &publication,
        &publication_sha256,
        &repository_commit,
        catalog_sha256,
        &qualification_sources_sha256,
        &qualification.assessments,
    )?;
    success_line(format!(
        "release evidence: admitted {} for {} tools",
        admitted.publication_sha256(),
        publication.tools.len()
    ))
}

fn generate_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let output = match args {
        [] => workspace.path(SCHEMA_PATH),
        [path] => path_from_arg(workspace, path),
        _ => bail!(USAGE),
    };
    bijux_dna_infra::atomic_write_json(&output, &bijux_dna_policies::release_evidence_schema())?;
    success_line(format!("release evidence: generated {}", output.display()))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        bail!(USAGE);
    }
    let checked: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(SCHEMA_PATH))?)?;
    let generated = bijux_dna_policies::release_evidence_schema();
    if checked != generated {
        return Ok(ContainerCommandOutcome::failure(format!(
            "{SCHEMA_PATH} drifted from the Rust release-evidence contract\n"
        )));
    }
    success_line("release evidence schema: OK")
}

fn load_qualification(workspace: &Workspace, path: &str) -> Result<QualificationReport> {
    let path = path_from_arg(workspace, path);
    let report: QualificationReport = serde_json::from_str(&read_utf8(&path)?)
        .with_context(|| format!("parse qualification report {}", path.display()))?;
    report.validate()?;
    Ok(report)
}

fn catalog_sha256(report: &QualificationReport) -> Result<&str> {
    report
        .sources
        .get("catalog")
        .map(|source| source.sha256.as_str())
        .context("qualification report omits the catalog source")
}
