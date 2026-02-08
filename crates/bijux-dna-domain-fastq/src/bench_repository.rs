use anyhow::Result;

use bijux_dna_core::contract::BenchResultRecord;
use bijux_dna_core::ids::StageId;

use crate::BenchCorpus;

pub trait BenchResultsRepository {
    /// # Errors
    /// Returns an error if benchmark records cannot be loaded for the request.
    fn bench_results(
        &self,
        stage: &StageId,
        tool: &str,
        corpus: &BenchCorpus,
    ) -> Result<Vec<BenchResultRecord>>;
}
