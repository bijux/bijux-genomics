# Schema Model

The authored evidence plane currently supports:

- `sources/*.yaml`
- `evidences/*.yaml`
- `claims/*.yaml`
- `assumptions/*.yaml`
- `reasoning/*.yaml`
- `decisions/*.yaml`
- `bindings/*.yaml`
- `science/specs/releases/manifests/*.yaml`

Every record uses a typed ID, explicit schema version, and explicit cross-record references.

The initial generated slice is `fastq_stage_tool_environment_matrix`, compiled from authored
bindings plus current repo authority files.

When science needs non-shareable or manually acquired evidence, the authored
record stays in `science/specs/**` and the local payload may live under
`science-docs/**`. The payload is supporting material, not the authored source of
truth.
