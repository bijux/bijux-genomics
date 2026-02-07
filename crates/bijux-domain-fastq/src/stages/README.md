# stages

This directory defines FASTQ stage semantics and contracts.

- **Stage semantics**: conceptual meaning of each stage (what it does, what it preserves).
- **Stage specs**: declarative inputs/outputs/constraints for planners.
- **Invariants**: boundary guarantees validated across stages.

Planners consume these definitions to select tools, but do not redefine the stage contracts.
