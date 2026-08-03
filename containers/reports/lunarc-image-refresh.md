# LUNARC Apptainer image rebuild evidence

Purpose: record whether every catalog-supported Apptainer image can be rebuilt from a frozen clean repository commit and admitted without leftover or unknown images.

## Verdict

- Verdict: **passed**
- Run: `run-1000`
- Repository commit: `32a447d03b36aeb617de08fda69f8b9fb1ff0f56`
- Selected tools: **2**
- Final execution: **2 passed**, **0 not passed**
- Final inventory: **85 current**, **4 unsupported**, **0 staged**, **0 unknown**, **0 unreferenced obsolete**

## Claim boundary

This report proves source, build, and freshness evidence for the complete retained Apptainer roster. It does not prove runtime behavior, stage integration, real-data correctness, scientific validity, benchmark quality, security acceptance, or release qualification.

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
| `bijux_dna` | `0.1.0` | `passed` | 1 | 0 | `5a7e64cf5cbe13f9af7396a32a082348c844c992351074a8f8b66ac37735ccde` | 460161024 | `af43138998e6d1e73fe21615736522515b880625086aca627c2eef71da5aef83` |
| `picard` | `3.3.0` | `passed` | 1 | 0 | `5d820089bef7854e698a7014f79df3f7b47568ed78b30e85842e9434ca6dcc92` | 160616448 | `3fb15c079232a0d558713a5c0f2becff6b7f9e99a61f9e6395402415f56d713e` |
