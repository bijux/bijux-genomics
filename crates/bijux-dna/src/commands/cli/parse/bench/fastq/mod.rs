mod preprocessing;
mod quality;
mod workflows;

pub use self::preprocessing::{
    BenchFastqDetectAdaptersArgs, BenchFastqTrimArgs, BenchFastqTrimPolygArgs,
    BenchFastqTrimTerminalDamageArgs, BenchFastqValidateArgs,
};
pub use self::quality::{
    BenchFastqCorrectArgs, BenchFastqFilterArgs, BenchFastqFilterLowComplexityArgs,
    BenchFastqProfileOverrepresentedArgs, BenchFastqProfileReadLengthsArgs, BenchFastqQcPostArgs,
    BenchFastqRemoveDuplicatesArgs, BenchFastqStatsArgs,
};
pub use self::workflows::{
    BenchFastqClusterOtusArgs, BenchFastqDepleteHostArgs,
    BenchFastqDepleteReferenceContaminantsArgs, BenchFastqDepleteRrnaArgs,
    BenchFastqIndexReferenceArgs, BenchFastqInferAsvsArgs, BenchFastqMergeArgs,
    BenchFastqNormalizeAbundanceArgs, BenchFastqNormalizePrimersArgs, BenchFastqPreprocessArgs,
    BenchFastqRemoveChimerasArgs, BenchFastqScreenArgs, BenchFastqUmiArgs,
};

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum BenchFastqCommand {
    #[command(name = "trim-reads", visible_alias = "trim")]
    Trim(BenchFastqTrimArgs),
    #[command(name = "trim-polyg-tails")]
    TrimPolygTails(BenchFastqTrimPolygArgs),
    #[command(name = "trim-terminal-damage")]
    TrimTerminalDamage(BenchFastqTrimTerminalDamageArgs),
    #[command(name = "validate-reads", visible_alias = "validate")]
    Validate(BenchFastqValidateArgs),
    #[command(name = "detect-adapters")]
    DetectAdapters(BenchFastqDetectAdaptersArgs),
    #[command(name = "profile-read-lengths")]
    ProfileReadLengths(BenchFastqProfileReadLengthsArgs),
    Filter(BenchFastqFilterArgs),
    #[command(name = "filter-low-complexity")]
    FilterLowComplexity(BenchFastqFilterLowComplexityArgs),
    Merge(BenchFastqMergeArgs),
    #[command(name = "remove-duplicates")]
    RemoveDuplicates(BenchFastqRemoveDuplicatesArgs),
    #[command(name = "remove-chimeras")]
    RemoveChimeras(BenchFastqRemoveChimerasArgs),
    #[command(name = "normalize-primers")]
    NormalizePrimers(BenchFastqNormalizePrimersArgs),
    #[command(name = "infer-asvs")]
    InferAsvs(BenchFastqInferAsvsArgs),
    #[command(name = "cluster-otus")]
    ClusterOtus(BenchFastqClusterOtusArgs),
    #[command(name = "normalize-abundance")]
    NormalizeAbundance(BenchFastqNormalizeAbundanceArgs),
    Correct(BenchFastqCorrectArgs),
    #[command(name = "report-qc", visible_alias = "qc-post", alias = "qc2")]
    ReportQc(BenchFastqQcPostArgs),
    Umi(BenchFastqUmiArgs),
    #[command(name = "index-reference")]
    IndexReference(BenchFastqIndexReferenceArgs),
    #[command(name = "screen-taxonomy", visible_alias = "screen")]
    Screen(BenchFastqScreenArgs),
    #[command(name = "deplete-host")]
    DepleteHost(BenchFastqDepleteHostArgs),
    #[command(name = "deplete-reference-contaminants")]
    DepleteReferenceContaminants(BenchFastqDepleteReferenceContaminantsArgs),
    #[command(name = "deplete-rrna")]
    DepleteRrna(BenchFastqDepleteRrnaArgs),
    #[command(name = "profile-reads", visible_alias = "stats")]
    Stats(BenchFastqStatsArgs),
    #[command(name = "profile-overrepresented-sequences", visible_alias = "overrepresented")]
    ProfileOverrepresentedSequences(BenchFastqProfileOverrepresentedArgs),
    Preprocess(BenchFastqPreprocessArgs),
}
