# Dependency Arrows

Allowed dependency direction is strictly:

core -> domain -> stages -> engine -> cli

## Rules

- A crate may only depend on crates to its left.
- Skip layers only when necessary (e.g., engine -> core).
- No reverse dependencies.

## Examples

Allowed:
- bijux-stages-fastq -> bijux-domain-fastq -> bijux-core
- bijux-engine -> bijux-core
- bijux-cli -> bijux-engine -> bijux-core

Not allowed:
- bijux-domain-fastq -> bijux-engine
- bijux-engine -> bijux-stages-fastq
- bijux-cli -> bijux-stages-fastq (must go through engine)
