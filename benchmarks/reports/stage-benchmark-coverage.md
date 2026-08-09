<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::stage_coverage -->
# Stage benchmark coverage

A qualification campaign proves container execution and runnable-tool closure,
and a reproducible measurement contract. It is not a comparative scientific
benchmark. Comparative readiness is governed separately by the corpus portfolio,
truth strategy, cohort size, output contract, and downstream substitution probes.

- Retained stages: **71**
- Defined campaigns: **12**
- Qualification-only campaigns: **6**
- Comparative campaigns: **6**
- Admitted campaigns: **0**
- Stale campaigns: **0**
- Failed campaigns: **0**
- Missing campaigns: **59**

| Stage | Retained tools | Campaign | Definition | Execution |
|---|---|---|---|---|
| `bam.align` | `bowtie2`, `bwa` | `run-2005` | comparative | not_admitted |
| `bam.authenticity` | `authenticct`, `damageprofiler`, `pmdtools` | - | missing | not_defined |
| `bam.bias_mitigation` | `mapdamage2` | - | missing | not_defined |
| `bam.complexity` | `preseq` | - | missing | not_defined |
| `bam.contamination` | `contammix`, `schmutzi`, `verifybamid2` | - | missing | not_defined |
| `bam.coverage` | `bedtools`, `mosdepth`, `samtools` | `run-2002` | qualification | not_admitted |
| `bam.damage` | `addeam`, `damageprofiler`, `mapdamage2`, `ngsbriggs`, `pmdtools`, `pydamage` | - | missing | not_defined |
| `bam.duplication_metrics` | `picard`, `samtools` | `run-2011` | qualification | not_admitted |
| `bam.endogenous_content` | `samtools` | `run-2003` | qualification | not_admitted |
| `bam.filter` | `bamtools`, `bedtools`, `samtools` | - | missing | not_defined |
| `bam.gc_bias` | `picard` | `run-2013` | comparative | not_admitted |
| `bam.genotyping` | `bcftools` | `run-2001` | qualification | not_admitted |
| `bam.haplogroups` | `yleaf` | - | missing | not_defined |
| `bam.insert_size` | `picard` | `run-2015` | comparative | not_admitted |
| `bam.kinship` | `angsd`, `king` | - | missing | not_defined |
| `bam.length_filter` | `picard`, `samtools` | - | missing | not_defined |
| `bam.mapping_summary` | `picard`, `samtools` | `run-2004` | qualification | not_admitted |
| `bam.mapq_filter` | `bamtools`, `samtools` | - | missing | not_defined |
| `bam.markdup` | `picard`, `samtools` | - | missing | not_defined |
| `bam.overlap_correction` | `bamutil` | - | missing | not_defined |
| `bam.qc_pre` | `multiqc`, `samtools` | - | missing | not_defined |
| `bam.recalibration` | `gatk` | - | missing | not_defined |
| `bam.sex` | `angsd`, `rxy`, `yleaf` | - | missing | not_defined |
| `bam.validate` | `bamtools`, `bedtools`, `samtools` | - | missing | not_defined |
| `fastq.cluster_otus` | `vsearch` | - | missing | not_defined |
| `fastq.correct_errors` | `bayeshammer`, `lighter`, `musket`, `rcorrector` | - | missing | not_defined |
| `fastq.deplete_host` | `bowtie2` | - | missing | not_defined |
| `fastq.deplete_reference_contaminants` | `bowtie2` | - | missing | not_defined |
| `fastq.deplete_rrna` | `sortmerna` | - | missing | not_defined |
| `fastq.detect_adapters` | `fastqc` | `run-2030` | qualification | not_admitted |
| `fastq.detect_duplicates_premerge` | `bijux_dna` | - | missing | not_defined |
| `fastq.estimate_library_complexity_prealign` | `bijux_dna` | - | missing | not_defined |
| `fastq.extract_umis` | `umi_tools` | - | missing | not_defined |
| `fastq.filter_low_complexity` | `bbduk`, `fastp`, `prinseq` | - | missing | not_defined |
| `fastq.filter_reads` | `bbduk`, `fastp`, `prinseq`, `seqkit` | - | missing | not_defined |
| `fastq.index_reference` | `bowtie2_build`, `star` | - | missing | not_defined |
| `fastq.infer_asvs` | `dada2` | - | missing | not_defined |
| `fastq.merge_pairs` | `adapterremoval`, `bbmerge`, `flash2`, `leehom`, `pear`, `seqprep`, `vsearch` | - | missing | not_defined |
| `fastq.normalize_abundance` | `seqkit` | - | missing | not_defined |
| `fastq.normalize_primers` | `cutadapt` | - | missing | not_defined |
| `fastq.profile_overrepresented_sequences` | `fastq_scan`, `fastqc`, `seqkit` | - | missing | not_defined |
| `fastq.profile_read_lengths` | `seqfu`, `seqkit` | `run-2042` | comparative | not_admitted |
| `fastq.profile_reads` | `seqfu`, `seqkit`, `seqkit_stats` | `run-2043` | comparative | not_admitted |
| `fastq.remove_chimeras` | `vsearch` | - | missing | not_defined |
| `fastq.remove_duplicates` | `clumpify`, `fastuniq` | - | missing | not_defined |
| `fastq.report_qc` | `multiqc` | - | missing | not_defined |
| `fastq.screen_taxonomy` | `centrifuge`, `kaiju`, `kraken2`, `krakenuniq` | - | missing | not_defined |
| `fastq.trim_polyg_tails` | `bbduk`, `fastp` | - | missing | not_defined |
| `fastq.trim_reads` | `adapterremoval`, `alientrimmer`, `atropos`, `bbduk`, `cutadapt`, `fastp`, `fastx_clipper`, `leehom`, `prinseq`, `seqkit`, `seqprep`, `skewer`, `trim_galore`, `trimmomatic` | - | missing | not_defined |
| `fastq.trim_terminal_damage` | `adapterremoval`, `cutadapt`, `seqkit` | - | missing | not_defined |
| `fastq.validate_reads` | `fastq_scan`, `fastqc`, `fastqvalidator`, `fqtools`, `seqtk` | `run-2051` | comparative | not_admitted |
| `vcf.admixture` | `plink`, `plink2` | - | missing | not_defined |
| `vcf.call` | `bcftools` | - | missing | not_defined |
| `vcf.call_diploid` | `bcftools` | - | missing | not_defined |
| `vcf.call_gl` | `angsd`, `bcftools` | - | missing | not_defined |
| `vcf.call_pseudohaploid` | `angsd`, `bcftools` | - | missing | not_defined |
| `vcf.damage_filter` | `bcftools` | - | missing | not_defined |
| `vcf.demography` | `ibdne` | - | missing | not_defined |
| `vcf.filter` | `bcftools` | - | missing | not_defined |
| `vcf.gl_propagation` | `bcftools` | - | missing | not_defined |
| `vcf.ibd` | `germline`, `ibdhap`, `ibdseq` | - | missing | not_defined |
| `vcf.imputation_metrics` | `beagle`, `glimpse`, `impute5`, `minimac4` | - | missing | not_defined |
| `vcf.impute` | `beagle`, `glimpse`, `impute5`, `minimac4` | - | missing | not_defined |
| `vcf.pca` | `eigensoft`, `plink2` | - | missing | not_defined |
| `vcf.phasing` | `beagle`, `eagle`, `shapeit`, `shapeit5` | - | missing | not_defined |
| `vcf.population_structure` | `eigensoft`, `plink`, `plink2` | - | missing | not_defined |
| `vcf.postprocess` | `bcftools` | - | missing | not_defined |
| `vcf.prepare_reference_panel` | `bcftools` | - | missing | not_defined |
| `vcf.qc` | `bcftools`, `plink`, `plink2` | - | missing | not_defined |
| `vcf.roh` | `plink2` | - | missing | not_defined |
| `vcf.stats` | `bcftools` | - | missing | not_defined |
