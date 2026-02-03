# FASTQ Validate Metrics Spec v1

This document defines the **FastQ Validate** benchmark schema and the rules that govern it. It is a contract.

## Scope

Applies to the `fastq.validate` stage and any tool that emits `FastqValidateMetrics` v1.

## Metrics

### reads_total
- **Definition:** Total number of reads observed in the input FASTQ.
- **Type:** `u64`
- **Measurement:** Parsed from the validation tool output or computed deterministically. For paired-end inputs, this is the sum across both mates.

### reads_valid
- **Definition:** Number of reads that pass validation.
- **Type:** `u64`
- **Measurement:** Reported by the validation tool. If the tool only signals pass/fail, `reads_valid` is either `reads_total` (pass) or `0` (fail).

### reads_invalid
- **Definition:** Number of reads that fail validation.
- **Type:** `u64`
- **Measurement:** `reads_total - reads_valid`.

### mean_q
- **Definition:** Mean Phred quality score across all bases.
- **Type:** `f64`
- **Measurement:** Weighted mean across all bases. For paired-end inputs, weights are per-base counts per mate. Reported with the tool's native precision or rounded to two decimals if computed externally.

## Invariants

- `reads_valid + reads_invalid == reads_total`
- `mean_q ∈ [0, 45]`
- counts are non-negative

Invalid metrics **must** hard-fail validation.

## What is intentionally excluded

- Per-base quality distributions
- Adapter presence/contamination estimates
- Read length histograms
- Duplicate or overrepresented sequence stats
- Any tool-specific warnings beyond the counts above

## Known limitations

- Some validators only expose pass/fail; in that case `reads_valid` may collapse to `reads_total` or `0`.
- `mean_q` computation can vary slightly by tool parsing strategy and rounding.
- The schema does not capture paired-end consistency checks beyond what each tool encodes.
