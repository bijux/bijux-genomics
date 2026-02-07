# ARCHITECTURE

Infra exports only low-level utilities (paths, logging, formats, IO helpers).
No domain semantics should live here.

`formats/yaml.rs` is intentionally allowlisted. YAML is only used for narrow
interop cases and should not be a default serialization target.
