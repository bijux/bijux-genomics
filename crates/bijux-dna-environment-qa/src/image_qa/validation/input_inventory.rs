use anyhow::{anyhow, Context, Result};

pub(crate) fn expected_input_hashes(
    stage: &str,
    platform: &crate::api::PlatformSpec,
) -> Result<Vec<String>> {
    let cwd = std::env::current_dir().map_err(|err| anyhow!("failed to resolve cwd: {err}"))?;
    let qa_sqlite = crate::image_qa::support::image_qa_sqlite_path(&cwd, &platform.name);
    if !qa_sqlite.exists() {
        return Err(anyhow!(
            "image QA results missing; run `bijux image-qa --platform {}`",
            platform.name
        ));
    }
    let conn = bijux_dna_analyze::open_sqlite(&qa_sqlite).context("open image qa sqlite")?;
    let runner = platform.runner.to_string();
    let mut expected_inputs = bijux_dna_analyze::load::sqlite::reports::image_qa_inputs(
        &conn,
        stage,
        &platform.name,
        &runner,
    )?;
    if expected_inputs.is_empty() {
        expected_inputs =
            bijux_dna_analyze::load::sqlite::reports::image_qa_input_hashes_from_records(
                &conn,
                stage,
                &platform.name,
                &runner,
            )?;
    }
    if expected_inputs.is_empty() {
        return Err(anyhow!(
            "image QA inputs missing for {stage}; run `bijux image-qa --platform {}`",
            platform.name
        ));
    }
    Ok(expected_inputs)
}
