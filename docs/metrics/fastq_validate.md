# FASTQ Validate Metrics Spec v1

This document defines the **FastQ Validate** benchmark schema and the rules that govern it. It is a contract.

## Scope

Applies to the `fastq.validate` stage and any tool that emits `FastqValidateMetrics` v1.

## Metrics

### reads
- **Definition:** Total number of reads observed in the input FASTQ.
- **Type:** `u64`
- **Measurement:** Parsed from the tool output or computed by a deterministic counter over the input. For paired-end inputs, `reads` is the sum of both mates.

### bases
- **Definition:** Total number of bases observed in the input FASTQ.
- **Type:** `u64`
- **Measurement:** Derived from the sum of per-read sequence lengths. For paired-end inputs, `bases` is the sum across both mates.

### mean_q
- **Definition:** Mean Phred quality score across all bases.
- **Type:** `f64`
- **Measurement:** Weighted mean across all bases. For paired-end inputs, weights are per-base counts per mate. Reported with the tool's native precision or rounded to two decimals if computed externally.

### format_valid
- **Definition:** Whether the input FASTQ is syntactically valid per tool-specific validation rules.
- **Type:** `bool`
- **Measurement:** `true` if the tool reports the input as valid (or if all internal checks pass); otherwise `false`.

## Invariants

- `reads` ≥ 0
- `bases` ≥ 0
- `mean_q` is a finite number
- If `reads == 0`, then `bases == 0`

Invalid metrics **must** hard-fail validation.

## What is intentionally excluded

- Per-base quality distributions
- Adapter presence/contamination estimates
- Read length histograms
- Duplicate or overrepresented sequence stats
- Any tool-specific warnings beyond `format_valid`

## Known limitations

- `format_valid` is tool-dependent and may differ across validators.
- `mean_q` computation can vary slightly by tool parsing strategy and rounding.
- The schema does not capture paired-end consistency checks beyond what each tool encodes in `format_valid`.
