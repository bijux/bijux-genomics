use super::{
    path_from_arg, read_utf8, success_line, write_utf8, ContainerCommandOutcome, Result, Workspace,
};

const INDEX_SCHEMA_PATH: &str = "containers/contracts/schemas/run-evidence-index.v2.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run run-evidence -- import <result-root> <runs-root> [<minimum-run> <maximum-run>]\n  bijux-dna-dev containers run run-evidence -- validate <runs-root>\n  bijux-dna-dev containers run run-evidence -- publish <runs-root> <attempt-ledger> [--apply]\n  bijux-dna-dev containers run run-evidence -- generate <runs-root> <json-output> <markdown-output>\n  bijux-dna-dev containers run run-evidence -- check <runs-root> <json-output> <markdown-output>\n  bijux-dna-dev containers run run-evidence -- generate-schema [<schema-output>]\n  bijux-dna-dev containers run run-evidence -- check-schema";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "import" => import(workspace, options),
        "validate" => validate(workspace, options),
        "publish" => publish(workspace, options),
        "generate" => generate(workspace, options),
        "check" => check(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn import(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let (result_root, runs_root, minimum_run, maximum_run) = match args {
        [result_root, runs_root] => (result_root, runs_root, None, None),
        [result_root, runs_root, minimum_run, maximum_run] => (
            result_root,
            runs_root,
            Some(minimum_run.parse::<u64>()?),
            Some(maximum_run.parse::<u64>()?),
        ),
        _ => anyhow::bail!(USAGE),
    };
    let count = bijux_dna_containers::api::import_run_evidence(
        &bijux_dna_containers::api::RunEvidenceImportRequest {
            result_root: path_from_arg(workspace, result_root),
            output_root: path_from_arg(workspace, runs_root),
            minimum_run,
            maximum_run,
        },
    )?;
    success_line(format!("imported {count} governed run records"))
}

fn validate(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [runs_root] = args else {
        anyhow::bail!(USAGE);
    };
    let runs_root = path_from_arg(workspace, runs_root);
    let records = bijux_dna_containers::api::validate_run_evidence_root(&runs_root)?;
    bijux_dna_containers::api::validate_run_attempt_ledger(
        &runs_root.join("attempt-ledger.json"),
        &records,
    )?;
    bijux_dna_containers::api::derive_run_evidence_index(&runs_root)?;
    success_line(format!("validated {} governed run records", records.len()))
}

fn publish(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let (runs_root, attempt_ledger, apply) = match args {
        [runs_root, attempt_ledger] => (runs_root, attempt_ledger, false),
        [runs_root, attempt_ledger, apply] if apply == "--apply" => {
            (runs_root, attempt_ledger, true)
        }
        _ => anyhow::bail!(USAGE),
    };
    let runs_root = path_from_arg(workspace, runs_root);
    let attempt_ledger = path_from_arg(workspace, attempt_ledger);
    let plan = bijux_dna_containers::api::plan_final_run_evidence(&runs_root, &attempt_ledger)?;
    if apply {
        bijux_dna_containers::api::apply_final_run_evidence_plan(
            &runs_root,
            &attempt_ledger,
            &plan,
        )?;
    }
    let action = if apply { "published" } else { "would publish" };
    success_line(format!(
        "{action} {} final records; {} attempts superseded",
        plan.selected_run_count,
        plan.ledger.attempts.len()
    ))
}

fn generate(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [runs_root, json_output, markdown_output] = args else {
        anyhow::bail!(USAGE);
    };
    let index =
        bijux_dna_containers::api::derive_run_evidence_index(&path_from_arg(workspace, runs_root))?;
    let json = serde_json::to_string_pretty(&index)? + "\n";
    let markdown = bijux_dna_containers::api::render_run_evidence_markdown(&index);
    let json_output = path_from_arg(workspace, json_output);
    let markdown_output = path_from_arg(workspace, markdown_output);
    write_utf8(&json_output, &json)?;
    write_utf8(&markdown_output, &markdown)?;
    success_line(format!(
        "generated {} selected run records at {} and {}",
        index.runs.len(),
        json_output.display(),
        markdown_output.display()
    ))
}

fn check(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [runs_root, json_output, markdown_output] = args else {
        anyhow::bail!(USAGE);
    };
    bijux_dna_containers::api::verify_run_evidence_publication(
        &path_from_arg(workspace, runs_root),
        &path_from_arg(workspace, json_output),
        &path_from_arg(workspace, markdown_output),
    )?;
    success_line("selected run-evidence publication: OK")
}

fn generate_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let path = match args {
        [] => workspace.path(INDEX_SCHEMA_PATH),
        [path] => path_from_arg(workspace, path),
        _ => anyhow::bail!(USAGE),
    };
    bijux_dna_containers::api::write_run_evidence_index_schema(&path)?;
    success_line(format!("generated selected run-evidence schema at {}", path.display()))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        anyhow::bail!(USAGE);
    }
    let checked: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(INDEX_SCHEMA_PATH))?)?;
    if checked != bijux_dna_containers::api::run_evidence_index_schema() {
        return Ok(ContainerCommandOutcome::failure(
            "selected run-evidence schema drifted from Rust types\n",
        ));
    }
    success_line("selected run-evidence schema: OK")
}
