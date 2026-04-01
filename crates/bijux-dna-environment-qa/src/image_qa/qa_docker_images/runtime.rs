use std::process::{Command, Output};

#[derive(Copy, Clone, Eq, PartialEq)]
pub(super) enum LogLevel {
    Info,
    Debug,
}

pub(super) trait Logger {
    fn log(&mut self, level: LogLevel, line: &str);
    fn is_quiet(&self) -> bool;
}

pub(super) struct StdoutLogger {
    level: LogLevel,
    quiet: bool,
}

impl StdoutLogger {
    pub(super) fn new(level: LogLevel, quiet: bool) -> Self {
        Self { level, quiet }
    }
}

impl Logger for StdoutLogger {
    fn log(&mut self, level: LogLevel, line: &str) {
        if level == LogLevel::Info || self.level == LogLevel::Debug {
            println!("{line}");
        }
    }

    fn is_quiet(&self) -> bool {
        self.quiet
    }
}

pub(super) trait CommandRunner {
    fn run(&self, args: &[&str]) -> Result<Output, std::io::Error>;
}

pub(super) struct RealRunner;

impl CommandRunner for RealRunner {
    fn run(&self, args: &[&str]) -> Result<Output, std::io::Error> {
        let mut cmd = Command::new(args[0]);
        if args.len() > 1 {
            cmd.args(&args[1..]);
        }
        cmd.output()
    }
}

pub(super) fn log_debug(logger: &mut dyn Logger, line: &str) {
    logger.log(LogLevel::Debug, line);
}
