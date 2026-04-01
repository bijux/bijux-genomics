mod stage_ids;

pub use stage_ids::{
    BAM_ALIGN, BAM_AUTHENTICITY, BAM_BIAS_MITIGATION, BAM_COMPLEXITY, BAM_CONTAMINATION,
    BAM_COVERAGE, BAM_DAMAGE, BAM_DUPLICATION_METRICS, BAM_ENDOGENOUS_CONTENT, BAM_FILTER,
    BAM_GC_BIAS, BAM_GENOTYPING, BAM_HAPLOGROUPS, BAM_INSERT_SIZE, BAM_KINSHIP,
    BAM_LENGTH_FILTER, BAM_MAPPING_SUMMARY, BAM_MAPQ_FILTER, BAM_MARKDUP,
    BAM_OVERLAP_CORRECTION, BAM_QC_PRE, BAM_RECALIBRATION, BAM_SEX, BAM_VALIDATE, BAM_PREFIX,
    CORE_PREFIX, CORE_PREPARE_REFERENCE, FASTQ_CLUSTER_OTUS, FASTQ_CORRECT,
    FASTQ_DEDUPLICATE, FASTQ_DEPLETE_HOST, FASTQ_DEPLETE_REFERENCE_CONTAMINANTS,
    FASTQ_DEPLETE_RRNA, FASTQ_DETECT_ADAPTERS, FASTQ_FILTER, FASTQ_INDEX_REFERENCE,
    FASTQ_INFER_ASVS, FASTQ_LOW_COMPLEXITY, FASTQ_MERGE, FASTQ_NORMALIZE_ABUNDANCE,
    FASTQ_NORMALIZE_PRIMERS, FASTQ_PREFIX, FASTQ_PREPROCESS,
    FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES, FASTQ_PROFILE_READ_LENGTHS, FASTQ_QC_POST,
    FASTQ_REMOVE_CHIMERAS, FASTQ_SCREEN, FASTQ_STATS_NEUTRAL, FASTQ_TRIM,
    FASTQ_TRIM_POLYG_TAILS, FASTQ_TRIM_TERMINAL_DAMAGE, FASTQ_UMI, FASTQ_VALIDATE_PRE,
    FASTQ_VALIDATE_READS, REPORT_AGGREGATE_STAGE, REPORT_AGGREGATE_STEP, VCF_CALL, VCF_FILTER,
    VCF_PREFIX, VCF_STATS,
};

pub const PIPELINE_FASTQ_DEFAULT: &str = "fastq-to-fastq__default__v1";
pub const PIPELINE_FASTQ_MINIMAL: &str = "fastq-to-fastq__minimal__v1";
pub const PIPELINE_FASTQ_ADNA: &str = "fastq-to-fastq__adna__v1";
pub const PIPELINE_FASTQ_REFERENCE_ADNA: &str = "fastq-to-fastq__reference_adna__v1";
pub const PIPELINE_BAM_DEFAULT: &str = "bam-to-bam__default__v1";
pub const PIPELINE_BAM_ADNA_SHOTGUN: &str = "bam-to-bam__adna_shotgun__v1";
pub const PIPELINE_BAM_ADNA_CAPTURE: &str = "bam-to-bam__adna_capture__v1";
pub const PIPELINE_BAM_REFERENCE_ADNA: &str = "bam-to-bam__reference_adna__v1";
pub const PIPELINE_FASTQ_TO_BAM_DEFAULT: &str = "fastq-to-bam__default__v1";
pub const PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN: &str = "fastq-to-bam__adna_shotgun__v1";
pub const PIPELINE_VCF_MINIMAL: &str = "vcf-to-vcf__minimal__v1";
pub const PIPELINE_VCF_REFERENCE_BASIC: &str = "vcf-to-vcf__reference_basic__v1";

pub const TOOL_FASTQVALIDATOR_OFFICIAL: &str = "fastqvalidator";
pub const TOOL_SEQKIT_STATS: &str = "seqkit_stats";
pub const TOOL_RCORRECTOR: &str = "rcorrector";
pub const TOOL_UMI_TOOLS: &str = "umi_tools";
pub const TOOL_FASTQC: &str = "fastqc";
pub const TOOL_FASTP: &str = "fastp";
pub const TOOL_ADAPTERREMOVAL: &str = "adapterremoval";
pub const TOOL_CUTADAPT: &str = "cutadapt";
pub const TOOL_SEQKIT: &str = "seqkit";
pub const TOOL_MULTIQC: &str = "multiqc";
pub const TOOL_PEAR: &str = "pear";
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
pub const TOOL_BCFTOOLS: &str = "bcftools";
pub const TOOL_BBDUK: &str = "bbduk";
