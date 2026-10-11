<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::stage_coverage -->
# Stage benchmark coverage

A qualification campaign proves container execution and runnable-tool closure,
and a reproducible measurement contract. It is not a comparative scientific
benchmark. Comparative readiness is governed separately by the corpus portfolio,
truth strategy, cohort size, output contract, and downstream substitution probes.

- Retained stages: **71**
- Defined campaigns: **71**
- Qualification-only campaigns: **34**
- Comparative campaigns: **37**
- Admitted campaigns: **0**
- Stale campaigns: **0**
- Failed campaigns: **0**
- Missing campaigns: **0**

| Stage | Retained tools | Campaign | Definition | Execution |
|---|---|---|---|---|
| `bam.align` | `bowtie2`, `bwa` | `run-2005` | comparative | not_admitted |
| `bam.authenticity` | `authenticct`, `damageprofiler`, `pmdtools` | `run-2006` | comparative | not_admitted |
| `bam.bias_mitigation` | `mapdamage2` | `run-2007` | qualification | not_admitted |
| `bam.complexity` | `preseq` | `run-2008` | qualification | not_admitted |
| `bam.contamination` | `contammix`, `schmutzi`, `verifybamid2` | `run-2009` | comparative | not_admitted |
| `bam.coverage` | `bedtools`, `mosdepth`, `samtools` | `run-2002` | qualification | not_admitted |
| `bam.damage` | `addeam`, `damageprofiler`, `mapdamage2`, `ngsbriggs`, `pmdtools`, `pydamage` | `run-2010` | qualification | not_admitted |
| `bam.duplication_metrics` | `picard`, `samtools` | `run-2011` | qualification | not_admitted |
| `bam.endogenous_content` | `samtools` | `run-2003` | qualification | not_admitted |
| `bam.filter` | `bamtools`, `bedtools`, `samtools` | `run-2012` | comparative | not_admitted |
| `bam.gc_bias` | `picard` | `run-2013` | comparative | not_admitted |
| `bam.genotyping` | `bcftools` | `run-2001` | qualification | not_admitted |
| `bam.haplogroups` | `yleaf` | `run-2014` | qualification | not_admitted |
| `bam.insert_size` | `picard` | `run-2015` | comparative | not_admitted |
| `bam.kinship` | `angsd`, `king` | `run-2016` | comparative | not_admitted |
| `bam.length_filter` | `picard`, `samtools` | `run-2017` | comparative | not_admitted |
| `bam.mapping_summary` | `picard`, `samtools` | `run-2004` | qualification | not_admitted |
| `bam.mapq_filter` | `bamtools`, `samtools` | `run-2018` | comparative | not_admitted |
| `bam.markdup` | `picard`, `samtools` | `run-2019` | comparative | not_admitted |
| `bam.overlap_correction` | `bamutil` | `run-2020` | qualification | not_admitted |
| `bam.qc_pre` | `multiqc`, `samtools` | `run-2021` | comparative | not_admitted |
| `bam.recalibration` | `gatk` | `run-2022` | qualification | not_admitted |
| `bam.sex` | `angsd`, `rxy`, `yleaf` | `run-2023` | comparative | not_admitted |
| `bam.validate` | `bamtools`, `bedtools`, `samtools` | `run-2024` | comparative | not_admitted |
| `fastq.cluster_otus` | `vsearch` | `run-2025` | qualification | not_admitted |
| `fastq.correct_errors` | `bayeshammer`, `lighter`, `musket`, `rcorrector` | `run-2026` | comparative | not_admitted |
| `fastq.deplete_host` | `bowtie2` | `run-2027` | qualification | not_admitted |
| `fastq.deplete_reference_contaminants` | `bowtie2` | `run-2028` | qualification | not_admitted |
| `fastq.deplete_rrna` | `sortmerna` | `run-2029` | qualification | not_admitted |
| `fastq.detect_adapters` | `fastqc` | `run-2030` | qualification | not_admitted |
| `fastq.detect_duplicates_premerge` | `bijux_dna` | `run-2031` | qualification | not_admitted |
| `fastq.estimate_library_complexity_prealign` | `bijux_dna` | `run-2032` | qualification | not_admitted |
| `fastq.extract_umis` | `umi_tools` | `run-2033` | qualification | not_admitted |
| `fastq.filter_low_complexity` | `bbduk`, `fastp`, `prinseq` | `run-2034` | comparative | not_admitted |
| `fastq.filter_reads` | `bbduk`, `fastp`, `prinseq`, `seqkit` | `run-2035` | comparative | not_admitted |
| `fastq.index_reference` | `bowtie2_build`, `star` | `run-2036` | comparative | not_admitted |
| `fastq.infer_asvs` | `dada2` | `run-2037` | qualification | not_admitted |
| `fastq.merge_pairs` | `adapterremoval`, `bbmerge`, `flash2`, `leehom`, `pear`, `seqprep`, `vsearch` | `run-2038` | comparative | not_admitted |
| `fastq.normalize_abundance` | `seqkit` | `run-2039` | qualification | not_admitted |
| `fastq.normalize_primers` | `cutadapt` | `run-2040` | qualification | not_admitted |
| `fastq.profile_overrepresented_sequences` | `fastq_scan`, `fastqc`, `seqkit` | `run-2041` | comparative | not_admitted |
| `fastq.profile_read_lengths` | `seqfu`, `seqkit` | `run-2042` | comparative | not_admitted |
| `fastq.profile_reads` | `seqfu`, `seqkit`, `seqkit_stats` | `run-2043` | comparative | not_admitted |
| `fastq.remove_chimeras` | `vsearch` | `run-2044` | qualification | not_admitted |
| `fastq.remove_duplicates` | `clumpify`, `fastuniq` | `run-2045` | comparative | not_admitted |
| `fastq.report_qc` | `multiqc` | `run-2046` | qualification | not_admitted |
| `fastq.screen_taxonomy` | `centrifuge`, `kaiju`, `kraken2`, `krakenuniq` | `run-2047` | comparative | not_admitted |
| `fastq.trim_polyg_tails` | `bbduk`, `fastp` | `run-2048` | comparative | not_admitted |
| `fastq.trim_reads` | `adapterremoval`, `alientrimmer`, `atropos`, `bbduk`, `cutadapt`, `fastp`, `fastx_clipper`, `leehom`, `prinseq`, `seqkit`, `seqprep`, `skewer`, `trim_galore`, `trimmomatic` | `run-2049` | comparative | not_admitted |
| `fastq.trim_terminal_damage` | `adapterremoval`, `cutadapt`, `seqkit` | `run-2050` | comparative | not_admitted |
| `fastq.validate_reads` | `fastq_scan`, `fastqc`, `fastqvalidator`, `fqtools`, `seqtk` | `run-2051` | comparative | not_admitted |
| `vcf.admixture` | `plink`, `plink2` | `run-2052` | comparative | not_admitted |
| `vcf.call` | `bcftools` | `run-2053` | qualification | not_admitted |
| `vcf.call_diploid` | `bcftools` | `run-2054` | qualification | not_admitted |
| `vcf.call_gl` | `angsd`, `bcftools` | `run-2055` | comparative | not_admitted |
| `vcf.call_pseudohaploid` | `angsd`, `bcftools` | `run-2056` | comparative | not_admitted |
| `vcf.damage_filter` | `bcftools` | `run-2057` | qualification | not_admitted |
| `vcf.demography` | `ibdne` | `run-2058` | qualification | not_admitted |
| `vcf.filter` | `bcftools` | `run-2059` | qualification | not_admitted |
| `vcf.gl_propagation` | `bcftools` | `run-2060` | qualification | not_admitted |
| `vcf.ibd` | `germline`, `ibdhap`, `ibdseq` | `run-2061` | comparative | not_admitted |
| `vcf.imputation_metrics` | `beagle`, `glimpse`, `impute5`, `minimac4` | `run-2062` | comparative | not_admitted |
| `vcf.impute` | `beagle`, `glimpse`, `impute5`, `minimac4` | `run-2063` | comparative | not_admitted |
| `vcf.pca` | `eigensoft`, `plink2` | `run-2064` | comparative | not_admitted |
| `vcf.phasing` | `beagle`, `eagle`, `shapeit`, `shapeit5` | `run-2065` | comparative | not_admitted |
| `vcf.population_structure` | `eigensoft`, `plink`, `plink2` | `run-2066` | comparative | not_admitted |
| `vcf.postprocess` | `bcftools` | `run-2067` | qualification | not_admitted |
| `vcf.prepare_reference_panel` | `bcftools` | `run-2068` | qualification | not_admitted |
| `vcf.qc` | `bcftools`, `plink`, `plink2` | `run-2069` | comparative | not_admitted |
| `vcf.roh` | `plink2` | `run-2070` | qualification | not_admitted |
| `vcf.stats` | `bcftools` | `run-2071` | qualification | not_admitted |
