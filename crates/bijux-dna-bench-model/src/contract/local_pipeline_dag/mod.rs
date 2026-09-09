//! Input declaration and persisted report contracts for local benchmark pipelines.

mod config;
mod execution;
mod report;

pub use config::{
    validate_benchmark_local_pipeline_dag_config,
    validate_benchmark_local_pipeline_dag_declarations,
    BENCHMARK_LOCAL_PIPELINE_DAG_CONFIG_SCHEMA_V1,
};
pub use report::{
    validate_benchmark_local_pipeline_dag_validation,
    validate_benchmark_local_pipeline_dag_validation_set,
};

pub use execution::validate_benchmark_local_pipeline_execution_context;
