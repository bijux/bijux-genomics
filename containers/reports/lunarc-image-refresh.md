# LUNARC Apptainer image rebuild evidence

Purpose: record whether the selected stale or missing Apptainer images can be rebuilt from a frozen clean repository commit and admitted into a clean canonical inventory.

## Verdict

- Verdict: **passed**
- Run: `run-1000`
- Repository commit: `5a2f9252ed08b6a55b6d086828d850c0b961dbfc`
- Campaign scope: **targeted**
- Selected tools: **1**
- Final execution: **1 passed**, **0 not passed**
- Final inventory: **85 current**, **4 unsupported**, **0 staged**, **0 unknown**, **0 unreferenced obsolete**

## Claim boundary

This report proves source, build, and freshness evidence only for the selected tools. The final inventory checks prove that the canonical root is internally classified; they do not rebuild or requalify every unselected image. It does not prove runtime behavior, stage integration, real-data correctness, scientific validity, benchmark quality, security acceptance, or release qualification.

The machine-readable report records SHA-256 digests and study-relative locations for the immutable campaign execution and after-build inventory retained on LUNARC.

## Acceptance checks

- PASS: `campaign_passed`
- PASS: `all_selected_tools_passed`
- PASS: `all_selected_images_current`
- PASS: `all_selected_records_match_inventory`
- PASS: `no_staged_images`
- PASS: `no_unknown_images`
- PASS: `no_unreferenced_obsolete_images`

## Per-tool evidence

| Tool | Version | Final status | Attempts | Failed attempts | Image SHA-256 | Size (bytes) | Build-log SHA-256 |
|---|---|---|---:|---:|---|---:|---|
| `bijux_dna` | `0.1.0` | `passed` | 1 | 0 | `4cb58a275c3301f2eb29a0227f0407746955c68e02824f2368349ec074935b4e` | 460161024 | `c9decdd0ddbf3181424b20c7e67fe5ecb2383e2c78c49ab4506afbf441feb2d3` |
