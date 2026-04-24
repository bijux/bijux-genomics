#![allow(clippy::map_unwrap_or, clippy::too_many_lines, clippy::uninlined_format_args)]

mod compiler;

pub use compiler::{
    compile_domain_configs, domain_coverage_report, validate_domain, CompileOptions,
    ValidateOptions,
};
