use std::path::Path;

use bijux_dna_bench::{
    load_stage_interface_catalog, render_bam_genotyping_markdown,
    render_bam_mapping_summary_markdown, render_fastq_profile_read_lengths_markdown,
    render_fastq_profile_reads_markdown, render_stage_report_markdown, BenchmarkStageReport,
};

type LegacyRenderer = fn(&BenchmarkStageReport) -> String;

#[test]
fn generic_renderer_preserves_historical_report_bytes() -> anyhow::Result<()> {
    let root = repository_root()?;
    let interfaces =
        load_stage_interface_catalog(&root.join("benchmarks/contracts/stage-interfaces.toml"))?;
    let cases: [(&str, &str, &str, LegacyRenderer); 4] = [
        (
            "bam.genotyping",
            "9348a2417384417ab59052193b56717b69cb0309",
            "run-2001-bam-genotyping",
            render_bam_genotyping_markdown,
        ),
        (
            "bam.mapping_summary",
            "520efb95cf753b65fca168f0610b1b4138932031",
            "run-2004-bam-mapping-summary",
            render_bam_mapping_summary_markdown,
        ),
        (
            "fastq.profile_read_lengths",
            "520efb95cf753b65fca168f0610b1b4138932031",
            "run-2042-fastq-profile-read-lengths",
            render_fastq_profile_read_lengths_markdown,
        ),
        (
            "fastq.profile_reads",
            "520efb95cf753b65fca168f0610b1b4138932031",
            "run-2043-fastq-profile-reads",
            render_fastq_profile_reads_markdown,
        ),
    ];
    for (stage_id, source_commit, stem, legacy_renderer) in cases {
        let interface = interfaces
            .interfaces
            .iter()
            .find(|interface| interface.stage_id == stage_id)
            .ok_or_else(|| anyhow::anyhow!("missing stage interface {stage_id}"))?;
        let report_root = root.join("benchmarks/reports/historical").join(source_commit);
        let report: BenchmarkStageReport =
            serde_json::from_slice(&std::fs::read(report_root.join(format!("{stem}.json")))?)?;
        let generic = render_stage_report_markdown(&report, &interface.report_presentation)?;
        assert_eq!(generic, legacy_renderer(&report), "legacy parity failed for {stage_id}");
        assert_eq!(
            generic.as_bytes(),
            std::fs::read(report_root.join(format!("{stem}.md")))?,
            "checked-in parity failed for {stage_id}"
        );
    }
    Ok(())
}

#[test]
fn every_interface_has_a_unique_data_driven_report_presentation() -> anyhow::Result<()> {
    let root = repository_root()?;
    let interfaces =
        load_stage_interface_catalog(&root.join("benchmarks/contracts/stage-interfaces.toml"))?;
    assert_eq!(interfaces.interfaces.len(), 11);
    let stems = interfaces
        .interfaces
        .iter()
        .map(|interface| interface.report_presentation.report_file_stem.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(stems.len(), interfaces.interfaces.len());
    Ok(())
}

fn repository_root() -> anyhow::Result<&'static Path> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark crate is outside the workspace"))
}
