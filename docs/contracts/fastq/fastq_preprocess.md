# fastq.preprocess contract (v1)

## Inputs
- SE: reads_r1 (FASTQ)
- PE: reads_r1 + reads_r2 (FASTQ)

## Outputs
- final_reads_r1 / final_reads_r2 (FASTQ)
- manifest_json (JSON)

## Mandatory metrics
- read_retention
- base_retention
- delta_mean_q

## Allowed side-effects
- Writes per-stage artifacts and manifests under artifacts/bench and artifacts/image-qa.

## Failure vs warning
- Fail: any stage contract violation or missing output.
- Warn: mean_q regression within tolerated bounds.
