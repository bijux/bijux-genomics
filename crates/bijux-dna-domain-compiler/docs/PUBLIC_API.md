# bijux-dna-domain-compiler Public API

The public API is intentionally small. Consumers should call these library functions instead of
reaching into internal compiler modules.

## Functions

- `compile_domain_configs(options: &CompileOptions) -> anyhow::Result<()>`
- `validate_domain(options: &ValidateOptions) -> anyhow::Result<()>`
- `domain_coverage_report(domain_dir: &Path) -> anyhow::Result<String>`

## Option Structs

- `CompileOptions`
  - `domain_dir: PathBuf`
  - `configs_dir: PathBuf`
  - `scope: String`
- `ValidateOptions`
  - `domain_dir: PathBuf`

## Defaults

- `DEFAULT_DOMAIN_DIR`
- `DEFAULT_CONFIGS_DIR`
- `DEFAULT_COMPILE_SCOPE`

## Stability Rules

- Public additions must be documented here and covered by contract or boundary tests.
- Internal model types in `src/compiler/` must remain private unless a consumer need is proven.
- Command behavior must stay aligned with [COMMANDS.md](COMMANDS.md).
