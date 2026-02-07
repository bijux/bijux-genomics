use std::collections::HashMap;

use crate::api::{load_image_catalog, load_platform};
use crate::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use anyhow::{anyhow, Context, Result};
use bijux_analyze::load::sqlite::bench::{ensure_image_qa_tables, insert_image_qa_v1};
use bijux_analyze::load::sqlite::reports::insert_image_qa_input_v1;
use bijux_analyze::{append_image_qa_jsonl, open_sqlite, ImageQaOutcome, ImageQaRecord};
use bijux_infra::atomic_write_bytes;

use super::support::{
    image_qa_jsonl_path, image_qa_sqlite_path, resolve_image_for_run, trace_enabled, StdoutLogger,
};

use super::behavioral::run_behavioral_qa;
use super::datasets::{
    dataset_input_hash, datasets_for_stage, discover_qa_datasets, hydrate_datasets,
};
use super::logging::{
    log_dataset, log_header, log_stage_header, log_tool, log_tool_result, log_tool_skip,
};
use super::records::{build_qa_record, qa_already_passed};
use super::static_qa::run_static_qa;
use super::QaStage;
use bijux_runtime::manifests::load_manifests;

/// Run image QA for the FASTQ domain.
///
/// # Errors
/// Returns an error if QA datasets or tool runs fail.
pub fn run_image_qa(platform_name: Option<&str>) -> Result<()> {
    if trace_enabled() {
        println!("[engine][composer] image_qa start");
    }
    let platform = load_platform(platform_name)?;
    let catalog = load_image_catalog()?;
    let logger = StdoutLogger::new();
    run_image_qa_with(&platform, &catalog, &logger)
}

#[allow(clippy::too_many_lines)]
fn run_image_qa_with(
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    logger: &StdoutLogger,
) -> Result<()> {
    if platform.runner != RuntimeKind::Docker {
        return Err(anyhow!("image QA supports docker only for now"));
    }

    let cwd = std::env::current_dir().context("resolve cwd")?;
    let qa_jsonl = image_qa_jsonl_path(&cwd, &platform.name);
    let qa_sqlite = image_qa_sqlite_path(&cwd, &platform.name);
    let qa_dir = qa_sqlite
        .parent()
        .ok_or_else(|| anyhow!("missing image QA directory"))?;
    bijux_infra::ensure_dir(qa_dir).context("create image qa dir")?;

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
        .ok_or_else(|| anyhow!("seqkit missing from images.toml"))?;
    let seqkit_image = resolve_image_for_run(seqkit_spec, platform)?;

    let registry = load_manifests(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow!("manifest validation failed: {err}"))?;
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
        QaStage::QcPost,
        QaStage::Umi,
        QaStage::Stats,
    ];
    if std::env::var("BIJUX_SCREEN_DB").is_ok() {
        stages.push(QaStage::Screen);
    }

    for stage in stages {
        let stage_id = stage.stage_id();
        log_stage_header(logger, stage);
        let stage_datasets = datasets_for_stage(stage, &datasets);
        for dataset in stage_datasets {
            log_dataset(logger, &dataset);
            let input_hash = dataset_input_hash(stage, &dataset);
            insert_image_qa_input_v1(
                &conn,
                stage_id.as_str(),
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
                let mut outcome = match run_static_qa(tool, platform, catalog) {
                    Ok(()) => run_behavioral_qa(
                        stage,
                        tool,
                        platform,
                        catalog,
                        &registry,
                        &dataset,
                        &seqkit_image,
                    ),
                    Err(err) => ImageQaOutcome::Fail(err.to_string()),
                };
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
                            stage: stage_id.as_str().to_string(),
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
    atomic_write_bytes(&qa_json, &serde_json::to_vec_pretty(&summary)?)
        .map_err(anyhow::Error::from)
        .context("write qa.json")?;

    println!("QA PASS: {pass}");
    println!("QA FAIL: {fail}");
    if fail > 0 {
        return Err(anyhow!("image QA failed for {fail} tools"));
    }
    Ok(())
}
