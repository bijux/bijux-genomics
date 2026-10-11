# BAM evidence contracts

`EvidenceContext` supplies an explicit root, canonical default paths and four provider
traits. `ServingEvidence`, `OutputEvidence`, `ReportEvidence` and `ConsistencyEvidence`
can be substituted independently without linking native executors into the owner.
Planner observations retain missing declarations and raw/normalized artifact roles;
the owner does not select a different report or infer compatibility from suffixes.

`command_coverage`, `parser_coverage`, `output_contract`, `report_map` and `consistency`
export the original BAM row/report types and render operations. The first four also
export collectors for canonical composition. Field names, enum snake-case labels,
TSV columns, report identities, error details and CLI defaults retain their contracts.
Enum `as_str` methods expose the same labels used by TSV and JSON serialization.

Coverage statuses describe existing declarations. Parser-fixture coverage does not
execute parsers, validate new payloads, or prove scientific equivalence. Synthetic
owner observations exercise engineering boundaries only. Canonical acquisition and
native adapter semantics remain separately tested by their existing owners.

## BAM readiness contracts

Six public `readiness` family modules export their existing row/report models and
`render_bam_*_ready` functions. `ReadinessContext` carries the repository root,
seven canonical evidence addresses, a `DeclarationEvidence` provider and a
`LocalProofWriter` provider. `LocalProofKind` enumerates the twelve supported
native proof operations; the evidence layer never silently substitutes another kind.

The `bam_downstream` feature controls haplogroups, genotyping and kinship bindings.
Schema input is the canonical normalized-metrics JSON declaration; published source
inputs reuse shared canonical report types. Serialized model fields, condition
precedence, report counts, artifact identities and CLI output defaults are retained.

Acquisition failure returns the original error before report replacement. An assessed
incomplete report is atomically published before returning its readiness failure.
Provider publications are independent effects, not a transaction. Duplicate report-map
binding identities are now rejected as ambiguous evidence.
