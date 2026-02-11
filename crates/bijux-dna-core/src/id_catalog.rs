#![allow(missing_docs)]

pub const FASTQ_PREFIX: &str = "fastq.";
pub const BAM_PREFIX: &str = "bam.";
pub const CORE_PREFIX: &str = "core.";

pub const CORE_PREPARE_REFERENCE: &str = "core.prepare_reference";
pub const REPORT_AGGREGATE_STAGE: &str = "report.aggregate";
pub const REPORT_AGGREGATE_STEP: &str = "report.aggregate";

pub const FASTQ_PREPROCESS: &str = "fastq.preprocess";
pub const FASTQ_VALIDATE_PRE: &str = "fastq.validate_pre";
pub const FASTQ_DETECT_ADAPTERS: &str = "fastq.detect_adapters";
pub const FASTQ_TRIM: &str = "fastq.trim";
pub const FASTQ_FILTER: &str = "fastq.filter";
pub const FASTQ_DEDUPLICATE: &str = "fastq.deduplicate";
pub const FASTQ_LOW_COMPLEXITY: &str = "fastq.low_complexity";
pub const FASTQ_MERGE: &str = "fastq.merge";
pub const FASTQ_CORRECT: &str = "fastq.correct";
pub const FASTQ_QC_POST: &str = "fastq.qc_post";
pub const FASTQ_UMI: &str = "fastq.umi";
pub const FASTQ_SCREEN: &str = "fastq.screen";
pub const FASTQ_STATS_NEUTRAL: &str = "fastq.stats_neutral";

pub const BAM_ALIGN: &str = "bam.align";
pub const BAM_VALIDATE: &str = "bam.validate";
pub const BAM_MARKDUP: &str = "bam.markdup";
pub const BAM_RECALIBRATION: &str = "bam.recalibration";
pub const BAM_COVERAGE: &str = "bam.coverage";
pub const BAM_INSERT_SIZE: &str = "bam.insert_size";
pub const BAM_GC_BIAS: &str = "bam.gc_bias";
pub const BAM_DAMAGE: &str = "bam.damage";
pub const BAM_COMPLEXITY: &str = "bam.complexity";
pub const BAM_AUTHENTICITY: &str = "bam.authenticity";
pub const BAM_HAPLOGROUPS: &str = "bam.haplogroups";
pub const BAM_KINSHIP: &str = "bam.kinship";
pub const BAM_CONTAMINATION: &str = "bam.contamination";
pub const BAM_SEX: &str = "bam.sex";

pub const PIPELINE_FASTQ_DEFAULT: &str = "fastq-to-fastq__default__v1";
pub const PIPELINE_FASTQ_MINIMAL: &str = "fastq-to-fastq__minimal__v1";
pub const PIPELINE_FASTQ_ADNA: &str = "fastq-to-fastq__adna__v1";
pub const PIPELINE_FASTQ_REFERENCE_ADNA: &str = "fastq-to-fastq__reference_adna__v1";
pub const PIPELINE_BAM_DEFAULT: &str = "bam-to-bam__default__v1";
pub const PIPELINE_BAM_ADNA_SHOTGUN: &str = "bam-to-bam__adna_shotgun__v1";
pub const PIPELINE_BAM_ADNA_CAPTURE: &str = "bam-to-bam__adna_capture__v1";
pub const PIPELINE_FASTQ_TO_BAM_DEFAULT: &str = "fastq-to-bam__default__v1";
pub const PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN: &str = "fastq-to-bam__adna_shotgun__v1";

pub const TOOL_FASTQVALIDATOR_OFFICIAL: &str = "fastqvalidator_official";
pub const TOOL_SEQKIT_STATS: &str = "seqkit_stats";
pub const TOOL_RCORRECTOR: &str = "rcorrector";
pub const TOOL_UMI_TOOLS: &str = "umi_tools";
pub const TOOL_FASTQC: &str = "fastqc";
pub const TOOL_FASTP: &str = "fastp";
pub const TOOL_ADAPTERREMOVAL: &str = "adapterremoval";
pub const TOOL_SEQKIT: &str = "seqkit";
pub const TOOL_MULTIQC: &str = "multiqc";
pub const TOOL_PLANNER: &str = "planner";
pub const TOOL_VSEARCH: &str = "vsearch";
pub const TOOL_LEEHOM: &str = "leehom";
pub const TOOL_KRAKEN2: &str = "kraken2";
pub const TOOL_SAMTOOLS: &str = "samtools";
pub const TOOL_BWA: &str = "bwa";
pub const TOOL_GATK: &str = "gatk";
pub const TOOL_PRESEQ: &str = "preseq";
pub const TOOL_MOSDEPTH: &str = "mosdepth";
pub const TOOL_PYDAMAGE: &str = "pydamage";
pub const TOOL_AUTHENTICCT: &str = "authenticct";
pub const TOOL_PMDTOOLS: &str = "pmdtools";
pub const TOOL_SCHMUTZI: &str = "schmutzi";
pub const TOOL_VERIFYBAMID2: &str = "verifybamid2";
pub const TOOL_CONTAMMIX: &str = "contammix";
pub const TOOL_RXY: &str = "rxy";
pub const TOOL_ANGSD: &str = "angsd";
pub const TOOL_YLEAF: &str = "yleaf";
pub const TOOL_KING: &str = "king";
