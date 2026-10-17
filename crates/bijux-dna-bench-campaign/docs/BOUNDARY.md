# Corpus campaign boundary

Owner: Corpus benchmark orchestration.
Scope: Admit resolved corpus requests, plan samples, run bounded execution through a caller port, manage cache/QC preparation and publish the existing run manifest.
Allowed inputs: Explicit request paths, source/provenance identities, resources, ordered defaults, admitted corpus contracts and caller-provided execution, paths and clock ports.
Forbidden dependencies: CLI parsing, application configuration discovery, native scientific engines, API presentation and sibling repository checkouts.
Forbidden effects: Direct process spawning, network requests and process-global environment mutation. File acquisition, cache preparation and manifest publication belong to this owner; callers must control concurrent writers to admitted output roots.
Validation command: `cargo test --locked -p bijux-dna-bench-campaign --all-targets`

Input and publication roots must be disjoint. Sample identities cannot escape their namespaces. Tool exits remain sample observations; transport errors leave the previous run manifest intact. Publication is per file, and admission checks do not establish a filesystem race sandbox or backend scientific equivalence.
