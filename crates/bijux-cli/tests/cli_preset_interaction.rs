use std::path::PathBuf;

use anyhow::Result;
use bijux::cli::{
    bench_args_from_trim, preprocess_args_from_cli, BenchCorpusArg, CommonArgs,
    FastqPreprocessArgs, FastqTrimArgs, ObjectiveArg,
};

#[test]
fn trim_bench_args_preserve_bank_presets() -> Result<()> {
    let args = FastqTrimArgs {
        common: CommonArgs::default(),
        list_adapter_presets: false,
        list_adapters: false,
        env: None,
        sample_id: Some("s1".to_string()),
        r1: Some(PathBuf::from("reads.fastq.gz")),
        out: Some(PathBuf::from("out")),
        tools: vec!["fastp".to_string()],
        adapter_bank_preset: Some("ancientdna-illumina".to_string()),
        adapter_bank: Some("preset:legacy".to_string()),
        adapter_bank_file: None,
        enable_adapter: vec!["adapter1".to_string()],
        disable_adapter: vec!["adapter2".to_string()],
        polyx_preset: Some("illumina_twocolor".to_string()),
        contaminant_preset: Some("illumina_default".to_string()),
    };
    let bench = bench_args_from_trim(&args)?;
    assert_eq!(
        bench.adapter_bank_preset.as_deref(),
        Some("ancientdna-illumina")
    );
    assert_eq!(bench.adapter_bank.as_deref(), Some("preset:legacy"));
    assert_eq!(bench.polyx_preset.as_deref(), Some("illumina_twocolor"));
    assert_eq!(
        bench.contaminant_preset.as_deref(),
        Some("illumina_default")
    );
    Ok(())
}

#[test]
fn preprocess_args_require_required_fields() -> Result<()> {
    let args = FastqPreprocessArgs {
        common: CommonArgs::default(),
        env: None,
        sample_id: Some("s1".to_string()),
        r1: Some(PathBuf::from("reads.fastq.gz")),
        r2: None,
        out: Some(PathBuf::from("out")),
        strict: false,
        auto: false,
        objective: ObjectiveArg::Balanced,
        bench_corpus: Some(BenchCorpusArg::Fastq5Set),
        allow_partial: false,
        list_adapter_presets: false,
        list_adapters: false,
        adapter_bank_preset: Some("ancientdna-illumina".to_string()),
        adapter_bank: None,
        adapter_bank_file: None,
        enable_adapter: Vec::new(),
        disable_adapter: Vec::new(),
        polyx_preset: Some("illumina_twocolor".to_string()),
        contaminant_preset: Some("illumina_default".to_string()),
    };
    let bench = preprocess_args_from_cli(&args)?;
    assert_eq!(bench.sample_id, "s1");
    assert_eq!(
        bench.adapter_bank_preset.as_deref(),
        Some("ancientdna-illumina")
    );
    Ok(())
}
