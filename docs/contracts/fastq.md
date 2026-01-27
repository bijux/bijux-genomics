# FASTQ output contract

This document freezes the on-disk output contract for FASTQ stages. Tests may
assert this structure in the future.

## Directory layout

All FASTQ stage runs write into a run directory with a stable layout:

```
<run_root>/
  artifacts/               # Stage outputs (FASTQ, reports, stats)
  logs/
    tool.log               # Tool stdout/stderr
  manifest.json            # Execution manifest
  metrics.json             # Execution + stage metrics
```

Bench outputs are grouped by stage/sample/tool, but the per-run layout above is
identical within each run directory.

## File naming (artifacts/)

Output filenames are tool-scoped and follow the tool manifest expectations. The
canonical filenames below are required when the corresponding tool is used.

### fastq.validate

No FASTQ outputs are produced.

Required artifacts:
- `artifacts/` may be empty

### fastq.trim

Required artifacts (per tool):
- fastp: `fastp.fastq.gz`
- cutadapt: `cutadapt.fastq.gz`
- atropos: `atropos.fastq.gz`
- bbduk: `bbduk.fastq.gz`
- adapterremoval: `adapterremoval.fastq.gz`
- trimmomatic: `trimmomatic.fastq.gz`
- trim_galore: `trimmed_trimmed.fq.gz`
- seqpurge: `seqpurge.fastq.gz`

### fastq.filter

Required artifacts (per tool):
- fastp: `fastp.fastq.gz`
- prinseq: `prinseq_good.fastq`
- seqkit: `seqkit.fastq.gz`

### fastq.merge

Required artifacts (per tool):
- pear: `pear.fastq.gz`
- vsearch: `vsearch.fastq.gz`
- bbmerge: `bbmerge.fastq.gz`
- flash2: `flash2.fastq.gz`

### fastq.correct

Required artifacts (per tool):
- rcorrector: `rcorrector.fastq.gz`
- spades: `spades.fastq.gz`
- bayeshammer: `bayeshammer.fastq.gz`
- lighter: `lighter.fastq.gz`
- musket: `musket.fastq.gz`

### fastq.qc2

Required artifacts (per tool):
- fastqc: `*_fastqc.zip` and extracted directory
- multiqc: `multiqc_report.html` and `multiqc_data/`

### fastq.umi

Required artifacts (per tool):
- umi_tools: `umi_tools.fastq.gz`

### fastq.stats

Required artifacts (per tool):
- seqkit_stats: `seqkit_stats.tsv`

### fastq.screen

Required artifacts (per tool):
- kraken2: `kraken2.report`
- centrifuge: `centrifuge.tsv`
- metaphlan: `metaphlan.tsv`
- kaiju: `kaiju.tsv`
- fastq_screen: `fastq_screen.txt`
