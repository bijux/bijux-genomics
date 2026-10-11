# bijux-dna-bench-genotypes

Genotype calling and likelihood propagation benchmark services with explicit input
declarations, native execution, scientific observations and retained publication.
The command binary acquires the requested matrix row and renders the returned report.

## Ownership

- `call`: compatible calling alias, resolved to diploid execution.
- `call_gl`: likelihood calling from governed BAM inputs.
- `call_diploid`: diploid genotype observations and metrics.
- `call_pseudohaploid`: haploid projection and canonical payload replay checks.
- `gl_propagation`: preservation of declared GL/PL/GP fields through native output.

`inputs` admits stage/adapter/parser identities and unique output declarations.
`bam` reuses corpus alignment validation, materializes the reference, executes typed
calling modes and materializes VCF/index artifacts. Each service separates admission,
observations, report schemas and publication. Shared artifact publication belongs to
`bijux-dna-bench-artifacts`.

## Boundaries

The crate operates without CLI parsing or rendering; catalog acquisition belongs
to composition and native backend implementation belongs to VCF stages.

The crate compiles independently of the CLI, API, benchmark umbrella, variant,
haplotype and population benchmark services. BAM dependencies are intentional:
these workflows consume and validate real alignments. Native dependency receipts
and platform measurements determine scheduling; module count is not a speed claim.

## Public API

[Services](docs/PUBLIC_API.md) accept admitted declarations and return compatible typed reports.

## Architecture

[Architecture](docs/ARCHITECTURE.md) describes execution, observation and publication ownership.

## Tests

[Tests](docs/TESTS.md) cover original semantics, input refusals and native artifact retention.
