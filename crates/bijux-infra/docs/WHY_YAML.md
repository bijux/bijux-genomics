# WHY_YAML

YAML is allowed only for configuration compatibility with external tooling.
The YAML API is intentionally narrow and must live in `formats/yaml.rs`.

No YAML parsing may be introduced elsewhere.
