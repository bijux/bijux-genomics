# Metric Semantics (FASTQ)

## retention
- numerator: reads_out
- denominator: reads_in
- units: reads
- failure modes: missing reads_in/out
- can be gamed by dropping low-quality reads without recording filters

## bases_kept
- numerator: bases_out
- denominator: bases_in
- units: bases
