<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::run_registry -->
# Permanent benchmark run registry

Run IDs identify durable stage protocols, not attempts or publications. Existing IDs are immutable; additions allocate monotonically from the declared frontier.

- Registry SHA-256: `e098eddc58a8e8c8c5e04da604abd9eec4929751fd8297713945ed7d7e43853b`
- Registered protocols: **71**
- Active protocols: **13**
- Reserved protocols: **58**
- Next allocation: **run-2072**

| Run | Stage | Profile | Tools | State | Interface |
|---|---|---|---|---|---|
| `run-2001` | `bam.genotyping` | `bam-biological-inference` | `bcftools` | `active` | `bijux.benchmark.bam.genotyping.bundle.v1` |
| `run-2002` | `bam.coverage` | `bam-observational-qc` | `bedtools`, `mosdepth`, `samtools` | `active` | `bijux.benchmark.bam.coverage.bundle.v1` |
| `run-2003` | `bam.endogenous_content` | `bam-observational-qc` | `samtools` | `active` | `bijux.benchmark.bam.endogenous_content.bundle.v1` |
| `run-2004` | `bam.mapping_summary` | `bam-observational-qc` | `picard`, `samtools` | `active` | `bijux.benchmark.bam.mapping_summary.bundle.v1` |
| `run-2005` | `bam.align` | `bam-alignment` | `bowtie2`, `bwa` | `active` | `bijux.benchmark.bam.align.bundle.v1` |
| `run-2006` | `bam.authenticity` | `bam-ancient-authentication` | `authenticct`, `damageprofiler`, `pmdtools` | `reserved` | - |
| `run-2007` | `bam.bias_mitigation` | `bam-transformation` | `mapdamage2` | `reserved` | - |
| `run-2008` | `bam.complexity` | `bam-observational-qc` | `preseq` | `reserved` | - |
| `run-2009` | `bam.contamination` | `bam-biological-inference` | `contammix`, `schmutzi`, `verifybamid2` | `reserved` | - |
| `run-2010` | `bam.damage` | `bam-ancient-authentication` | `addeam`, `damageprofiler`, `mapdamage2`, `ngsbriggs`, `pmdtools`, `pydamage` | `active` | `bijux.benchmark.bam.damage.bundle.v1` |
| `run-2011` | `bam.duplication_metrics` | `bam-observational-qc` | `picard`, `samtools` | `active` | `bijux.benchmark.bam.duplication_metrics.bundle.v1` |
| `run-2012` | `bam.filter` | `bam-transformation` | `bamtools`, `bedtools`, `samtools` | `reserved` | - |
| `run-2013` | `bam.gc_bias` | `bam-observational-qc` | `picard` | `active` | `bijux.benchmark.bam.gc_bias.bundle.v1` |
| `run-2014` | `bam.haplogroups` | `bam-biological-inference` | `yleaf` | `reserved` | - |
| `run-2015` | `bam.insert_size` | `bam-observational-qc` | `picard` | `active` | `bijux.benchmark.bam.insert_size.bundle.v1` |
| `run-2016` | `bam.kinship` | `bam-biological-inference` | `angsd`, `king` | `reserved` | - |
| `run-2017` | `bam.length_filter` | `bam-transformation` | `picard`, `samtools` | `reserved` | - |
| `run-2018` | `bam.mapq_filter` | `bam-transformation` | `bamtools`, `samtools` | `reserved` | - |
| `run-2019` | `bam.markdup` | `bam-transformation` | `picard`, `samtools` | `reserved` | - |
| `run-2020` | `bam.overlap_correction` | `bam-transformation` | `bamutil` | `reserved` | - |
| `run-2021` | `bam.qc_pre` | `bam-observational-qc` | `multiqc`, `samtools` | `reserved` | - |
| `run-2022` | `bam.recalibration` | `bam-transformation` | `gatk` | `reserved` | - |
| `run-2023` | `bam.sex` | `bam-biological-inference` | `angsd`, `rxy`, `yleaf` | `reserved` | - |
| `run-2024` | `bam.validate` | `bam-observational-qc` | `bamtools`, `bedtools`, `samtools` | `reserved` | - |
| `run-2025` | `fastq.cluster_otus` | `fastq-amplicon` | `vsearch` | `reserved` | - |
| `run-2026` | `fastq.correct_errors` | `fastq-read-transformation` | `bayeshammer`, `lighter`, `musket`, `rcorrector` | `reserved` | - |
| `run-2027` | `fastq.deplete_host` | `fastq-depletion` | `bowtie2` | `reserved` | - |
| `run-2028` | `fastq.deplete_reference_contaminants` | `fastq-depletion` | `bowtie2` | `reserved` | - |
| `run-2029` | `fastq.deplete_rrna` | `fastq-depletion` | `sortmerna` | `reserved` | - |
| `run-2030` | `fastq.detect_adapters` | `fastq-observational-qc` | `fastqc` | `active` | `bijux.benchmark.fastq.detect_adapters.bundle.v1` |
| `run-2031` | `fastq.detect_duplicates_premerge` | `fastq-observational-qc` | `bijux_dna` | `reserved` | - |
| `run-2032` | `fastq.estimate_library_complexity_prealign` | `fastq-observational-qc` | `bijux_dna` | `reserved` | - |
| `run-2033` | `fastq.extract_umis` | `fastq-read-transformation` | `umi_tools` | `reserved` | - |
| `run-2034` | `fastq.filter_low_complexity` | `fastq-read-transformation` | `bbduk`, `fastp`, `prinseq` | `reserved` | - |
| `run-2035` | `fastq.filter_reads` | `fastq-read-transformation` | `bbduk`, `fastp`, `prinseq`, `seqkit` | `reserved` | - |
| `run-2036` | `fastq.index_reference` | `reference-index-build` | `bowtie2_build`, `star` | `reserved` | - |
| `run-2037` | `fastq.infer_asvs` | `fastq-amplicon` | `dada2` | `reserved` | - |
| `run-2038` | `fastq.merge_pairs` | `fastq-read-transformation` | `adapterremoval`, `bbmerge`, `flash2`, `leehom`, `pear`, `seqprep`, `vsearch` | `reserved` | - |
| `run-2039` | `fastq.normalize_abundance` | `fastq-amplicon` | `seqkit` | `reserved` | - |
| `run-2040` | `fastq.normalize_primers` | `fastq-read-transformation` | `cutadapt` | `reserved` | - |
| `run-2041` | `fastq.profile_overrepresented_sequences` | `fastq-observational-qc` | `fastq_scan`, `fastqc`, `seqkit` | `reserved` | - |
| `run-2042` | `fastq.profile_read_lengths` | `fastq-observational-qc` | `seqfu`, `seqkit` | `active` | `bijux.benchmark.fastq.profile_read_lengths.bundle.v1` |
| `run-2043` | `fastq.profile_reads` | `fastq-observational-qc` | `seqfu`, `seqkit`, `seqkit_stats` | `active` | `bijux.benchmark.fastq.profile_reads.bundle.v1` |
| `run-2044` | `fastq.remove_chimeras` | `fastq-amplicon` | `vsearch` | `reserved` | - |
| `run-2045` | `fastq.remove_duplicates` | `fastq-read-transformation` | `clumpify`, `fastuniq` | `reserved` | - |
| `run-2046` | `fastq.report_qc` | `fastq-observational-qc` | `multiqc` | `reserved` | - |
| `run-2047` | `fastq.screen_taxonomy` | `fastq-taxonomy` | `centrifuge`, `kaiju`, `kraken2`, `krakenuniq` | `reserved` | - |
| `run-2048` | `fastq.trim_polyg_tails` | `fastq-read-transformation` | `bbduk`, `fastp` | `reserved` | - |
| `run-2049` | `fastq.trim_reads` | `fastq-read-transformation` | `adapterremoval`, `alientrimmer`, `atropos`, `bbduk`, `cutadapt`, `fastp`, `fastx_clipper`, `leehom`, `prinseq`, `seqkit`, `seqprep`, `skewer`, `trim_galore`, `trimmomatic` | `reserved` | - |
| `run-2050` | `fastq.trim_terminal_damage` | `fastq-read-transformation` | `adapterremoval`, `cutadapt`, `seqkit` | `reserved` | - |
| `run-2051` | `fastq.validate_reads` | `fastq-observational-qc` | `fastq_scan`, `fastqc`, `fastqvalidator`, `fqtools`, `seqtk` | `active` | `bijux.benchmark.fastq.validate_reads.bundle.v1` |
| `run-2052` | `vcf.admixture` | `vcf-population-inference` | `plink`, `plink2` | `reserved` | - |
| `run-2053` | `vcf.call` | `vcf-calling` | `bcftools` | `reserved` | - |
| `run-2054` | `vcf.call_diploid` | `vcf-calling` | `bcftools` | `reserved` | - |
| `run-2055` | `vcf.call_gl` | `vcf-calling` | `angsd`, `bcftools` | `reserved` | - |
| `run-2056` | `vcf.call_pseudohaploid` | `vcf-calling` | `angsd`, `bcftools` | `reserved` | - |
| `run-2057` | `vcf.damage_filter` | `vcf-transformation` | `bcftools` | `reserved` | - |
| `run-2058` | `vcf.demography` | `vcf-population-inference` | `ibdne` | `reserved` | - |
| `run-2059` | `vcf.filter` | `vcf-transformation` | `bcftools` | `reserved` | - |
| `run-2060` | `vcf.gl_propagation` | `vcf-transformation` | `bcftools` | `reserved` | - |
| `run-2061` | `vcf.ibd` | `vcf-population-inference` | `germline`, `ibdhap`, `ibdseq` | `reserved` | - |
| `run-2062` | `vcf.imputation_metrics` | `vcf-phasing-imputation` | `beagle`, `glimpse`, `impute5`, `minimac4` | `reserved` | - |
| `run-2063` | `vcf.impute` | `vcf-phasing-imputation` | `beagle`, `glimpse`, `impute5`, `minimac4` | `reserved` | - |
| `run-2064` | `vcf.pca` | `vcf-population-inference` | `eigensoft`, `plink2` | `reserved` | - |
| `run-2065` | `vcf.phasing` | `vcf-phasing-imputation` | `beagle`, `eagle`, `shapeit`, `shapeit5` | `reserved` | - |
| `run-2066` | `vcf.population_structure` | `vcf-population-inference` | `eigensoft`, `plink`, `plink2` | `reserved` | - |
| `run-2067` | `vcf.postprocess` | `vcf-transformation` | `bcftools` | `reserved` | - |
| `run-2068` | `vcf.prepare_reference_panel` | `vcf-transformation` | `bcftools` | `reserved` | - |
| `run-2069` | `vcf.qc` | `vcf-transformation` | `bcftools`, `plink`, `plink2` | `reserved` | - |
| `run-2070` | `vcf.roh` | `vcf-population-inference` | `plink2` | `reserved` | - |
| `run-2071` | `vcf.stats` | `vcf-transformation` | `bcftools` | `reserved` | - |
