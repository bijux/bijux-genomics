# CLI

Commands:

- `validate`: load authored specs and fail on schema or reference drift
- `build`: validate then emit deterministic generated outputs
- `trace`: print the compiled FASTQ environment rows for a selected stage or tool
- `release`: freeze a named science release manifest under `artifacts/science-releases/**`

Examples:

```bash
cargo run -p bijux-dna-science -- validate
cargo run -p bijux-dna-science -- build
cargo run -p bijux-dna-science -- trace --stage fastq.trim_reads
cargo run -p bijux-dna-science -- release --release-id release.fastq-environment-baseline
```
