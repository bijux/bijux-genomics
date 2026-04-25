# Reference Asset Evidence

## What
Evidence and provenance notes for governed FASTQ reference assets under `assets/reference/`.

## Why
Reference assets affect scientific interpretation as much as tool choice. Adapter banks, primer banks, contaminant references, and QC thresholds must be explicit about whether they are production references, sentinel records, or synthetic test motifs.

## Asset Status
| Asset group | Current files | Evidence status | Review rule |
| --- | --- | --- | --- |
| Adapter bank | `adapters/bank.v1.yaml`, `adapters/presets.v1.yaml` | Mixed vendor-derived motifs and synthetic fallback motifs | Treat vendor-derived motifs as supported only when a matching source note is present; treat synthetic motifs as test coverage, not kit truth. |
| Primer bank | `primers/*.fasta` | Literature-derived marker primers | Keep marker, sequence, and citation together when adding or changing primers. |
| Contaminant motifs | `contaminants/contaminant_motifs.v1.yaml` | Small deterministic motif set | Use for deterministic detection tests and policy wiring, not as a complete contamination database. |
| Contaminant references | `contaminants/references/phix174.fasta`, `contaminants/references/univec.fasta` | Sentinel FASTA records in the current repository | Do not describe these as complete PhiX174 or UniVec references until replaced by pinned upstream snapshots. |
| PolyX bank | `polyx/bank.v1.yaml`, `polyx/presets.v1.yaml` | Deterministic sequence-tail policy assets | Use as policy inputs for trimming behavior, not as external biological references. |
| QC thresholds | `qc_thresholds.yaml` | Governed thresholds | Interpret only with the matching FASTQ stage assumptions and profile defaults. |

## Primer References
| Primer set | Marker | Current sequences | Primary evidence |
| --- | --- | --- | --- |
| `COI_folmer_v1` | mitochondrial COI barcode | LCO1490, HCO2198 | Folmer et al. 1994, published COI primer pair for invertebrate mitochondrial cytochrome c oxidase I. |
| `16S_universal_v1` | bacterial 16S rRNA | 27F, 1492R | Weisburg et al. 1991, broad bacterial 16S amplification primers. |
| `ITS2_plant_v1` | plant ITS2 | S2F, S3R | Chen et al. 2010, ITS2 plant DNA barcode primer set. |

## Sentinel Contaminant References
The current PhiX174 and UniVec FASTA payloads are deliberately tiny records:

- `phix174.fasta` is 48 bytes in the current checkout.
- `univec.fasta` is 43 bytes in the current checkout.

Those sizes are incompatible with complete upstream PhiX174 or UniVec reference payloads. They are suitable for path, checksum, parser, and policy tests only. Production contaminant depletion or screening requires pinned upstream replacement payloads, checksum review, and an updated lock note.

## Update Requirements
- Stage candidate updates under `artifacts/assets-refresh/reference/`.
- Record upstream URL, retrieval date, checksum, sequence count, and total bases.
- Diff sequence headers and lengths before replacing a tracked reference.
- Recompute affected `CHECKSUMS.sha256` files in the same change.
- Add uncertain source or missing production payload questions to `/Users/bijan/bijux/NEEDED.md`.
