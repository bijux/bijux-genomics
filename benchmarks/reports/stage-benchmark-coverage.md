<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::stage_coverage -->
# Stage benchmark coverage

A qualification campaign proves container execution, retained-tool closure,
and a reproducible measurement contract. It is not a comparative scientific
benchmark. Comparative readiness is governed separately by the corpus portfolio,
truth strategy, cohort size, output contract, and downstream substitution probes.

- Retained stages: **71**
- Defined campaigns: **6**
- Qualification-only campaigns: **4**
- Comparative campaigns: **2**
- Admitted campaigns: **0**
- Stale campaigns: **6**
- Failed campaigns: **0**
- Missing campaigns: **65**

| Stage | Retained tools | Campaign | Definition | Execution |
|---|---|---|---|---|
| `bam.align` | `bowtie2`, `bwa` | - | missing | not_defined |
| `bam.authenticity` | `authenticct`, `damageprofiler`, `pmdtools` | - | missing | not_defined |
| `bam.bias_mitigation` | `mapdamage2` | - | missing | not_defined |
| `bam.complexity` | `preseq` | - | missing | not_defined |
| `bam.contamination` | `contammix`, `schmutzi`, `verifybamid2` | - | missing | not_defined |
| `bam.coverage` | `bedtools`, `mosdepth`, `samtools` | `run-2002` | qualification | stale |
| `bam.damage` | `addeam`, `damageprofiler`, `mapdamage2`, `pmdtools`, `pydamage` | - | missing | not_defined |
| `bam.duplication_metrics` | `picard`, `samtools` | - | missing | not_defined |
| `bam.endogenous_content` | `samtools` | `run-2003` | qualification | stale |
| `bam.filter` | `bamtools`, `bedtools`, `samtools` | - | missing | not_defined |
| `bam.gc_bias` | `picard` | - | missing | not_defined |
| `bam.genotyping` | `bcftools` | `run-2001` | qualification | stale |
| `bam.haplogroups` | `yleaf` | - | missing | not_defined |
| `bam.insert_size` | `picard` | - | missing | not_defined |
| `bam.kinship` | `king` | - | missing | not_defined |
| `bam.length_filter` | `picard`, `samtools` | - | missing | not_defined |
| `bam.mapping_summary` | `picard`, `samtools` | `run-2004` | qualification | stale |
| `bam.mapq_filter` | `bamtools`, `samtools` | - | missing | not_defined |
| `bam.markdup` | `picard`, `samtools` | - | missing | not_defined |
| `bam.overlap_correction` | `bamutil` | - | missing | not_defined |
| `bam.qc_pre` | `multiqc`, `samtools` | - | missing | not_defined |
| `bam.recalibration` | `gatk` | - | missing | not_defined |
| `bam.sex` | `rxy`, `yleaf` | - | missing | not_defined |
| `bam.validate` | `bamtools`, `bedtools`, `samtools` | - | missing | not_defined |
| `fastq.cluster_otus` | `vsearch` | - | missing | not_defined |
| `fastq.correct_errors` | `bayeshammer`, `lighter`, `musket`, `rcorrector` | - | missing | not_defined |
| `fastq.deplete_host` | `bowtie2` | - | missing | not_defined |
| `fastq.deplete_reference_contaminants` | `bowtie2` | - | missing | not_defined |
| `fastq.deplete_rrna` | `sortmerna` | - | missing | not_defined |
| `fastq.detect_adapters` | `fastqc` | - | missing | not_defined |
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
| `fastq.profile_read_lengths` | `seqfu`, `seqkit` | `run-2042` | comparative | stale |
| `fastq.profile_reads` | `seqfu`, `seqkit`, `seqkit_stats` | `run-2043` | comparative | stale |
| `fastq.remove_chimeras` | `vsearch` | - | missing | not_defined |
| `fastq.remove_duplicates` | `clumpify` | - | missing | not_defined |
| `fastq.report_qc` | `multiqc` | - | missing | not_defined |
| `fastq.screen_taxonomy` | `centrifuge`, `kaiju`, `kraken2`, `krakenuniq` | - | missing | not_defined |
| `fastq.trim_polyg_tails` | `bbduk`, `fastp` | - | missing | not_defined |
| `fastq.trim_reads` | `adapterremoval`, `alientrimmer`, `atropos`, `bbduk`, `cutadapt`, `fastp`, `fastx_clipper`, `leehom`, `prinseq`, `seqkit`, `seqprep`, `skewer`, `trim_galore`, `trimmomatic` | - | missing | not_defined |
| `fastq.trim_terminal_damage` | `adapterremoval`, `cutadapt`, `seqkit` | - | missing | not_defined |
| `fastq.validate_reads` | `fastq_scan`, `fastqc`, `fastqvalidator`, `fqtools`, `seqtk` | - | missing | not_defined |
| `vcf.admixture` | `plink`, `plink2` | - | missing | not_defined |
| `vcf.call` | `bcftools` | - | missing | not_defined |
| `vcf.call_diploid` | `bcftools` | - | missing | not_defined |
| `vcf.call_gl` | `bcftools` | - | missing | not_defined |
| `vcf.call_pseudohaploid` | `bcftools` | - | missing | not_defined |
| `vcf.damage_filter` | `bcftools` | - | missing | not_defined |
| `vcf.demography` | `ibdne` | - | missing | not_defined |
| `vcf.filter` | `bcftools` | - | missing | not_defined |
| `vcf.gl_propagation` | `bcftools` | - | missing | not_defined |
| `vcf.ibd` | `germline`, `ibdhap`, `ibdseq` | - | missing | not_defined |
| `vcf.imputation_metrics` | `beagle`, `glimpse`, `minimac4` | - | missing | not_defined |
| `vcf.impute` | `beagle`, `glimpse`, `minimac4` | - | missing | not_defined |
| `vcf.pca` | `eigensoft`, `plink2` | - | missing | not_defined |
| `vcf.phasing` | `beagle`, `eagle`, `shapeit`, `shapeit5` | - | missing | not_defined |
| `vcf.population_structure` | `eigensoft`, `plink`, `plink2` | - | missing | not_defined |
| `vcf.postprocess` | `bcftools` | - | missing | not_defined |
| `vcf.prepare_reference_panel` | `bcftools` | - | missing | not_defined |
| `vcf.qc` | `bcftools`, `plink`, `plink2` | - | missing | not_defined |
| `vcf.roh` | `plink2` | - | missing | not_defined |
| `vcf.stats` | `bcftools` | - | missing | not_defined |
