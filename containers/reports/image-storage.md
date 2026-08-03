# LUNARC Apptainer canonical storage evidence

Purpose: record the reversible removal of unreferenced obsolete SIFs from the active canonical image root.

## Verdict

- Verdict: **passed**
- Repository commit: `c53e1e768744d854eff4678be1d775040a0b0e5a`
- Quarantined files: **14**
- Quarantined bytes: **2235879424**
- Historical run dependencies preserved: **1**

No SIF was deleted. Every move remained on the governed filesystem and is recoverable through the checksummed source plan after target digests are revalidated.

## Acceptance checks

- PASS: `plan_applied`
- PASS: `all_moves_applied`
- PASS: `move_count_matches`
- PASS: `move_size_matches`
- PASS: `only_current_or_unsupported_tools_remain`
- PASS: `no_unknown_images`
- PASS: `no_staged_images`
- PASS: `no_unreferenced_obsolete_images`

## Quarantined images

| Image | Category | Size (bytes) | SHA-256 | Quarantine path |
|---|---|---:|---|---|
| `alientrimmer-3.2-linux-amd64-5b4cb0770404.sif` | `obsolete` | 177602560 | `5b4cb0770404aad2b8a5288e90be805d2c2a10e17033d369f12e6e5aee12b8da` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/alientrimmer-3.2-linux-amd64-5b4cb0770404.sif` |
| `bracken-2.9-linux-amd64-aa477f012d9f.sif` | `obsolete` | 44494848 | `aa477f012d9f7b54ecf92ae39c4b2071d7174f9df4d6c163e1e97ea9af0e73c7` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/bracken-2.9-linux-amd64-aa477f012d9f.sif` |
| `dada2-1.30.0-linux-amd64-46ff3cf7673a.sif` | `obsolete` | 243642368 | `46ff3cf7673a5b100396db766159af6722deb6729f0381dcd21193e958bb9f4e` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/dada2-1.30.0-linux-amd64-46ff3cf7673a.sif` |
| `damageprofiler-1.1-linux-amd64-4c135c9ca508.sif` | `obsolete` | 167612416 | `4c135c9ca508fa0edb68ae59472cb4dafd2fde99cfcfcb9e73f79e6607f60420` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/damageprofiler-1.1-linux-amd64-4c135c9ca508.sif` |
| `damageprofiler-1.1-linux-amd64-860802600f73.sif` | `obsolete` | 164499456 | `860802600f732b9f038487a19117fea7a013734975879204235b2edc3caa5de8` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/damageprofiler-1.1-linux-amd64-860802600f73.sif` |
| `damageprofiler-1.1-linux-amd64-eb310de6eefe.sif` | `obsolete` | 164499456 | `eb310de6eefec837f53969e20b164b9e813ed69d11e599bb231268b7962fdb52` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/damageprofiler-1.1-linux-amd64-eb310de6eefe.sif` |
| `fastq_scan-1.0.1-linux-amd64-ef2aa335402f.sif` | `obsolete` | 120717312 | `ef2aa335402f3012886855d29ad62220ead94178974a7af3f5d97d26b69213cf` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/fastq_scan-1.0.1-linux-amd64-ef2aa335402f.sif` |
| `mapdamage2-2.2.2-linux-amd64-eebdbec567fb.sif` | `obsolete` | 175280128 | `eebdbec567fbeb14cb3d434d838a28a781b29a4a5238152ec7b76073b09dd17d` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/mapdamage2-2.2.2-linux-amd64-eebdbec567fb.sif` |
| `mapdamage2-2.2.2-linux-amd64-f762917cc1ad.sif` | `obsolete` | 175280128 | `f762917cc1adb21f95d56298831c06ec37e3fc44e240272768aca193cebc8fdf` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/mapdamage2-2.2.2-linux-amd64-f762917cc1ad.sif` |
| `mapdamage2-2.2.2-linux-amd64-fe844de7efe8.sif` | `obsolete` | 175280128 | `fe844de7efe86c58b1bd5a7682836257215d8af661c322daf23f5c0ce61aa74d` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/mapdamage2-2.2.2-linux-amd64-fe844de7efe8.sif` |
| `pear-0.9.6-linux-amd64-1fa2df569ac9.sif` | `obsolete` | 134995968 | `1fa2df569ac9197cffe5f0fc8375a53029a463c9557a3bf28d9f775823118ee1` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/pear-0.9.6-linux-amd64-1fa2df569ac9.sif` |
| `qualimap-2.3-linux-amd64-5c45a0c0f945.sif` | `obsolete` | 130244608 | `5c45a0c0f94538be96b28351a2ed0ad087b770395cf10fe32663175a1e200d5e` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/qualimap-2.3-linux-amd64-5c45a0c0f945.sif` |
| `qualimap-2.3-linux-amd64-ccf357b5a38d.sif` | `obsolete` | 134848512 | `ccf357b5a38da71b3ab17ab98e59dd1108c18843fdf3a9b28f796bcfb1257054` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/qualimap-2.3-linux-amd64-ccf357b5a38d.sif` |
| `spades-4.2.0-linux-amd64-12ec3751a5cc.sif` | `obsolete` | 226881536 | `12ec3751a5ccabc4225569f492434119c2d064f2df04a1433c31bed343469547` | `containers/apptainer/.quarantine/c53e1e768744d854eff4678be1d775040a0b0e5a/obsolete/spades-4.2.0-linux-amd64-12ec3751a5cc.sif` |
