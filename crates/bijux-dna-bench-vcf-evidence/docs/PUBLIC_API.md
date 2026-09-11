# Public API

`model` contains the established serialized VCF readiness records, with their
existing fields, enum labels and report schema identities.

`snapshot::VcfEvidenceRows` accepts untrusted observations. `ActiveVcfBinding` is
an explicit projection from the all-domain authority; it carries only the full
binding identity and adapter/parser/schema declarations required by VCF consumers.
`VcfEvidenceSnapshot::admit` validates all rows before returning any observation.
`stage_bindings` returns a borrowed stable slice or an error for an unretained stage.
The snapshot exposes no mutable records and can be reused across stage requests.

`commands::admission` validates argv, command identities and pipe topology.
`commands::projection` preserves the shell and argv JSONL report formats. Callers
must admit untrusted rows before projection. Filesystem publication is external.

Evidence coverage is not native tool execution, payload normalization, or a proof
of interchangeability between scientific tools.
