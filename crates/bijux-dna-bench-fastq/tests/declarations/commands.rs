use anyhow::Result;
use bijux_dna_bench_fastq::readiness::{
    active::{FastqActiveStageToolMatrixCollection, FastqActiveStageToolMatrixRow},
    commands::render_fastq_commands,
    declarations::{
        observations::RenderedCommandObservation, ports::CommandEvidence, CommandContext,
    },
};
use std::path::Path;
struct Evidence {
    active_count: usize,
    rendered: Vec<RenderedCommandObservation>,
}
fn active() -> FastqActiveStageToolMatrixRow {
    FastqActiveStageToolMatrixRow {
        stage_id: "fastq.trim_reads".into(),
        tool_id: "fastp".into(),
        corpus_id: "corpus".into(),
        asset_profile_id: "assets".into(),
        adapter_id: "adapter".into(),
        parser_id: "parser".into(),
        schema_id: "schema".into(),
        benchmark_status: "benchmark_ready".into(),
        support_status: "supported".into(),
        adapter_status: "runnable".into(),
        parser_status: "complete".into(),
        corpus_status: "complete".into(),
        scope_proof_path: "scope.tsv".into(),
        reason: "fixture".into(),
    }
}
fn command() -> RenderedCommandObservation {
    RenderedCommandObservation {
        stage_id: "fastq.trim_reads".into(),
        tool_id: "fastp".into(),
        readiness_kind: "smoke".into(),
        argv: vec!["fastp".into(), "--in1".into(), "reads with spaces.fastq".into()],
        command: "fastp --in1 'reads with spaces.fastq'".into(),
    }
}
impl CommandEvidence for Evidence {
    fn active(&self, _: &Path) -> Result<FastqActiveStageToolMatrixCollection> {
        Ok(FastqActiveStageToolMatrixCollection {
            retained_row_count: self.active_count,
            retained_stage_count: 1,
            retained_tool_count: 1,
            removed_row_count: 0,
            removed_stage_count: 0,
            removed_tool_count: 0,
            rows: (0..self.active_count).map(|_| active()).collect(),
        })
    }
    fn rendered(&self, _: &Path) -> Result<Vec<RenderedCommandObservation>> {
        Ok(self.rendered.clone())
    }
}
fn refuses(evidence: &Evidence, message: &str) -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("commands.sh");
    std::fs::write(&output, b"prior command evidence")?;
    let result =
        render_fastq_commands(&CommandContext { repo_root: root.path(), evidence }, &output);
    assert!(result.err().is_some_and(|error| error.to_string().contains(message)));
    assert_eq!(std::fs::read(&output)?, b"prior command evidence");
    assert!(!output.with_extension("argv.jsonl").exists());
    Ok(())
}
#[test]
fn missing_command_refuses_before_overwriting_published_evidence() -> Result<()> {
    refuses(
        &Evidence { active_count: 1, rendered: vec![] },
        "missing benchmark-ready command coverage",
    )
}
#[test]
fn duplicate_active_binding_refuses_before_overwriting_published_evidence() -> Result<()> {
    refuses(&Evidence { active_count: 2, rendered: vec![command()] }, "duplicate")
}
#[test]
fn undeclared_command_binding_refuses_before_overwriting_published_evidence() -> Result<()> {
    let mut extra = command();
    extra.tool_id = "foreign".into();
    refuses(&Evidence { active_count: 1, rendered: vec![command(), extra] }, "unexpected rows")
}
#[test]
fn argv_argument_boundaries_and_shell_text_remain_independent() -> Result<()> {
    let root = tempfile::tempdir()?;
    let original = command();
    let evidence = Evidence { active_count: 1, rendered: vec![original.clone()] };
    let report = render_fastq_commands(
        &CommandContext { repo_root: root.path(), evidence: &evidence },
        &root.path().join("commands.sh"),
    )?;
    assert_eq!(report.rows[0].command_steps[0].argv, original.argv);
    assert_eq!(report.rows[0].script_commands, std::slice::from_ref(&original.command));
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.path().join(report.argv_output_path))?)?;
    assert_eq!(json["command_steps"][0]["argv"][2], "reads with spaces.fastq");
    assert!(
        std::fs::read_to_string(root.path().join(report.output_path))?.contains(&original.command)
    );
    Ok(())
}

#[test]
fn duplicate_rendered_observations_preserve_both_published_command_outputs() -> Result<()> {
    for conflicting in [false, true] {
        let root = tempfile::tempdir()?;
        let output = root.path().join("commands.sh");
        let argv_output = output.with_extension("argv.jsonl");
        std::fs::write(&output, b"previous shell")?;
        std::fs::write(&argv_output, b"previous argv")?;
        let first = command();
        let mut second = first.clone();
        if conflicting {
            second.argv.push("--foreign".into());
            second.command.push_str(" --foreign");
        }
        let evidence = Evidence { active_count: 1, rendered: vec![first, second] };
        let result = render_fastq_commands(
            &CommandContext { repo_root: root.path(), evidence: &evidence },
            &output,
        );
        assert!(
            result.err().is_some_and(|error| error.to_string().contains("duplicate")),
            "rendered multiplicity must not be lost before admission; conflicting={conflicting}"
        );
        assert_eq!(std::fs::read(&output)?, b"previous shell");
        assert_eq!(std::fs::read(&argv_output)?, b"previous argv");
    }
    Ok(())
}
