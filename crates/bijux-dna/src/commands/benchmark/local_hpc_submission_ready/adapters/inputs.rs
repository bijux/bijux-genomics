use crate::commands::benchmark::local_corpus_fixture::{amplicon, bam, damage, edna, fastq};
use crate::commands::benchmark::local_corpus_stage_compatibility::DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH;
use crate::commands::benchmark::local_stage_fake_runs::DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT;
use crate::commands::benchmark::local_taxonomy_database_fixture::DEFAULT_TAXONOMY_MINI_MANIFEST_PATH;
use bijux_dna_bench_hpc::readiness::ReadinessInputPaths;

pub(in crate::commands::benchmark::local_hpc_submission_ready) fn input_paths(
) -> ReadinessInputPaths<'static> {
    ReadinessInputPaths {
        fastq_fixture: fastq::DEFAULT_CORPUS_01_MINI_MANIFEST_PATH,
        bam_fixture: bam::DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH,
        damage_fixture: damage::DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH,
        edna_fixture: edna::DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH,
        amplicon_fixture: amplicon::DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH,
        taxonomy_fixture: DEFAULT_TAXONOMY_MINI_MANIFEST_PATH,
        corpus_stage_compatibility: DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
        fake_run_root: DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT,
    }
}
