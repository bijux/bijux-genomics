//! Local stage schemas, admission and optional repository and execution services.
mod error;
mod model;
mod validation;
mod versions;
pub use error::StageContractError;
pub use model::BenchmarkLocalStageReadinessKind;
pub use model::{
    benchmark_local_all_domain_stage_inventory_schema, benchmark_local_stage_inventory_schema,
    benchmark_local_stage_matrix_config_schema, BenchmarkLocalAllDomainStageInventory,
    BenchmarkLocalStageInventory, BenchmarkLocalStageInventoryEntry,
    BenchmarkLocalStageMatrixConfig, BENCHMARK_LOCAL_BAM_STAGE_MATRIX_SCHEMA_V1,
    BENCHMARK_LOCAL_FASTQ_STAGE_MATRIX_SCHEMA_V1,
};
pub use model::{
    benchmark_local_stage_commands_schema, BenchmarkLocalStageArtifactEntry,
    BenchmarkLocalStageCommandArgvEntry, BenchmarkLocalStageCommandEntry,
    BenchmarkLocalStageCommandManifest,
};
pub use validation::validate_benchmark_local_stage_commands;
pub use validation::{
    validate_benchmark_local_all_domain_stage_inventory, validate_benchmark_local_stage_inventory,
};
pub use versions::{
    BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1, BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
    BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
};
#[cfg(feature = "execution")]
pub mod execution;
#[cfg(any(feature = "repository", feature = "execution"))]
mod paths;
#[cfg(feature = "repository")]
pub mod repository;
