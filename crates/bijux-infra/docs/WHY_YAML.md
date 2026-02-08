# WHY_YAML

## Who uses YAML
- `bijux-environment` uses YAML for tool image spec configuration files.

## Why JSON is not sufficient there
Tool image specs are often managed by operators in YAML-first pipelines and
must remain compatible with existing YAML inventories.

## Scope
YAML is permitted only for config compatibility and must not be used for
contract JSON schemas. JSON remains the format for contracts.
