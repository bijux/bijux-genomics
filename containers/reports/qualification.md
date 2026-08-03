<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: containers/contracts/qualification_report.py -->
# Container evidence qualification

Qualification is derived from independent evidence dimensions. A current image
or successful command does not become a scientific or release claim.

## Summary

- Catalog tools: **89**
- Qualified: **0**
- Retained but unverified: **87**
- Unsupported: **2**
- Scientific, benchmark, security, and release evidence remain mandatory
  even after build, runtime, and real-data execution pass.

## Dimension coverage

| Dimension | Retained tools with passing evidence |
|---|---:|
| `catalog` | 87 |
| `source` | 87 |
| `build` | 87 |
| `runtime` | 87 |
| `stage` | 67 |
| `real_data` | 67 |
| `science` | 0 |
| `benchmark` | 0 |
| `security` | 0 |
| `release` | 0 |
| `freshness` | 87 |

## Tool matrix

`pass` means evidence exists for that dimension; `missing` means no claim is
permitted. Unsupported tools use `n/a`.

| Tool | Version | Disposition | Build | Runtime | Real data | Science | Benchmark | Security | Release | Qualification |
|---|---|---|---|---|---|---|---|---|---|---|
| `adapterremoval` | `2.3.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `addeam` | `1.0.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `alientrimmer` | `3.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `angsd` | `0.941` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `atropos` | `1.1.31` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `authenticct` | `1.0.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bamtools` | `2.5.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bamutil` | `1.0.15` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bayeshammer` | `4.2.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bbduk` | `39.06` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bbmerge` | `39.06` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bcftools` | `1.20` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `beagle` | `5.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bedtools` | `2.31.1` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bijux_dna` | `0.1.0` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `bowtie2` | `2.5.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bowtie2_build` | `2.5.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bracken` | `2.9` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `bwa` | `0.7.17` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `centrifuge` | `1.0.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `clumpify` | `39.06` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `contammix` | `1.0.11` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `cutadapt` | `4.9` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `dada2` | `1.30.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `damageprofiler` | `1.1` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `diamond` | `2.1.10` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `dustmasker` | `2.15.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `eagle` | `2.4.1` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `eigensoft` | `8.0.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `fastp` | `0.23.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `fastq_scan` | `1.0.1` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `fastq_screen` | `0.15.3` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `fastqc` | `0.12.1` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `fastqvalidator` | `0.1.1` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `fastuniq` | `1.1` | unsupported | n/a | n/a | n/a | n/a | n/a | n/a | n/a | **unsupported** |
| `fastx_clipper` | `0.0.14` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `flash2` | `2.2.00` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `fqtools` | `2.3` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `gatk` | `4.6.2.0` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `germline` | `1.5.3` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `glimpse` | `1.1.1` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `ibdhap` | `3.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `ibdne` | `23Apr20.ae9` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `ibdseq` | `r1206` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `impute5` | `1.2.0` | unsupported | n/a | n/a | n/a | n/a | n/a | n/a | n/a | **unsupported** |
| `kaiju` | `1.10.0` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `king` | `2.3.0` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `kraken2` | `2.1.3` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `krakenuniq` | `1.0.4` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `leehom` | `1.2.18` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `lighter` | `1.1.2` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `mapdamage2` | `2.2.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `metaphlan` | `4.1.1` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `minimac4` | `4.1.6` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `mosdepth` | `0.3.11` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `multiqc` | `1.24` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `musket` | `1.1` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `ngsbriggs` | `0.01` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `pear` | `0.9.6` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `picard` | `3.3.0` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `plink` | `1.90b7.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `plink2` | `2.00a6` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `pmdtools` | `0.60` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `preseq` | `3.2.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `prinseq` | `1.2.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `pydamage` | `1.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `qualimap` | `2.3` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `rcorrector` | `1.0.7` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `rxy` | `1.0.0` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `samtools` | `1.20` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `schmutzi` | `1.5.4` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `seqfu` | `1.26.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `seqkit` | `2.8.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `seqkit_stats` | `2.8.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `seqprep` | `1.3.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `seqpurge` | `2026_06` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `seqtk` | `1.4` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `shapeit` | `4.2.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `shapeit5` | `5.1.1` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `skewer` | `0.2.2` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `sortmerna` | `4.3.7` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `spades` | `4.2.0` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `star` | `2.7.11b` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `trim_galore` | `0.6.10` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `trimmomatic` | `0.39` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `umi_tools` | `1.1.6` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `verifybamid2` | `2.0.1` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |
| `vsearch` | `2.29.3` | retained | pass | pass | pass | missing | missing | missing | missing | **unverified** |
| `yleaf` | `4.1.4` | retained | pass | pass | missing | missing | missing | missing | missing | **unverified** |

The machine-readable companion is
[qualification.json](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/qualification.json). Real-run details and failures are in
the [run evidence report](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/run-evidence.md).
