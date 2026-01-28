use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    append_image_qa_jsonl, ensure_image_qa_tables, insert_image_qa_input_v1, insert_image_qa_v1,
    open_sqlite, ImageQaOutcome, ImageQaRecord,
};
use bijux_environment::api::{load_image_catalog, load_platform};
use bijux_environment::api::{PlatformSpec, RunnerKind, ToolImageSpec};

use bijux_engine::api::{image_qa_jsonl_path, image_qa_sqlite_path};

use super::datasets::{
    dataset_input_hash, datasets_for_stage, discover_qa_datasets, hydrate_datasets,
};
use super::helpers::{build_qa_record, qa_already_passed};
use super::logging::{
    log_dataset, log_header, log_stage_header, log_tool, log_tool_result, log_tool_skip,
};
use super::stages::run_stage_qa;
use super::QaStage;
use bijux_engine::api::load_registry;

/// Run image QA for the FASTQ domain.
///
/// # Errors
/// Returns an error if QA datasets or tool runs fail.
pub fn run_image_qa(platform_name: Option<&str>) -> Result<()> {
    if bijux_engine::api::trace_enabled() {
        println!("[engine][composer] image_qa start");
    }
    let platform = load_platform(platform_name)?;
    let catalog = load_image_catalog()?;
    let logger = bijux_engine::api::StdoutLogger::new();
    run_image_qa_with(&platform, &catalog, &logger)
}

#[allow(clippy::too_many_lines)]
fn run_image_qa_with(
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    logger: &bijux_engine::api::StdoutLogger,
) -> Result<()> {
    if platform.runner != RunnerKind::Docker {
        return Err(anyhow!("image QA supports docker only for now"));
    }

    let cwd = std::env::current_dir().context("resolve cwd")?;
    let qa_jsonl = image_qa_jsonl_path(&cwd, &platform.name);
    let qa_sqlite = image_qa_sqlite_path(&cwd, &platform.name);
    let qa_dir = qa_sqlite
        .parent()
        .ok_or_else(|| anyhow!("missing image QA directory"))?;
    std::fs::create_dir_all(qa_dir).context("create image qa dir")?;

    let conn = open_sqlite(&qa_sqlite).context("open image qa sqlite")?;
    ensure_image_qa_tables(&conn).context("ensure image qa tables")?;
    conn.execute(
        "DELETE FROM image_qa_inputs_v1 WHERE platform = ?1 AND runner = ?2",
        (&platform.name, &platform.runner.to_string()),
    )
    .context("reset image qa inputs")?;
    conn.execute(
        "DELETE FROM image_qa_v1 WHERE platform = ?1 AND runner = ?2",
        (&platform.name, &platform.runner.to_string()),
    )
    .context("reset image qa records")?;

    let seqkit_spec = catalog
        .get("seqkit")
        .ok_or_else(|| anyhow!("seqkit missing from images.yaml"))?;
    let seqkit_image = bijux_engine::api::resolve_image_for_run(seqkit_spec, platform)?;

    let registry =
        load_registry(&std::env::current_dir()?.join("domain")).context("load manifests")?;
    let mut datasets = discover_qa_datasets()?;
    hydrate_datasets(&mut datasets, &seqkit_image)?;

    log_header(logger, &platform.name, platform.runner, &datasets);

    let mut pass = 0;
    let mut fail = 0;
    let mut summary_records: Vec<ImageQaRecord> = Vec::new();

    let mut stages = vec![
        QaStage::Trim,
        QaStage::Validate,
        QaStage::Filter,
        QaStage::Merge,
        QaStage::Correct,
        QaStage::Qc2,
        QaStage::Umi,
        QaStage::Stats,
    ];
    if std::env::var("BIJUX_SCREEN_DB").is_ok() {
        stages.push(QaStage::Screen);
    }

    for stage in stages {
        log_stage_header(logger, stage);
        let stage_datasets = datasets_for_stage(stage, &datasets);
        for dataset in stage_datasets {
            log_dataset(logger, &dataset);
            let input_hash = dataset_input_hash(stage, &dataset);
            insert_image_qa_input_v1(
                &conn,
                stage.stage_id(),
                &platform.name,
                &platform.runner.to_string(),
                &input_hash,
            )
            .context("write qa inputs sqlite")?;
            for &tool in stage.tools() {
                if stage != QaStage::Trim
                    && qa_already_passed(&conn, stage, tool, platform, catalog, &input_hash)
                        .unwrap_or(false)
                {
                    log_tool_skip(logger, stage, tool, &dataset);
                    continue;
                }
                log_tool(logger, stage, tool);
                let mut outcome = run_stage_qa(
                    stage,
                    tool,
                    platform,
                    catalog,
                    &registry,
                    &dataset,
                    &seqkit_image,
                );
                let record = match build_qa_record(
                    stage,
                    tool,
                    platform,
                    catalog,
                    &input_hash,
                    outcome.clone(),
                ) {
                    Ok(record) => record,
                    Err(err) => {
                        outcome = ImageQaOutcome::Fail(err.to_string());
                        ImageQaRecord {
                            tool: tool.to_string(),
                            stage: stage.stage_id().to_string(),
                            tool_version: "unknown".to_string(),
                            image_digest: "unknown".to_string(),
                            runner: platform.runner.to_string(),
                            platform: platform.name.clone(),
                            input_hash: input_hash.clone(),
                            outcome: outcome.clone(),
                        }
                    }
                };
                log_tool_result(logger, stage, tool, &dataset, &outcome);
                if matches!(outcome, ImageQaOutcome::Pass) {
                    pass += 1;
                } else {
                    fail += 1;
                }
                append_image_qa_jsonl(&qa_jsonl, &record).context("write qa jsonl")?;
                insert_image_qa_v1(&conn, &record).context("write qa sqlite")?;
                summary_records.push(record);
            }
        }
    }

    let qa_json = qa_dir.join("qa.json");
    let summary = serde_json::json!({
        "pass": pass,
        "fail": fail,
        "records": summary_records,
    });
    std::fs::write(&qa_json, serde_json::to_vec_pretty(&summary)?).context("write qa.json")?;

    println!("QA PASS: {pass}");
    println!("QA FAIL: {fail}");
    if fail > 0 {
        return Err(anyhow!("image QA failed for {fail} tools"));
    }
    Ok(())
}
