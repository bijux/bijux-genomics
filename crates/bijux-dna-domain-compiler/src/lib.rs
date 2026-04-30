#![allow(clippy::map_unwrap_or, clippy::too_many_lines, clippy::uninlined_format_args)]

mod compiler;

pub use compiler::bundle::{
    build_domain_registry_bundle, domain_artifact_contract_snapshots, domain_defaults_snapshot,
    domain_deprecation_catalogs, domain_metric_catalogs, load_domain_registry_bundle,
    query_domain_registry_bundle, write_domain_registry_bundle, ArtifactContractSnapshot,
    ArtifactRoleSnapshot, CompiledDomainDefaultsSnapshot, CompiledDomainRegistry,
    DefaultSettingsContract, DomainDeprecationCatalog, DomainMetricCatalog, DomainMetricEntry,
    DomainRegistryQuery, DomainRegistryQueryKind, DomainRegistryReleaseBundle,
    DomainRegistrySchemas, DomainStageContract, DomainToolContract, RegistryDeprecationRecord,
    RegistryFixtureBinding, StageMetricContract, StageParameterDefault, ToolContainerContract,
};
pub use compiler::{
    compile_domain_configs, domain_coverage_report, validate_domain, CompileOptions,
    ValidateOptions, DEFAULT_COMPILE_SCOPE, DEFAULT_CONFIGS_DIR, DEFAULT_DOMAIN_DIR,
};
