# FASTQ Tools Roster

| Stage | Supported tools | Rationale |
| --- | --- | --- |
| fastq.validate_pre | fastqvalidator, seqkit | FastQ integrity + counts |
| fastq.detect_adapters | fastp | Integrated adapter detection |
| fastq.trim | fastp, cutadapt, trimmomatic | Proven trimming strategies |
| fastq.filter | seqkit, prinseq, fastp | Quality/length filtering |
| fastq.stats_neutral | seqkit_stats | Fast summaries |
| fastq.merge | pear, flash2, bbmerge, vsearch | Paired‑end merging variants |
| fastq.correct | rcorrector, spades/bayeshammer, lighter, musket | Error correction options |
| fastq.umi | umi_tools | UMI‑aware handling |
| fastq.qc_post | multiqc | Aggregated QC reporting |
| fastq.screen | kraken2, centrifuge | Taxonomic screening |
