use bijux_dna_environment::resolve::RuntimeKind;

use super::models::{ImagePlan, Summary};
use super::runtime::{LogLevel, Logger};

pub(super) fn log_header(
    logger: &mut dyn Logger,
    platform: Option<&str>,
    runner: RuntimeKind,
    total: usize,
) {
    let platform = platform.unwrap_or("unknown");
    logger.log(LogLevel::Info, &format!("Platform {platform} ({runner}) - {total} images"));
}

pub(super) fn log_discovered_images(logger: &mut dyn Logger, plans: &[ImagePlan]) {
    if logger.is_quiet() {
        return;
    }
    for plan in plans {
        logger.log(LogLevel::Info, &format!("image: {}", plan.image_name));
    }
}

pub(super) fn log_summary(logger: &mut dyn Logger, summary: &Summary) {
    logger.log(LogLevel::Info, &format!("Summary: {} pass / {} fail", summary.pass, summary.fail));
}
