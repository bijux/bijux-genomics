# Tool ID Contract

Purpose: Define the strict format and lifecycle contract for `containers/TOOL_IDS.txt`.

## Format
- File is generated-only from registry data by `scripts/containers/generate-tool-ids.sh`.
- Header lines are required:
  - `# GENERATED FILE - DO NOT EDIT`
  - `# Regenerate with: scripts/containers/generate-tool-ids.sh`
  - `# format: <tool_id><TAB><status>`
- Data rows must be exactly `<tool_id>\t<status>`.

## Naming Rules
- `tool_id` must match `^[a-z][a-z0-9_]*$`.
- `tool_id` values are unique within the file.
- `status` must be one of `production`, `experimental`, `planned`.

## Authority
- `containers/TOOL_IDS.txt` is authoritative for allowed container filename IDs.
- Container defs and Dockerfiles must not introduce tool IDs absent from this file.
