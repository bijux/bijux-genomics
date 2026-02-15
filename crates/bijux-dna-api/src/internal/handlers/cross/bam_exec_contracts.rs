#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
    use bijux_dna_core::ids::{StageId, StageVersion, ToolId};
    use bijux_dna_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1};
    use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};

    fn mock_plan(stage: bijux_dna_planner_bam::stage_api::BamStage) -> StagePlanV1 {
        StagePlanV1 {
            stage_id: StageId::new(stage.as_str()),
            stage_version: StageVersion(1),
            tool_id: ToolId::new("samtools"),
            tool_version: "1.20".to_string(),
            image: ContainerImageRefV1 {
                image: "example/tool".to_string(),
                digest: Some("sha256:deadbeef".to_string()),
            },
            command: CommandSpecV1 {
                template: vec!["samtools".to_string(), "--version".to_string()],
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("in"),
                    PathBuf::from("in.bam"),
                    ArtifactRole::Bam,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("out"),
                    PathBuf::from("out.bam"),
                    ArtifactRole::Bam,
                )],
            },
            out_dir: PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            reason: PlanDecisionReason {
                kind: PlanReasonKind::Default,
                summary: "test".to_string(),
                details: serde_json::json!({}),
            },
        }
    }

    #[test]
    fn validate_and_mapping_summary_postprocess_emit_standardized_outputs() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let validate_dir = temp.path().join("validate");
        bijux_dna_infra::ensure_dir(&validate_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &validate_dir.join("flagstat.txt"),
            b"10 + 0 in total (QC-passed reads + QC-failed reads)\n8 + 0 mapped (80.00% : N/A)\n2 + 0 duplicates\n",
        )?;
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Validate,
            &validate_dir,
            &mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Validate),
        )?;
        let validate_summary: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(validate_dir.join("validation.summary.json"))?)?;
        assert_eq!(
            validate_summary
                .get("schema_version")
                .and_then(serde_json::Value::as_str),
            Some("bijux.bam.validate.v1")
        );

        let mapping_dir = temp.path().join("mapping_summary");
        bijux_dna_infra::ensure_dir(&mapping_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &mapping_dir.join("flagstat.txt"),
            b"20 + 0 in total (QC-passed reads + QC-failed reads)\n15 + 0 mapped (75.00% : N/A)\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(&mapping_dir.join("samtools_stats.txt"), b"SN\traw total sequences:\t20\n")?;
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::MappingSummary,
            &mapping_dir,
            &mock_plan(bijux_dna_planner_bam::stage_api::BamStage::MappingSummary),
        )?;
        let mapping_summary: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(mapping_dir.join("mapping_summary.json"))?)?;
        assert_eq!(
            mapping_summary
                .get("schema_version")
                .and_then(serde_json::Value::as_str),
            Some("bijux.bam.mapping_summary.v1")
        );
        Ok(())
    }

    #[test]
    fn damage_and_authenticity_postprocess_emit_composite_artifacts() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let bam_root = temp.path().join("bam");
        let damage_dir = bam_root.join("damage");
        let authenticity_dir = bam_root.join("authenticity");
        bijux_dna_infra::ensure_dir(&damage_dir)?;
        bijux_dna_infra::ensure_dir(&authenticity_dir)?;
        bijux_dna_infra::atomic_write_json(
            &damage_dir.join("damage.pydamage.json"),
            &serde_json::json!({
                "schema_version":"bijux.bam.damage.v1",
                "c_to_t_5p": 0.20,
                "g_to_a_3p": 0.15
            }),
        )?;
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Damage,
            &damage_dir,
            &mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Damage),
        )?;
        assert!(damage_dir.join("damage.unified_metrics.json").exists());

        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Authenticity,
            &authenticity_dir,
            &mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Authenticity),
        )?;
        let authenticity: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            authenticity_dir.join("authenticity_composite.json"),
        )?)?;
        assert!(authenticity.get("composite_score").is_some());
        Ok(())
    }

    #[test]
    fn refusal_rules_enforce_align_reference_and_sex_contigs() {
        let temp = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let bam = temp.path().join("x.bam");
        std::fs::write(&bam, b"bam").unwrap_or_else(|err| panic!("write bam: {err}"));
        let Err(err) = enforce_stage_refusal_rules(
            bijux_dna_planner_bam::stage_api::BamStage::Align,
            &bam,
            None,
            None,
        ) else {
            panic!("align must require reference");
        };
        assert!(err.to_string().contains("requires resolved reference fasta"));

        let ref_fa = temp.path().join("ref.fa");
        let ref_fai = temp.path().join("ref.fa.fai");
        std::fs::write(&ref_fa, b">1\nACGT\n").unwrap_or_else(|err| panic!("write ref: {err}"));
        std::fs::write(&ref_fai, b"1\t4\t0\t4\t5\n").unwrap_or_else(|err| panic!("write fai: {err}"));
        let Err(err) = enforce_stage_refusal_rules(
            bijux_dna_planner_bam::stage_api::BamStage::Sex,
            &bam,
            Some(&temp.path().join("x.bam.bai")),
            Some(&ref_fa),
        ) else {
            panic!("sex must require X/Y contigs");
        };
        assert!(err.to_string().contains("lacks required X/Y contigs"));
    }

    #[test]
    fn bam_invariants_and_wrapper_contracts_are_emitted() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("validate");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        let bam = temp.path().join("input.bam");
        let bai = temp.path().join("input.bam.bai");
        std::fs::write(&bam, b"@HD\tVN:1.6\tSO:coordinate\n@RG\tID:rg1\tSM:s1\n")?;
        std::fs::write(&bai, b"index")?;

        let stage = bijux_dna_planner_bam::stage_api::BamStage::Validate;
        let plan = mock_plan(stage);
        let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&plan);
        write_bam_invariants(&stage_dir, stage, &bam, Some(&bai), None)?;
        write_tool_wrapper_contract(&stage_dir, stage, &plan, &step)?;
        let invariants: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(stage_dir.join("bam_invariants.json"))?)?;
        assert_eq!(
            invariants
                .get("schema_version")
                .and_then(serde_json::Value::as_str),
            Some("bijux.bam.invariants.v1")
        );
        assert_eq!(
            invariants
                .get("sort_order")
                .and_then(serde_json::Value::as_str),
            Some("coordinate")
        );
        assert_eq!(
            invariants
                .pointer("/read_groups/status")
                .and_then(serde_json::Value::as_str),
            Some("present")
        );
        Ok(())
    }

    #[test]
    fn bam_output_contract_and_resume_marker_follow_artifacts() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("align");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        let stage = bijux_dna_planner_bam::stage_api::BamStage::Align;
        std::fs::write(stage_dir.join("align.bam"), b"bam")?;
        std::fs::write(stage_dir.join("align.bam.bai"), b"bai")?;
        std::fs::write(stage_dir.join("stage_loss_accounting.json"), b"{}")?;
        write_bam_output_contract(stage, &stage_dir)?;

        let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&mock_plan(stage));
        let resumed = maybe_resume_bam_stage(stage, &stage_dir, &step)?
            .expect("resume should trigger with complete artifacts");
        assert_eq!(resumed.result.command, "resume-skip");
        assert!(stage_dir.join("stage_resume.json").exists());
        Ok(())
    }

}
