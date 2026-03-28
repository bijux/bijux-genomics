# configs/lab

Purpose: local lab execution defaults.

Files:
- `configs/lab/config_example.toml`

Contract:
- `configs/lab/config.toml` is the runtime config consumed by `cargo run -q -p bijux-dna-dev -- lab run ...`
- `CONFIG_PATH` selects a different lab config file when needed
- string values may use environment placeholders such as `${BIJUX_LAB_CORPUS_ROOT}` and `${BIJUX_LAB_OUTPUT_DIR}`
- `pipeline_ids` may be declared as a TOML array instead of a comma-delimited string
