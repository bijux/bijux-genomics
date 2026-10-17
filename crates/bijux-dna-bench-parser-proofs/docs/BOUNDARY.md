# Parser proof boundary

Owner: Governed parser proof aggregation.
Scope: Admit configured proof families, request materialization through an explicit port, parse the selected representation, aggregate deterministic rows and publish the report.
Allowed inputs: Governed family declarations, proof paths and observed tool identities supplied by `ProbeMaterializer`; FASTQ and BAM native parsers only through their explicit features.
Forbidden dependencies: CLI arguments, repository configuration discovery, application presentation and sibling repository checkouts. Default compilation does not admit a native scientific engine.
Forbidden effects: Direct backend process execution, network requests and process-global environment mutation. Proof reads and atomic per-file report publication are supported effects; native generation belongs to the supplied producer.
Validation command: `cargo test --locked -p bijux-dna-bench-parser-proofs --all-targets --all-features`

Family uniqueness and compiled parser support are admitted before producer invocation. Failed report builds preserve prior aggregate reports; individual producer effects are not a multi-file transaction. Required VCF key admission and synthetic fixture rows establish representation contracts, not value-level scientific accuracy.
