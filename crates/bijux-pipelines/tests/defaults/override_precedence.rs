use bijux_pipelines::{merge_effective_defaults, EffectiveDefaults};

#[test]
fn overrides_apply_in_expected_order() {
    let mut base = EffectiveDefaults::default();
    base.tools
        .insert("fastq.trim".to_string(), "fastp".to_string());
    base.params
        .insert("fastq.trim".to_string(), serde_json::json!({"min_len": 0}));

    let mut config = EffectiveDefaults::default();
    config
        .tools
        .insert("fastq.trim".to_string(), "trimmomatic".to_string());
    config
        .params
        .insert("fastq.trim".to_string(), serde_json::json!({"min_len": 5}));

    let mut cli = EffectiveDefaults::default();
    cli.tools
        .insert("fastq.trim".to_string(), "fastp".to_string());
    cli.params
        .insert("fastq.trim".to_string(), serde_json::json!({"min_len": 10}));

    let mut api = EffectiveDefaults::default();
    api.tools
        .insert("fastq.trim".to_string(), "cutadapt".to_string());
    api.params
        .insert("fastq.trim".to_string(), serde_json::json!({"min_len": 20}));

    let merged = merge_effective_defaults(&base, Some(&config), Some(&cli), Some(&api)).unwrap();
    assert_eq!(
        merged.tools.get("fastq.trim"),
        Some(&"cutadapt".to_string())
    );
    assert_eq!(
        merged.params.get("fastq.trim"),
        Some(&serde_json::json!({"min_len": 20}))
    );
    assert_eq!(
        merged.rationales.get("fastq.trim"),
        Some(&"api override".to_string())
    );

    let snapshot = serde_json::json!({
        "tools": merged.tools,
        "params": merged.params,
        "rationales": merged.rationales,
    });
    insta::assert_json_snapshot!("override_precedence", snapshot);
}
