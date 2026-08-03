# LUNARC Apptainer image rebuild evidence

Purpose: record whether the selected stale or missing Apptainer images can be rebuilt from a frozen clean repository commit and admitted into a clean canonical inventory.

## Verdict

- Verdict: **passed**
- Run: `run-1000`
- Repository commit: `915f5feb3091a3c78127fdddd9e0377e79b7ed33`
- Campaign scope: **targeted**
- Selected tools: **3**
- Final execution: **3 passed**, **0 not passed**
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
| `bedtools` | `2.31.1` | `passed` | 2 | 1 | `fe469da8f256a3a5730207531c03886b72675fecadbed2b3979c29e34634597e` | 45867008 | `112abcf6e352ad22f70f335f33d288cbcc05634d2775df23aae6fccfdc3b5f98` |
| `mosdepth` | `0.3.11` | `passed` | 1 | 0 | `8bd0a039c887f56a6fa9d68e0ac94edfc1177843bc1321eb5e941a1f516dde04` | 38289408 | `0cbb64fa048b09b8bc096ebabb56b0bb89bf662b94926ea8354dc5bf76e69e1c` |
| `samtools` | `1.20` | `passed` | 1 | 0 | `a29122faccca9bcba0c51adc528400970502b32e0a57f54e5c15cadda879b85f` | 122376192 | `82b5acc75f74f3766b0a159d2267ead97082fe10596dc743d32e14c73fb78c70` |
