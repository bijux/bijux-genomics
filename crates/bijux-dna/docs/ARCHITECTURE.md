# bijux-dna Architecture

`bijux-dna` is the user-facing CLI package. Its ideal tree is a small public root over an explicit command control plane:

```text
src/
├── lib.rs
├── cli_entrypoint.rs
├── process_exit.rs
├── public_api/
├── bin/
│   └── bijux-dna.rs
└── commands/
    ├── bam/
    ├── benchmark/
    ├── cli/
    │   ├── env/
    │   ├── parse/
    │   │   └── bench/
    │   │       └── fastq/
    │   ├── plan/
    │   └── render/
    ├── corpus/
    ├── ena/
    ├── example.rs
    ├── example/
    ├── fastq/
    │   ├── api_bridge.rs
    │   └── meta/
    ├── hpc/
    ├── planning/
    ├── router/
    ├── status/
    ├── support/
    └── vcf/
```

Root responsibilities:

- `lib.rs` owns only the curated public surface.
- `cli_entrypoint.rs` owns crate-local CLI startup.
- `process_exit.rs` owns operator-facing refusal printing and exit-code policy.
- `public_api/` is the explicit namespace for the stable testable surface.
- `bin/bijux-dna.rs` is a thin process wrapper.

Command responsibilities:

- `commands/router/` owns argv parsing, cwd/environment setup, root-command routing, and CLI execution entry.
- `commands/support/` owns cross-command helpers: shared imports, report input resolution, run profile loading, workspace audit policy, and workspace root discovery.
- `commands/planning/` owns run-plan assembly and dry-run planning entrypoints.
- `commands/status/` owns status inspection flows.
- `commands/corpus/` owns curated corpus workflows.
- `commands/benchmark/` owns all benchmark-specific configuration, corpus, workspace, publication, suite, and execution flows.
- `commands/benchmark/fastq_bench/` owns FASTQ benchmark execution entry, adapter-bank inspection, stage discovery, stage explanation, and tool-tier policy as separate internal concerns.
- `commands/benchmark/corpus_fastq/` owns governed corpus benchmark execution, with run models, stage preparation, runtime support, report-qc support, sortmerna support, and artifact-bundle hashing separated by responsibility.
- `commands/benchmark/workspace/` owns benchmark config contracts, config queries, publication contract lookup, stage-run layout policy, and workspace value queries.
- `commands/cli/` owns operator-facing parse, render, plan, validation, and environment command support. Within that tree, `commands/cli/parse/bench/` owns bench-specific CLI parsing, and `commands/cli/parse/bench/fastq/` keeps preprocessing, quality, and workflow argument families separate.
- `commands/fastq/meta/` owns FASTQ meta-command routing, with dedicated handlers for pipeline, analysis, and environment command families plus focused debug dispatch; `commands/fastq/api_bridge.rs` stays focused on API mediation.
- `commands/cli/env/` owns environment registry queries, promotion policy, runtime support, registry commands, and benchmark HPC root support as separate internal concerns instead of one include-driven command blob.
- `commands/bam/`, `commands/fastq/`, and `commands/vcf/` own domain-facing CLI dispatch only.
- `commands/ena/`, `commands/hpc/`, and `example.rs` own focused operator-facing helpers that do not belong in the routing or support layers.

This boundary keeps CLI startup, process exit policy, routing, planning, parse contracts, support helpers, and domain dispatch from collapsing into ambiguous root files.
