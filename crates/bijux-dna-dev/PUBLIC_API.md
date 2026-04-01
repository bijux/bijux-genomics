# bijux-dna-dev Public API

`bijux-dna-dev` is a binary crate. Its durable entrypoints are:

- `src/main.rs`: process entrypoint for `cargo run -p bijux-dna-dev -- ...`
- `src/dev_entrypoint.rs::run`: crate-local launcher that wires the CLI into the development control plane
- `src/cli/`: versioned command schema, routing, and execution reporting for the developer-facing surface
