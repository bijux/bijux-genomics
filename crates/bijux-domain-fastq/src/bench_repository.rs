use anyhow::Result;

use bijux_core::contract::BenchResultRecord;
use bijux_core::ids::StageId;

use crate::BenchCorpus;

pub trait BenchResultsRepository {
    fn bench_results(
        &self,
        stage: &StageId,
        tool: &str,
        corpus: &BenchCorpus,
    ) -> Result<Vec<BenchResultRecord>>;
}
