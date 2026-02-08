use crate::api::RuntimeKind;

use super::support::StdoutLogger;

use super::{QaDataset, QaStage};
use bijux_dna_analyze::ImageQaOutcome;

pub(crate) fn log_header(
    logger: &StdoutLogger,
    platform: &str,
    runner: RuntimeKind,
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
    let stage_id = stage.stage_id();
    logger.info(&format!("[bijux][image-qa] Stage: {}", stage_id.as_str()));
}

pub(crate) fn log_dataset(logger: &StdoutLogger, dataset: &QaDataset) {
    logger.debug(&format!("[bijux][image-qa][dataset] {}", dataset.name));
}

pub(crate) fn log_tool(logger: &StdoutLogger, stage: QaStage, tool: &str) {
    let stage_id = stage.stage_id();
    logger.debug(&format!(
        "[bijux][image-qa][run] {}::{tool}",
        stage_id.as_str()
    ));
}

pub(crate) fn log_tool_result(
    logger: &StdoutLogger,
    stage: QaStage,
    tool: &str,
    dataset: &QaDataset,
    outcome: &ImageQaOutcome,
) {
    let stage_id = stage.stage_id();
    let status = match outcome {
        ImageQaOutcome::Pass => "pass",
        ImageQaOutcome::Fail(_) => "fail",
    };
    let mut line = format!(
        "[bijux][image-qa][{status}] {}::{tool} ({})",
        stage_id.as_str(),
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
    let stage_id = stage.stage_id();
    logger.info(&format!(
        "[bijux][image-qa][skip] {}::{tool} ({})",
        stage_id.as_str(),
        dataset.name
    ));
}
