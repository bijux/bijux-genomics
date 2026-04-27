# FASTQ Reference Governance

## Purpose
Define the governed bank boundary for FASTQ stages that depend on external reference, primer,
adapter, taxonomy, or asset registries.

## Scope
This document covers every FASTQ stage whose manifest declares non-empty `bank_hooks` under
[domain/fastq/stages/](../../../domain/fastq/stages/).

## Non-goals
- Replacing the lower-level bank-hook implementation in the FASTQ stage manifests.
- Claiming that stages with `bank_hooks: ["none"]` are free from all scientific assumptions.
- Listing every file emitted by reference or asset materialization.

## Contracts
- Any FASTQ stage with non-empty `bank_hooks` in
  [domain/fastq/stages/](../../../domain/fastq/stages/) must appear exactly once here.
- Route-level required assets in
  [domain/fastq/route_policies.toml](../../../domain/fastq/route_policies.toml) remain refusal
  boundaries, not best-effort hints.
- Pinned default bank-owning stages and baseline policy live in
  [domain/fastq/docs/DEFAULT_SETTINGS.md](../../../domain/fastq/docs/DEFAULT_SETTINGS.md).

| Stage | Required banks | Why it exists |
| --- | --- | --- |
| fastq.build_contaminant_db | contaminant_database_lock | Contaminant database builds must publish a governed lock identity before downstream depletion contracts can reuse them. |
| fastq.build_rrna_db | rrna_database_lock | rRNA database builds must stay pinned so depletion results are comparable across runs. |
| fastq.build_taxonomy_db | taxonomy_database_lock | Taxonomy database builds need governed identity before classification outputs are scientifically interpretable. |
| fastq.capture_provenance_snapshot | provenance_registry | Provenance capture only makes sense against the governed registry of route, tool, and asset identities. |
| fastq.demultiplex_reads | barcode_kit_bank | Barcode-kit interpretation is a governed bank choice, not an incidental inline string. |
| fastq.deplete_host | reference_bank | Host depletion is only interpretable when the admitted host reference bundle is explicit. |
| fastq.deplete_reference_contaminants | contaminant_db_bank | Reference-guided contaminant depletion depends on a governed contaminant database identity. |
| fastq.deplete_rrna | rrna_database_lock | rRNA depletion must identify the governed rRNA database build used by the run. |
| fastq.detect_adapters | adapter_bank | Adapter detection claims only compare cleanly when the governed adapter bank is explicit. |
| fastq.detect_instrument_artifacts | instrument_artifact_bank | Instrument-artifact screening depends on the governed artifact signature registry. |
| fastq.filter_reads | contaminant_db_bank | Read filtering can depend on governed contaminant signatures that must stay explicit in provenance. |
| fastq.index_reference | reference_bank | Reference indexing must remain tied to the governed reference bundle it materializes. |
| fastq.normalize_primers | primer_bank | Primer normalization only makes sense when the governed primer bank and orientation contract are explicit. |
| fastq.prepare_adapter_bank | adapter_bank | Adapter-bank preparation owns the admitted adapter source inventory used by downstream detection and trimming. |
| fastq.prepare_host_reference_bundle | reference_bank | Host-reference preparation must publish the governed bundle identity that later depletion stages consume. |
| fastq.prepare_primer_bank | primer_bank | Primer-bank preparation owns the primer catalog that later normalization and amplicon routes consume. |
| fastq.screen_taxonomy | taxonomy_database_lock | Taxonomy screening depends on a locked classifier database, not a floating local install. |
| fastq.trim_polyg_tails | polyx_bank | PolyG and other homopolymer trimming requires a governed polyX policy bank. |
| fastq.trim_reads | adapter_bank, contaminant_db_bank, polyx_bank | Primary trimming crosses adapter, contaminant, and polyX governance boundaries and must keep them explicit together. |
| fastq.trim_terminal_damage | adapter_bank | Damage-aware terminal trimming still depends on the governed adapter bank before any damage policy is meaningful. |
| fastq.verify_assets | asset_lock_registry | Asset verification is only valid against the governed lock registry it is checking. |

## Runtime Rules
- Do not hardcode host filesystem paths for reference-like banks.
- Bank-backed runs must emit provenance that identifies the admitted bank or lock identity.
- Route-level required assets and manifest-level `bank_hooks` must agree; a route cannot silently weaken a stage bank boundary.

## Failure modes
- Using the right tool against the wrong bank makes FASTQ outputs look operationally valid while being scientifically non-comparable.
- Hiding bank identity in route or stage provenance makes amplicon and depletion comparisons non-auditable across runs.
