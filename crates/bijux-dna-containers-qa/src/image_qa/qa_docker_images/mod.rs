mod args;
#[cfg(test)]
mod contracts;
mod models;
mod planning;
mod probe;
mod reporting;
mod runtime;

use args::parse_run_options;
use models::{ImagePlan, ImageTestOutcome, Summary};
use planning::{build_image_plans, filter_tools, load_platform_spec};
use probe::run_image_test;
use reporting::{log_discovered_images, log_header, log_summary};
use runtime::{CommandRunner, LogLevel, Logger, RealRunner, StdoutLogger};

/// # Errors
/// Returns an error if loading specs, building plans, or executing image checks fails.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let options = parse_run_options(&args);
    let platform_spec = load_platform_spec(options.platform.as_deref())?;
    let tools = filter_tools(options.tools_filter)?;
    let image_plans = build_image_plans(&platform_spec, &tools)?;

    let mut logger = StdoutLogger::new(
        if options.debug { LogLevel::Debug } else { LogLevel::Info },
        options.quiet,
    );
    log_header(
        &mut logger,
        Some(platform_spec.name.as_str()),
        platform_spec.runner,
        image_plans.len(),
    );
    log_discovered_images(&mut logger, &image_plans);

    let runner = RealRunner;
    let summary = run_image_tests(&runner, &mut logger, &image_plans)?;
    log_summary(&mut logger, &summary);

    if summary.fail > 0 {
        return Err(format!("image tests failed: {}", summary.fail).into());
    }

    Ok(())
}

fn run_image_tests(
    runner: &dyn CommandRunner,
    logger: &mut dyn Logger,
    plans: &[ImagePlan],
) -> Result<Summary, Box<dyn std::error::Error>> {
    let mut summary = Summary::default();
    for plan in plans {
        let outcome = run_image_test(runner, logger, plan)?;
        match outcome {
            ImageTestOutcome::Pass(kind) => {
                summary.pass += 1;
                if !logger.is_quiet() {
                    logger.log(LogLevel::Info, &format!("PASS [{}] {}", kind, plan.image_name));
                }
            }
            ImageTestOutcome::Fail(reason) => {
                summary.fail += 1;
                logger.log(LogLevel::Info, &format!("FAIL [{}] {}", reason, plan.image_name));
            }
        }
    }
    Ok(summary)
}
