# Tool ID Contract

Purpose: Define the strict format and lifecycle contract for `containers/TOOL_IDS.txt`.

Authority surfaces:
- [../README.md](../README.md)
- [../index.md](../index.md)
- [../TOOL_IDS.txt](../TOOL_IDS.txt)

## Format
- File is generated-only from registry data by `cargo run -p bijux-dna-dev -- containers run generate-tool-ids`.
- Header lines are required:
  - `# GENERATED FILE - DO NOT EDIT`
  - `# Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-tool-ids`
  - `# format: <tool_id><TAB><status>`
- Data rows must be exactly `<tool_id>\t<status>`.

## Naming Rules
- `tool_id` must match `^[a-z][a-z0-9_]*$`.
- `tool_id` values are unique within the file.
- `status` must be one of `production`, `experimental`, `planned`.

## Authority
- `containers/TOOL_IDS.txt` is authoritative for allowed container filename IDs.
- Container defs and Dockerfiles must not introduce tool IDs absent from this file.
