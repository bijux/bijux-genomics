<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: python3 containers/runs/manage.py render-report -->
# LUNARC container run evidence

This report indexes the final successful immutable execution admitted for each
tool. A successful process is not scientific validation, and no tool is qualified
by this report.

## Archive boundary

- Final governed runs: **70**
- Tools represented: **70**
- Execution status: **passed 70**
- Scientific status: **not_assessed 70**

Superseded successful, failed, and incomplete attempts are summarized in the
checksummed attempt ledger without presenting them as current evidence. Their raw
results remain on the governed execution site. Large results stay there; each final
run directory records paths, sizes, and SHA-256 digests.

The machine-readable [attempt ledger](../runs/attempt-ledger.json) preserves
the identities and lock hashes of superseded attempts.

## Evidence by tool

| Tool | Final run | Corpus | Declared stage | Execution | Scientific claim |
|---|---|---|---|---|---|
| `adapterremoval` | [run-1002](../runs/run-1002-adapterremoval/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.trim_reads` | passed | not_assessed |
| `addeam` | [run-1096](../runs/run-1096-addeam/) | `ena-prjeb22390-err2112767` | `bam.damage_profile` | passed | not_assessed |
| `alientrimmer` | [run-1091](../runs/run-1091-alientrimmer/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.trim_reads` | passed | not_assessed |
| `angsd` | [run-1094](../runs/run-1094-angsd/) | `ena-prjeb22390-err2112767` | `bam.genotyping` | passed | not_assessed |
| `atropos` | [run-1098](../runs/run-1098-atropos/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.trim_reads` | passed | not_assessed |
| `authenticct` | [run-1093](../runs/run-1093-authenticct/) | `ena-prjeb22390-err2112767` | `bam.authenticity` | passed | not_assessed |
| `bamtools` | [run-1086](../runs/run-1086-bamtools/) | `ena-prjeb10854-err1021817-read-prefix-100000-equcab3` | `bam.filter` | passed | not_assessed |
| `bamutil` | [run-1087](../runs/run-1087-bamutil/) | `ena-prjeb10854-err1021817-read-prefix-100000-equcab3` | `bam.overlap_correction` | passed | not_assessed |
| `bayeshammer` | [run-1099](../runs/run-1099-bayeshammer/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.correct_errors` | passed | not_assessed |
| `bbduk` | [run-1006](../runs/run-1006-bbduk/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.trim` | passed | not_assessed |
| `bbmerge` | [run-1007](../runs/run-1007-bbmerge/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.merge_pairs` | passed | not_assessed |
| `bcftools` | [run-1140](../runs/run-1140-bcftools/) | `ena-prjeb22390-err2112767-bam` | `bam.genotyping` | passed | not_assessed |
| `beagle` | [run-1062](../runs/run-1062-beagle/) | `1000-genomes-phase3-chr22-biallelic-100-samples-full-chromosome` | `vcf.phasing` | passed | not_assessed |
| `bedtools` | [run-1074](../runs/run-1074-bedtools/) | `ena-prjeb22390-err2112767-bam` | `bam.coverage` | passed | not_assessed |
| `bowtie2` | [run-1101](../runs/run-1101-bowtie2/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `bam.align` | passed | not_assessed |
| `bowtie2_build` | [run-1100](../runs/run-1100-bowtie2_build/) | `equcab3-nc-009144.3-first-megabase` | `fastq.index_reference` | passed | not_assessed |
| `bracken` | [run-1104](../runs/run-1104-bracken/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `container.runtime` | passed | not_assessed |
| `bwa` | [run-1004](../runs/run-1004-bwa/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `bam.align` | passed | not_assessed |
| `centrifuge` | [run-1106](../runs/run-1106-centrifuge/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.screen_taxonomy` | passed | not_assessed |
| `clumpify` | [run-1008](../runs/run-1008-clumpify/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.deduplicate` | passed | not_assessed |
| `contammix` | [run-1108](../runs/run-1108-contammix/) | `ena-prjeb22390-err2112767` | `bam.contamination` | passed | not_assessed |
| `cutadapt` | [run-1109](../runs/run-1109-cutadapt/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.normalize_primers` | passed | not_assessed |
| `dada2` | [run-1060](../runs/run-1060-dada2/) | `mothur-miseq-sop-f3d0` | `fastq.infer_asvs` | passed | not_assessed |
| `damageprofiler` | [run-1115](../runs/run-1115-damageprofiler/) | `ena-prjeb22390-err2112767` | `bam.damage` | passed | not_assessed |
| `diamond` | [run-1117](../runs/run-1117-diamond/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `container.runtime` | passed | not_assessed |
| `dustmasker` | [run-1055](../runs/run-1055-dustmasker/) | `equcab3-nc_009144.3-first-megabase` | `reference.mask_low_complexity` | passed | not_assessed |
| `eagle` | [run-1036](../runs/run-1036-eagle/) | `1000-genomes-phase3-chr22-biallelic` | `vcf.phasing` | passed | not_assessed |
| `eigensoft` | [run-1070](../runs/run-1070-eigensoft/) | `1000-genomes-phase3-chr22-biallelic-100-samples-full-chromosome` | `vcf.principal_components` | passed | not_assessed |
| `fastp` | [run-1075](../runs/run-1075-fastp/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.filter_low_complexity` | passed | not_assessed |
| `fastq_scan` | [run-1118](../runs/run-1118-fastq_scan/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.profile_overrepresented_sequences` | passed | not_assessed |
| `fastqc` | [run-1076](../runs/run-1076-fastqc/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.detect_adapters` | passed | not_assessed |
| `fastqvalidator` | [run-1077](../runs/run-1077-fastqvalidator/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.validate_reads` | passed | not_assessed |
| `fastuniq` | [run-1023](../runs/run-1023-fastuniq/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.deduplicate` | passed | not_assessed |
| `fastx_clipper` | [run-1119](../runs/run-1119-fastx_clipper/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.trim_reads` | passed | not_assessed |
| `flash2` | [run-1120](../runs/run-1120-flash2/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.merge_pairs` | passed | not_assessed |
| `fqtools` | [run-1121](../runs/run-1121-fqtools/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.validate_reads` | passed | not_assessed |
| `germline` | [run-1069](../runs/run-1069-germline/) | `1000-genomes-phase3-chr22-biallelic-100-samples-full-chromosome` | `vcf.ibd_segments` | passed | not_assessed |
| `ibdhap` | [run-1067](../runs/run-1067-ibdhap/) | `1000-genomes-phase3-chr22-biallelic-100-samples-full-chromosome` | `vcf.ibd_state_probabilities` | passed | not_assessed |
| `ibdne` | [run-1053](../runs/run-1053-ibdne/) | `1000-genomes-phase3-chr22-100-samples-ibdseq-segments-full-chromosome` | `vcf.demography` | passed | not_assessed |
| `ibdseq` | [run-1052](../runs/run-1052-ibdseq/) | `1000-genomes-phase3-chr22-biallelic-100-samples-full-chromosome` | `vcf.ibd` | passed | not_assessed |
| `kraken2` | [run-1102](../runs/run-1102-kraken2/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.screen_taxonomy` | passed | not_assessed |
| `leehom` | [run-1016](../runs/run-1016-leehom/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `adapter-trimming-and-merging` | passed | not_assessed |
| `mapdamage2` | [run-1050](../runs/run-1050-mapdamage2/) | `ena-prjeb22390-err2112767` | `bam.damage` | passed | not_assessed |
| `mosdepth` | [run-1088](../runs/run-1088-mosdepth/) | `ena-prjeb10854-err1021817-read-prefix-100000-equcab3` | `bam.coverage` | passed | not_assessed |
| `multiqc` | [run-1129](../runs/run-1129-multiqc/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.report_qc` | passed | not_assessed |
| `musket` | [run-1025](../runs/run-1025-musket/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.correct_errors` | passed | not_assessed |
| `ngsbriggs` | [run-1046](../runs/run-1046-ngsbriggs/) | `ena-prjeb22390-err2112767-ngsbriggs-input` | `bam.damage` | passed | not_assessed |
| `pear` | [run-1024](../runs/run-1024-pear/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.merge_pairs` | passed | not_assessed |
| `picard` | [run-1138](../runs/run-1138-picard/) | `ena-prjeb22390-err2112767` | `bam.mapping_summary` | passed | not_assessed |
| `plink` | [run-1066](../runs/run-1066-plink/) | `1000-genomes-phase3-chr22-biallelic-100-samples-full-chromosome` | `vcf.allele_frequency` | passed | not_assessed |
| `plink2` | [run-1064](../runs/run-1064-plink2/) | `1000-genomes-phase3-chr22-biallelic-100-samples-full-chromosome` | `vcf.export_ped` | passed | not_assessed |
| `pmdtools` | [run-1137](../runs/run-1137-pmdtools/) | `ena-prjeb22390-err2112767` | `bam.authenticity` | passed | not_assessed |
| `preseq` | [run-1136](../runs/run-1136-preseq/) | `ena-prjeb22390-err2112767` | `bam.complexity` | passed | not_assessed |
| `prinseq` | [run-1018](../runs/run-1018-prinseq/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `low-complexity-filter` | passed | not_assessed |
| `pydamage` | [run-1134](../runs/run-1134-pydamage/) | `ena-prjeb22390-err2112767` | `bam.damage` | passed | not_assessed |
| `qualimap` | [run-1030](../runs/run-1030-qualimap/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `bam.quality_control` | passed | not_assessed |
| `samtools` | [run-1112](../runs/run-1112-samtools/) | `ena-prjeb22390-err2112767` | `bam.validate` | passed | not_assessed |
| `seqfu` | [run-1031](../runs/run-1031-seqfu/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.profile_read_lengths` | passed | not_assessed |
| `seqkit` | [run-1071](../runs/run-1071-seqkit/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.profile_reads` | passed | not_assessed |
| `seqkit_stats` | [run-1072](../runs/run-1072-seqkit_stats/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.profile_read_lengths` | passed | not_assessed |
| `seqprep` | [run-1082](../runs/run-1082-seqprep/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.merge_pairs` | passed | not_assessed |
| `seqpurge` | [run-1012](../runs/run-1012-seqpurge/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `adapter-trimming` | passed | not_assessed |
| `seqtk` | [run-1073](../runs/run-1073-seqtk/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.validate_reads` | passed | not_assessed |
| `shapeit` | [run-1037](../runs/run-1037-shapeit/) | `1000-genomes-phase3-chr22-biallelic` | `vcf.phasing` | passed | not_assessed |
| `shapeit5` | [run-1033](../runs/run-1033-shapeit5/) | `1000-genomes-phase3-chr22` | `vcf.phasing` | passed | not_assessed |
| `skewer` | [run-1079](../runs/run-1079-skewer/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.trim_reads` | passed | not_assessed |
| `spades` | [run-1135](../runs/run-1135-spades/) | `mothur-miseq-sop-f3d0` | `container.runtime` | passed | not_assessed |
| `trim_galore` | [run-1124](../runs/run-1124-trim_galore/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.trim_reads` | passed | not_assessed |
| `trimmomatic` | [run-1125](../runs/run-1125-trimmomatic/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.trim_reads` | passed | not_assessed |
| `vsearch` | [run-1084](../runs/run-1084-vsearch/) | `ena-prjeb10854-err1021817-read-prefix-100000` | `fastq.cluster_otus` | passed | not_assessed |

## Interpretation

`passed` means the archived Slurm execution completed its declared runtime
contract. `not_assessed` means no predeclared scientific acceptance threshold
was admitted for that run. Release readiness additionally requires reproducible
build, security, benchmark, and freshness evidence; none of those claims can be
inferred from process exit status.

The machine-readable companion is [run-evidence.json](run-evidence.json).
