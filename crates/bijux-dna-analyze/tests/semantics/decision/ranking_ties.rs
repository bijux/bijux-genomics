use bijux_dna_analyze::{build_rankings, RankInput};

#[test]
fn ranking_breaks_ties_by_tool_name() -> anyhow::Result<()> {
    let inputs = vec![
        RankInput {
            tool: "b".to_string(),
            runtime_s: 1.0,
            memory_mb: 1.0,
            read_retention: Some(0.9),
            base_retention: Some(0.9),
            error_reduction_proxy: Some(0.0),
        },
        RankInput {
            tool: "a".to_string(),
            runtime_s: 1.0,
            memory_mb: 1.0,
            read_retention: Some(0.9),
            base_retention: Some(0.9),
            error_reduction_proxy: Some(0.0),
        },
    ];
    let rankings = build_rankings(&inputs)?;
    let fastest = &rankings["FastestAcceptable"];
    assert_eq!(fastest[0].tool, "a");
    Ok(())
}
