#![cfg(test)]

use super::*;
use bijux_dna_core::contract::{Objective, StageIO};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageId, StepId,
    ToolConstraints,
};
use bijux_dna_planner_fastq::stage_api::args::{BenchFastqPreprocessArgs, FastqPlannerMode};

fn batch_args(root: &std::path::Path) -> BenchFastqPreprocessArgs {
    BenchFastqPreprocessArgs {
        sample_id: "sample".to_string(),
        profile: None,
        r1: root.join("input.fastq"),
        r2: None,
        reference_fasta: None,
        out: root.to_path_buf(),
        strict: false,
        auto: false,
        objective: Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        dry_run: false,
        replicates: 1,
        jobs: 2,
        ci_bootstrap: None,
        adapter_bank_preset: None,
        adapter_bank: None,
        adapter_bank_file: None,
        enable_adapters: Vec::new(),
        disable_adapters: Vec::new(),
        polyx_preset: None,
        contaminant_preset: None,
        enable_contaminant_removal: false,
        no_qc_post: true,
        force_merge: false,
        enable_correct: false,
        run_all_governed_tools: false,
        allow_planned: false,
        mode: FastqPlannerMode::Shotgun,
    }
}

fn local_step(root: &std::path::Path, label: &str) -> ExecutionStep {
    let out_dir = root.join(label);
    std::fs::create_dir_all(&out_dir).expect("create local step directory");
    let output = out_dir.join("observed.txt");
    ExecutionStep {
        step_id: StepId::new(format!("fastq.trim_reads.{label}")),
        stage_id: StageId::from_static("fastq.trim_reads"),
        command: CommandSpecV1 {
            template: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "printf '%s' \"$1\" > \"$2\"; printf '%s' \"$1\"".to_string(),
                "batch-output".to_string(),
                label.to_string(),
                output.display().to_string(),
            ],
        },
        image: ContainerImageRefV1 {
            image: "local-shell".to_string(),
            digest: Some(format!("sha256:{}", "a".repeat(64))),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: Vec::new(),
            outputs: vec![ArtifactSpec::required(
                ArtifactId::new(label),
                output,
                ArtifactRole::Log,
            )],
        },
        out_dir,
        aux_images: std::collections::BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

#[test]
fn mixed_resumed_and_pending_preprocess_batch_preserves_ordered_native_outputs() -> Result<()> {
    for jobs in [1, 2] {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let args = batch_args(root);
        std::fs::write(&args.r1, b"@read\nACGT\n+\nIIII\n")?;
        let batch =
            [local_step(root, "retained"), local_step(root, "left"), local_step(root, "right")];
        let retained = &batch[0].io.outputs[0].path;
        std::fs::write(retained, b"retained original output")?;
        let stage_root = run_artifacts_dir_for_out(root).join(batch[0].step_id.as_str());
        std::fs::create_dir_all(&stage_root)?;
        std::fs::write(stage_root.join("runtime_provenance.json"), b"{}")?;

        let results =
            execute_preprocess_batch(&batch, RuntimeKind::Local, jobs, root, "sample", &args)?;
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].command, "resume");
        assert_eq!(std::fs::read(retained)?, b"retained original output");
        for (slot, label) in [(1, "left"), (2, "right")] {
            assert_eq!(results[slot].exit_code, 0);
            assert_eq!(results[slot].stdout, label);
            assert_eq!(results[slot].outputs, vec![batch[slot].io.outputs[0].path.clone()]);
            assert_eq!(std::fs::read_to_string(&batch[slot].io.outputs[0].path)?, label);
        }
    }
    Ok(())
}
