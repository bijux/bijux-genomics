# FASTQ Domain (Bijux)

This document is authority, not marketing.

## What FASTQ means in Bijux
- FASTQ is treated as *read sets with integrity and pairing semantics*.
- The domain enforces stage contracts before and after execution.
- Outputs are normalized at stage boundaries to canonical names.

## Stage guarantees
- **validate**: reports strict format correctness, no data mutation.
- **trim**: may drop reads/bases; improves quality metrics.
- **filter**: may drop reads; quality shifts should improve.
- **merge**: requires paired input; emits merged reads.
- **correct**: preserves pairing; aims to reduce error proxies.
- **stats**: observational; never changes data.

## Not guaranteed
- No silent tool switching.
- No implicit QC scoring.
- No hidden heuristics.

## Non-authoritative tools
Some tools remain non-authoritative due to known variability or undocumented behavior. They can run, but they do not define truth.

## Why this is strict
FASTQ outputs are used in downstream scientific interpretation. The domain must be explicit about guarantees, or results become non-reproducible.
