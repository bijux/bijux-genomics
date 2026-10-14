//! Runs of per-contig homozygosity flags for retained mini-fixture truth.
use super::VcfFixtureSampleGenotypes;

impl VcfFixtureSampleGenotypes {
    /// Count runs of at least two adjacent homozygous observations per contig.
    ///
    /// Record order is preserved. Runs never bridge contigs or false flags;
    /// these counts are fixture observations rather than physical intervals.
    #[must_use]
    pub fn homozygous_run_count(&self) -> u64 {
        self.contig_is_homozygous.values().map(|flags| count_true_runs(flags)).sum::<u64>()
    }

    /// Count runs where both samples have adjacent homozygous observations.
    ///
    /// The flags are paired in their existing order and truncate at the shorter
    /// vector. An absent counterpart contig contributes zero. Dosage equality
    /// and matching alleles are separate claims, outside this observation.
    #[must_use]
    pub fn shared_homozygous_run_count(&self, other: &Self) -> u64 {
        self.contig_is_homozygous
            .iter()
            .map(|(contig, left_flags)| {
                let Some(right_flags) = other.contig_is_homozygous.get(contig) else {
                    return 0;
                };
                let shared_flags = left_flags
                    .iter()
                    .zip(right_flags.iter())
                    .map(|(left_flag, right_flag)| *left_flag && *right_flag)
                    .collect::<Vec<_>>();
                count_true_runs(&shared_flags)
            })
            .sum::<u64>()
    }
}

fn count_true_runs(flags: &[bool]) -> u64 {
    let mut run_length = 0usize;
    let mut count = 0u64;
    for flag in flags {
        if *flag {
            run_length += 1;
        } else {
            if run_length >= 2 {
                count += 1;
            }
            run_length = 0;
        }
    }
    if run_length >= 2 {
        count += 1;
    }
    count
}

#[cfg(test)]
mod contracts;
