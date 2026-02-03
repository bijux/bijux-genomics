# fastq.correct metrics

Required
- reads_in
- reads_out
- bases_in
- bases_out
- mean_q_before
- mean_q_after
- kmer_fix_rate (proxy)

Derived
- read_retention = reads_out / reads_in
- base_retention = bases_out / bases_in
- error_reduction_proxy = max(0, mean_q_after - mean_q_before)

Invariants
- reads_out == reads_in
- bases_out <= bases_in
- mean_q_after >= mean_q_before (warn)
- kmer_fix_rate in [0, 1]
