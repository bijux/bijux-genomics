# Binding evidence boundary

Owner: bijux-dna-bench-bindings
Scope: FASTQ/BAM serving-map and expected-result admission, projection and publication.
Allowed inputs: Explicit roots, canonical scheduler layout values and typed source observations.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; API, benchmark umbrella/model, native domains, planners and runner are excluded from normal and dev closures.
Forbidden effects: No process execution, networking, environment mutation, root discovery or recursive output deletion. Tests must not become production dependencies.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-bindings --all-targets

Output publication uses atomic infrastructure replacement per file. Providers retain
their canonical local acquisition effects. No multi-file transaction is implied.
