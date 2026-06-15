pub(crate) fn include_fastq_active_benchmark_pair(stage_id: &str, tool_id: &str) -> bool {
    if stage_id != "fastq.screen_taxonomy" {
        return true;
    }
    matches!(tool_id, "centrifuge" | "kaiju" | "kraken2" | "krakenuniq")
}
