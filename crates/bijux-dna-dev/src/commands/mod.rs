mod automation_boundary;
mod checks;
mod command_support;
mod containers;
mod domain;
mod native_dispatch;
mod ops;
mod repo_checks;

pub use native_dispatch::{
    run_native_check, run_native_container_command, run_native_domain_command,
    run_native_ops_command,
};
