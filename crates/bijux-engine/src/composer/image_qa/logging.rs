use bijux_environment::api::RunnerKind;

use crate::types::StdoutLogger;

use super::{QaDataset, QaStage};
use bijux_bench::ImageQaOutcome;

pub(crate) fn log_header(
    logger: &StdoutLogger,
    platform: &str,
    runner: RunnerKind,
    datasets: &[QaDataset],
) {
    logger.info(&format!("[bijux] Image QA started ({runner})"));
    logger.info(&format!(
        "[bijux] Platform: {platform} | Datasets: {}",
        datasets.len()
    ));
    if !datasets.is_empty() {
        logger.info("[bijux] QA datasets:");
        for dataset in datasets {
            logger.info(&format!("  - {}", dataset.name));
        }
    }
}

pub(crate) fn log_stage_header(logger: &StdoutLogger, stage: QaStage) {
    logger.info(&format!("[bijux][image-qa] Stage: {}", stage.stage_id()));
}

pub(crate) fn log_dataset(logger: &StdoutLogger, dataset: &QaDataset) {
    logger.debug(&format!("[bijux][image-qa][dataset] {}", dataset.name));
}

pub(crate) fn log_tool(logger: &StdoutLogger, stage: QaStage, tool: &str) {
    logger.debug(&format!(
        "[bijux][image-qa][run] {}::{tool}",
        stage.stage_id()
    ));
}

pub(crate) fn log_tool_result(
    logger: &StdoutLogger,
    stage: QaStage,
    tool: &str,
    dataset: &QaDataset,
    outcome: &ImageQaOutcome,
) {
    let status = match outcome {
        ImageQaOutcome::Pass => "pass",
        ImageQaOutcome::Fail(_) => "fail",
    };
    let mut line = format!(
        "[bijux][image-qa][{status}] {}::{tool} ({})",
        stage.stage_id(),
        dataset.name
    );
    if let ImageQaOutcome::Fail(reason) = outcome {
        let _ = std::fmt::Write::write_fmt(&mut line, format_args!(" - {reason}"));
    }
    logger.info(&line);
}

pub(crate) fn log_tool_skip(
    logger: &StdoutLogger,
    stage: QaStage,
    tool: &str,
    dataset: &QaDataset,
) {
    logger.info(&format!(
        "[bijux][image-qa][skip] {}::{tool} ({})",
        stage.stage_id(),
        dataset.name
    ));
}
