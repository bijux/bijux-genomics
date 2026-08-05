mod render;

use super::{
    out_path_arg, path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result,
    Workspace,
};

const SCAN_SCHEMA_PATH: &str = "containers/contracts/schemas/image-inventory-scan.v1.json";
const PUBLICATION_SCHEMA_PATH: &str =
    "containers/contracts/schemas/image-inventory-publication.v2.json";
const PUBLICATION_PATH: &str = "containers/reports/image-inventory.json";
const REPORT_PATH: &str = "containers/reports/image-inventory.md";
const GENERATE_USAGE: &str = "Usage: cargo run -p bijux-dna-dev -- containers run generate-image-inventory -- <repository-root> <study-root> <image-root> <build-evidence-root> <run-evidence-root> <scan-output> <publication-output>";

pub(super) fn generate(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(GENERATE_USAGE);
    }
    let (request, scan_output, publication_output) = parse_generate_args(workspace, args)?;
    if scan_output == publication_output {
        anyhow::bail!("image-inventory scan and publication outputs must be distinct");
    }
    if scan_output.try_exists()? || publication_output.try_exists()? {
        anyhow::bail!("refusing to replace existing image-inventory evidence output");
    }
    let evidence = bijux_dna_environment::evidence::derive_image_inventory_evidence(&request)?;
    bijux_dna_infra::atomic_create_json(&scan_output, &evidence.scan)?;
    bijux_dna_infra::atomic_create_json(&publication_output, &evidence.publication)?;
    success_line(format!(
        "generated immutable image inventory for {} tools at {} and {}",
        evidence.publication.tools.len(),
        scan_output.display(),
        publication_output.display()
    ))
}

fn parse_generate_args(
    workspace: &Workspace,
    args: &[String],
) -> Result<(
    bijux_dna_environment::evidence::ImageInventoryDerivationRequest,
    std::path::PathBuf,
    std::path::PathBuf,
)> {
    let [repository_root, study_root, image_root, build_evidence_root, run_evidence_root, scan_output, publication_output] =
        args
    else {
        anyhow::bail!(GENERATE_USAGE);
    };
    Ok((
        bijux_dna_environment::evidence::ImageInventoryDerivationRequest {
            repository_root: path_from_arg(workspace, repository_root),
            study_root: path_from_arg(workspace, study_root),
            image_root: path_from_arg(workspace, image_root),
            build_evidence_root: path_from_arg(workspace, build_evidence_root),
            run_evidence_root: path_from_arg(workspace, run_evidence_root),
        },
        path_from_arg(workspace, scan_output),
        path_from_arg(workspace, publication_output),
    ))
}

pub(super) fn generate_scan_schema(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run generate-image-inventory-scan-schema -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let output = out_path_arg(workspace, args, SCAN_SCHEMA_PATH, usage)?;
    bijux_dna_environment::evidence::write_image_inventory_scan_schema(&output)?;
    success_line(format!("generated {}", output.display()))
}

pub(super) fn generate_publication_schema(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run generate-image-inventory-publication-schema -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let output = out_path_arg(workspace, args, PUBLICATION_SCHEMA_PATH, usage)?;
    bijux_dna_environment::evidence::write_image_inventory_publication_schema(&output)?;
    success_line(format!("generated {}", output.display()))
}

pub(super) fn check_scan_schema(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    check_schema(
        workspace,
        SCAN_SCHEMA_PATH,
        bijux_dna_environment::evidence::image_inventory_scan_schema(),
        "scan",
    )
}

pub(super) fn check_publication_schema(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    check_schema(
        workspace,
        PUBLICATION_SCHEMA_PATH,
        bijux_dna_environment::evidence::image_inventory_publication_schema(),
        "publication",
    )
}

fn check_schema(
    workspace: &Workspace,
    relative: &str,
    generated: serde_json::Value,
    name: &str,
) -> Result<ContainerCommandOutcome> {
    let path = workspace.path(relative);
    if !path.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "missing {relative}; generate it with the governed container command\n"
        )));
    }
    let checked: serde_json::Value = serde_json::from_str(&read_utf8(&path)?)?;
    if checked != generated {
        return Ok(ContainerCommandOutcome::failure(format!(
            "{relative} drifted from Rust environment types\n"
        )));
    }
    success_line(format!("image-inventory {name} schema: OK"))
}

pub(super) fn generate_report(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run generate-image-inventory-report -- [<publication> <output>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let (publication_path, report_path) = match args {
        [] => (workspace.path(PUBLICATION_PATH), workspace.path(REPORT_PATH)),
        [publication, output] => {
            (path_from_arg(workspace, publication), path_from_arg(workspace, output))
        }
        _ => anyhow::bail!(usage),
    };
    let publication =
        bijux_dna_environment::evidence::load_image_inventory_publication(&publication_path)?;
    let report = render::markdown(&publication);
    bijux_dna_infra::atomic_write_bytes(&report_path, report.as_bytes())?;
    success_line(format!(
        "generated image-inventory report for {} tools at {}",
        publication.tools.len(),
        report_path.display()
    ))
}

pub(super) fn check_report(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let publication = bijux_dna_environment::evidence::load_image_inventory_publication(
        &workspace.path(PUBLICATION_PATH),
    )?;
    let expected = render::markdown(&publication);
    if read_utf8(&workspace.path(REPORT_PATH))? != expected {
        return Ok(ContainerCommandOutcome::failure(format!(
            "{REPORT_PATH} drifted from {PUBLICATION_PATH}; regenerate it with the governed container command\n"
        )));
    }
    success_line("image-inventory report: OK")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_preserves_explicit_evidence_roots() -> Result<()> {
        let workspace = Workspace { root: "/automation".into() };
        let args = [
            "/source",
            "/study",
            "/study/containers/staging/commit",
            "/study/containers/evidence/image-campaigns/commit",
            "/runs",
            "/evidence/scan.json",
            "/evidence/publication.json",
        ]
        .map(str::to_string);

        let (request, scan, publication) = parse_generate_args(&workspace, &args)?;

        assert_eq!(request.repository_root, std::path::Path::new("/source"));
        assert_eq!(request.study_root, std::path::Path::new("/study"));
        assert_eq!(request.image_root, std::path::Path::new("/study/containers/staging/commit"));
        assert_eq!(
            request.build_evidence_root,
            std::path::Path::new("/study/containers/evidence/image-campaigns/commit")
        );
        assert_eq!(request.run_evidence_root, std::path::Path::new("/runs"));
        assert_eq!(scan, std::path::Path::new("/evidence/scan.json"));
        assert_eq!(publication, std::path::Path::new("/evidence/publication.json"));
        Ok(())
    }

    #[test]
    fn generation_rejects_implicit_evidence_roots() {
        let workspace = Workspace { root: "/automation".into() };
        let legacy_args =
            ["/study", "/evidence/scan.json", "/evidence/publication.json"].map(str::to_string);

        let error =
            parse_generate_args(&workspace, &legacy_args).err().map(|error| error.to_string());

        assert_eq!(error.as_deref(), Some(GENERATE_USAGE));
    }
}
