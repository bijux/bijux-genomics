# Examples Catalog

Examples are numbered by domain family and stage progression:

- `1xx` = FASTQ examples
- `2xx` = BAM examples
- `3xx` = VCF examples

Stage numbering semantics:

- The last two digits encode the stage-catalog progression within the domain family.
- `example-101` = FASTQ benchmark stage-01
- `example-102` = FASTQ benchmark stage-02
- `example-103` = FASTQ benchmark stage-03

Each example must be fully self-contained in `examples/example-XYZ/` with:

- `README.md`
- `example.toml`
- `bench-suite.toml` (example-pinned suite shape)
- `helpers/`
- `golden/plan.json`
- `golden/explain.json`
