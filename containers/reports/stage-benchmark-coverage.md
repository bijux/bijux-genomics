<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: containers/benchmarks/benchmark_registry.py -->
# Stage benchmark coverage

A defined campaign has complete retained-tool closure and a reproducible
measurement contract. It is not marked executed until Slurm evidence is
admitted separately.

- Retained stages: **71**
- Defined campaigns: **1**
- Missing campaigns: **70**

| Stage | Retained tools | Campaign | Definition |
|---|---|---|---|
| `bam.align` | `bowtie2`, `bwa` | — | missing |
| `bam.authenticity` | `authenticct`, `damageprofiler`, `pmdtools` | — | missing |
| `bam.bias_mitigation` | `mapdamage2` | — | missing |
| `bam.complexity` | `preseq` | — | missing |
| `bam.contamination` | `contammix`, `schmutzi`, `verifybamid2` | — | missing |
| `bam.coverage` | `bedtools`, `mosdepth`, `samtools` | — | missing |
| `bam.damage` | `addeam`, `damageprofiler`, `mapdamage2`, `pmdtools`, `pydamage` | — | missing |
| `bam.duplication_metrics` | `picard`, `samtools` | — | missing |
| `bam.endogenous_content` | `samtools` | — | missing |
| `bam.filter` | `bamtools`, `bedtools`, `samtools` | — | missing |
| `bam.gc_bias` | `picard` | — | missing |
| `bam.genotyping` | `bcftools` | `run-2001` | defined |
| `bam.haplogroups` | `yleaf` | — | missing |
| `bam.insert_size` | `picard` | — | missing |
| `bam.kinship` | `king` | — | missing |
| `bam.length_filter` | `picard`, `samtools` | — | missing |
| `bam.mapping_summary` | `picard`, `samtools` | — | missing |
| `bam.mapq_filter` | `bamtools`, `samtools` | — | missing |
| `bam.markdup` | `picard`, `samtools` | — | missing |
| `bam.overlap_correction` | `bamutil` | — | missing |
| `bam.qc_pre` | `multiqc`, `samtools` | — | missing |
| `bam.recalibration` | `gatk` | — | missing |
| `bam.sex` | `rxy`, `yleaf` | — | missing |
| `bam.validate` | `bamtools`, `bedtools`, `samtools` | — | missing |
| `fastq.cluster_otus` | `vsearch` | — | missing |
| `fastq.correct_errors` | `bayeshammer`, `lighter`, `musket`, `rcorrector` | — | missing |
| `fastq.deplete_host` | `bowtie2` | — | missing |
| `fastq.deplete_reference_contaminants` | `bowtie2` | — | missing |
| `fastq.deplete_rrna` | `sortmerna` | — | missing |
| `fastq.detect_adapters` | `fastqc` | — | missing |
| `fastq.detect_duplicates_premerge` | `bijux_dna` | — | missing |
| `fastq.estimate_library_complexity_prealign` | `bijux_dna` | — | missing |
| `fastq.extract_umis` | `umi_tools` | — | missing |
| `fastq.filter_low_complexity` | `bbduk`, `fastp`, `prinseq` | — | missing |
| `fastq.filter_reads` | `bbduk`, `fastp`, `prinseq`, `seqkit` | — | missing |
| `fastq.index_reference` | `bowtie2_build`, `star` | — | missing |
| `fastq.infer_asvs` | `dada2` | — | missing |
| `fastq.merge_pairs` | `adapterremoval`, `bbmerge`, `flash2`, `leehom`, `pear`, `seqprep`, `vsearch` | — | missing |
| `fastq.normalize_abundance` | `seqkit` | — | missing |
| `fastq.normalize_primers` | `cutadapt` | — | missing |
| `fastq.profile_overrepresented_sequences` | `fastq_scan`, `fastqc`, `seqkit` | — | missing |
| `fastq.profile_read_lengths` | `fastp`, `prinseq`, `seqfu`, `seqkit_stats` | — | missing |
| `fastq.profile_reads` | `seqfu`, `seqkit`, `seqkit_stats` | — | missing |
| `fastq.remove_chimeras` | `vsearch` | — | missing |
| `fastq.remove_duplicates` | `clumpify` | — | missing |
| `fastq.report_qc` | `multiqc` | — | missing |
| `fastq.screen_taxonomy` | `centrifuge`, `kaiju`, `kraken2`, `krakenuniq` | — | missing |
| `fastq.trim_polyg_tails` | `bbduk`, `fastp` | — | missing |
| `fastq.trim_reads` | `adapterremoval`, `alientrimmer`, `atropos`, `bbduk`, `cutadapt`, `fastp`, `fastx_clipper`, `leehom`, `prinseq`, `seqkit`, `seqprep`, `skewer`, `trim_galore`, `trimmomatic` | — | missing |
| `fastq.trim_terminal_damage` | `adapterremoval`, `cutadapt`, `seqkit` | — | missing |
| `fastq.validate_reads` | `fastq_scan`, `fastqc`, `fastqvalidator`, `fqtools`, `seqtk` | — | missing |
| `vcf.admixture` | `plink`, `plink2` | — | missing |
| `vcf.call` | `bcftools` | — | missing |
| `vcf.call_diploid` | `bcftools` | — | missing |
| `vcf.call_gl` | `bcftools` | — | missing |
| `vcf.call_pseudohaploid` | `bcftools` | — | missing |
| `vcf.damage_filter` | `bcftools` | — | missing |
| `vcf.demography` | `ibdne` | — | missing |
| `vcf.filter` | `bcftools` | — | missing |
| `vcf.gl_propagation` | `bcftools` | — | missing |
| `vcf.ibd` | `germline`, `ibdhap`, `ibdseq` | — | missing |
| `vcf.imputation_metrics` | `beagle`, `glimpse`, `minimac4` | — | missing |
| `vcf.impute` | `beagle`, `glimpse`, `minimac4` | — | missing |
| `vcf.pca` | `eigensoft`, `plink2` | — | missing |
| `vcf.phasing` | `beagle`, `eagle`, `shapeit`, `shapeit5` | — | missing |
| `vcf.population_structure` | `eigensoft`, `plink`, `plink2` | — | missing |
| `vcf.postprocess` | `bcftools` | — | missing |
| `vcf.prepare_reference_panel` | `bcftools` | — | missing |
| `vcf.qc` | `bcftools`, `plink`, `plink2` | — | missing |
| `vcf.roh` | `plink2` | — | missing |
| `vcf.stats` | `bcftools` | — | missing |
