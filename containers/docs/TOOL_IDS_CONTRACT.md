# Tool ID Contract

Purpose: Define the strict identity and support-disposition contract for
`containers/TOOL_IDS.txt`.

Authority surfaces:
- [../README.md](../README.md)
- [../index.md](../index.md)
- [../TOOL_IDS.txt](../TOOL_IDS.txt)

## Format
- File is generated-only from registry data by `cargo run -p bijux-dna-dev -- containers run generate-tool-ids`.
- Header lines are required:
  - `# GENERATED FILE - DO NOT EDIT`
  - `# Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-tool-ids`
  - `# format: <tool_id><TAB><support_disposition>`
- Data rows must be exactly `<tool_id>\t<support_disposition>`.

## Naming Rules
- `tool_id` must match `^[a-z][a-z0-9_]*$`.
- `tool_id` values are unique within the file.
- `support_disposition` must be one of `retained`, `unsupported`, `retired`,
  or `excluded`.
- Disposition answers whether Bijux Genomics ships the implementation. It is
  not a production claim. Qualification remains evidence-derived in
  [qualification.md](../reports/qualification.md).

## Authority
- The canonical catalog generates `containers/TOOL_IDS.txt`; the manifest is
  the compact filename-ID boundary consumed by validation commands.
- Container defs and Dockerfiles must not introduce tool IDs absent from this file.
