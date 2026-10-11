# Tool ID Contract

Purpose: Define the strict identity, product-support, artifact-delivery, and redistribution contract for
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
  - `# format: <tool_id><TAB><product_support><TAB><artifact_delivery><TAB><redistribution_permitted>`
- Data rows must contain exactly those four tab-separated fields.

## Naming Rules
- `tool_id` must match `^[a-z][a-z0-9_]*$`.
- `tool_id` values are unique within the file.
- `product_support` must be one of `supported`, `retired`, or `excluded`.
- `artifact_delivery` must be one of `redistributable_image`,
  `external_artifact`, `private_site_build`, or `site_module`.
- `redistribution_permitted` must be `true` or `false`; redistributable images
  require `true`, while external and site delivery require `false`.
- Product support does not depend on redistribution permission or delivery
  mechanism. These dimensions are not production claims. Qualification remains evidence-derived in
  [qualification.md](../reports/qualification.md).
- External-artifact availability remains evidence-derived and is not encoded in
  this identity manifest.

## Authority
- The canonical catalog generates `containers/TOOL_IDS.txt`; the manifest is
  the compact filename-ID boundary consumed by validation commands.
- Container defs and Dockerfiles must not introduce tool IDs absent from this file.
