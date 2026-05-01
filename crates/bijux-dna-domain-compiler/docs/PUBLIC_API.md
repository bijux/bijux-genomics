# bijux-dna-domain-compiler Public API

The public API is intentionally small. Consumers should call these library functions instead of
reaching into internal compiler modules.

## Functions

- `compile_domain_configs(options: &CompileOptions) -> anyhow::Result<()>`
- `validate_domain(options: &ValidateOptions) -> anyhow::Result<()>`
- `domain_coverage_report(domain_dir: &Path) -> anyhow::Result<String>`
- `build_domain_registry_bundle(domain_dir: &Path, source_ref: impl Into<String>) -> anyhow::Result<DomainRegistryReleaseBundle>`
- `load_domain_registry_bundle(path: &Path) -> anyhow::Result<DomainRegistryReleaseBundle>`
- `write_domain_registry_bundle(configs_dir: &Path, bundle: &DomainRegistryReleaseBundle) -> anyhow::Result<Vec<PathBuf>>`
- `query_domain_registry_bundle(bundle: &DomainRegistryReleaseBundle, query: &DomainRegistryQuery) -> serde_json::Value`
- `domain_defaults_snapshot(bundle: &DomainRegistryReleaseBundle) -> Vec<CompiledDomainDefaultsSnapshot>`
- `domain_artifact_contract_snapshots(bundle: &DomainRegistryReleaseBundle) -> Vec<ArtifactContractSnapshot>`
- `domain_metric_catalogs(bundle: &DomainRegistryReleaseBundle) -> Vec<DomainMetricCatalog>`
- `domain_deprecation_catalogs(bundle: &DomainRegistryReleaseBundle) -> Vec<DomainDeprecationCatalog>`
- `domain_invariant_catalogs(bundle: &DomainRegistryReleaseBundle) -> Vec<DomainInvariantCatalog>`
- `domain_evidence_catalogs(bundle: &DomainRegistryReleaseBundle) -> Vec<DomainEvidenceCatalog>`

## Option Structs

- `CompileOptions`
  - `domain_dir: PathBuf`
  - `configs_dir: PathBuf`
  - `scope: String`
- `ValidateOptions`
  - `domain_dir: PathBuf`
- `DomainRegistryReleaseBundle`
- `DomainRegistryQuery`
- `DomainRegistryQueryKind`
- `CompiledDomainRegistry`

## Defaults

- `DEFAULT_DOMAIN_DIR`
- `DEFAULT_CONFIGS_DIR`
- `DEFAULT_COMPILE_SCOPE`

## Stability Rules

- Public additions must be documented here and covered by contract or boundary tests.
- Internal model types in `src/compiler/` must remain private unless a consumer need is proven.
- Command behavior must stay aligned with [COMMANDS.md](COMMANDS.md).
- Generated bundle file names must stay aligned with [CONTRACTS.md](CONTRACTS.md).
