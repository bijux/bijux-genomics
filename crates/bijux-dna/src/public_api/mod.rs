pub mod cli {
    pub use crate::commands::cli::*;
}

pub mod hpc {
    pub use crate::commands::hpc::*;
}

pub use crate::cli_entrypoint::{run_from_args, run_from_env};
pub use crate::commands::{run_with_args, run_with_cli};
