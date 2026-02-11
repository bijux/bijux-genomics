# FASTQ Metrics Contract

Schema: `bijux.metrics.summary.v1`

Required stage metrics in default FASTQ pipelines:
- `fastq.trim`: trim retention and quality deltas (`reads_in/out`, `bases_in/out`, `delta_metrics`, `retention`)
- `fastq.filter`: filter removals and retention (`reads_removed_*`, `delta_metrics`, `retention`)
- `fastq.qc_post`: QC summary with adapter/duplication/N-content indicators and report paths

Tool metrics schemas covered by parsers:
- `bijux.fastp.metrics.v1`
- `bijux.adapterremoval.metrics.v1`
- `bijux.seqkit.metrics.v1`
- `bijux.samtools.flagstat.v1`
- `bijux.fastqc.metrics.v1`
- `bijux.multiqc.metrics.v1`

Provenance requirements:
- Every metrics envelope must include `metric_provenance`.
- `metric_provenance.params_hash` must be the stage parameter hash.
- `metric_provenance.tool_id/tool_version` must match the executed tool.
- `metric_provenance.input_artifact_hashes` must include all hashed stage inputs available at metric build time.
