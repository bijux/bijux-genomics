<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-containers::evidence::qualification_registry -->
# Container run index

Run IDs are permanent protocols; scheduler jobs and retries are attempts under those IDs.

- Registered protocols: **159**
- Active protocols: **68**
- Allocated protocols: **21**
- Historical protocols: **67**
- Reserved IDs: **2**

| Run | State | Stage | Tools | Citation | Evidence |
|---|---|---|---|---|---|
| `run-1000` | `campaign` | `container.image_campaign` | `adapterremoval`, `addeam`, `alientrimmer`, `atropos`, `authenticct`, `bamtools`, `bamutil`, `bayeshammer`, `bbduk`, `bbmerge`, `bcftools`, `beagle`, `bedtools`, `bijux_dna`, `bowtie2`, `bowtie2_build`, `bracken`, `bwa`, `centrifuge`, `clumpify`, `contammix`, `cutadapt`, `dada2`, `damageprofiler`, `diamond`, `dustmasker`, `eagle`, `eigensoft`, `fastp`, `fastq_scan`, `fastq_screen`, `fastqc`, `fastqvalidator`, `fastx_clipper`, `flash2`, `fqtools`, `gatk`, `germline`, `glimpse`, `ibdhap`, `ibdne`, `ibdseq`, `kaiju`, `king`, `kraken2`, `krakenuniq`, `leehom`, `lighter`, `mapdamage2`, `metaphlan`, `minimac4`, `mosdepth`, `multiqc`, `musket`, `pear`, `picard`, `plink`, `plink2`, `pmdtools`, `preseq`, `prinseq`, `pydamage`, `qualimap`, `rcorrector`, `rxy`, `samtools`, `schmutzi`, `seqfu`, `seqkit`, `seqkit_stats`, `seqprep`, `seqpurge`, `seqtk`, `shapeit`, `shapeit5`, `skewer`, `sortmerna`, `spades`, `star`, `trim_galore`, `trimmomatic`, `umi_tools`, `verifybamid2`, `vsearch`, `yleaf` | `bijux-container-run-1000` | indexed |
| `run-1001` | `historical` | `fastq.merge_pairs` | `adapterremoval` | `bijux-container-run-1001` | indexed |
| `run-1002` | `active` | `fastq.merge_pairs` | `adapterremoval` | `bijux-container-run-1002` | indexed |
| `run-1003` | `historical` | `bam.align` | `bwa` | `bijux-container-run-1003` | indexed |
| `run-1004` | `active` | `bam.align` | `bwa` | `bijux-container-run-1004` | indexed |
| `run-1005` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1005` | indexed |
| `run-1006` | `active` | `fastq.filter_low_complexity` | `bbduk` | `bijux-container-run-1006` | indexed |
| `run-1007` | `active` | `fastq.merge_pairs` | `bbmerge` | `bijux-container-run-1007` | indexed |
| `run-1008` | `active` | `fastq.remove_duplicates` | `clumpify` | `bijux-container-run-1008` | indexed |
| `run-1009` | `historical` | `fastq.validate_reads` | `fqtools` | `bijux-container-run-1009` | indexed |
| `run-1010` | `historical` | `fastq.profile_overrepresented_sequences` | `fastq_scan` | `bijux-container-run-1010` | indexed |
| `run-1011` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1011` | indexed |
| `run-1012` | `active` | `container.runtime` | `seqpurge` | `bijux-container-run-1012` | indexed |
| `run-1013` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1013` | indexed |
| `run-1014` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1014` | indexed |
| `run-1015` | `historical` | `fastq.remove_duplicates` | `fastuniq` | `bijux-container-run-1015` | indexed |
| `run-1016` | `active` | `fastq.merge_pairs` | `leehom` | `bijux-container-run-1016` | indexed |
| `run-1017` | `historical` | `fastq.merge_pairs` | `pear` | `bijux-container-run-1017` | indexed |
| `run-1018` | `active` | `fastq.filter_low_complexity` | `prinseq` | `bijux-container-run-1018` | indexed |
| `run-1019` | `historical` | `fastq.validate_reads` | `fqtools` | `bijux-container-run-1019` | indexed |
| `run-1020` | `historical` | `fastq.validate_reads` | `fqtools` | `bijux-container-run-1020` | indexed |
| `run-1021` | `historical` | `fastq.validate_reads` | `fqtools` | `bijux-container-run-1021` | indexed |
| `run-1022` | `historical` | `fastq.validate_reads` | `fqtools` | `bijux-container-run-1022` | indexed |
| `run-1023` | `allocated` | `fastq.remove_duplicates` | `fastuniq` | `bijux-container-run-1023` | indexed |
| `run-1024` | `active` | `fastq.merge_pairs` | `pear` | `bijux-container-run-1024` | indexed |
| `run-1025` | `active` | `fastq.correct_errors` | `musket` | `bijux-container-run-1025` | indexed |
| `run-1026` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1026` | indexed |
| `run-1027` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1027` | indexed |
| `run-1028` | `historical` | `container.runtime` | `qualimap` | `bijux-container-run-1028` | indexed |
| `run-1029` | `historical` | `container.runtime` | `qualimap` | `bijux-container-run-1029` | indexed |
| `run-1030` | `active` | `container.runtime` | `qualimap` | `bijux-container-run-1030` | indexed |
| `run-1031` | `active` | `fastq.profile_read_lengths` | `seqfu` | `bijux-container-run-1031` | indexed |
| `run-1032` | `historical` | `vcf.phasing` | `eagle` | `bijux-container-run-1032` | indexed |
| `run-1033` | `active` | `vcf.phasing` | `shapeit5` | `bijux-container-run-1033` | indexed |
| `run-1034` | `historical` | `vcf.phasing` | `eagle` | `bijux-container-run-1034` | indexed |
| `run-1035` | `historical` | `vcf.call` | `bcftools` | `bijux-container-run-1035` | indexed |
| `run-1036` | `active` | `vcf.phasing` | `eagle` | `bijux-container-run-1036` | indexed |
| `run-1037` | `active` | `vcf.phasing` | `shapeit` | `bijux-container-run-1037` | indexed |
| `run-1038` | `historical` | `vcf.call` | `bcftools` | `bijux-container-run-1038` | indexed |
| `run-1039` | `historical` | `vcf.ibd` | `ibdseq` | `bijux-container-run-1039` | indexed |
| `run-1040` | `historical` | `vcf.demography` | `ibdne` | `bijux-container-run-1040` | indexed |
| `run-1041` | `historical` | `vcf.demography` | `ibdne` | `bijux-container-run-1041` | indexed |
| `run-1042` | `historical` | `vcf.demography` | `ibdne` | `bijux-container-run-1042` | indexed |
| `run-1043` | `historical` | `bam.damage` | `ngsbriggs` | `bijux-container-run-1043` | indexed |
| `run-1044` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1044` | indexed |
| `run-1045` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1045` | indexed |
| `run-1046` | `allocated` | `bam.damage` | `ngsbriggs` | `bijux-container-run-1046` | indexed |
| `run-1047` | `historical` | `bam.damage` | `mapdamage2` | `bijux-container-run-1047` | indexed |
| `run-1048` | `historical` | `bam.damage` | `mapdamage2` | `bijux-container-run-1048` | indexed |
| `run-1049` | `historical` | `bam.damage` | `mapdamage2` | `bijux-container-run-1049` | indexed |
| `run-1050` | `active` | `bam.damage` | `mapdamage2` | `bijux-container-run-1050` | indexed |
| `run-1051` | `historical` | `vcf.call` | `bcftools` | `bijux-container-run-1051` | indexed |
| `run-1052` | `active` | `vcf.ibd` | `ibdseq` | `bijux-container-run-1052` | indexed |
| `run-1053` | `active` | `vcf.demography` | `ibdne` | `bijux-container-run-1053` | indexed |
| `run-1054` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1054` | indexed |
| `run-1055` | `active` | `container.runtime` | `dustmasker` | `bijux-container-run-1055` | indexed |
| `run-1056` | `historical` | `fastq.infer_asvs` | `dada2` | `bijux-container-run-1056` | indexed |
| `run-1057` | `historical` | `fastq.infer_asvs` | `dada2` | `bijux-container-run-1057` | indexed |
| `run-1058` | `historical` | `fastq.infer_asvs` | `dada2` | `bijux-container-run-1058` | indexed |
| `run-1059` | `historical` | `fastq.infer_asvs` | `dada2` | `bijux-container-run-1059` | indexed |
| `run-1060` | `active` | `fastq.infer_asvs` | `dada2` | `bijux-container-run-1060` | indexed |
| `run-1061` | `historical` | `vcf.ibd` | `ibdhap` | `bijux-container-run-1061` | indexed |
| `run-1062` | `active` | `vcf.imputation_metrics` | `beagle` | `bijux-container-run-1062` | indexed |
| `run-1063` | `historical` | `vcf.admixture` | `plink2` | `bijux-container-run-1063` | indexed |
| `run-1064` | `active` | `vcf.admixture` | `plink2` | `bijux-container-run-1064` | indexed |
| `run-1065` | `historical` | `vcf.ibd` | `germline` | `bijux-container-run-1065` | indexed |
| `run-1066` | `active` | `vcf.admixture` | `plink` | `bijux-container-run-1066` | indexed |
| `run-1067` | `active` | `vcf.ibd` | `ibdhap` | `bijux-container-run-1067` | indexed |
| `run-1068` | `historical` | `vcf.ibd` | `germline` | `bijux-container-run-1068` | indexed |
| `run-1069` | `active` | `vcf.ibd` | `germline` | `bijux-container-run-1069` | indexed |
| `run-1070` | `active` | `vcf.pca` | `eigensoft` | `bijux-container-run-1070` | indexed |
| `run-1071` | `active` | `fastq.filter_reads` | `seqkit` | `bijux-container-run-1071` | indexed |
| `run-1072` | `active` | `fastq.profile_reads` | `seqkit_stats` | `bijux-container-run-1072` | indexed |
| `run-1073` | `active` | `fastq.validate_reads` | `seqtk` | `bijux-container-run-1073` | indexed |
| `run-1074` | `active` | `bam.coverage` | `bedtools` | `bijux-container-run-1074` | indexed |
| `run-1075` | `active` | `fastq.filter_low_complexity` | `fastp` | `bijux-container-run-1075` | indexed |
| `run-1076` | `active` | `fastq.detect_adapters` | `fastqc` | `bijux-container-run-1076` | indexed |
| `run-1077` | `active` | `fastq.validate_reads` | `fastqvalidator` | `bijux-container-run-1077` | indexed |
| `run-1078` | `historical` | `fastq.merge_pairs` | `flash2` | `bijux-container-run-1078` | indexed |
| `run-1079` | `active` | `fastq.trim_reads` | `skewer` | `bijux-container-run-1079` | indexed |
| `run-1080` | `historical` | `fastq.trim_reads` | `fastx_clipper` | `bijux-container-run-1080` | indexed |
| `run-1081` | `reserved` | `container.run_reservation` |  | `bijux-container-run-1081` | indexed |
| `run-1082` | `active` | `fastq.merge_pairs` | `seqprep` | `bijux-container-run-1082` | indexed |
| `run-1083` | `historical` | `fastq.trim_reads` | `fastx_clipper` | `bijux-container-run-1083` | indexed |
| `run-1084` | `active` | `fastq.cluster_otus` | `vsearch` | `bijux-container-run-1084` | indexed |
| `run-1085` | `reserved` | `container.run_reservation` |  | `bijux-container-run-1085` | indexed |
| `run-1086` | `active` | `bam.filter` | `bamtools` | `bijux-container-run-1086` | indexed |
| `run-1087` | `active` | `bam.overlap_correction` | `bamutil` | `bijux-container-run-1087` | indexed |
| `run-1088` | `active` | `bam.coverage` | `mosdepth` | `bijux-container-run-1088` | indexed |
| `run-1089` | `historical` | `fastq.trim_reads` | `alientrimmer` | `bijux-container-run-1089` | indexed |
| `run-1090` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1090` | indexed |
| `run-1091` | `active` | `fastq.trim_reads` | `alientrimmer` | `bijux-container-run-1091` | indexed |
| `run-1092` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1092` | indexed |
| `run-1093` | `active` | `bam.authenticity` | `authenticct` | `bijux-container-run-1093` | indexed |
| `run-1094` | `allocated` | `bam.kinship` | `angsd` | `bijux-container-run-1094` | indexed |
| `run-1095` | `historical` | `bam.damage` | `addeam` | `bijux-container-run-1095` | indexed |
| `run-1096` | `active` | `bam.damage` | `addeam` | `bijux-container-run-1096` | indexed |
| `run-1097` | `historical` | `fastq.trim_reads` | `atropos` | `bijux-container-run-1097` | indexed |
| `run-1098` | `active` | `fastq.trim_reads` | `atropos` | `bijux-container-run-1098` | indexed |
| `run-1099` | `active` | `fastq.correct_errors` | `bayeshammer` | `bijux-container-run-1099` | indexed |
| `run-1100` | `active` | `fastq.index_reference` | `bowtie2_build` | `bijux-container-run-1100` | indexed |
| `run-1101` | `active` | `bam.align` | `bowtie2` | `bijux-container-run-1101` | indexed |
| `run-1102` | `active` | `fastq.screen_taxonomy` | `kraken2` | `bijux-container-run-1102` | indexed |
| `run-1103` | `historical` | `container.runtime` | `bracken` | `bijux-container-run-1103` | indexed |
| `run-1104` | `active` | `container.runtime` | `bracken` | `bijux-container-run-1104` | indexed |
| `run-1105` | `historical` | `fastq.screen_taxonomy` | `centrifuge` | `bijux-container-run-1105` | indexed |
| `run-1106` | `active` | `fastq.screen_taxonomy` | `centrifuge` | `bijux-container-run-1106` | indexed |
| `run-1107` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1107` | indexed |
| `run-1108` | `active` | `bam.contamination` | `contammix` | `bijux-container-run-1108` | indexed |
| `run-1109` | `active` | `fastq.normalize_primers` | `cutadapt` | `bijux-container-run-1109` | indexed |
| `run-1110` | `historical` | `bam.authenticity` | `damageprofiler` | `bijux-container-run-1110` | indexed |
| `run-1111` | `historical` | `bam.coverage` | `samtools` | `bijux-container-run-1111` | indexed |
| `run-1112` | `active` | `bam.coverage` | `samtools` | `bijux-container-run-1112` | indexed |
| `run-1113` | `historical` | `bam.authenticity` | `damageprofiler` | `bijux-container-run-1113` | indexed |
| `run-1114` | `historical` | `bam.authenticity` | `damageprofiler` | `bijux-container-run-1114` | indexed |
| `run-1115` | `active` | `bam.authenticity` | `damageprofiler` | `bijux-container-run-1115` | indexed |
| `run-1116` | `historical` | `container.runtime` | `diamond` | `bijux-container-run-1116` | indexed |
| `run-1117` | `active` | `container.runtime` | `diamond` | `bijux-container-run-1117` | indexed |
| `run-1118` | `active` | `fastq.profile_overrepresented_sequences` | `fastq_scan` | `bijux-container-run-1118` | indexed |
| `run-1119` | `active` | `fastq.trim_reads` | `fastx_clipper` | `bijux-container-run-1119` | indexed |
| `run-1120` | `active` | `fastq.merge_pairs` | `flash2` | `bijux-container-run-1120` | indexed |
| `run-1121` | `active` | `fastq.validate_reads` | `fqtools` | `bijux-container-run-1121` | indexed |
| `run-1122` | `historical` | `fastq.trim_reads` | `trim_galore` | `bijux-container-run-1122` | indexed |
| `run-1123` | `historical` | `fastq.trim_reads` | `trimmomatic` | `bijux-container-run-1123` | indexed |
| `run-1124` | `active` | `fastq.trim_reads` | `trim_galore` | `bijux-container-run-1124` | indexed |
| `run-1125` | `active` | `fastq.trim_reads` | `trimmomatic` | `bijux-container-run-1125` | indexed |
| `run-1126` | `historical` | `bam.qc_pre` | `multiqc` | `bijux-container-run-1126` | indexed |
| `run-1127` | `historical` | `container.runtime` | `spades` | `bijux-container-run-1127` | indexed |
| `run-1128` | `historical` | `bam.qc_pre` | `multiqc` | `bijux-container-run-1128` | indexed |
| `run-1129` | `active` | `bam.qc_pre` | `multiqc` | `bijux-container-run-1129` | indexed |
| `run-1130` | `historical` | `bam.damage` | `pydamage` | `bijux-container-run-1130` | indexed |
| `run-1131` | `historical` | `bam.damage` | `pydamage` | `bijux-container-run-1131` | indexed |
| `run-1132` | `historical` | `bam.damage` | `pydamage` | `bijux-container-run-1132` | indexed |
| `run-1133` | `historical` | `bam.damage` | `pydamage` | `bijux-container-run-1133` | indexed |
| `run-1134` | `active` | `bam.damage` | `pydamage` | `bijux-container-run-1134` | indexed |
| `run-1135` | `active` | `container.runtime` | `spades` | `bijux-container-run-1135` | indexed |
| `run-1136` | `active` | `bam.complexity` | `preseq` | `bijux-container-run-1136` | indexed |
| `run-1137` | `active` | `bam.authenticity` | `pmdtools` | `bijux-container-run-1137` | indexed |
| `run-1138` | `active` | `bam.duplication_metrics` | `picard` | `bijux-container-run-1138` | indexed |
| `run-1139` | `active` | `fastq.correct_errors` | `lighter` | `bijux-container-run-1139` | indexed |
| `run-1140` | `active` | `vcf.call` | `bcftools` | `bijux-container-run-1140` | indexed |
| `run-1141` | `allocated` | `fastq.detect_duplicates_premerge` | `bijux_dna` | `bijux-container-run-1141` | not selected |
| `run-1142` | `allocated` | `container.runtime` | `fastq_screen` | `bijux-container-run-1142` | not selected |
| `run-1143` | `allocated` | `bam.recalibration` | `gatk` | `bijux-container-run-1143` | not selected |
| `run-1144` | `allocated` | `vcf.imputation_metrics` | `glimpse` | `bijux-container-run-1144` | not selected |
| `run-1145` | `allocated` | `fastq.screen_taxonomy` | `kaiju` | `bijux-container-run-1145` | not selected |
| `run-1146` | `allocated` | `bam.kinship` | `king` | `bijux-container-run-1146` | not selected |
| `run-1147` | `allocated` | `fastq.screen_taxonomy` | `krakenuniq` | `bijux-container-run-1147` | not selected |
| `run-1148` | `allocated` | `container.runtime` | `metaphlan` | `bijux-container-run-1148` | not selected |
| `run-1149` | `allocated` | `vcf.imputation_metrics` | `minimac4` | `bijux-container-run-1149` | not selected |
| `run-1150` | `allocated` | `fastq.correct_errors` | `rcorrector` | `bijux-container-run-1150` | not selected |
| `run-1151` | `allocated` | `bam.sex` | `rxy` | `bijux-container-run-1151` | not selected |
| `run-1152` | `allocated` | `bam.contamination` | `schmutzi` | `bijux-container-run-1152` | not selected |
| `run-1153` | `allocated` | `fastq.deplete_rrna` | `sortmerna` | `bijux-container-run-1153` | not selected |
| `run-1154` | `allocated` | `fastq.index_reference` | `star` | `bijux-container-run-1154` | not selected |
| `run-1155` | `allocated` | `fastq.extract_umis` | `umi_tools` | `bijux-container-run-1155` | not selected |
| `run-1156` | `allocated` | `bam.contamination` | `verifybamid2` | `bijux-container-run-1156` | not selected |
| `run-1157` | `allocated` | `bam.haplogroups` | `yleaf` | `bijux-container-run-1157` | not selected |
| `run-1158` | `allocated` | `vcf.imputation_metrics` | `impute5` | `bijux-container-run-1158` | not selected |
