# bijux-dna Architecture

`bijux-dna` is the user-facing CLI package. Its ideal tree is a small public root over a decomposed command surface:

```text
src/
├── lib.rs
├── cli_entrypoint.rs
├── process_exit.rs
├── public_api/
├── bin/
│   └── bijux-dna.rs
└── commands/
    ├── cli/
    ├── router/
    ├── benchmark/
    ├── bam/
    ├── fastq/
    ├── vcf/
    ├── ena/
    ├── hpc/
    ├── example/
    └── ...
```

Root responsibilities:

- `lib.rs` owns only the curated public surface.
- `cli_entrypoint.rs` owns crate-local CLI startup.
- `process_exit.rs` owns operator-facing refusal printing and exit-code policy.
- `public_api/` is the explicit namespace for the stable testable surface.
- `bin/bijux-dna.rs` is a thin process wrapper.

Command responsibilities:

- `commands/router/` owns argv parsing, cwd/environment setup, root-command routing, and CLI execution entry.
- `commands/benchmark/` owns all benchmark-specific configuration, corpus, workspace, publication, suite, and execution flows.
- `commands/bam/`, `commands/fastq/`, and `commands/vcf/` own domain-facing CLI dispatch only.
- The remaining command modules own focused root-level helpers such as examples, HPC layout, ENA materialization, and reporting.

This boundary keeps CLI startup, process exit policy, routing, and benchmark workflows from collapsing into ambiguous roots.
