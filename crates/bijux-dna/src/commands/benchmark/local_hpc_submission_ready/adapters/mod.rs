//! Compose existing local services into explicit readiness evidence ports.
pub(super) struct LocalReadinessAdapter;
mod campaign;
mod corpora;
mod harness;
mod inventory;
mod pipelines;
mod slurm;

fn inventory_domain(
    domain: bijux_dna_bench_hpc::readiness::ports::ReadinessDomain,
) -> crate::commands::benchmark::local_stage_inventory::BenchLocalDomain {
    use crate::commands::benchmark::local_stage_inventory::BenchLocalDomain;
    use bijux_dna_bench_hpc::readiness::ports::ReadinessDomain;
    match domain {
        ReadinessDomain::Fastq => BenchLocalDomain::Fastq,
        ReadinessDomain::Bam => BenchLocalDomain::Bam,
    }
}

mod inputs;
pub(super) use inputs::input_paths;
