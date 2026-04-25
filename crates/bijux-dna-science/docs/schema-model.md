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
