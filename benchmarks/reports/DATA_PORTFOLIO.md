# Benchmark data portfolio audit

This report joins portfolio declarations to checksum-verified STUDY records. Data-index
eligibility proves governed file identity; it does not by itself admit comparative science.

- Source commit: `f62473c409b7cbd8c87dcc21040adf876f239718`
- Portfolio corpora: **16**
- Publishable-data ready: **7**
- Bound governed catalogs: **24**
- Unbound governed catalogs: **10**
- Verified bound bytes: **42415666966**
- Corpora without storage projections: **0**
- Corpora without compute projections: **5**
- Publishable contamination findings: **0**

## Corpus inventory and projections

| Corpus | Class | State | Samples | Catalogs | Verified bytes | Projected bytes | Projected CPU hours | Ready | Blockers |
|---|---|---|---:|---:|---:|---:|---:|---|---|
| `alignment-truth-panel` | production_real | source_defined | 0/10 | 0 | 0 | 1000000000000 | 5000 | no | source selection is not frozen by exact accessions; source-defined corpus has not been admitted |
| `amplicon-community-challenge` | production_real | source_defined | 0/10 | 0 | 0 | 25000000000 | 500 | no | source-defined corpus has not been admitted |
| `amplicon-stage-qualification` | qualification_real | admitted | 1/1 | 3 | 11778203 | 11778203 | unknown | yes | - |
| `ancient-dna-bam-diversity` | production_real | admitted | 10/10 | 2 | 5711625731 | 250000000000 | 3000 | yes | - |
| `ancient-dna-damage-controls` | production_real | source_defined | 0/10 | 0 | 0 | 100000000000 | 1500 | no | source-defined corpus has not been admitted |
| `ancient-horse-alignment-qualification` | qualification_real | admitted | 1/1 | 7 | 3020996176 | 3020996176 | unknown | yes | - |
| `ancient-population-panel` | production_real | source_defined | 0/10 | 0 | 0 | 200000000000 | 3000 | no | source selection is not frozen by exact accessions; source-defined corpus has not been admitted |
| `environmental-taxonomy-challenge` | production_real | source_defined | 0/10 | 0 | 0 | 100000000000 | 2000 | no | source-defined corpus has not been admitted |
| `equine-read-stage-qualification` | qualification_real | admitted | 1/1 | 4 | 27299073539 | 27299073539 | unknown | yes | - |
| `human-population-stage-qualification` | qualification_real | admitted | 100/100 | 5 | 478974528 | 478974528 | unknown | yes | - |
| `human-short-read-diversity` | production_real | admitted | 20/20 | 2 | 7950810128 | 5457128348 | 240 | yes | - |
| `masked-genotype-truth-panel` | production_real | source_defined | 0/100 | 0 | 0 | 750000000000 | 4000 | no | source selection is not frozen by exact accessions; source-defined corpus has not been admitted |
| `population-inference-truth-panel` | production_real | source_defined | 0/100 | 0 | 0 | 300000000000 | 2500 | no | source selection is not frozen by exact accessions; source-defined corpus has not been admitted |
| `read-processing-challenge` | production_real | source_defined | 0/10 | 0 | 0 | 100000000000 | 1000 | no | source-defined corpus has not been admitted |
| `variant-truth-panel` | production_real | source_defined | 0/10 | 0 | 0 | 1000000000000 | 5000 | no | source selection is not frozen by exact accessions; source-defined corpus has not been admitted |
| `viral-taxonomy-stage-qualification` | qualification_real | admitted | 1/1 | 2 | 2255133224 | 2255133224 | unknown | yes | - |

## Source selection closure

A source declaration is acquisition-ready only when every sample-bearing collection is frozen by exact accession.

| Corpus | Source specification | Selection groups | Samples | Selected/planned accessions | Frozen |
|---|---|---:|---:|---:|---|
| `alignment-truth-panel` | `benchmarks/corpora/alignment-truth-panel.toml` | 2 | 17 | 0/17 | no |
| `amplicon-community-challenge` | `benchmarks/corpora/amplicon-community-challenge.toml` | 2 | 11 | 11/11 | yes |
| `ancient-dna-bam-diversity` | `benchmarks/corpora/ancient-dna-bam-diversity.toml` | 2 | 10 | 10/10 | yes |
| `ancient-dna-damage-controls` | `benchmarks/corpora/ancient-dna-damage-controls.toml` | 4 | 21 | 31/31 | yes |
| `ancient-population-panel` | `benchmarks/corpora/ancient-population-panel.toml` | 1 | 100 | 0/100 | no |
| `environmental-taxonomy-challenge` | `benchmarks/corpora/environmental-taxonomy-challenge.toml` | 3 | 20 | 20/20 | yes |
| `human-short-read-diversity` | `benchmarks/corpora/human-short-read-diversity.toml` | 1 | 20 | 20/20 | yes |
| `masked-genotype-truth-panel` | `benchmarks/corpora/masked-genotype-truth-panel.toml` | 1 | 100 | 0/100 | no |
| `population-inference-truth-panel` | `benchmarks/corpora/population-inference-truth-panel.toml` | 1 | 100 | 0/100 | no |
| `read-processing-challenge` | `benchmarks/corpora/read-processing-challenge.toml` | 2 | 20 | 20/20 | yes |
| `variant-truth-panel` | `benchmarks/corpora/variant-truth-panel.toml` | 2 | 17 | 0/17 | no |

## Truth-role matrix

| Corpus | Evidence scope | Data types | Truth roles | Truth strategy |
|---|---|---|---|---|
| `alignment-truth-panel` | comparative | bam, fasta, fastq | alignment_truth | real_reference_material_with_orthogonal_truth_and_supplementary_simulation |
| `amplicon-community-challenge` | comparative | amplicon_reference, fastq | amplicon_truth | real_mock_communities_with_known_composition |
| `amplicon-stage-qualification` | qualification | fastq | amplicon_stage_qualification | observational_only |
| `ancient-dna-bam-diversity` | comparative | bam, fasta | bam_diversity | observational_alignment_diversity_with_separate_orthogonal_controls |
| `ancient-dna-damage-controls` | comparative | bam, fasta, fastq, orthogonal_assay, sample_metadata | ancient_damage_truth | paired_udg_treatment_labels_with_extraction_blank_and_observational_modern_negatives |
| `ancient-horse-alignment-qualification` | qualification | bam | ancient_bam_qualification | observational_only |
| `ancient-population-panel` | comparative | genotype_likelihoods, sample_metadata, vcf | ancient_population_diversity | real_cohort_with_independent_metadata_and_sensitivity_analysis |
| `environmental-taxonomy-challenge` | comparative | fastq, taxonomy_database | environmental_taxonomy_truth, host_and_rrna_challenge | real_edna_with_mock_community_and_orthogonal_assays |
| `equine-read-stage-qualification` | qualification | fasta, fastq | equine_read_stage_qualification | observational_only |
| `human-population-stage-qualification` | qualification | ibd_segments, vcf | human_population_stage_qualification | observational_only |
| `human-short-read-diversity` | comparative | fasta, fastq | short_read_diversity, ancient_and_modern_reads, single_and_paired_reads | observational_only |
| `masked-genotype-truth-panel` | comparative | genetic_map, reference_panel, sample_metadata, vcf | masked_genotype_truth | held_out_high_coverage_genotypes_with_panel_exclusion |
| `population-inference-truth-panel` | comparative | pedigree, sample_metadata, vcf | population_inference_truth | masked_population_metadata_and_pedigree_relationship_truth |
| `read-processing-challenge` | comparative | fastq | read_processing_truth, adapter_damage_complexity_challenge | real_reads_with_calibrated_spike_ins |
| `variant-truth-panel` | comparative | bam, genetic_map, reference_panel, vcf | variant_truth | independently_curated_genotypes_with_mendelian_consistency |
| `viral-taxonomy-stage-qualification` | qualification | fasta, taxonomy_database | viral_taxonomy_stage_qualification | observational_only |

## Bound catalog access matrix

Access terms are copied from each exact governed catalog. Legacy null or absent terms are reported explicitly rather than inferred.

| Corpus | Catalog | Present | Access | Redistribution | Access terms | Observed samples | Sample-count authority |
|---|---|---|---|---|---|---:|---|
| `amplicon-stage-qualification` | `ena-prjdb2953-drr021371` | yes | public | public-source-copy | not declared by governed catalog | 1 | catalog.sample_identity |
| `amplicon-stage-qualification` | `ena-prjdb4000-drr037766` | yes | public | public-source-copy | not declared by governed catalog | 1 | catalog.sample_identity |
| `amplicon-stage-qualification` | `mothur-miseq-sop-f3d0` | yes | public | public-tutorial-copy | not declared by governed catalog | 1 | catalog.sample_identity |
| `ancient-dna-bam-diversity` | `ancient-dna-bam-diversity` | yes | public | accession-referenced-derived-alignments | not declared by governed catalog | 10 | catalog.samples |
| `ancient-dna-bam-diversity` | `grch38-reference` | yes | public | public-reference-assembly | not declared by governed catalog | unknown | not available |
| `ancient-horse-alignment-qualification` | `ena-prjeb22390-err2112767-bam` | yes | public | accession-referenced-derived-alignment | not declared by governed catalog | 1 | catalog.sample_identity |
| `ancient-horse-alignment-qualification` | `ena-prjeb22390-err2112767-ngsbriggs-input` | yes | public | accession-referenced-derived-alignment | not declared by governed catalog | 1 | catalog.samples |
| `ancient-horse-alignment-qualification` | `ensembl-113-equcab3-refseq-genotyping-inputs-10mb` | yes | public | public-derived-reference-database | not declared by governed catalog | unknown | not available |
| `ancient-horse-alignment-qualification` | `ensembl-113-equcab3-refseq-known-sites-10mb` | yes | public | public-derived-reference-database | not declared by governed catalog | unknown | not available |
| `ancient-horse-alignment-qualification` | `ensembl-113-equcab3-variation` | yes | public | public-reference-database | not declared by governed catalog | unknown | not available |
| `ancient-horse-alignment-qualification` | `equcab3-gcf-002863925.1` | yes | public | public-reference-accession | not declared by governed catalog | unknown | not available |
| `ancient-horse-alignment-qualification` | `equus-mitochondrial-candidates` | yes | public | public-reference-sequences | not declared by governed catalog | unknown | not available |
| `equine-read-stage-qualification` | `ena-prjeb10854-err1021817-full` | yes | public | accession-referenced-source-copy | not declared by governed catalog | 1 | catalog.sample_identity |
| `equine-read-stage-qualification` | `ena-prjeb10854-err1021817-read-prefix-100000` | yes | public | accession-referenced-derived-subset | not declared by governed catalog | 1 | catalog.sample_identity |
| `equine-read-stage-qualification` | `ena-prjeb10854-err1021817-split-paired-fastq` | yes | public | accession-referenced-source-copy | not declared by governed catalog | 1 | catalog.sample_identity |
| `equine-read-stage-qualification` | `equcab3-nc_009144.3-first-megabase` | yes | public | public-derived-subset | not declared by governed catalog | unknown | not available |
| `human-population-stage-qualification` | `1000-genomes-phase3-chr22-100-samples-ibdseq-segments-full-chromosome` | yes | public | public-derived-analysis | not declared by governed catalog | 100 | catalog.sample_count |
| `human-population-stage-qualification` | `1000-genomes-phase3-chr22-biallelic-100-samples` | yes | public | public-derived-subset | not declared by governed catalog | 100 | catalog.sample_count |
| `human-population-stage-qualification` | `1000-genomes-phase3-chr22-biallelic-100-samples-full-chromosome` | yes | public | public-derived-subset | not declared by governed catalog | 100 | catalog.sample_count |
| `human-population-stage-qualification` | `1000-genomes-phase3-chr22-biallelic-20-22mb` | yes | public | public-derived-subset | not declared by governed catalog | unknown | not available |
| `human-population-stage-qualification` | `1000-genomes-phase3-chr22-ibdseq-segments` | yes | public | public-derived-result | not declared by governed catalog | 100 | catalog.sample_count |
| `human-short-read-diversity` | `grch38-reference` | yes | public | public-reference-assembly | not declared by governed catalog | unknown | not available |
| `human-short-read-diversity` | `human-short-read-diversity` | yes | public | accession-referenced | ENA public data access terms apply; publication identifies source accessions. | 20 | catalog.samples |
| `viral-taxonomy-stage-qualification` | `centrifuge-nc-038500.1-single-reference` | yes | public | public-reference-sequence | not declared by governed catalog | 1 | catalog.sample_identity |
| `viral-taxonomy-stage-qualification` | `kraken2-viral-20260626` | yes | public | public-reference-database | not declared by governed catalog | unknown | not available |

## Unbound governed assets

These records are checksum-verified holdings but are not portfolio admission evidence.

| Catalog | Access | Redistribution | Files | Bytes | Non-production suspicion | Reasons |
|---|---|---|---:|---:|---|---|
| `amplicon-community-challenge-extreme-mock-community` | public | accession-referenced-source-copy | 2 | 567380588 | no | - |
| `amplicon-community-challenge-hmp-amplicon-mock` | public | accession-referenced-source-copy | 10 | 32745634 | no | - |
| `ancient-dna-damage-controls-ena-prjeb19769-matched-udg-pairs` | public | accession-referenced-derived-alignments | 40 | 18171459349 | no | - |
| `ancient-dna-damage-controls-ncbi-prjna688065-extraction-blank` | public | accession-referenced-derived-alignments | 1 | 1018364 | no | - |
| `ancient-dna-damage-controls-public-modern-human-observational-controls` | public | accession-referenced-derived-alignments | 15 | 2442546526 | no | - |
| `corpus-01` | public | public-accession-referenced-source-copy | 30 | 3638085565 | no | - |
| `environmental-taxonomy-challenge-ena-tara-oceans` | public | accession-referenced-source-copy | 19 | 17167643384 | no | - |
| `environmental-taxonomy-challenge-hmp-mock-community` | public | accession-referenced-source-copy | 10 | 32745634 | no | - |
| `read-processing-challenge-ena-ancient-read-panel` | mixed-public | accession-referenced-source-copy | 15 | 1195539039 | no | - |
| `read-processing-challenge-seqc-reference-rna` | mixed-public | accession-referenced-source-copy | 20 | 14812606713 | no | - |

## Publishable contamination findings

None.
