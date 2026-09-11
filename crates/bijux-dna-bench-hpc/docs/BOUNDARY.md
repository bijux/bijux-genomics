# Local HPC benchmark evidence boundary

Owner: bijux-dna-bench-hpc
Scope: Deterministic watchdog policy simulations, input admission and report publication.
Allowed inputs: Explicit canonical benchmark DAG evidence, scenario selection and caller-supplied provenance/output paths.
Forbidden dependencies: CLI, API, general benchmark orchestration, planners and backend transports.
Forbidden effects: Working-directory discovery, global environment mutation, process launch and remote submission. Publication occurs through infrastructure atomic JSON writing after admission.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-hpc --all-targets

The benchmark-model dependency disables default features; its own transitive domain contracts remain part of the native closure. Simulation timing and injected failure reports establish policy examples. They do not demonstrate remote backend execution or scientific performance. See [public inputs and outputs](PUBLIC_API.md) and [architecture](ARCHITECTURE.md).


Readiness orchestration has no CLI, API, benchmark umbrella, planner or runner dependency. It accepts explicit provider ports and writes local diagnostic/support artifacts through infrastructure publication helpers. The local campaign port cannot request remote submission. TOML serialization owns escaping of generated local profile path values.
