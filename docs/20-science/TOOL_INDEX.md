<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-tool-index -->

# TOOL_INDEX

## Purpose
Generated index of registry tools with stage bindings and container references/self-reports.

## Scope
Source of truth = registry contracts + `artifacts/containers/summary.json` self-reports when available.

## Non-goals
- Replacing full scientific method docs for each domain.

## Contracts
- Manual edits are forbidden; regenerate via native control-plane.
- Source of truth is registry + containers; this file is a rendered view.
- Tool admission policy is documented in `docs/50-reference/TOOL_ADMISSION.md`.

See also: [Tool Admission](../50-reference/TOOL_ADMISSION.md)
See also: [VCF Downstream Roadmap](vcf/ROADMAP.md)

## VCF Downstream / IBD Toolkit

- `angsd` (planned) : vcf.call_gl, vcf.call_pseudohaploid, vcf.damage_filter, vcf.gl_propagation
- `bcftools` (production) : vcf.call, vcf.call_diploid, vcf.call_gl, vcf.call_pseudohaploid, vcf.damage_filter, vcf.filter, vcf.gl_propagation, vcf.postprocess, vcf.prepare_reference_panel, vcf.qc, vcf.stats
- `beagle` (experimental) : vcf.phasing
- `eagle` (experimental) : vcf.phasing
- `eigensoft` (experimental) : vcf.pca, vcf.population_structure
- `germline` (experimental) : vcf.ibd
- `glimpse` (planned) : vcf.impute, vcf.imputation_metrics
- `ibdhap` (planned) : vcf.ibd
- `ibdne` (planned) : vcf.ibd, vcf.demography
- `ibdseq` (planned) : vcf.ibd
- `impute5` (planned) : vcf.impute, vcf.imputation_metrics
- `minimac4` (planned) : vcf.impute, vcf.imputation_metrics
- `plink` (experimental) : vcf.qc, vcf.admixture
- `plink2` (experimental) : vcf.qc, vcf.pca, vcf.population_structure, vcf.roh, vcf.admixture
- `shapeit` (planned) : vcf.phasing
- `shapeit5` (experimental) : vcf.phasing

| Tool ID | Purpose | Stage Bindings | Container Ref | Version | Citation | Status |
|---|---|---|---|---|---|---|
| `adapterremoval` | `merger` | `fastq.merge_pairs, fastq.trim_reads, fastq.trim_terminal_damage` | `bijuxdna/adapterremoval@sha256:5b618834ce9fc6376c9605c3a69d738236b9be48fdf493c1bc0945568a50808d` | `2.3.3` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC4751634/ | `production` |
| `addeam` | `transform` | `bam.damage` | `bijuxdna/addeam:1.0.0` | `1.0.0` | paper:https://academic.oup.com/bioinformatics/article/doi/10.1093/bioinformatics/btaf407/8205671 | `production` |
| `alientrimmer` | `trimmer` | `fastq.trim_reads` | `bijuxdna/alientrimmer:3.2` | `3.2` | paper:https://pubmed.ncbi.nlm.nih.gov/23912058/ | `production` |
| `angsd` | `genotyping` | `vcf.call_gl, vcf.call_pseudohaploid, vcf.damage_filter, vcf.gl_propagation` | `@` | `0.940` | Korneliussen et al. 2014, BMC Bioinformatics 15:356 | `planned` |
| `atropos` | `trimmer` | `fastq.trim_reads` | `bijuxdna/atropos@sha256:5a2fab22811303ced83475111a2427eaa894fcb39436618bb26be1da391f300d` | `1.1.31` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC5581536/ | `production` |
| `authenticct` | `transform` | `bam.authenticity` | `bijuxdna/authenticct:1.0.0` | `1.0.0` | paper:https://genomebiology.biomedcentral.com/articles/10.1186/s13059-020-02123-y | `production` |
| `bamtools` | `filter` | `bam.filter, bam.mapq_filter, bam.validate` | `bijuxdna/bamtools:2.5.2` | `2.5.2` | paper:https://academic.oup.com/bioinformatics/article/27/12/1691/255399 | `production` |
| `bamutil` | `corrector` | `bam.overlap_correction` | `bijuxdna/bamutil:1.0.15` | `1.0.15` | upstream:https://github.com/statgen/bamUtil | `production` |
| `bayeshammer` | `corrector` | `fastq.correct_errors` | `bijuxdna/bayeshammer@sha256:99849ca7576f392a125ce5836e915481ada762cbb4fa9b69593b81c4ca359cd7` | `4.2.0` | paper:https://link.springer.com/article/10.1186/1471-2164-14-S1-S7 | `production` |
| `bbduk` | `filter` | `fastq.filter_low_complexity, fastq.filter_reads, fastq.trim_polyg_tails, fastq.trim_reads` | `bijuxdna/bbduk@sha256:da5764715915a5edeb0e40e2c18a5ce7142f31dac8e4844bd2dcb463403b8bd4` | `39.08` | software:https://bbmap.org/tools/bbduk | `production` |
| `bbmerge` | `merger` | `fastq.merge_pairs` | `bijuxdna/bbmerge@sha256:6182848b989c8dbf094e06c486190b5d54243ac8eea542daa2c5c059a11bba54` | `39.01` | paper:https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0185056 | `production` |
| `bcftools` | `unknown` | `vcf.call, vcf.call_diploid, vcf.call_gl, vcf.call_pseudohaploid, vcf.damage_filter, vcf.filter, vcf.gl_propagation, vcf.postprocess, vcf.prepare_reference_panel, vcf.qc, vcf.stats` | `quay.io/biocontainers/bcftools:1.20--h8b25389_0@sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6` | `1.20` | DOI:10.1093/gigascience/giab008 | `production` |
| `beagle` | `phasing` | `vcf.phasing` | `registry_lock` | `5.4` | planned | `experimental` |
| `bedtools` | `transform` | `bam.coverage, bam.filter, bam.validate` | `bijuxdna/bedtools:2.31.1` | `2.31.1` | paper:https://academic.oup.com/bioinformatics/article/26/6/841/244688 | `production` |
| `bijux_dna` | `merger` | `fastq.detect_duplicates_premerge, fastq.estimate_library_complexity_prealign` | `bijuxdna/bijux_dna:workspace` | `workspace` | software:https://github.com/bijux/bijux-genomics | `production` |
| `bowtie2` | `aligner` | `bam.align, fastq.deplete_host, fastq.deplete_reference_contaminants` | `bijuxdna/bowtie2:2.5.4` | `2.5.4` | paper:https://www.nature.com/articles/nmeth.1923 | `production` |
| `bowtie2_build` | `transform` | `fastq.index_reference` | `bijuxdna/bowtie2_build:2.5.4` | `2.5.4` | paper:https://www.nature.com/articles/nmeth.1923 | `production` |
| `bwa` | `aligner` | `bam.align` | `bijuxdna/bwa:0.7.17` | `0.7.17` | paper:https://academic.oup.com/bioinformatics/article/25/14/1754/225615 | `production` |
| `centrifuge` | `screen` | `fastq.screen_taxonomy` | `bijuxdna/centrifuge@sha256:7a29ee8d8dc156513a23e05e9f8fda2299032ab7e6d5cb3c241c9f34c3f81553` | `1.0.4` | paper:https://genome.cshlp.org/content/26/12/1721 | `production` |
| `clumpify` | `transform` | `fastq.remove_duplicates` | `bijuxdna/clumpify@sha256:fa8159777183edae8355363ba8cc5e8f8f069bee09cb6f2fb70a7ca4220a5c81` | `39.08` | software:https://bbmap.org/tools/clumpify | `production` |
| `contammix` | `transform` | `bam.contamination` | `bijuxdna/contammix:1.0.11` | `1.0.11` | upstream:https://bioconductor.org/packages/contamMix | `production` |
| `cutadapt` | `transform` | `fastq.normalize_primers, fastq.trim_reads, fastq.trim_terminal_damage` | `bijuxdna/cutadapt@sha256:4405f2effc1a195c93098408aa36268357c25b758348bfe6da8790bbe7e842ba` | `4.9` | paper:https://journal.embnet.org/index.php/embnetjournal/article/view/200 | `production` |
| `dada2` | `transform` | `fastq.infer_asvs` | `bijuxdna/dada2@sha256:930e06fb8e27965ccafae61d4161ab5e32933aea2555e68ad409ef7af43c8245` | `1.30.0` | paper:https://www.nature.com/articles/nmeth.3869 | `production` |
| `damageprofiler` | `transform` | `bam.authenticity, bam.damage` | `bijuxdna/damageprofiler:1.1` | `1.1` | paper:https://academic.oup.com/bioinformatics/article/37/20/3652/6247758 | `production` |
| `eagle` | `phasing` | `vcf.phasing` | `registry_lock` | `5.4` | planned | `experimental` |
| `eigensoft` | `population_structure` | `vcf.pca, vcf.population_structure` | `registry_lock` | `8.0.0` | planned | `experimental` |
| `fastp` | `filter` | `fastq.filter_reads, fastq.profile_read_lengths, fastq.trim_polyg_tails, fastq.trim_reads` | `bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10` | `0.23.4` | paper:https://academic.oup.com/bioinformatics/article/34/17/i884/5093234 | `production` |
| `fastq_scan` | `transform` | `fastq.profile_overrepresented_sequences, fastq.validate_reads` | `bijuxdna/fastq_scan@sha256:973e958d84a4a5d779a71e34f890b0a5335607e1fe56207c8a145409fd2937ad` | `0.0.1` | software:https://github.com/rpetit3/fastq-scan | `production` |
| `fastqc` | `trimmer` | `fastq.detect_adapters, fastq.profile_overrepresented_sequences, fastq.validate_reads` | `bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789` | `0.12.1` | software:https://www.bioinformatics.babraham.ac.uk/projects/fastqc/ | `production` |
| `fastqvalidator` | `validator` | `fastq.validate_reads` | `bijuxdna/fastqvalidator@sha256:3e01ccfed0313ffb63e8b9a8c4044a47249906d0b7b32c68caccf81f0a532edc` | `v0.1.1` | software:https://github.com/statgen/fastQValidator | `production` |
| `fastuniq` | `transform` | `fastq.remove_duplicates` | `bijuxdna/fastuniq@sha256:4e0aba0be50a2894b7d36e34599cb4ff727be3e57fe47895f1a35b04985fcacf` | `1.1` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC3527383/ | `production` |
| `fastx_clipper` | `trimmer` | `fastq.trim_reads` | `bijuxdna/fastx_clipper:0.0.14` | `0.0.14` | software:http://hannonlab.cshl.edu/fastx_toolkit/ | `production` |
| `flash2` | `merger` | `fastq.merge_pairs` | `bijuxdna/flash2@sha256:e3dfc866d56d1ca6d62c58ade5981e0b00fc3c8bf8148ecbd196ab56293e1dd5` | `2.2.00` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC3198573/ | `production` |
| `fqtools` | `validator` | `fastq.validate_reads` | `bijuxdna/fqtools@sha256:d7190221f5582bdabccecb3bae06d463d81c0cb06d47a80fa4c1b3c4704d4bf6` | `v2.3` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC4908325/ | `production` |
| `gatk` | `transform` | `bam.recalibration` | `bijuxdna/gatk:4.6.2.0` | `4.6.2.0` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC2928508/ | `production` |
| `germline` | `relatedness` | `vcf.ibd` | `registry_lock` | `1.5.3` | planned | `experimental` |
| `glimpse` | `imputation` | `vcf.impute, vcf.imputation_metrics` | `registry_lock` | `0.0.0-planned` | planned | `planned` |
| `ibdhap` | `relatedness` | `vcf.ibd` | `registry_lock` | `0.1.0-planned` | planned | `planned` |
| `ibdne` | `demography` | `vcf.ibd, vcf.demography` | `registry_lock` | `1.0-planned` | planned | `planned` |
| `ibdseq` | `relatedness` | `vcf.ibd` | `registry_lock` | `3.0-planned` | planned | `planned` |
| `impute5` | `imputation` | `vcf.impute, vcf.imputation_metrics` | `registry_lock` | `0.0.0-planned` | planned | `planned` |
| `kaiju` | `screen` | `fastq.screen_taxonomy` | `bijuxdna/kaiju@sha256:4f30fd9becc62e873bc223231c717bba5b42db8a4f993979bf26c7fc00979f9b` | `1.10.0` | paper:https://www.nature.com/articles/ncomms11257 | `production` |
| `king` | `transform` | `bam.kinship` | `bijuxdna/king:2.3.0` | `2.3.0` | paper:https://academic.oup.com/bioinformatics/article/26/22/2867/228512 | `production` |
| `kraken2` | `screen` | `fastq.screen_taxonomy` | `bijuxdna/kraken2@sha256:e493061d26aeea71812ed8b587a0e65f67617b192e225d8d53eba896dfe7cb35` | `2.1.3` | paper:https://genomebiology.biomedcentral.com/articles/10.1186/s13059-019-1891-0 | `production` |
| `krakenuniq` | `screen` | `fastq.screen_taxonomy` | `bijuxdna/krakenuniq:1.0.4` | `1.0.4` | paper:https://genomebiology.biomedcentral.com/articles/10.1186/s13059-018-1568-0 | `production` |
| `leehom` | `merger` | `fastq.merge_pairs, fastq.trim_reads` | `bijuxdna/leehom@sha256:146f111ee336a7d01cba5861a9672d75a7739ad7ece923925bbd90c2550341b1` | `bbddce1542ce` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC4191382/ | `production` |
| `lighter` | `corrector` | `fastq.correct_errors` | `bijuxdna/lighter@sha256:e65d145062ddedd1e584f7aec7db45c13ddf33f849888e61d6c3b91e89730085` | `1.1.2` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC4248469/ | `production` |
| `mapdamage2` | `transform` | `bam.bias_mitigation, bam.damage` | `bijuxdna/mapdamage2:2.2.2` | `2.2.2` | paper:https://academic.oup.com/bioinformatics/article/29/13/1682/184965 | `production` |
| `minimac4` | `imputation` | `vcf.impute, vcf.imputation_metrics` | `registry_lock` | `0.0.0-planned` | planned | `planned` |
| `mosdepth` | `transform` | `bam.coverage` | `bijuxdna/mosdepth:0.3.10` | `0.3.10` | paper:https://academic.oup.com/bioinformatics/article-abstract/34/5/867/4583630 | `production` |
| `multiqc` | `qc` | `bam.qc_pre, fastq.report_qc` | `bijuxdna/multiqc@sha256:40af0025fcc5bc4ea15e5cd2a4fd7bcfc98ea06c9ca781e6268f3c81d12787ec` | `1.24` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC5039924/ | `production` |
| `musket` | `corrector` | `fastq.correct_errors` | `bijuxdna/musket@sha256:d2761679719e961709c0c4842842c0bcb5b3fdc23a5046aa6cd2b28a6dfaca68` | `1.1` | paper:https://academic.oup.com/bioinformatics/article/29/3/308/257257 | `production` |
| `ngsbriggs` | `transform` | `bam.damage` | `bijuxdna/ngsbriggs:0.1.3` | `0.1.3` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC11646307/ | `production` |
| `pear` | `merger` | `fastq.merge_pairs` | `bijuxdna/pear@sha256:4e00e9ffabc5ed46115efab5b6bae946913f1713bb314fd4acb7c379c37efae6` | `0.9.6` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC3933873/ | `production` |
| `picard` | `transform` | `bam.duplication_metrics, bam.gc_bias, bam.insert_size, bam.length_filter, bam.mapping_summary, bam.markdup` | `bijuxdna/picard:3.3.0` | `3.3.0` | software:https://broadinstitute.github.io/picard/ | `production` |
| `plink` | `qc_admixture` | `vcf.qc, vcf.admixture` | `registry_lock` | `1.90` | planned | `experimental` |
| `plink2` | `analysis` | `vcf.qc, vcf.pca, vcf.population_structure, vcf.roh, vcf.admixture` | `registry_lock` | `2.00a5` | planned | `experimental` |
| `pmdtools` | `transform` | `bam.authenticity, bam.damage` | `bijuxdna/pmdtools:0.60` | `0.60` | paper:https://doi.org/10.1073/pnas.1318934111 | `production` |
| `preseq` | `transform` | `bam.complexity` | `bijuxdna/preseq:3.2.0` | `3.2.0` | paper:https://www.nature.com/articles/nmeth.2375 | `production` |
| `prinseq` | `filter` | `fastq.filter_low_complexity, fastq.filter_reads, fastq.profile_read_lengths, fastq.trim_reads` | `bijuxdna/prinseq@sha256:7216ffecd7913edaea33ec76b3775ab0cb0d60064f31e96c63e043d578a3f971` | `1.2.4` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC3051327/ | `production` |
| `pydamage` | `transform` | `bam.damage` | `bijuxdna/pydamage:1.0.0` | `1.0.0` | paper:https://pubmed.ncbi.nlm.nih.gov/34395085/ | `production` |
| `rcorrector` | `corrector` | `fastq.correct_errors` | `bijuxdna/rcorrector@sha256:cfec0e32dc980f61681376e78961785c8f74804bd90ebabccb98f825e5fdb454` | `1.0.7` | paper:https://gigascience.biomedcentral.com/articles/10.1186/s13742-015-0089-y | `production` |
| `rxy` | `transform` | `bam.sex` | `bijuxdna/rxy:1.0.0` | `1.0.0` | upstream:https://github.com/bijux/bijux-genomics | `production` |
| `samtools` | `transform` | `bam.coverage, bam.duplication_metrics, bam.endogenous_content, bam.filter, bam.length_filter, bam.mapping_summary, bam.mapq_filter, bam.markdup, bam.qc_pre, bam.validate` | `bijuxdna/samtools:1.21` | `1.21` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC2723002/ | `production` |
| `schmutzi` | `transform` | `bam.contamination` | `bijuxdna/schmutzi:1.5.4` | `1.5.4` | paper:https://genomebiology.biomedcentral.com/articles/10.1186/s13059-015-0776-0 | `production` |
| `seqfu` | `transform` | `fastq.profile_read_lengths, fastq.profile_reads` | `bijuxdna/seqfu:2.4.0` | `2.4.0` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC8148589/ | `production` |
| `seqkit` | `filter` | `fastq.filter_reads, fastq.normalize_abundance, fastq.profile_overrepresented_sequences, fastq.profile_reads, fastq.trim_reads, fastq.trim_terminal_damage` | `bijuxdna/seqkit@sha256:ca3dc13e3fef5d34927c44b2d8cd2bc6708c2c256f42e51369d7b1203b0d2991` | `2.8.2` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC5051824/ | `production` |
| `seqkit_stats` | `transform` | `fastq.profile_read_lengths, fastq.profile_reads` | `bijuxdna/seqkit@sha256:ca3dc13e3fef5d34927c44b2d8cd2bc6708c2c256f42e51369d7b1203b0d2991` | `2.7.0` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC5051824/ | `production` |
| `seqtk` | `validator` | `fastq.validate_reads` | `bijuxdna/seqtk@sha256:16e615286a66f1654278a862ad47c3d62bdfabbbf17cb2e12a568256d1b05024` | `1.5-r133` | software:https://github.com/lh3/seqtk | `production` |
| `shapeit` | `phasing` | `vcf.phasing` | `registry_lock` | `4.2.2-planned` | planned | `planned` |
| `shapeit5` | `phasing` | `vcf.phasing` | `registry_lock` | `5.4` | planned | `experimental` |
| `skewer` | `trimmer` | `fastq.trim_reads` | `bijuxdna/skewer:978e8e46cba4` | `978e8e46cba4` | paper:https://bmcbioinformatics.biomedcentral.com/articles/10.1186/1471-2105-15-182 | `production` |
| `sortmerna` | `transform` | `fastq.deplete_rrna` | `bijuxdna/sortmerna@sha256:2021b21d075d06404339ec019b9729f2dfb820685c86835df654c2fb7d8b447c` | `4.3.7` | paper:https://academic.oup.com/bioinformatics/article/28/24/3211/246053 | `production` |
| `star` | `transform` | `fastq.index_reference` | `bijuxdna/star:2.7.11b` | `2.7.11b` | paper:https://academic.oup.com/bioinformatics/article/29/1/15/272537 | `production` |
| `trim_galore` | `trimmer` | `fastq.trim_reads` | `bijuxdna/trim_galore@sha256:f323405a5a0ba19bbdae765dd4269e9156c415993977996079b63c5eb5bb0a61` | `0.6.10` | software:https://doi.org/10.5281/zenodo.7598955 | `production` |
| `trimmomatic` | `trimmer` | `fastq.trim_reads` | `bijuxdna/trimmomatic@sha256:41c0d161444ee7bb6b36ead3bbceb998af611be6ead6784231c5440e092bd5a4` | `0.39` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC4103590/ | `production` |
| `umi_tools` | `transform` | `fastq.extract_umis` | `bijuxdna/umi_tools@sha256:b2913af8c02c1eeea5de7a4b5c120f65e2003b90479c8873f0ec37689d36296c` | `1.1.6` | paper:https://genome.cshlp.org/content/27/3/491 | `production` |
| `verifybamid2` | `transform` | `bam.contamination` | `bijuxdna/verifybamid2:2.0.1` | `2.0.1` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC7050530/ | `production` |
| `vsearch` | `transform` | `fastq.cluster_otus, fastq.merge_pairs, fastq.remove_chimeras` | `bijuxdna/vsearch@sha256:c16ef98d6fd67ac0b8eea3ebb4f3dc6df9c582d6f838317d5f6ccc7a09e60bb3` | `2.28.1` | paper:https://pmc.ncbi.nlm.nih.gov/articles/PMC5075697/ | `production` |
| `yleaf` | `transform` | `bam.haplogroups, bam.sex` | `bijuxdna/yleaf:3.0.3` | `3.0.3` | paper:https://academic.oup.com/mbe/article/35/5/1291/4922696 | `production` |
