# fastq.umi metrics

Required
- reads_in
- reads_out
- dedup_rate

Derived
- read_retention = reads_out / reads_in

Invariants
- reads_out <= reads_in
- dedup_rate in [0, 1]
