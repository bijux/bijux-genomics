use super::{path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result, Workspace};

const PLAN_SCHEMA_PATH: &str = "containers/contracts/schemas/image-activation-plan.v1.json";

const USAGE: &str = "Usage:\n  bijux-dna-dev containers run image-activation -- plan <study-root> <inventory> <source-root> <active-root> <plan> <created-at-utc>\n  bijux-dna-dev containers run image-activation -- apply <plan> <source-root> <active-root> <observed-at-utc>\n  bijux-dna-dev containers run image-activation -- verify <plan> <source-root> <active-root>\n  bijux-dna-dev containers run image-activation -- rollback <plan> <source-root> <active-root> <observed-at-utc>\n  bijux-dna-dev containers run image-activation -- generate-schema [<plan-schema>]\n  bijux-dna-dev containers run image-activation -- check-schema";

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
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn plan(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [study_root, inventory, source_root, active_root, output, created_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let inventory = path_from_arg(workspace, inventory);
    let study_root = path_from_arg(workspace, study_root);
    let source_root = path_from_arg(workspace, source_root);
    let active_root = path_from_arg(workspace, active_root);
    let request = bijux_dna_containers::api::ImageActivationPlanRequest {
        inventory_path: &inventory,
        study_root: &study_root,
        source_root: &source_root,
        active_root: &active_root,
        created_at_utc,
    };
    let output = path_from_arg(workspace, output);
    let plan = bijux_dna_containers::api::create_image_activation_plan(&request, &output)?;
    success_line(format!(
        "planned {} canonical image activations at {}",
        plan.file_count,
        output.display()
    ))
}

fn apply(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [plan, source_root, active_root, observed_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let applied = bijux_dna_containers::api::apply_image_activation_plan(
        &path_from_arg(workspace, plan),
        &path_from_arg(workspace, source_root),
        &path_from_arg(workspace, active_root),
        observed_at_utc,
    )?;
    success_line(format!("activated {} canonical images", applied.file_count))
}

fn verify(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [plan, source_root, active_root] = args else {
        anyhow::bail!(USAGE);
    };
    let verified = bijux_dna_containers::api::verify_image_activation_plan(
        &path_from_arg(workspace, plan),
        &path_from_arg(workspace, source_root),
        &path_from_arg(workspace, active_root),
    )?;
    success_line(format!("verified {} canonical image links", verified.file_count))
}

fn rollback(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [plan, source_root, active_root, observed_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let rolled_back = bijux_dna_containers::api::rollback_image_activation_plan(
        &path_from_arg(workspace, plan),
        &path_from_arg(workspace, source_root),
        &path_from_arg(workspace, active_root),
        observed_at_utc,
    )?;
    success_line(format!("removed {} canonical image links", rolled_back.file_count))
}

fn generate_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let output = match args {
        [] => workspace.path(PLAN_SCHEMA_PATH),
        [output] => path_from_arg(workspace, output),
        _ => anyhow::bail!(USAGE),
    };
    bijux_dna_containers::api::write_image_activation_plan_schema(&output)?;
    success_line(format!("generated image-activation schema at {}", output.display()))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        anyhow::bail!(USAGE);
    }
    let checked: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(PLAN_SCHEMA_PATH))?)?;
    if checked != bijux_dna_containers::api::image_activation_plan_schema() {
        return Ok(ContainerCommandOutcome::failure(
            "image-activation schema drifted from Rust environment types\n",
        ));
    }
    success_line("image-activation schema: OK")
}
