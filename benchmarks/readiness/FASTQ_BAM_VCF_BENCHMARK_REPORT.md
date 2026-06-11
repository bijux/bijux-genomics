# FASTQ + BAM + VCF Benchmark Report

- Report rows: 126
- Expected-result rows: 125
- Explicit unsupported rows: 1
- Present rows: 122
- Missing-result rows: 3
- Unsupported-pair rows: 1
- Failure rows: 125
- Comparable metric rows: 103

## Stage-Centric

| Domain | Stage | Rows | Present | Missing | Unsupported | Tools | Pipelines |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| bam | bam.align | 2 | 2 | 0 | 0 | bowtie2, bwa | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf, core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf |
| bam | bam.authenticity | 3 | 3 | 0 | 0 | authenticct, damageprofiler, pmdtools | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf |
| bam | bam.bias_mitigation | 1 | 1 | 0 | 0 | mapdamage2 |  |
| bam | bam.complexity | 1 | 1 | 0 | 0 | preseq | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf |
| bam | bam.contamination | 3 | 3 | 0 | 0 | contammix, schmutzi, verifybamid2 | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf |
| bam | bam.coverage | 3 | 2 | 1 | 0 | bedtools, mosdepth, samtools | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf, bam-genotyping-to-vcf-downstream, core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf |
| bam | bam.damage | 6 | 6 | 0 | 0 | addeam, damageprofiler, mapdamage2, ngsbriggs, pmdtools, pydamage | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf |
| bam | bam.duplication_metrics | 2 | 2 | 0 | 0 | picard, samtools |  |
| bam | bam.endogenous_content | 1 | 1 | 0 | 0 | samtools |  |
| bam | bam.filter | 3 | 3 | 0 | 0 | bamtools, bedtools, samtools | bam-genotyping-to-vcf-downstream, diploid-small-fastq-bam-vcf |
| bam | bam.gc_bias | 1 | 1 | 0 | 0 | picard |  |
| bam | bam.genotyping | 1 | 1 | 0 | 0 | angsd |  |
| bam | bam.haplogroups | 1 | 1 | 0 | 0 | yleaf |  |
| bam | bam.insert_size | 1 | 1 | 0 | 0 | picard |  |
| bam | bam.kinship | 2 | 2 | 0 | 0 | angsd, king |  |
| bam | bam.length_filter | 2 | 2 | 0 | 0 | picard, samtools |  |
| bam | bam.mapping_summary | 2 | 2 | 0 | 0 | picard, samtools | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf, diploid-small-fastq-bam-vcf |
| bam | bam.mapq_filter | 2 | 2 | 0 | 0 | bamtools, samtools |  |
| bam | bam.markdup | 2 | 2 | 0 | 0 | picard, samtools |  |
| bam | bam.overlap_correction | 1 | 1 | 0 | 0 | bamutil |  |
| bam | bam.qc_pre | 2 | 2 | 0 | 0 | multiqc, samtools | core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf |
| bam | bam.recalibration | 1 | 1 | 0 | 0 | gatk | bam-genotyping-to-vcf-downstream, diploid-small-fastq-bam-vcf |
| bam | bam.sex | 3 | 3 | 0 | 0 | angsd, rxy, yleaf |  |
| bam | bam.validate | 3 | 3 | 0 | 0 | bamtools, bedtools, samtools | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf, core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf |
| fastq | fastq.cluster_otus | 1 | 1 | 0 | 0 | vsearch | amplicon-asv-otu-no-vcf |
| fastq | fastq.correct_errors | 4 | 4 | 0 | 0 | bayeshammer, lighter, musket, rcorrector |  |
| fastq | fastq.deplete_host | 1 | 1 | 0 | 0 | bowtie2 |  |
| fastq | fastq.deplete_reference_contaminants | 1 | 1 | 0 | 0 | bowtie2 |  |
| fastq | fastq.deplete_rrna | 1 | 1 | 0 | 0 | sortmerna |  |
| fastq | fastq.detect_adapters | 1 | 1 | 0 | 0 | fastqc | edna-taxonomy-no-vcf |
| fastq | fastq.detect_duplicates_premerge | 1 | 1 | 0 | 0 | bijux_dna |  |
| fastq | fastq.extract_umis | 1 | 1 | 0 | 0 | umi_tools |  |
| fastq | fastq.filter_low_complexity | 2 | 2 | 0 | 0 | bbduk, prinseq |  |
| fastq | fastq.filter_reads | 4 | 4 | 0 | 0 | bbduk, fastp, prinseq, seqkit | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf, core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf, edna-taxonomy-no-vcf |
| fastq | fastq.infer_asvs | 1 | 1 | 0 | 0 | dada2 | amplicon-asv-otu-no-vcf |
| fastq | fastq.merge_pairs | 6 | 6 | 0 | 0 | adapterremoval, bbmerge, flash2, leehom, pear, vsearch |  |
| fastq | fastq.normalize_abundance | 1 | 1 | 0 | 0 | seqkit | amplicon-asv-otu-no-vcf |
| fastq | fastq.normalize_primers | 1 | 1 | 0 | 0 | cutadapt | amplicon-asv-otu-no-vcf |
| fastq | fastq.profile_read_lengths | 4 | 4 | 0 | 0 | fastp, prinseq, seqfu, seqkit_stats |  |
| fastq | fastq.profile_reads | 3 | 3 | 0 | 0 | seqfu, seqkit, seqkit_stats | core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf |
| fastq | fastq.remove_chimeras | 1 | 1 | 0 | 0 | vsearch | amplicon-asv-otu-no-vcf |
| fastq | fastq.remove_duplicates | 2 | 2 | 0 | 0 | clumpify, fastuniq | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf |
| fastq | fastq.screen_taxonomy | 4 | 3 | 1 | 0 | centrifuge, kaiju, kraken2, krakenuniq | edna-taxonomy-no-vcf |
| fastq | fastq.trim_polyg_tails | 2 | 2 | 0 | 0 | bbduk, fastp |  |
| fastq | fastq.trim_reads | 13 | 13 | 0 | 0 | adapterremoval, alientrimmer, atropos, bbduk, cutadapt, fastp, fastx_clipper, leehom, prinseq, seqkit, skewer, trim_galore, trimmomatic | core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf, edna-taxonomy-no-vcf |
| fastq | fastq.trim_terminal_damage | 3 | 3 | 0 | 0 | adapterremoval, cutadapt, seqkit | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf |
| fastq | fastq.validate_reads | 5 | 5 | 0 | 0 | fastq_scan, fastqc, fastqvalidator, fqtools, seqtk | adna-gl-fastq-bam-vcf, adna-pseudohaploid-fastq-bam-vcf, amplicon-asv-otu-no-vcf, core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf, edna-taxonomy-no-vcf |
| vcf | vcf.call | 1 | 1 | 0 | 0 | bcftools | core-germline-fastq-bam-vcf |
| vcf | vcf.call_diploid | 1 | 1 | 0 | 0 | bcftools | diploid-small-fastq-bam-vcf |
| vcf | vcf.call_gl | 1 | 1 | 0 | 0 | bcftools | adna-gl-fastq-bam-vcf |
| vcf | vcf.call_pseudohaploid | 1 | 1 | 0 | 0 | bcftools | adna-pseudohaploid-fastq-bam-vcf |
| vcf | vcf.damage_filter | 1 | 1 | 0 | 0 | bcftools | adna-pseudohaploid-fastq-bam-vcf |
| vcf | vcf.filter | 2 | 1 | 0 | 1 | bcftools, samtools | bam-genotyping-to-vcf-downstream, core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf |
| vcf | vcf.gl_propagation | 1 | 1 | 0 | 0 | bcftools | adna-gl-fastq-bam-vcf |
| vcf | vcf.postprocess | 1 | 1 | 0 | 0 | bcftools |  |
| vcf | vcf.prepare_reference_panel | 1 | 1 | 0 | 0 | bcftools | reference-panel-imputation |
| vcf | vcf.qc | 3 | 3 | 0 | 0 | bcftools, plink, plink2 | adna-gl-fastq-bam-vcf, bam-genotyping-to-vcf-downstream, core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf, popgen-structure-vcf, reference-panel-imputation, relatedness-segments-vcf |
| vcf | vcf.stats | 1 | 0 | 1 | 0 | bcftools | adna-pseudohaploid-fastq-bam-vcf, bam-genotyping-to-vcf-downstream, core-germline-fastq-bam-vcf, diploid-small-fastq-bam-vcf |

## Tool-Centric

| Tool | Rows | Present | Missing | Unsupported | Domains | Stages |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| adapterremoval | 3 | 3 | 0 | 0 | fastq | fastq.merge_pairs, fastq.trim_reads, fastq.trim_terminal_damage |
| addeam | 1 | 1 | 0 | 0 | bam | bam.damage |
| alientrimmer | 1 | 1 | 0 | 0 | fastq | fastq.trim_reads |
| angsd | 3 | 3 | 0 | 0 | bam | bam.genotyping, bam.kinship, bam.sex |
| atropos | 1 | 1 | 0 | 0 | fastq | fastq.trim_reads |
| authenticct | 1 | 1 | 0 | 0 | bam | bam.authenticity |
| bamtools | 3 | 3 | 0 | 0 | bam | bam.filter, bam.mapq_filter, bam.validate |
| bamutil | 1 | 1 | 0 | 0 | bam | bam.overlap_correction |
| bayeshammer | 1 | 1 | 0 | 0 | fastq | fastq.correct_errors |
| bbduk | 4 | 4 | 0 | 0 | fastq | fastq.filter_low_complexity, fastq.filter_reads, fastq.trim_polyg_tails, fastq.trim_reads |
| bbmerge | 1 | 1 | 0 | 0 | fastq | fastq.merge_pairs |
| bcftools | 11 | 10 | 1 | 0 | vcf | vcf.call, vcf.call_diploid, vcf.call_gl, vcf.call_pseudohaploid, vcf.damage_filter, vcf.filter, vcf.gl_propagation, vcf.postprocess, vcf.prepare_reference_panel, vcf.qc, vcf.stats |
| bedtools | 3 | 3 | 0 | 0 | bam | bam.coverage, bam.filter, bam.validate |
| bijux_dna | 1 | 1 | 0 | 0 | fastq | fastq.detect_duplicates_premerge |
| bowtie2 | 3 | 3 | 0 | 0 | bam, fastq | bam.align, fastq.deplete_host, fastq.deplete_reference_contaminants |
| bwa | 1 | 1 | 0 | 0 | bam | bam.align |
| centrifuge | 1 | 1 | 0 | 0 | fastq | fastq.screen_taxonomy |
| clumpify | 1 | 1 | 0 | 0 | fastq | fastq.remove_duplicates |
| contammix | 1 | 1 | 0 | 0 | bam | bam.contamination |
| cutadapt | 3 | 3 | 0 | 0 | fastq | fastq.normalize_primers, fastq.trim_reads, fastq.trim_terminal_damage |
| dada2 | 1 | 1 | 0 | 0 | fastq | fastq.infer_asvs |
| damageprofiler | 2 | 2 | 0 | 0 | bam | bam.authenticity, bam.damage |
| fastp | 4 | 4 | 0 | 0 | fastq | fastq.filter_reads, fastq.profile_read_lengths, fastq.trim_polyg_tails, fastq.trim_reads |
| fastq_scan | 1 | 1 | 0 | 0 | fastq | fastq.validate_reads |
| fastqc | 2 | 2 | 0 | 0 | fastq | fastq.detect_adapters, fastq.validate_reads |
| fastqvalidator | 1 | 1 | 0 | 0 | fastq | fastq.validate_reads |
| fastuniq | 1 | 1 | 0 | 0 | fastq | fastq.remove_duplicates |
| fastx_clipper | 1 | 1 | 0 | 0 | fastq | fastq.trim_reads |
| flash2 | 1 | 1 | 0 | 0 | fastq | fastq.merge_pairs |
| fqtools | 1 | 1 | 0 | 0 | fastq | fastq.validate_reads |
| gatk | 1 | 1 | 0 | 0 | bam | bam.recalibration |
| kaiju | 1 | 1 | 0 | 0 | fastq | fastq.screen_taxonomy |
| king | 1 | 1 | 0 | 0 | bam | bam.kinship |
| kraken2 | 1 | 0 | 1 | 0 | fastq | fastq.screen_taxonomy |
| krakenuniq | 1 | 1 | 0 | 0 | fastq | fastq.screen_taxonomy |
| leehom | 2 | 2 | 0 | 0 | fastq | fastq.merge_pairs, fastq.trim_reads |
| lighter | 1 | 1 | 0 | 0 | fastq | fastq.correct_errors |
| mapdamage2 | 2 | 2 | 0 | 0 | bam | bam.bias_mitigation, bam.damage |
| mosdepth | 1 | 1 | 0 | 0 | bam | bam.coverage |
| multiqc | 1 | 1 | 0 | 0 | bam | bam.qc_pre |
| musket | 1 | 1 | 0 | 0 | fastq | fastq.correct_errors |
| ngsbriggs | 1 | 1 | 0 | 0 | bam | bam.damage |
| pear | 1 | 1 | 0 | 0 | fastq | fastq.merge_pairs |
| picard | 6 | 6 | 0 | 0 | bam | bam.duplication_metrics, bam.gc_bias, bam.insert_size, bam.length_filter, bam.mapping_summary, bam.markdup |
| plink | 1 | 1 | 0 | 0 | vcf | vcf.qc |
| plink2 | 1 | 1 | 0 | 0 | vcf | vcf.qc |
| pmdtools | 2 | 2 | 0 | 0 | bam | bam.authenticity, bam.damage |
| preseq | 1 | 1 | 0 | 0 | bam | bam.complexity |
| prinseq | 4 | 4 | 0 | 0 | fastq | fastq.filter_low_complexity, fastq.filter_reads, fastq.profile_read_lengths, fastq.trim_reads |
| pydamage | 1 | 1 | 0 | 0 | bam | bam.damage |
| rcorrector | 1 | 1 | 0 | 0 | fastq | fastq.correct_errors |
| rxy | 1 | 1 | 0 | 0 | bam | bam.sex |
| samtools | 11 | 9 | 1 | 1 | bam, vcf | bam.coverage, bam.duplication_metrics, bam.endogenous_content, bam.filter, bam.length_filter, bam.mapping_summary, bam.mapq_filter, bam.markdup, bam.qc_pre, bam.validate, vcf.filter |
| schmutzi | 1 | 1 | 0 | 0 | bam | bam.contamination |
| seqfu | 2 | 2 | 0 | 0 | fastq | fastq.profile_read_lengths, fastq.profile_reads |
| seqkit | 5 | 5 | 0 | 0 | fastq | fastq.filter_reads, fastq.normalize_abundance, fastq.profile_reads, fastq.trim_reads, fastq.trim_terminal_damage |
| seqkit_stats | 2 | 2 | 0 | 0 | fastq | fastq.profile_read_lengths, fastq.profile_reads |
| seqtk | 1 | 1 | 0 | 0 | fastq | fastq.validate_reads |
| skewer | 1 | 1 | 0 | 0 | fastq | fastq.trim_reads |
| sortmerna | 1 | 1 | 0 | 0 | fastq | fastq.deplete_rrna |
| trim_galore | 1 | 1 | 0 | 0 | fastq | fastq.trim_reads |
| trimmomatic | 1 | 1 | 0 | 0 | fastq | fastq.trim_reads |
| umi_tools | 1 | 1 | 0 | 0 | fastq | fastq.extract_umis |
| verifybamid2 | 1 | 1 | 0 | 0 | bam | bam.contamination |
| vsearch | 3 | 3 | 0 | 0 | fastq | fastq.cluster_otus, fastq.merge_pairs, fastq.remove_chimeras |
| yleaf | 2 | 2 | 0 | 0 | bam | bam.haplogroups, bam.sex |

## Corpus-Centric

| Corpus | Rows | Present | Missing | Unsupported | Domains | Stages |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| corpus-01-adna-bam-mini | 7 | 7 | 0 | 0 | bam | bam.contamination, bam.haplogroups, bam.sex |
| corpus-01-adna-damage-mini | 9 | 9 | 0 | 0 | bam | bam.authenticity, bam.damage |
| corpus-01-bam-mini | 28 | 27 | 1 | 0 | bam | bam.bias_mitigation, bam.complexity, bam.coverage, bam.duplication_metrics, bam.endogenous_content, bam.filter, bam.gc_bias, bam.insert_size, bam.length_filter, bam.mapping_summary, bam.mapq_filter, bam.markdup, bam.overlap_correction, bam.qc_pre, bam.recalibration, bam.validate |
| corpus-01-genotyping-mini | 1 | 1 | 0 | 0 | bam | bam.genotyping |
| corpus-01-kinship-mini | 2 | 2 | 0 | 0 | bam | bam.kinship |
| corpus-01-mini | 56 | 56 | 0 | 0 | bam, fastq | bam.align, fastq.correct_errors, fastq.deplete_host, fastq.deplete_reference_contaminants, fastq.deplete_rrna, fastq.detect_adapters, fastq.detect_duplicates_premerge, fastq.extract_umis, fastq.filter_low_complexity, fastq.filter_reads, fastq.merge_pairs, fastq.profile_read_lengths, fastq.profile_reads, fastq.remove_duplicates, fastq.trim_polyg_tails, fastq.trim_reads, fastq.trim_terminal_damage, fastq.validate_reads |
| corpus-02-edna-mini | 4 | 3 | 1 | 0 | fastq | fastq.screen_taxonomy |
| corpus-03-amplicon-mini | 5 | 5 | 0 | 0 | fastq | fastq.cluster_otus, fastq.infer_asvs, fastq.normalize_abundance, fastq.normalize_primers, fastq.remove_chimeras |
| not_applicable | 1 | 0 | 0 | 1 | vcf | vcf.filter |
| vcf_production_regression | 13 | 12 | 1 | 0 | vcf | vcf.call, vcf.call_diploid, vcf.call_gl, vcf.call_pseudohaploid, vcf.damage_filter, vcf.filter, vcf.gl_propagation, vcf.postprocess, vcf.prepare_reference_panel, vcf.qc, vcf.stats |

## Pipeline-Centric

- Pipeline rows: 10
- Unmapped report rows: 92

| Pipeline | Rows | Present | Missing | Unsupported | Domains | Stages |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| adna-gl-fastq-bam-vcf | 15 | 14 | 1 | 0 | bam, fastq, vcf | bam.align, bam.authenticity, bam.complexity, bam.contamination, bam.coverage, bam.damage, bam.mapping_summary, bam.validate, fastq.filter_reads, fastq.remove_duplicates, fastq.trim_terminal_damage, fastq.validate_reads, vcf.call_gl, vcf.gl_propagation, vcf.qc |
| adna-pseudohaploid-fastq-bam-vcf | 15 | 13 | 2 | 0 | bam, fastq, vcf | bam.align, bam.authenticity, bam.complexity, bam.contamination, bam.coverage, bam.damage, bam.mapping_summary, bam.validate, fastq.filter_reads, fastq.remove_duplicates, fastq.trim_terminal_damage, fastq.validate_reads, vcf.call_pseudohaploid, vcf.damage_filter, vcf.stats |
| amplicon-asv-otu-no-vcf | 6 | 6 | 0 | 0 | fastq | fastq.cluster_otus, fastq.infer_asvs, fastq.normalize_abundance, fastq.normalize_primers, fastq.remove_chimeras, fastq.validate_reads |
| bam-genotyping-to-vcf-downstream | 6 | 4 | 2 | 0 | bam, vcf | bam.coverage, bam.filter, bam.recalibration, vcf.filter, vcf.qc, vcf.stats |
| core-germline-fastq-bam-vcf | 12 | 10 | 2 | 0 | bam, fastq, vcf | bam.align, bam.coverage, bam.qc_pre, bam.validate, fastq.filter_reads, fastq.profile_reads, fastq.trim_reads, fastq.validate_reads, vcf.call, vcf.filter, vcf.qc, vcf.stats |
| diploid-small-fastq-bam-vcf | 15 | 13 | 2 | 0 | bam, fastq, vcf | bam.align, bam.coverage, bam.filter, bam.mapping_summary, bam.qc_pre, bam.recalibration, bam.validate, fastq.filter_reads, fastq.profile_reads, fastq.trim_reads, fastq.validate_reads, vcf.call_diploid, vcf.filter, vcf.qc, vcf.stats |
| edna-taxonomy-no-vcf | 5 | 4 | 1 | 0 | fastq | fastq.detect_adapters, fastq.filter_reads, fastq.screen_taxonomy, fastq.trim_reads, fastq.validate_reads |
| popgen-structure-vcf | 1 | 1 | 0 | 0 | vcf | vcf.qc |
| reference-panel-imputation | 2 | 2 | 0 | 0 | vcf | vcf.prepare_reference_panel, vcf.qc |
| relatedness-segments-vcf | 1 | 1 | 0 | 0 | vcf | vcf.qc |

## Runtime

| Report Row | Domain | Stage | Tool | Status | Simulated Elapsed Seconds | Real-Smoke Elapsed Seconds | Source |
| --- | --- | --- | --- | --- | ---: | ---: | --- |
| bam:corpus-01-mini:bam.align:sample-set:bowtie2 | bam | bam.align | bowtie2 | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-mini:bam.align:sample-set:bwa | bam | bam.align | bwa | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:authenticct | bam | bam.authenticity | authenticct | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:damageprofiler | bam | bam.authenticity | damageprofiler | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:pmdtools | bam | bam.authenticity | pmdtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.bias_mitigation:sample-set:mapdamage2 | bam | bam.bias_mitigation | mapdamage2 | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.complexity:sample-set:preseq | bam | bam.complexity | preseq | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:contammix | bam | bam.contamination | contammix | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:schmutzi | bam | bam.contamination | schmutzi | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:verifybamid2 | bam | bam.contamination | verifybamid2 | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.coverage:sample-set:bedtools | bam | bam.coverage | bedtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.coverage:sample-set:mosdepth | bam | bam.coverage | mosdepth | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools | bam | bam.coverage | samtools | missing_result | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:addeam | bam | bam.damage | addeam | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:damageprofiler | bam | bam.damage | damageprofiler | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:mapdamage2 | bam | bam.damage | mapdamage2 | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:ngsbriggs | bam | bam.damage | ngsbriggs | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:pmdtools | bam | bam.damage | pmdtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:pydamage | bam | bam.damage | pydamage | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.duplication_metrics:sample-set:picard | bam | bam.duplication_metrics | picard | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.duplication_metrics:sample-set:samtools | bam | bam.duplication_metrics | samtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.endogenous_content:sample-set:samtools | bam | bam.endogenous_content | samtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.filter:sample-set:bamtools | bam | bam.filter | bamtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.filter:sample-set:bedtools | bam | bam.filter | bedtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.filter:sample-set:samtools | bam | bam.filter | samtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.gc_bias:sample-set:picard | bam | bam.gc_bias | picard | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-genotyping-mini:bam.genotyping:human_like_genotyping_candidate_panel:angsd | bam | bam.genotyping | angsd | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-bam-mini:bam.haplogroups:sample-set:yleaf | bam | bam.haplogroups | yleaf | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.insert_size:sample-set:picard | bam | bam.insert_size | picard | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-kinship-mini:bam.kinship:sample-set:angsd | bam | bam.kinship | angsd | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-kinship-mini:bam.kinship:sample-set:king | bam | bam.kinship | king | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.length_filter:sample-set:picard | bam | bam.length_filter | picard | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.length_filter:sample-set:samtools | bam | bam.length_filter | samtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.mapping_summary:sample-set:picard | bam | bam.mapping_summary | picard | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.mapping_summary:sample-set:samtools | bam | bam.mapping_summary | samtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.mapq_filter:sample-set:bamtools | bam | bam.mapq_filter | bamtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.mapq_filter:sample-set:samtools | bam | bam.mapq_filter | samtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.markdup:sample-set:picard | bam | bam.markdup | picard | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.markdup:sample-set:samtools | bam | bam.markdup | samtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.overlap_correction:sample-set:bamutil | bam | bam.overlap_correction | bamutil | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc | bam | bam.qc_pre | multiqc | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.qc_pre:sample-set:samtools | bam | bam.qc_pre | samtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.recalibration:sample-set:gatk | bam | bam.recalibration | gatk | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-bam-mini:bam.sex:sample-set:angsd | bam | bam.sex | angsd | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-bam-mini:bam.sex:sample-set:rxy | bam | bam.sex | rxy | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-adna-bam-mini:bam.sex:sample-set:yleaf | bam | bam.sex | yleaf | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.validate:sample-set:bamtools | bam | bam.validate | bamtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.validate:sample-set:bedtools | bam | bam.validate | bedtools | present | 1.250 |  | fake_run_simulated |
| bam:corpus-01-bam-mini:bam.validate:sample-set:samtools | bam | bam.validate | samtools | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-03-amplicon-mini:fastq.cluster_otus:sample-set:vsearch | fastq | fastq.cluster_otus | vsearch | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.correct_errors:sample-set:bayeshammer | fastq | fastq.correct_errors | bayeshammer | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.correct_errors:sample-set:lighter | fastq | fastq.correct_errors | lighter | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.correct_errors:sample-set:musket | fastq | fastq.correct_errors | musket | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.correct_errors:sample-set:rcorrector | fastq | fastq.correct_errors | rcorrector | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.deplete_host:sample-set:bowtie2 | fastq | fastq.deplete_host | bowtie2 | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.deplete_reference_contaminants:sample-set:bowtie2 | fastq | fastq.deplete_reference_contaminants | bowtie2 | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.deplete_rrna:sample-set:sortmerna | fastq | fastq.deplete_rrna | sortmerna | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.detect_adapters:sample-set:fastqc | fastq | fastq.detect_adapters | fastqc | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.detect_duplicates_premerge:sample-set:bijux_dna | fastq | fastq.detect_duplicates_premerge | bijux_dna | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.extract_umis:sample-set:umi_tools | fastq | fastq.extract_umis | umi_tools | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.filter_low_complexity:sample-set:bbduk | fastq | fastq.filter_low_complexity | bbduk | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.filter_low_complexity:sample-set:prinseq | fastq | fastq.filter_low_complexity | prinseq | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.filter_reads:sample-set:bbduk | fastq | fastq.filter_reads | bbduk | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.filter_reads:sample-set:fastp | fastq | fastq.filter_reads | fastp | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.filter_reads:sample-set:prinseq | fastq | fastq.filter_reads | prinseq | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.filter_reads:sample-set:seqkit | fastq | fastq.filter_reads | seqkit | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-03-amplicon-mini:fastq.infer_asvs:sample-set:dada2 | fastq | fastq.infer_asvs | dada2 | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:adapterremoval | fastq | fastq.merge_pairs | adapterremoval | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:bbmerge | fastq | fastq.merge_pairs | bbmerge | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:flash2 | fastq | fastq.merge_pairs | flash2 | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:leehom | fastq | fastq.merge_pairs | leehom | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:pear | fastq | fastq.merge_pairs | pear | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:vsearch | fastq | fastq.merge_pairs | vsearch | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-03-amplicon-mini:fastq.normalize_abundance:sample-set:seqkit | fastq | fastq.normalize_abundance | seqkit | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-03-amplicon-mini:fastq.normalize_primers:sample-set:cutadapt | fastq | fastq.normalize_primers | cutadapt | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:fastp | fastq | fastq.profile_read_lengths | fastp | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:prinseq | fastq | fastq.profile_read_lengths | prinseq | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:seqfu | fastq | fastq.profile_read_lengths | seqfu | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:seqkit_stats | fastq | fastq.profile_read_lengths | seqkit_stats | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqfu | fastq | fastq.profile_reads | seqfu | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit | fastq | fastq.profile_reads | seqkit | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats | fastq | fastq.profile_reads | seqkit_stats | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-03-amplicon-mini:fastq.remove_chimeras:sample-set:vsearch | fastq | fastq.remove_chimeras | vsearch | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.remove_duplicates:sample-set:clumpify | fastq | fastq.remove_duplicates | clumpify | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.remove_duplicates:sample-set:fastuniq | fastq | fastq.remove_duplicates | fastuniq | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:centrifuge | fastq | fastq.screen_taxonomy | centrifuge | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kaiju | fastq | fastq.screen_taxonomy | kaiju | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2 | fastq | fastq.screen_taxonomy | kraken2 | missing_result | 1.250 |  | fake_run_simulated |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:krakenuniq | fastq | fastq.screen_taxonomy | krakenuniq | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_polyg_tails:sample-set:bbduk | fastq | fastq.trim_polyg_tails | bbduk | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_polyg_tails:sample-set:fastp | fastq | fastq.trim_polyg_tails | fastp | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:adapterremoval | fastq | fastq.trim_reads | adapterremoval | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:alientrimmer | fastq | fastq.trim_reads | alientrimmer | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:atropos | fastq | fastq.trim_reads | atropos | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:bbduk | fastq | fastq.trim_reads | bbduk | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:cutadapt | fastq | fastq.trim_reads | cutadapt | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:fastp | fastq | fastq.trim_reads | fastp | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:fastx_clipper | fastq | fastq.trim_reads | fastx_clipper | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:leehom | fastq | fastq.trim_reads | leehom | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:prinseq | fastq | fastq.trim_reads | prinseq | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:seqkit | fastq | fastq.trim_reads | seqkit | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:skewer | fastq | fastq.trim_reads | skewer | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:trim_galore | fastq | fastq.trim_reads | trim_galore | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:trimmomatic | fastq | fastq.trim_reads | trimmomatic | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:adapterremoval | fastq | fastq.trim_terminal_damage | adapterremoval | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:cutadapt | fastq | fastq.trim_terminal_damage | cutadapt | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:seqkit | fastq | fastq.trim_terminal_damage | seqkit | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastq_scan | fastq | fastq.validate_reads | fastq_scan | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqc | fastq | fastq.validate_reads | fastqc | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqvalidator | fastq | fastq.validate_reads | fastqvalidator | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:fqtools | fastq | fastq.validate_reads | fqtools | present | 1.250 |  | fake_run_simulated |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:seqtk | fastq | fastq.validate_reads | seqtk | present | 1.250 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools | vcf | vcf.call | bcftools | present | 1.750 | 0.159 | fake_run_and_real_smoke |
| vcf:vcf_production_regression:vcf.call_diploid:bam_bundle:bcftools | vcf | vcf.call_diploid | bcftools | present | 1.750 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.call_gl:bam_bundle:bcftools | vcf | vcf.call_gl | bcftools | present | 1.750 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.call_pseudohaploid:bam_bundle:bcftools | vcf | vcf.call_pseudohaploid | bcftools | present | 1.750 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.damage_filter:vcf_single_sample:bcftools | vcf | vcf.damage_filter | bcftools | present | 1.500 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.filter:vcf_single_sample:bcftools | vcf | vcf.filter | bcftools | present | 1.500 |  | fake_run_simulated |
| unsupported:vcf:vcf.filter:samtools | vcf | vcf.filter | samtools | unsupported_pair |  |  | not_applicable |
| vcf:vcf_production_regression:vcf.gl_propagation:vcf_single_sample:bcftools | vcf | vcf.gl_propagation | bcftools | present | 1.500 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools | vcf | vcf.postprocess | bcftools | present | 1.500 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.prepare_reference_panel:vcf_reference_panel:bcftools | vcf | vcf.prepare_reference_panel | bcftools | present | 1.500 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.qc:vcf_cohort:bcftools | vcf | vcf.qc | bcftools | present | 1.500 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.qc:vcf_cohort:plink | vcf | vcf.qc | plink | present | 1.250 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.qc:vcf_cohort:plink2 | vcf | vcf.qc | plink2 | present | 1.250 |  | fake_run_simulated |
| vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools | vcf | vcf.stats | bcftools | missing_result | 1.250 | 0.040 | fake_run_and_real_smoke |

## Memory

| Report Row | Domain | Stage | Tool | Status | Declared Memory MB | Declared CPU Threads | Real-Smoke Memory MB | Real-Smoke CPU Threads | Source |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- |
| bam:corpus-01-mini:bam.align:sample-set:bowtie2 | bam | bam.align | bowtie2 | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-mini:bam.align:sample-set:bwa | bam | bam.align | bwa | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:authenticct | bam | bam.authenticity | authenticct | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:damageprofiler | bam | bam.authenticity | damageprofiler | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:pmdtools | bam | bam.authenticity | pmdtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.bias_mitigation:sample-set:mapdamage2 | bam | bam.bias_mitigation | mapdamage2 | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.complexity:sample-set:preseq | bam | bam.complexity | preseq | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:contammix | bam | bam.contamination | contammix | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:schmutzi | bam | bam.contamination | schmutzi | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:verifybamid2 | bam | bam.contamination | verifybamid2 | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.coverage:sample-set:bedtools | bam | bam.coverage | bedtools | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.coverage:sample-set:mosdepth | bam | bam.coverage | mosdepth | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools | bam | bam.coverage | samtools | missing_result | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:addeam | bam | bam.damage | addeam | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:damageprofiler | bam | bam.damage | damageprofiler | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:mapdamage2 | bam | bam.damage | mapdamage2 | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:ngsbriggs | bam | bam.damage | ngsbriggs | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:pmdtools | bam | bam.damage | pmdtools | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:pydamage | bam | bam.damage | pydamage | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.duplication_metrics:sample-set:picard | bam | bam.duplication_metrics | picard | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.duplication_metrics:sample-set:samtools | bam | bam.duplication_metrics | samtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.endogenous_content:sample-set:samtools | bam | bam.endogenous_content | samtools | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.filter:sample-set:bamtools | bam | bam.filter | bamtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.filter:sample-set:bedtools | bam | bam.filter | bedtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.filter:sample-set:samtools | bam | bam.filter | samtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.gc_bias:sample-set:picard | bam | bam.gc_bias | picard | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-genotyping-mini:bam.genotyping:human_like_genotyping_candidate_panel:angsd | bam | bam.genotyping | angsd | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-bam-mini:bam.haplogroups:sample-set:yleaf | bam | bam.haplogroups | yleaf | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.insert_size:sample-set:picard | bam | bam.insert_size | picard | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-kinship-mini:bam.kinship:sample-set:angsd | bam | bam.kinship | angsd | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-kinship-mini:bam.kinship:sample-set:king | bam | bam.kinship | king | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.length_filter:sample-set:picard | bam | bam.length_filter | picard | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.length_filter:sample-set:samtools | bam | bam.length_filter | samtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.mapping_summary:sample-set:picard | bam | bam.mapping_summary | picard | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.mapping_summary:sample-set:samtools | bam | bam.mapping_summary | samtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.mapq_filter:sample-set:bamtools | bam | bam.mapq_filter | bamtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.mapq_filter:sample-set:samtools | bam | bam.mapq_filter | samtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.markdup:sample-set:picard | bam | bam.markdup | picard | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.markdup:sample-set:samtools | bam | bam.markdup | samtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.overlap_correction:sample-set:bamutil | bam | bam.overlap_correction | bamutil | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc | bam | bam.qc_pre | multiqc | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.qc_pre:sample-set:samtools | bam | bam.qc_pre | samtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.recalibration:sample-set:gatk | bam | bam.recalibration | gatk | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-bam-mini:bam.sex:sample-set:angsd | bam | bam.sex | angsd | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-bam-mini:bam.sex:sample-set:rxy | bam | bam.sex | rxy | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-adna-bam-mini:bam.sex:sample-set:yleaf | bam | bam.sex | yleaf | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.validate:sample-set:bamtools | bam | bam.validate | bamtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.validate:sample-set:bedtools | bam | bam.validate | bedtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam:corpus-01-bam-mini:bam.validate:sample-set:samtools | bam | bam.validate | samtools | present | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| fastq:corpus-03-amplicon-mini:fastq.cluster_otus:sample-set:vsearch | fastq | fastq.cluster_otus | vsearch | present | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.correct_errors:sample-set:bayeshammer | fastq | fastq.correct_errors | bayeshammer | present | 16384.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.correct_errors:sample-set:lighter | fastq | fastq.correct_errors | lighter | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.correct_errors:sample-set:musket | fastq | fastq.correct_errors | musket | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.correct_errors:sample-set:rcorrector | fastq | fastq.correct_errors | rcorrector | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.deplete_host:sample-set:bowtie2 | fastq | fastq.deplete_host | bowtie2 | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.deplete_reference_contaminants:sample-set:bowtie2 | fastq | fastq.deplete_reference_contaminants | bowtie2 | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.deplete_rrna:sample-set:sortmerna | fastq | fastq.deplete_rrna | sortmerna | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.detect_adapters:sample-set:fastqc | fastq | fastq.detect_adapters | fastqc | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.detect_duplicates_premerge:sample-set:bijux_dna | fastq | fastq.detect_duplicates_premerge | bijux_dna | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.extract_umis:sample-set:umi_tools | fastq | fastq.extract_umis | umi_tools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.filter_low_complexity:sample-set:bbduk | fastq | fastq.filter_low_complexity | bbduk | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.filter_low_complexity:sample-set:prinseq | fastq | fastq.filter_low_complexity | prinseq | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.filter_reads:sample-set:bbduk | fastq | fastq.filter_reads | bbduk | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.filter_reads:sample-set:fastp | fastq | fastq.filter_reads | fastp | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.filter_reads:sample-set:prinseq | fastq | fastq.filter_reads | prinseq | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.filter_reads:sample-set:seqkit | fastq | fastq.filter_reads | seqkit | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-03-amplicon-mini:fastq.infer_asvs:sample-set:dada2 | fastq | fastq.infer_asvs | dada2 | present | 16384.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:adapterremoval | fastq | fastq.merge_pairs | adapterremoval | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:bbmerge | fastq | fastq.merge_pairs | bbmerge | present | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:flash2 | fastq | fastq.merge_pairs | flash2 | present | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:leehom | fastq | fastq.merge_pairs | leehom | present | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:pear | fastq | fastq.merge_pairs | pear | present | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.merge_pairs:sample-set:vsearch | fastq | fastq.merge_pairs | vsearch | present | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-03-amplicon-mini:fastq.normalize_abundance:sample-set:seqkit | fastq | fastq.normalize_abundance | seqkit | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-03-amplicon-mini:fastq.normalize_primers:sample-set:cutadapt | fastq | fastq.normalize_primers | cutadapt | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:fastp | fastq | fastq.profile_read_lengths | fastp | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:prinseq | fastq | fastq.profile_read_lengths | prinseq | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:seqfu | fastq | fastq.profile_read_lengths | seqfu | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:seqkit_stats | fastq | fastq.profile_read_lengths | seqkit_stats | present | 2048.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqfu | fastq | fastq.profile_reads | seqfu | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit | fastq | fastq.profile_reads | seqkit | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats | fastq | fastq.profile_reads | seqkit_stats | present | 2048.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-03-amplicon-mini:fastq.remove_chimeras:sample-set:vsearch | fastq | fastq.remove_chimeras | vsearch | present | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.remove_duplicates:sample-set:clumpify | fastq | fastq.remove_duplicates | clumpify | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.remove_duplicates:sample-set:fastuniq | fastq | fastq.remove_duplicates | fastuniq | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:centrifuge | fastq | fastq.screen_taxonomy | centrifuge | present | 16384.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kaiju | fastq | fastq.screen_taxonomy | kaiju | present | 16384.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2 | fastq | fastq.screen_taxonomy | kraken2 | missing_result | 16384.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:krakenuniq | fastq | fastq.screen_taxonomy | krakenuniq | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_polyg_tails:sample-set:bbduk | fastq | fastq.trim_polyg_tails | bbduk | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_polyg_tails:sample-set:fastp | fastq | fastq.trim_polyg_tails | fastp | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:adapterremoval | fastq | fastq.trim_reads | adapterremoval | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:alientrimmer | fastq | fastq.trim_reads | alientrimmer | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:atropos | fastq | fastq.trim_reads | atropos | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:bbduk | fastq | fastq.trim_reads | bbduk | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:cutadapt | fastq | fastq.trim_reads | cutadapt | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:fastp | fastq | fastq.trim_reads | fastp | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:fastx_clipper | fastq | fastq.trim_reads | fastx_clipper | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:leehom | fastq | fastq.trim_reads | leehom | present | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:prinseq | fastq | fastq.trim_reads | prinseq | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:seqkit | fastq | fastq.trim_reads | seqkit | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:skewer | fastq | fastq.trim_reads | skewer | present | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:trim_galore | fastq | fastq.trim_reads | trim_galore | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_reads:sample-set:trimmomatic | fastq | fastq.trim_reads | trimmomatic | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:adapterremoval | fastq | fastq.trim_terminal_damage | adapterremoval | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:cutadapt | fastq | fastq.trim_terminal_damage | cutadapt | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:seqkit | fastq | fastq.trim_terminal_damage | seqkit | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastq_scan | fastq | fastq.validate_reads | fastq_scan | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqc | fastq | fastq.validate_reads | fastqc | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqvalidator | fastq | fastq.validate_reads | fastqvalidator | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:fqtools | fastq | fastq.validate_reads | fqtools | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq:corpus-01-mini:fastq.validate_reads:sample-set:seqtk | fastq | fastq.validate_reads | seqtk | present | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools | vcf | vcf.call | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.call_diploid:bam_bundle:bcftools | vcf | vcf.call_diploid | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.call_gl:bam_bundle:bcftools | vcf | vcf.call_gl | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.call_pseudohaploid:bam_bundle:bcftools | vcf | vcf.call_pseudohaploid | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.damage_filter:vcf_single_sample:bcftools | vcf | vcf.damage_filter | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.filter:vcf_single_sample:bcftools | vcf | vcf.filter | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| unsupported:vcf:vcf.filter:samtools | vcf | vcf.filter | samtools | unsupported_pair |  |  |  |  | not_applicable |
| vcf:vcf_production_regression:vcf.gl_propagation:vcf_single_sample:bcftools | vcf | vcf.gl_propagation | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools | vcf | vcf.postprocess | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.prepare_reference_panel:vcf_reference_panel:bcftools | vcf | vcf.prepare_reference_panel | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.qc:vcf_cohort:bcftools | vcf | vcf.qc | bcftools | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.qc:vcf_cohort:plink | vcf | vcf.qc | plink | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.qc:vcf_cohort:plink2 | vcf | vcf.qc | plink2 | present | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools | vcf | vcf.stats | bcftools | missing_result | 4096.000 | 2 |  |  | declared_stage_tool_resource |

## Failures

- Simulated failure rows: 125
- Failure classification rows: 7

| Failure Class | Domain | Stage | Tool | Source Surface | Status | Detail |
| --- | --- | --- | --- | --- | --- | --- |
| command_failed | fastq | fastq.cluster_otus | vsearch | governed_command_failed_probe | command_failed | governed shell probe exits with code 23 after writing stderr |
| insufficient_data | vcf | vcf.demography | ibdne | vcf_segment_fixture_bank | insufficient_data | ibdne |
| missing_input | vcf | vcf.call | bcftools | vcf_adapter_missing_input_tests | missing_input | governed VCF missing-input probe removed `bam` and expected `required input `input_bam`` |
| missing_output | vcf | vcf.call | bcftools | governed_missing_output_probe | missing_output | runs/bench/readiness-probes/all-domains/failure-classification/missing-output/vcf/vcf.call/bcftools/expected-output.json |
| parser_failed | vcf | vcf.call | bcftools | vcf_parser_failure_tests | parser_failed | parse_bcftools_call_metrics expected `raw VCF is missing #CHROM header` |
| tool_not_found | bam | bam.align | bowtie2 | governed_tool_not_found_probe | tool_not_found | governed probe derived from a benchmark-ready binding uses an absent executable path |
| unsupported_pair | vcf | vcf.filter | samtools | all_domain_stage_tool_table | unsupported_pair | unsupported-pair classification must remain explicit instead of collapsing into a generic failed status |

## Missing Results

| Result ID | Domain | Stage | Tool | Corpus | Section | Manifest Path | Reason |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools | bam | bam.coverage | samtools | corpus-01-bam-mini | coverage_quality | runs/bench/readiness-probes/all-domains/missing-result-test/bam/corpus-01-bam-mini/bam.coverage/corpus_only/samtools/stage-result.json | expected all-domain benchmark result `bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools` remains visible even though its fake-run manifest is missing |
| fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2 | fastq | fastq.screen_taxonomy | kraken2 | corpus-02-edna-mini | contamination_screening | runs/bench/readiness-probes/all-domains/missing-result-test/fastq/corpus-02-edna-mini/fastq.screen_taxonomy/database_artifact_id+taxonomy_database_root/kraken2/stage-result.json | expected all-domain benchmark result `fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2` remains visible even though its fake-run manifest is missing |
| vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools | vcf | vcf.stats | bcftools | vcf_production_regression | quality_control | runs/bench/readiness-probes/all-domains/missing-result-test/vcf/vcf_production_regression/vcf.stats/vcf_cohort/bcftools/stage-result.json | expected all-domain benchmark result `vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools` remains visible even though its fake-run manifest is missing |

## Comparable Metrics

| Domain | Stage | Metric ID | Metric Name | Unit | Direction | Required | Tools | Contract Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.align | alignment_rate | alignment_rate |  |  |  | bowtie2, bwa | declared |
| bam | bam.align | mapped_reads | mapped_reads |  |  |  | bowtie2, bwa | declared |
| bam | bam.authenticity | confidence | confidence |  |  |  | authenticct, damageprofiler, pmdtools | declared |
| bam | bam.authenticity | consumed_metric_ids | consumed_metric_ids |  |  |  | authenticct, damageprofiler, pmdtools | declared |
| bam | bam.authenticity | missing_metric_ids | missing_metric_ids |  |  |  | authenticct, damageprofiler, pmdtools | declared |
| bam | bam.authenticity | pmd_like_signal_present | pmd_like_signal_present |  |  |  | authenticct, damageprofiler, pmdtools | declared |
| bam | bam.authenticity | score | score |  |  |  | authenticct, damageprofiler, pmdtools | declared |
| bam | bam.contamination | ci_high | ci_high |  |  |  | contammix, schmutzi, verifybamid2 | declared |
| bam | bam.contamination | ci_low | ci_low |  |  |  | contammix, schmutzi, verifybamid2 | declared |
| bam | bam.contamination | estimate | estimate |  |  |  | contammix, schmutzi, verifybamid2 | declared |
| bam | bam.contamination | prerequisites_passed | prerequisites_passed |  |  |  | contammix, schmutzi, verifybamid2 | declared |
| bam | bam.contamination | scope | scope |  |  |  | contammix, schmutzi, verifybamid2 | declared |
| bam | bam.coverage | breadth_1x | breadth_1x |  |  |  | bedtools, mosdepth, samtools | declared |
| bam | bam.coverage | covered_bases | covered_bases |  |  |  | bedtools, mosdepth, samtools | declared |
| bam | bam.coverage | mean_depth | mean_depth |  |  |  | bedtools, mosdepth, samtools | declared |
| bam | bam.coverage | observed_region_count | observed_region_count |  |  |  | bedtools, mosdepth, samtools | declared |
| bam | bam.coverage | region_ids | region_ids |  |  |  | bedtools, mosdepth, samtools | declared |
| bam | bam.damage | damage_signal | damage_signal |  |  |  | addeam, damageprofiler, mapdamage2, ngsbriggs, pmdtools, pydamage | declared |
| bam | bam.damage | memory_mb | memory_mb |  |  |  | addeam, damageprofiler, mapdamage2, ngsbriggs, pmdtools, pydamage | declared |
| bam | bam.damage | runtime_s | runtime_s |  |  |  | addeam, damageprofiler, mapdamage2, ngsbriggs, pmdtools, pydamage | declared |
| bam | bam.damage | terminal_c_to_t_5p | terminal_c_to_t_5p |  |  |  | addeam, damageprofiler, mapdamage2, ngsbriggs, pmdtools, pydamage | declared |
| bam | bam.damage | terminal_g_to_a_3p | terminal_g_to_a_3p |  |  |  | addeam, damageprofiler, mapdamage2, ngsbriggs, pmdtools, pydamage | declared |
| bam | bam.duplication_metrics | duplicate_count | duplicate_count |  |  |  | picard, samtools | declared |
| bam | bam.duplication_metrics | duplicate_fraction | duplicate_fraction |  |  |  | picard, samtools | declared |
| bam | bam.duplication_metrics | estimated_library_size | estimated_library_size |  |  |  | picard, samtools | declared |
| bam | bam.filter | active_filters | active_filters |  |  |  | bamtools, bedtools, samtools | declared |
| bam | bam.filter | input_reads | input_reads |  |  |  | bamtools, bedtools, samtools | declared |
| bam | bam.filter | kept_reads | kept_reads |  |  |  | bamtools, bedtools, samtools | declared |
| bam | bam.filter | removed_reads | removed_reads |  |  |  | bamtools, bedtools, samtools | declared |
| bam | bam.kinship | observed_max_overlap_snps | observed_max_overlap_snps |  |  |  | angsd, king | declared |
| bam | bam.kinship | pair_count | pair_count |  |  |  | angsd, king | declared |
| bam | bam.kinship | pairwise_results | pairwise_results |  |  |  | angsd, king | declared |
| bam | bam.kinship | status | status |  |  |  | angsd, king | declared |
| bam | bam.length_filter | filtered_bam | filtered_bam |  |  |  | picard, samtools | declared |
| bam | bam.length_filter | kept_reads | kept_reads |  |  |  | picard, samtools | declared |
| bam | bam.length_filter | min_length_threshold | min_length_threshold |  |  |  | picard, samtools | declared |
| bam | bam.length_filter | removed_reads | removed_reads |  |  |  | picard, samtools | declared |
| bam | bam.mapping_summary | mapped_reads | mapped_reads |  |  |  | picard, samtools | declared |
| bam | bam.mapping_summary | mapping_fraction | mapping_fraction |  |  |  | picard, samtools | declared |
| bam | bam.mapping_summary | secondary_reads | secondary_reads |  |  |  | picard, samtools | declared |
| bam | bam.mapping_summary | supplementary_reads | supplementary_reads |  |  |  | picard, samtools | declared |
| bam | bam.mapping_summary | unmapped_reads | unmapped_reads |  |  |  | picard, samtools | declared |
| bam | bam.mapq_filter | filtered_bam | filtered_bam |  |  |  | bamtools, samtools | declared |
| bam | bam.mapq_filter | kept_reads | kept_reads |  |  |  | bamtools, samtools | declared |
| bam | bam.mapq_filter | mapq_threshold | mapq_threshold |  |  |  | bamtools, samtools | declared |
| bam | bam.mapq_filter | removed_reads | removed_reads |  |  |  | bamtools, samtools | declared |
| bam | bam.markdup | duplicate_count | duplicate_count |  |  |  | picard, samtools | declared |
| bam | bam.markdup | duplicate_fraction | duplicate_fraction |  |  |  | picard, samtools | declared |
| bam | bam.markdup | duplicate_metrics | duplicate_metrics |  |  |  | picard, samtools | declared |
| bam | bam.markdup | marked_bam | marked_bam |  |  |  | picard, samtools | declared |
| bam | bam.qc_pre | contig_summary | contig_summary |  |  |  | multiqc, samtools | declared |
| bam | bam.qc_pre | duplicate_flagged_reads | duplicate_flagged_reads |  |  |  | multiqc, samtools | declared |
| bam | bam.qc_pre | mapped_reads | mapped_reads |  |  |  | multiqc, samtools | declared |
| bam | bam.qc_pre | total_reads | total_reads |  |  |  | multiqc, samtools | declared |
| bam | bam.qc_pre | unmapped_reads | unmapped_reads |  |  |  | multiqc, samtools | declared |
| bam | bam.sex | autosomal_coverage | autosomal_coverage |  |  |  | angsd, rxy, yleaf | declared |
| bam | bam.sex | call | call |  |  |  | angsd, rxy, yleaf | declared |
| bam | bam.sex | confidence | confidence |  |  |  | angsd, rxy, yleaf | declared |
| bam | bam.sex | status | status |  |  |  | angsd, rxy, yleaf | declared |
| bam | bam.sex | x_coverage | x_coverage |  |  |  | angsd, rxy, yleaf | declared |
| bam | bam.sex | y_coverage | y_coverage |  |  |  | angsd, rxy, yleaf | declared |
| bam | bam.validate | input_bam_identity | input_bam_identity |  |  |  | bamtools, bedtools, samtools | declared |
| bam | bam.validate | validation_errors | validation_errors |  |  |  | bamtools, bedtools, samtools | declared |
| bam | bam.validate | validation_status | validation_status |  |  |  | bamtools, bedtools, samtools | declared |
| bam | bam.validate | validation_warnings | validation_warnings |  |  |  | bamtools, bedtools, samtools | declared |
| fastq | fastq.index_reference | index_build_exit_code | index_build_exit_code |  |  |  | bowtie2_build, star | declared |
| fastq | fastq.profile_overrepresented_sequences | flagged_sequences | flagged_sequences |  |  |  | fastq_scan, fastqc, seqkit | declared |
| fastq | fastq.profile_overrepresented_sequences | sequence_count | sequence_count |  |  |  | fastq_scan, fastqc, seqkit | declared |
| fastq | fastq.profile_overrepresented_sequences | top_fraction | top_fraction |  |  |  | fastq_scan, fastqc, seqkit | declared |
| fastq | fastq.validate_reads | format_validation_pass_rate | format_validation_pass_rate |  |  |  | fastq_scan, fastqc, fastqvalidator, fqtools, seqtk | declared |
| vcf | vcf.admixture | population_count | population count | populations | exact_match_preferred | true | plink, plink2 | declared |
| vcf | vcf.admixture | sample_count | sample count | samples | exact_match_preferred | true | plink, plink2 | declared |
| vcf | vcf.admixture | selected_k | selected cluster count | clusters | exact_match_preferred | true | plink, plink2 | declared |
| vcf | vcf.call_gl | missing_likelihoods | missing likelihoods | sites | lower_is_better | true | angsd, bcftools | declared |
| vcf | vcf.call_gl | sites_with_likelihoods | sites with likelihoods | sites | higher_is_better | true | angsd, bcftools | declared |
| vcf | vcf.call_pseudohaploid | called_sites | called sites | sites | higher_is_better | true | angsd, bcftools | declared |
| vcf | vcf.call_pseudohaploid | missing_sites | missing sites | sites | lower_is_better | true | angsd, bcftools | declared |
| vcf | vcf.damage_filter | removed_variants | removed variants | variants | exact_match_preferred | true | angsd, bcftools | declared |
| vcf | vcf.damage_filter | retained_variants | retained variants | variants | exact_match_preferred | true | angsd, bcftools | declared |
| vcf | vcf.damage_filter | terminal_damage_filtered_variants | terminal damage filtered variants | variants | exact_match_preferred | true | angsd, bcftools | declared |
| vcf | vcf.gl_propagation | sample_count | sample count | samples | exact_match_preferred | true | angsd, bcftools | declared |
| vcf | vcf.gl_propagation | site_count_after | site count after propagation | sites | exact_match_preferred | true | angsd, bcftools | declared |
| vcf | vcf.gl_propagation | site_count_before | site count before propagation | sites | exact_match_preferred | true | angsd, bcftools | declared |
| vcf | vcf.ibd | pair_count | pair count | pairs | exact_match_preferred | true | germline, ibdhap, ibdseq | declared |
| vcf | vcf.imputation_metrics | low_confidence_sites | low-confidence sites | sites | lower_is_better | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.imputation_metrics | masked_truth_sites | masked-truth sites | sites | exact_match_preferred | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.imputation_metrics | mean_info_score | mean info score | score | higher_is_better | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.impute | imputed_genotypes | imputed genotypes | genotypes | higher_is_better | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.impute | low_confidence_count | low-confidence sites | sites | lower_is_better | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.impute | masked_truth_match_count | masked-truth matches | sites | higher_is_better | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.impute | missing_after | missing genotypes after imputation | genotypes | lower_is_better | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.impute | missing_before | missing genotypes before imputation | genotypes | lower_is_better | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.impute | unresolved_count | unresolved sites | sites | lower_is_better | true | beagle, glimpse, impute5, minimac4 | declared |
| vcf | vcf.pca | sample_count | sample count | samples | exact_match_preferred | true | eigensoft, plink2 | declared |
| vcf | vcf.pca | variant_count | variant count | variants | exact_match_preferred | true | eigensoft, plink2 | declared |
| vcf | vcf.phasing | phase_block_n50 | phase block n50 | bases | higher_is_better | true | beagle, eagle, shapeit5 | declared |
| vcf | vcf.phasing | switch_error_proxy | switch error proxy | fraction | lower_is_better | true | beagle, eagle, shapeit5 | declared |
| vcf | vcf.population_structure | pair_count | pair count | pairs | exact_match_preferred | true | eigensoft, plink2 | declared |
| vcf | vcf.population_structure | sample_count | sample count | samples | exact_match_preferred | true | eigensoft, plink2 | declared |
| vcf | vcf.qc | concordance | concordance | fraction | higher_is_better | true | bcftools, plink, plink2 | declared |
| vcf | vcf.qc | imputation_info_mean | mean imputation info | score | higher_is_better | true | bcftools, plink, plink2 | declared |
| vcf | vcf.qc | missingness_post | post-qc missingness | fraction | lower_is_better | true | bcftools, plink, plink2 | declared |
| vcf | vcf.qc | rsq_mean | mean r-squared | score | higher_is_better | true | bcftools, plink, plink2 | declared |

## Unsupported Pairs

| Domain | Stage | Tool | Evidence Path | Detail |
| --- | --- | --- | --- | --- |
| vcf | vcf.filter | samtools | benchmarks/readiness/all-domain-stage-tool-table.tsv | unsupported-pair classification must remain explicit instead of collapsing into a generic failed status |
