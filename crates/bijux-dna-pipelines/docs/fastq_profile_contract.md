# FASTQ Profile Contract

This document defines what the FASTQ profile layer guarantees and how
`invariants_preset = "adna"` changes those guarantees.

## Base FASTQ guarantees

- Required processing stages include:
  - `fastq.validate_pre`
  - `fastq.detect_adapters`
  - `fastq.trim`
  - `fastq.filter`
  - `fastq.qc_post`
- Required parameter payloads are present and parseable for:
  - detect adapters
  - trim
  - filter

## aDNA guarantees (`invariants_preset = "adna"`)

- Includes all base FASTQ stages.
- Includes `fastq.merge` for paired-end short-fragment overlap handling.
- Uses aDNA-safe trimming defaults:
  - `trim.min_len > 0`
  - `trim.adapter_policy != "none"`
  - quality trimming enabled (`trim.q_cutoff`)
  - poly-X trimming enabled (`trim.polyx_policy`)
- Uses explicit short-read merge defaults:
  - `merge.min_len > 0`
  - `merge.merge_overlap` set
- Tool compatibility constraints:
  - `fastq.trim` tool must be one of `{adapterremoval, leehom}`
  - `fastq.merge` tool must be `leehom`

## Validation API

`validate_fastq_profile(profile)` returns a structured report:

- `valid`: boolean
- `violations`: list of invariant violations with codes and stage context

This API is used by contract tests and CLI explain surfaces to make profile
intent auditable.
