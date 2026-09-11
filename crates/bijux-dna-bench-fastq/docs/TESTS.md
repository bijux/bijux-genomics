# FASTQ probe tests

`tests/contracts.rs` covers foreign stage identity, unknown correction tools, malformed inputs preserving existing evidence and synthetic correction reports with valid gzip output. Private stream tests cover plain and multi-member gzip observations, malformed record framing and header IO errors. Publication tests preserve an existing destination when source opening fails.

Run owner cases with `cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-fastq --all-targets`, then repeat with `--all-features`. The CLI's original real-output parser smoke cases exercise mixed-domain composition separately. These fixtures establish parser and publication contracts; they do not certify scientific tool execution.

Readiness regressions exercise closed parser-status admission, duplicate and contradictory source cardinalities, missing active-binding coverage, the governed typed-fixture consumer and schema/stage/tool/truncated-payload rejection. Original CLI FASTQ readiness and cross-domain consumers preserve their public contract assertions.
