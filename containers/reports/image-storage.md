# LUNARC Apptainer canonical storage evidence

Purpose: record the reversible removal of unreferenced obsolete SIFs from the active canonical image root.

## Verdict

- Verdict: **passed**
- Repository commit: `2738ed91e391016513bbcaa17a783245970c0bb0`
- Quarantined files: **3**
- Quarantined bytes: **206467072**
- Historical run dependencies preserved: **3**

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
| `bedtools-2.31.1-linux-amd64-4e2c0d7c8141.sif` | `obsolete` | 45846528 | `4e2c0d7c8141fe95d9e0d6cdff8a2042b7f1344c0adc6cbd4d21e577f50a1ebd` | `containers/apptainer/.quarantine/2738ed91e391016513bbcaa17a783245970c0bb0/obsolete/bedtools-2.31.1-linux-amd64-4e2c0d7c8141.sif` |
| `mosdepth-0.3.11-linux-amd64-79e73d45f00f.sif` | `obsolete` | 38264832 | `79e73d45f00f841ba587d400ea641165b39dea15c2fde3ec080d99a9024b580b` | `containers/apptainer/.quarantine/2738ed91e391016513bbcaa17a783245970c0bb0/obsolete/mosdepth-0.3.11-linux-amd64-79e73d45f00f.sif` |
| `samtools-1.20-linux-amd64-4bc914c8549b.sif` | `obsolete` | 122355712 | `4bc914c8549bd325b57927f693d21f6c37e463e8a3d2b5e4dc5d3ad19e1d0861` | `containers/apptainer/.quarantine/2738ed91e391016513bbcaa17a783245970c0bb0/obsolete/samtools-1.20-linux-amd64-4bc914c8549b.sif` |
