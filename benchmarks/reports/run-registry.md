<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::run_registry -->
# Permanent benchmark run registry

Run IDs identify durable stage protocols, not attempts or publications. Existing IDs are immutable; additions allocate monotonically from the declared frontier.

- Registry SHA-256: `ccad111cdfcb726577275c2d315ea8fb30c63a884238d4684d793fac07a4b585`
- Registered protocols: **71**
- Active protocols: **71**
- Reserved protocols: **0**
- Next allocation: **run-2072**

| Run | Stage | Profile | Tools | State | Interface |
|---|---|---|---|---|---|
| `run-2001` | `bam.genotyping` | `bam-biological-inference` | `bcftools` | `active` | `bijux.benchmark.bam.genotyping.bundle.v1` |
| `run-2002` | `bam.coverage` | `bam-observational-qc` | `bedtools`, `mosdepth`, `samtools` | `active` | `bijux.benchmark.bam.coverage.bundle.v1` |
| `run-2003` | `bam.endogenous_content` | `bam-observational-qc` | `samtools` | `active` | `bijux.benchmark.bam.endogenous_content.bundle.v1` |
| `run-2004` | `bam.mapping_summary` | `bam-observational-qc` | `picard`, `samtools` | `active` | `bijux.benchmark.bam.mapping_summary.bundle.v1` |
| `run-2005` | `bam.align` | `bam-alignment` | `bowtie2`, `bwa` | `active` | `bijux.benchmark.bam.align.bundle.v1` |
| `run-2006` | `bam.authenticity` | `bam-ancient-authentication` | `authenticct`, `damageprofiler`, `pmdtools` | `active` | `bijux.benchmark.bam.authenticity.bundle.v1` |
| `run-2007` | `bam.bias_mitigation` | `bam-transformation` | `mapdamage2` | `active` | `bijux.benchmark.bam.bias_mitigation.bundle.v1` |
| `run-2008` | `bam.complexity` | `bam-observational-qc` | `preseq` | `active` | `bijux.benchmark.bam.complexity.bundle.v1` |
| `run-2009` | `bam.contamination` | `bam-biological-inference` | `contammix`, `schmutzi`, `verifybamid2` | `active` | `bijux.benchmark.bam.contamination.bundle.v1` |
| `run-2010` | `bam.damage` | `bam-ancient-authentication` | `addeam`, `damageprofiler`, `mapdamage2`, `ngsbriggs`, `pmdtools`, `pydamage` | `active` | `bijux.benchmark.bam.damage.bundle.v1` |
| `run-2011` | `bam.duplication_metrics` | `bam-observational-qc` | `picard`, `samtools` | `active` | `bijux.benchmark.bam.duplication_metrics.bundle.v1` |
| `run-2012` | `bam.filter` | `bam-transformation` | `bamtools`, `bedtools`, `samtools` | `active` | `bijux.benchmark.bam.filter.bundle.v1` |
| `run-2013` | `bam.gc_bias` | `bam-observational-qc` | `picard` | `active` | `bijux.benchmark.bam.gc_bias.bundle.v1` |
| `run-2014` | `bam.haplogroups` | `bam-biological-inference` | `yleaf` | `active` | `bijux.benchmark.bam.haplogroups.bundle.v1` |
| `run-2015` | `bam.insert_size` | `bam-observational-qc` | `picard` | `active` | `bijux.benchmark.bam.insert_size.bundle.v1` |
| `run-2016` | `bam.kinship` | `bam-biological-inference` | `angsd`, `king` | `active` | `bijux.benchmark.bam.kinship.bundle.v1` |
| `run-2017` | `bam.length_filter` | `bam-transformation` | `picard`, `samtools` | `active` | `bijux.benchmark.bam.length_filter.bundle.v1` |
| `run-2018` | `bam.mapq_filter` | `bam-transformation` | `bamtools`, `samtools` | `active` | `bijux.benchmark.bam.mapq_filter.bundle.v1` |
| `run-2019` | `bam.markdup` | `bam-transformation` | `picard`, `samtools` | `active` | `bijux.benchmark.bam.markdup.bundle.v1` |
| `run-2020` | `bam.overlap_correction` | `bam-transformation` | `bamutil` | `active` | `bijux.benchmark.bam.overlap_correction.bundle.v1` |
| `run-2021` | `bam.qc_pre` | `bam-observational-qc` | `multiqc`, `samtools` | `active` | `bijux.benchmark.bam.qc_pre.bundle.v1` |
| `run-2022` | `bam.recalibration` | `bam-transformation` | `gatk` | `active` | `bijux.benchmark.bam.recalibration.bundle.v1` |
| `run-2023` | `bam.sex` | `bam-biological-inference` | `angsd`, `rxy`, `yleaf` | `active` | `bijux.benchmark.bam.sex.bundle.v1` |
| `run-2024` | `bam.validate` | `bam-observational-qc` | `bamtools`, `bedtools`, `samtools` | `active` | `bijux.benchmark.bam.validate.bundle.v1` |
| `run-2025` | `fastq.cluster_otus` | `fastq-amplicon` | `vsearch` | `active` | `bijux.benchmark.fastq.cluster_otus.bundle.v1` |
| `run-2026` | `fastq.correct_errors` | `fastq-read-transformation` | `bayeshammer`, `lighter`, `musket`, `rcorrector` | `active` | `bijux.benchmark.fastq.correct_errors.bundle.v1` |
| `run-2027` | `fastq.deplete_host` | `fastq-depletion` | `bowtie2` | `active` | `bijux.benchmark.fastq.deplete_host.bundle.v1` |
| `run-2028` | `fastq.deplete_reference_contaminants` | `fastq-depletion` | `bowtie2` | `active` | `bijux.benchmark.fastq.deplete_reference_contaminants.bundle.v1` |
| `run-2029` | `fastq.deplete_rrna` | `fastq-depletion` | `sortmerna` | `active` | `bijux.benchmark.fastq.deplete_rrna.bundle.v1` |
| `run-2030` | `fastq.detect_adapters` | `fastq-observational-qc` | `fastqc` | `active` | `bijux.benchmark.fastq.detect_adapters.bundle.v1` |
| `run-2031` | `fastq.detect_duplicates_premerge` | `fastq-observational-qc` | `bijux_dna` | `active` | `bijux.benchmark.fastq.detect_duplicates_premerge.bundle.v1` |
| `run-2032` | `fastq.estimate_library_complexity_prealign` | `fastq-observational-qc` | `bijux_dna` | `active` | `bijux.benchmark.fastq.estimate_library_complexity_prealign.bundle.v1` |
| `run-2033` | `fastq.extract_umis` | `fastq-read-transformation` | `umi_tools` | `active` | `bijux.benchmark.fastq.extract_umis.bundle.v1` |
| `run-2034` | `fastq.filter_low_complexity` | `fastq-read-transformation` | `bbduk`, `fastp`, `prinseq` | `active` | `bijux.benchmark.fastq.filter_low_complexity.bundle.v1` |
| `run-2035` | `fastq.filter_reads` | `fastq-read-transformation` | `bbduk`, `fastp`, `prinseq`, `seqkit` | `active` | `bijux.benchmark.fastq.filter_reads.bundle.v1` |
| `run-2036` | `fastq.index_reference` | `reference-index-build` | `bowtie2_build`, `star` | `active` | `bijux.benchmark.fastq.index_reference.bundle.v1` |
| `run-2037` | `fastq.infer_asvs` | `fastq-amplicon` | `dada2` | `active` | `bijux.benchmark.fastq.infer_asvs.bundle.v1` |
| `run-2038` | `fastq.merge_pairs` | `fastq-read-transformation` | `adapterremoval`, `bbmerge`, `flash2`, `leehom`, `pear`, `seqprep`, `vsearch` | `active` | `bijux.benchmark.fastq.merge_pairs.bundle.v1` |
| `run-2039` | `fastq.normalize_abundance` | `fastq-amplicon` | `seqkit` | `active` | `bijux.benchmark.fastq.normalize_abundance.bundle.v1` |
| `run-2040` | `fastq.normalize_primers` | `fastq-read-transformation` | `cutadapt` | `active` | `bijux.benchmark.fastq.normalize_primers.bundle.v1` |
| `run-2041` | `fastq.profile_overrepresented_sequences` | `fastq-observational-qc` | `fastq_scan`, `fastqc`, `seqkit` | `active` | `bijux.benchmark.fastq.profile_overrepresented_sequences.bundle.v1` |
| `run-2042` | `fastq.profile_read_lengths` | `fastq-observational-qc` | `seqfu`, `seqkit` | `active` | `bijux.benchmark.fastq.profile_read_lengths.bundle.v1` |
| `run-2043` | `fastq.profile_reads` | `fastq-observational-qc` | `seqfu`, `seqkit`, `seqkit_stats` | `active` | `bijux.benchmark.fastq.profile_reads.bundle.v1` |
| `run-2044` | `fastq.remove_chimeras` | `fastq-amplicon` | `vsearch` | `active` | `bijux.benchmark.fastq.remove_chimeras.bundle.v1` |
| `run-2045` | `fastq.remove_duplicates` | `fastq-read-transformation` | `clumpify`, `fastuniq` | `active` | `bijux.benchmark.fastq.remove_duplicates.bundle.v1` |
| `run-2046` | `fastq.report_qc` | `fastq-observational-qc` | `multiqc` | `active` | `bijux.benchmark.fastq.report_qc.bundle.v1` |
| `run-2047` | `fastq.screen_taxonomy` | `fastq-taxonomy` | `centrifuge`, `kaiju`, `kraken2`, `krakenuniq` | `active` | `bijux.benchmark.fastq.screen_taxonomy.bundle.v1` |
| `run-2048` | `fastq.trim_polyg_tails` | `fastq-read-transformation` | `bbduk`, `fastp` | `active` | `bijux.benchmark.fastq.trim_polyg_tails.bundle.v1` |
| `run-2049` | `fastq.trim_reads` | `fastq-read-transformation` | `adapterremoval`, `alientrimmer`, `atropos`, `bbduk`, `cutadapt`, `fastp`, `fastx_clipper`, `leehom`, `prinseq`, `seqkit`, `seqprep`, `skewer`, `trim_galore`, `trimmomatic` | `active` | `bijux.benchmark.fastq.trim_reads.bundle.v1` |
| `run-2050` | `fastq.trim_terminal_damage` | `fastq-read-transformation` | `adapterremoval`, `cutadapt`, `seqkit` | `active` | `bijux.benchmark.fastq.trim_terminal_damage.bundle.v1` |
| `run-2051` | `fastq.validate_reads` | `fastq-observational-qc` | `fastq_scan`, `fastqc`, `fastqvalidator`, `fqtools`, `seqtk` | `active` | `bijux.benchmark.fastq.validate_reads.bundle.v1` |
| `run-2052` | `vcf.admixture` | `vcf-population-inference` | `plink`, `plink2` | `active` | `bijux.benchmark.vcf.admixture.bundle.v1` |
| `run-2053` | `vcf.call` | `vcf-calling` | `bcftools` | `active` | `bijux.benchmark.vcf.call.bundle.v1` |
| `run-2054` | `vcf.call_diploid` | `vcf-calling` | `bcftools` | `active` | `bijux.benchmark.vcf.call_diploid.bundle.v1` |
| `run-2055` | `vcf.call_gl` | `vcf-calling` | `angsd`, `bcftools` | `active` | `bijux.benchmark.vcf.call_gl.bundle.v1` |
| `run-2056` | `vcf.call_pseudohaploid` | `vcf-calling` | `angsd`, `bcftools` | `active` | `bijux.benchmark.vcf.call_pseudohaploid.bundle.v1` |
| `run-2057` | `vcf.damage_filter` | `vcf-transformation` | `bcftools` | `active` | `bijux.benchmark.vcf.damage_filter.bundle.v1` |
| `run-2058` | `vcf.demography` | `vcf-population-inference` | `ibdne` | `active` | `bijux.benchmark.vcf.demography.bundle.v1` |
| `run-2059` | `vcf.filter` | `vcf-transformation` | `bcftools` | `active` | `bijux.benchmark.vcf.filter.bundle.v1` |
| `run-2060` | `vcf.gl_propagation` | `vcf-transformation` | `bcftools` | `active` | `bijux.benchmark.vcf.gl_propagation.bundle.v1` |
| `run-2061` | `vcf.ibd` | `vcf-population-inference` | `germline`, `ibdhap`, `ibdseq` | `active` | `bijux.benchmark.vcf.ibd.bundle.v1` |
| `run-2062` | `vcf.imputation_metrics` | `vcf-phasing-imputation` | `beagle`, `glimpse`, `impute5`, `minimac4` | `active` | `bijux.benchmark.vcf.imputation_metrics.bundle.v1` |
| `run-2063` | `vcf.impute` | `vcf-phasing-imputation` | `beagle`, `glimpse`, `impute5`, `minimac4` | `active` | `bijux.benchmark.vcf.impute.bundle.v1` |
| `run-2064` | `vcf.pca` | `vcf-population-inference` | `eigensoft`, `plink2` | `active` | `bijux.benchmark.vcf.pca.bundle.v1` |
| `run-2065` | `vcf.phasing` | `vcf-phasing-imputation` | `beagle`, `eagle`, `shapeit`, `shapeit5` | `active` | `bijux.benchmark.vcf.phasing.bundle.v1` |
| `run-2066` | `vcf.population_structure` | `vcf-population-inference` | `eigensoft`, `plink`, `plink2` | `active` | `bijux.benchmark.vcf.population_structure.bundle.v1` |
| `run-2067` | `vcf.postprocess` | `vcf-transformation` | `bcftools` | `active` | `bijux.benchmark.vcf.postprocess.bundle.v1` |
| `run-2068` | `vcf.prepare_reference_panel` | `vcf-transformation` | `bcftools` | `active` | `bijux.benchmark.vcf.prepare_reference_panel.bundle.v1` |
| `run-2069` | `vcf.qc` | `vcf-transformation` | `bcftools`, `plink`, `plink2` | `active` | `bijux.benchmark.vcf.qc.bundle.v1` |
| `run-2070` | `vcf.roh` | `vcf-population-inference` | `plink2` | `active` | `bijux.benchmark.vcf.roh.bundle.v1` |
| `run-2071` | `vcf.stats` | `vcf-transformation` | `bcftools` | `active` | `bijux.benchmark.vcf.stats.bundle.v1` |
