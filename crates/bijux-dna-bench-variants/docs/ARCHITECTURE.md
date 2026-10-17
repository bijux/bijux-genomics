# Variant benchmark architecture

Each capability separates `admission`, `execution`, `observations`, `model` and
`publication`. The workflow composes typed prepared inputs, native stage outputs,
artifact paths and scientific evidence. Identity declarations, scientific records
and public wire reports have separate model modules.

QC admits report formats independently of execution. Cohort metric validation,
sample exclusion and variant exclusion preserve the governed fixture's exact
scientific expectations. Tool-table projections use the admitted format and are
kept out of fixture generation and scientific observation code.

The shared artifact owner admits the capability's inventory filename/schema and
report owner. New execution files are prepared beneath `executions/run-*`; native
VCF files, manifests, checksums and provenance retain their physical paths.
Stable files are copied after path/owner admission, and the public benchmark
stage-result index projects only output addresses. Unknown files survive.

Postprocessing hashes exact final-manifest bytes, but hashes compact JSON values
for normalization and validation contracts. Neither representation is rewritten.
Prior executions remain retained because downstream reports may refer to them.
Individual file replacements are atomic; publication is not a multi-file
transaction. Concurrent writers to one publication root are unsupported.

## Readiness records

Readiness flags observe independent proof surfaces, and incomplete combinations
remain valid diagnostic data. Flat serialized row fields preserve the existing
report contract. Pure field projections stay explicit in their capability owner;
scientific admission remains in coverage/science modules. Scoped lint allowances
document these boundaries without changing field identities or assessment behavior.
