use super::{path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result, Workspace};

const PLAN_SCHEMA_PATH: &str = "containers/contracts/schemas/image-storage-plan.v1.json";
const REPORT_SCHEMA_PATH: &str = "containers/contracts/schemas/image-storage-report.v1.json";

const USAGE: &str = "Usage:\n  bijux-dna-dev containers run image-storage -- plan <study-root> <inventory> <image-root> <quarantine-root> <plan> <created-at-utc>\n  bijux-dna-dev containers run image-storage -- apply <plan> <image-root> <observed-at-utc>\n  bijux-dna-dev containers run image-storage -- verify <plan> <image-root>\n  bijux-dna-dev containers run image-storage -- rollback <plan> <image-root> <observed-at-utc>\n  bijux-dna-dev containers run image-storage -- report <plan> <after-inventory> <json-output> <markdown-output>\n  bijux-dna-dev containers run image-storage -- check-report <plan> <after-inventory> <json-output> <markdown-output>\n  bijux-dna-dev containers run image-storage -- generate-schemas [<plan-schema> <report-schema>]\n  bijux-dna-dev containers run image-storage -- check-schemas";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "plan" => plan(workspace, options),
        "apply" => apply(workspace, options),
        "verify" => verify(workspace, options),
        "rollback" => rollback(workspace, options),
        "report" => report(workspace, options, false),
        "check-report" => report(workspace, options, true),
        "generate-schemas" => generate_schemas(workspace, options),
        "check-schemas" => check_schemas(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn plan(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [study_root, inventory, image_root, quarantine_root, output, created_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let inventory = path_from_arg(workspace, inventory);
    let image_root = path_from_arg(workspace, image_root);
    let quarantine_root = path_from_arg(workspace, quarantine_root);
    let study_root = path_from_arg(workspace, study_root);
    let request = bijux_dna_containers::api::ImageStoragePlanRequest {
        inventory_path: &inventory,
        image_root: &image_root,
        quarantine_root: &quarantine_root,
        study_root: &study_root,
        created_at_utc,
    };
    let output = path_from_arg(workspace, output);
    let plan = bijux_dna_containers::api::create_image_storage_plan(&request, &output)?;
    success_line(format!(
        "planned {} reversible image moves at {}",
        plan.file_count,
        output.display()
    ))
}

fn apply(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [plan, image_root, observed_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let plan = path_from_arg(workspace, plan);
    let applied = bijux_dna_containers::api::apply_image_storage_plan(
        &plan,
        &path_from_arg(workspace, image_root),
        observed_at_utc,
    )?;
    success_line(format!("applied {} reversible image moves", applied.file_count))
}

fn verify(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [plan, image_root] = args else {
        anyhow::bail!(USAGE);
    };
    let verified = bijux_dna_containers::api::verify_image_storage_plan(
        &path_from_arg(workspace, plan),
        &path_from_arg(workspace, image_root),
    )?;
    success_line(format!("verified {} quarantined image files", verified.file_count))
}

fn rollback(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [plan, image_root, observed_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let restored = bijux_dna_containers::api::rollback_image_storage_plan(
        &path_from_arg(workspace, plan),
        &path_from_arg(workspace, image_root),
        observed_at_utc,
    )?;
    success_line(format!("restored {} quarantined image files", restored.file_count))
}

fn report(workspace: &Workspace, args: &[String], check: bool) -> Result<ContainerCommandOutcome> {
    let [plan, after_inventory, json_output, markdown_output] = args else {
        anyhow::bail!(USAGE);
    };
    let plan = path_from_arg(workspace, plan);
    let after_inventory = path_from_arg(workspace, after_inventory);
    let json_output = path_from_arg(workspace, json_output);
    let markdown_output = path_from_arg(workspace, markdown_output);
    if check {
        let publication =
            bijux_dna_containers::api::derive_image_storage_report(&plan, &after_inventory)?;
        let markdown =
            bijux_dna_containers::api::render_image_storage_report_markdown(&publication);
        let expected_json = serde_json::to_string_pretty(&publication)?;
        if read_utf8(&json_output)? != expected_json || read_utf8(&markdown_output)? != markdown {
            return Ok(ContainerCommandOutcome::failure(
                "image-storage report drifted from governed evidence\n",
            ));
        }
        return success_line("image-storage report: OK");
    }
    bijux_dna_containers::api::publish_image_storage_report(
        &plan,
        &after_inventory,
        &json_output,
        &markdown_output,
    )?;
    success_line(format!(
        "published image-storage report at {} and {}",
        json_output.display(),
        markdown_output.display()
    ))
}

fn generate_schemas(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let (plan_path, report_path) = schema_paths(workspace, args)?;
    bijux_dna_containers::api::write_image_storage_plan_schema(&plan_path)?;
    bijux_dna_containers::api::write_image_storage_report_schema(&report_path)?;
    success_line(format!(
        "generated image-storage schemas at {} and {}",
        plan_path.display(),
        report_path.display()
    ))
}

fn check_schemas(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        anyhow::bail!(USAGE);
    }
    let plan_path = workspace.path(PLAN_SCHEMA_PATH);
    let report_path = workspace.path(REPORT_SCHEMA_PATH);
    let checked_plan: serde_json::Value = serde_json::from_str(&read_utf8(&plan_path)?)?;
    let checked_report: serde_json::Value = serde_json::from_str(&read_utf8(&report_path)?)?;
    if checked_plan != bijux_dna_containers::api::image_storage_plan_schema()
        || checked_report != bijux_dna_containers::api::image_storage_report_schema()
    {
        return Ok(ContainerCommandOutcome::failure(
            "image-storage schemas drifted from Rust environment types\n",
        ));
    }
    success_line("image-storage schemas: OK")
}

fn schema_paths(
    workspace: &Workspace,
    args: &[String],
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    match args {
        [] => Ok((workspace.path(PLAN_SCHEMA_PATH), workspace.path(REPORT_SCHEMA_PATH))),
        [plan, report] => Ok((path_from_arg(workspace, plan), path_from_arg(workspace, report))),
        _ => anyhow::bail!(USAGE),
    }
}
