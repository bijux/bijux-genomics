pub(super) struct RunOptions {
    pub(super) platform: Option<String>,
    pub(super) tools_filter: Option<String>,
    pub(super) debug: bool,
    pub(super) quiet: bool,
}

pub(super) fn parse_run_options(args: &[String]) -> RunOptions {
    RunOptions {
        platform: parse_arg_value(args, "--platform"),
        tools_filter: parse_arg_value(args, "--tools"),
        debug: has_flag(args, "--debug") || env_flag("DEBUG"),
        quiet: has_flag(args, "--quiet") || env_flag("QUIET"),
    }
}

fn parse_arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .filter(|value| !value.starts_with("--"))
        .cloned()
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| {
        let value = value.trim();
        value == "1" || value.eq_ignore_ascii_case("true")
    })
}
