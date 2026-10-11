# LUNARC Apptainer canonical storage evidence

Purpose: record the reversible removal of unreferenced obsolete SIFs from the active canonical image root.

## Verdict

- Verdict: **passed**
- Repository commit: `5a2f9252ed08b6a55b6d086828d850c0b961dbfc`
- Quarantined files: **2**
- Quarantined bytes: **920322048**
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
| `bijux_dna-0.1.0-linux-amd64-ed8434190891.sif` | `obsolete` | 460161024 | `ed8434190891ee95da032f818ad10f62fcee8ee5453a404c1861455681a78e8a` | `containers/apptainer/.quarantine/5a2f9252ed08b6a55b6d086828d850c0b961dbfc/obsolete/bijux_dna-0.1.0-linux-amd64-ed8434190891.sif` |
| `bijux_dna-0.1.0-linux-amd64-fa2e18e4b00a.sif` | `obsolete` | 460161024 | `fa2e18e4b00a0d54d0e024ef12b5a3b8793c9f42434d8936fcb1b3b1997e838b` | `containers/apptainer/.quarantine/5a2f9252ed08b6a55b6d086828d850c0b961dbfc/obsolete/bijux_dna-0.1.0-linux-amd64-fa2e18e4b00a.sif` |
