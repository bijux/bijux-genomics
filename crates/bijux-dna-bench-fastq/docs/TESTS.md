# FASTQ probe tests

`tests/contracts.rs` covers foreign stage identity, unknown correction tools, malformed inputs preserving existing evidence and synthetic correction reports with valid gzip output. Private stream tests cover plain and multi-member gzip observations, malformed record framing and header IO errors. Publication tests preserve an existing destination when source opening fails.

Run owner cases with `cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-fastq --all-targets`, then repeat with `--all-features`. The CLI's original real-output parser smoke cases exercise mixed-domain composition separately. These fixtures establish parser and publication contracts; they do not certify scientific tool execution.

Readiness regressions exercise closed parser-status admission, duplicate and contradictory source cardinalities, missing active-binding coverage, the governed typed-fixture consumer and schema/stage/tool/truncated-payload rejection. Original CLI FASTQ readiness and cross-domain consumers preserve their public contract assertions.

Declaration contracts independently test unavailable adapters, canonical assessment errors, artifact ordering, run identity, command input boundaries, duplicate/missing/extra bindings, preserved publication bytes, default/support anchor selection and unknown stage refusal. Original complete CLI declaration assertions remain consumer tests.

Proof source tests cover acquisition failure before publication, schema source ordering, missing-smoke materialization failure and refusal of malformed retained smoke without overwrite. Original public positive report contracts remain CLI consumer tests. Declaration proof is not native-tool or biological equivalence evidence.
