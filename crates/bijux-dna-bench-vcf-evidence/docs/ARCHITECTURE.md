# VCF evidence ownership

The acquisition boundary supplies `VcfEvidenceRows`. Admission consumes the rows
and produces an immutable `VcfEvidenceSnapshot`; selecting any number of stages
performs no further acquisition or publication.

Retained, active, parser and expected-result observations use stage/tool/corpus/asset
identity. Output declarations, command rows and report placement have stage/tool
identity and may apply to several corpus bindings. Duplicate identities are errors,
even when duplicate records have equal values. Parser observations are never joined
by tool alone. Active and parser adapter/parser/schema declarations must match the
retained binding; a covered parser fixture must name its native parser and schema. Native fixture
contracts intentionally use a different namespace from stage-level normalized
metrics contracts. Acquisition validates their actual inventory and payloads.

Missing surfaces are preserved as absent observations. Admission rejects foreign or
unretained observations and inconsistent report placement; it does not invent missing
coverage, inspect payloads, run tools or make scientific support claims. Snapshot
validation cannot detect evidence discarded by an upstream adapter before admission.

`commands` requires exact correspondence between structured argv, per-step shell
commands, pipe dependencies and the script command list. Quoting preserves empty
arguments and shell metacharacters. Existing report field names, schema versions and
serialization order remain the compatibility contract.

The crate intentionally depends only on serialization and error libraries. This
allows scientific capability services to share evidence without importing the
adapter acquisition service, benchmark umbrella or public CLI. Native compilation
measurements determine speed claims; this boundary alone is not a CI timing proof.
