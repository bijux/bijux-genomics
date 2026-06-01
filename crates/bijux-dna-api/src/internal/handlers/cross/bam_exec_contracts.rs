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
            stage_instance_id: None,
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
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
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
        let validate_bam = validate_dir.join("in.bam");
        let validate_index = validate_dir.join("in.bam.bai");
        std::fs::write(&validate_bam, b"@HD\tVN:1.6\n")?;
        std::fs::write(&validate_index, b"bai")?;
        bijux_dna_infra::atomic_write_bytes(
            &validate_dir.join("flagstat.txt"),
            b"10 + 0 in total (QC-passed reads + QC-failed reads)\n8 + 0 mapped (80.00% : N/A)\n2 + 0 duplicates\n",
        )?;
        let mut validate_plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Validate);
        validate_plan.io.inputs[0].path = validate_bam;
        validate_plan.io.inputs.push(ArtifactRef::required(
            ArtifactId::new("bai"),
            validate_index.clone(),
            ArtifactRole::Index,
        ));
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Validate,
            &validate_dir,
            &validate_plan,
        )?;
        let validate_summary: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            validate_dir.join("validation.summary.json"),
        )?)?;
        assert_eq!(
            validate_summary.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.validate.v1")
        );
        assert_eq!(
            validate_summary.get("bam_index").and_then(serde_json::Value::as_str),
            Some(validate_index.to_string_lossy().as_ref())
        );

        let mapping_dir = temp.path().join("mapping_summary");
        bijux_dna_infra::ensure_dir(&mapping_dir)?;
        let mapping_bam = mapping_dir.join("in.bam");
        let mapping_index = mapping_dir.join("in.bam.bai");
        std::fs::write(&mapping_bam, b"@HD\tVN:1.6\n")?;
        std::fs::write(&mapping_index, b"bai")?;
        bijux_dna_infra::atomic_write_bytes(
            &mapping_dir.join("flagstat.txt"),
            b"20 + 0 in total (QC-passed reads + QC-failed reads)\n15 + 0 mapped (75.00% : N/A)\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(
            &mapping_dir.join("samtools_stats.txt"),
            b"SN\traw total sequences:\t20\n",
        )?;
        let mut mapping_plan =
            mock_plan(bijux_dna_planner_bam::stage_api::BamStage::MappingSummary);
        mapping_plan.io.inputs[0].path = mapping_bam;
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::MappingSummary,
            &mapping_dir,
            &mapping_plan,
        )?;
        let mapping_summary: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            mapping_dir.join("mapping_summary.json"),
        )?)?;
        assert_eq!(
            mapping_summary.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.mapping_summary.v1")
        );
        Ok(())
    }

    #[test]
    fn qc_pre_postprocess_emits_core_summary_outputs() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("qc_pre");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("flagstat.txt"),
            b"3 + 0 in total (QC-passed reads + QC-failed reads)\n3 + 0 mapped (100.00% : N/A)\n1 + 0 duplicates\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("idxstats.txt"),
            b"chr1\t100\t2\t0\nchr2\t80\t1\t0\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("samtools_stats.txt"),
            b"RL\t8\t3\nMQ\t10\t1\nMQ\t25\t1\nMQ\t60\t1\n",
        )?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::QcPre);
        plan.io.inputs[0].path = stage_dir.join("input.sam");
        stage_postprocess(bijux_dna_planner_bam::stage_api::BamStage::QcPre, &stage_dir, &plan)?;
        let summary: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(stage_dir.join("qc_pre.summary.json"))?)?;
        assert_eq!(summary["stage_id"], serde_json::json!("bam.qc_pre"));
        assert_eq!(summary["total_reads"], serde_json::json!(3));
        assert_eq!(summary["mapped_reads"], serde_json::json!(3));
        assert_eq!(summary["unmapped_reads"], serde_json::json!(0));
        assert_eq!(summary["duplicate_flagged_reads"], serde_json::json!(1));
        assert_eq!(
            summary["contig_summary"],
            serde_json::json!([
                {"contig":"chr1","length":100,"mapped":2,"unmapped":0},
                {"contig":"chr2","length":80,"mapped":1,"unmapped":0}
            ])
        );
        Ok(())
    }

    #[test]
    fn mapq_filter_postprocess_emits_typed_summary() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("mapq_filter");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("flagstat.before.txt"),
            b"20 + 0 in total (QC-passed reads + QC-failed reads)\n15 + 0 mapped (75.00% : N/A)\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("flagstat.after.txt"),
            b"12 + 0 in total (QC-passed reads + QC-failed reads)\n9 + 0 mapped (75.00% : N/A)\n",
        )?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::MapqFilter);
        plan.params = serde_json::json!({ "mapq_threshold": 30 });
        plan.io.inputs[0].path = stage_dir.join("input.bam");
        plan.io.outputs[0].path = stage_dir.join("filtered.bam");
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::MapqFilter,
            &stage_dir,
            &plan,
        )?;
        let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            stage_dir.join("mapq_filter.summary.json"),
        )?)?;
        assert_eq!(
            payload.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.mapq_filter.v1")
        );
        assert_eq!(payload.get("mapq_threshold").and_then(serde_json::Value::as_u64), Some(30));
        assert_eq!(
            payload.get("mapped_reads_removed").and_then(serde_json::Value::as_u64),
            Some(6)
        );
        assert_eq!(payload.get("input_reads").and_then(serde_json::Value::as_u64), Some(20));
        assert_eq!(payload.get("kept_reads").and_then(serde_json::Value::as_u64), Some(12));
        assert_eq!(payload.get("removed_reads").and_then(serde_json::Value::as_u64), Some(8));
        assert_eq!(
            payload.get("mapped_fraction_retained").and_then(serde_json::Value::as_f64),
            Some(0.6)
        );
        Ok(())
    }

    #[test]
    fn complexity_postprocess_emits_typed_summary() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("complexity");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        bijux_dna_infra::atomic_write_bytes(&stage_dir.join("preseq.txt"), b"3\t2\n")?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Complexity);
        plan.params = serde_json::json!({ "min_reads": 3, "projection_points": [6, 12] });
        plan.io.inputs[0].path = stage_dir.join("input.bam");
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Complexity,
            &stage_dir,
            &plan,
        )?;
        let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            stage_dir.join("complexity.summary.json"),
        )?)?;
        assert_eq!(
            payload.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.complexity.v1")
        );
        assert_eq!(payload.get("method").and_then(serde_json::Value::as_str), Some("samtools"));
        assert_eq!(
            payload.get("observed_total_reads").and_then(serde_json::Value::as_u64),
            Some(3)
        );
        assert_eq!(
            payload.get("observed_unique_reads").and_then(serde_json::Value::as_u64),
            Some(2)
        );
        assert_eq!(
            payload.get("insufficient_data_reason").and_then(serde_json::Value::as_str),
            Some("insufficient_observed_unique_reads_for_complexity_extrapolation")
        );
        assert!(stage_dir.join("complexity.artifacts.json").exists());
        Ok(())
    }

    #[test]
    fn insert_size_postprocess_emits_typed_summary() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("insert_size");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("insert_size.metrics.txt"),
            b"## htsjdk.samtools.metrics.StringHeader\n\
# picard CollectInsertSizeMetrics synthetic fixture\n\
## METRICS CLASS\tpicard.analysis.InsertSizeMetrics\n\
MEDIAN_INSERT_SIZE\tMODE_INSERT_SIZE\tMEDIAN_ABSOLUTE_DEVIATION\tMIN_INSERT_SIZE\tMAX_INSERT_SIZE\tMEAN_INSERT_SIZE\tSTANDARD_DEVIATION\tREAD_PAIRS\tPAIR_ORIENTATION\tWIDTH_OF_10_PERCENT\tWIDTH_OF_20_PERCENT\tWIDTH_OF_30_PERCENT\tWIDTH_OF_40_PERCENT\tWIDTH_OF_50_PERCENT\tWIDTH_OF_60_PERCENT\tWIDTH_OF_70_PERCENT\tWIDTH_OF_80_PERCENT\tWIDTH_OF_90_PERCENT\tWIDTH_OF_99_PERCENT\tSAMPLE\tLIBRARY\tREAD_GROUP\n\
20\t20\t5\t15\t30\t21.666666666666668\t6.236095644623236\t3\tFR\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0\tNA\tNA\tNA\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("insert_size.histogram.pdf"),
            b"%PDF-1.4\n",
        )?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::InsertSize);
        plan.io.inputs[0].path = stage_dir.join("input.bam");
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::InsertSize,
            &stage_dir,
            &plan,
        )?;
        let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            stage_dir.join("insert_size.summary.json"),
        )?)?;
        assert_eq!(
            payload.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.insert_size.v1")
        );
        assert_eq!(payload.get("report_present").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            payload.get("histogram_present").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(payload.get("read_pairs").and_then(serde_json::Value::as_u64), Some(3));
        assert_eq!(
            payload.get("median_insert_size").and_then(serde_json::Value::as_f64),
            Some(20.0)
        );
        assert_eq!(
            payload.get("mean_insert_size").and_then(serde_json::Value::as_f64),
            Some(21.666666666666668)
        );
        assert_eq!(payload.get("min_insert_size").and_then(serde_json::Value::as_u64), Some(15));
        assert_eq!(payload.get("max_insert_size").and_then(serde_json::Value::as_u64), Some(30));
        assert_eq!(payload.get("insufficient_pairs_reason"), Some(&serde_json::Value::Null));
        Ok(())
    }

    #[test]
    fn gc_bias_postprocess_emits_typed_summary() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("gc_bias");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("gc_bias.metrics.txt"),
            b"## htsjdk.samtools.metrics.StringHeader\n\
# picard CollectGcBiasMetrics synthetic fixture\n\
## METRICS CLASS\tpicard.analysis.GcBiasMetrics\n\
ACCUMULATION_LEVEL\tREADS_USED\tWINDOW_SIZE\tTOTAL_CLUSTERS\tALIGNED_READS\tAT_DROPOUT\tGC_DROPOUT\tWINDOWS\tREAD_STARTS\n\
ALL_READS\tALL\t10\t4\t4\t25.0\t25.0\t3\t4\n\
",
        )?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("gc_bias.plot.pdf"),
            b"%PDF-1.4\n",
        )?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::GcBias);
        plan.io.inputs[0].path = stage_dir.join("input.bam");
        plan.io.inputs.push(ArtifactRef::required(
            ArtifactId::new("reference"),
            stage_dir.join("reference.fasta"),
            ArtifactRole::Reference,
        ));
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::GcBias,
            &stage_dir,
            &plan,
        )?;
        let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            stage_dir.join("gc_bias.summary.json"),
        )?)?;
        assert_eq!(
            payload.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.gc_bias.v1")
        );
        assert_eq!(payload.get("report_present").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(payload.get("plot_present").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(payload.get("total_clusters").and_then(serde_json::Value::as_u64), Some(4));
        assert_eq!(payload.get("aligned_reads").and_then(serde_json::Value::as_u64), Some(4));
        assert_eq!(payload.get("windows").and_then(serde_json::Value::as_u64), Some(3));
        assert_eq!(payload.get("read_starts").and_then(serde_json::Value::as_u64), Some(4));
        assert_eq!(payload.get("at_dropout").and_then(serde_json::Value::as_f64), Some(25.0));
        assert_eq!(payload.get("gc_dropout").and_then(serde_json::Value::as_f64), Some(25.0));
        assert_eq!(payload.get("gc_bias_score").and_then(serde_json::Value::as_f64), Some(0.25));
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
        assert!(damage_dir.join("advisory_boundary.json").exists());

        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Authenticity,
            &authenticity_dir,
            &mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Authenticity),
        )?;
        let authenticity: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            authenticity_dir.join("authenticity_composite.json"),
        )?)?;
        assert!(authenticity.get("composite_score").is_some());
        let canonical: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            authenticity_dir.join("authenticity.json"),
        )?)?;
        assert_eq!(
            canonical.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.authenticity.v1")
        );
        assert!(authenticity_dir.join("advisory_boundary.json").exists());
        Ok(())
    }

    #[test]
    fn duplicate_policy_outputs_are_typed_and_stage_specific() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let dup_dir = temp.path().join("duplication_metrics");
        let markdup_dir = temp.path().join("markdup");
        bijux_dna_infra::ensure_dir(&dup_dir)?;
        bijux_dna_infra::ensure_dir(&markdup_dir)?;

        let mut dup_plan =
            mock_plan(bijux_dna_planner_bam::stage_api::BamStage::DuplicationMetrics);
        dup_plan.params = serde_json::json!({
            "optical_duplicates": "mark_only",
            "umi_policy": "ignore",
            "duplicate_action": "mark"
        });
        bijux_dna_infra::atomic_write_json(
            &dup_dir.join("duplication.metrics.json"),
            &serde_json::json!({
                "method": "samtools",
                "source": dup_dir.join("duplication.histogram.txt"),
                "examined_pairs": 3,
                "duplicate_pairs": 1
            }),
        )?;
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::DuplicationMetrics,
            &dup_dir,
            &dup_plan,
        )?;
        let duplication_policy: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dup_dir.join("duplication.policy.json"))?,
        )?;
        assert_eq!(
            duplication_policy.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.duplicate_policy.v1")
        );
        assert_eq!(
            duplication_policy.get("policy_scope").and_then(serde_json::Value::as_str),
            Some("observation_only")
        );
        let duplication_summary: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dup_dir.join("duplication.summary.json"))?,
        )?;
        assert_eq!(
            duplication_summary.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.duplication_metrics.v1")
        );
        assert_eq!(
            duplication_summary.get("method").and_then(serde_json::Value::as_str),
            Some("samtools")
        );
        assert_eq!(
            duplication_summary.get("duplicate_action").and_then(serde_json::Value::as_str),
            Some("mark")
        );
        assert_eq!(
            duplication_summary.get("examined_reads").and_then(serde_json::Value::as_u64),
            Some(3)
        );
        assert_eq!(
            duplication_summary.get("duplicate_reads").and_then(serde_json::Value::as_u64),
            Some(1)
        );
        assert_eq!(
            duplication_summary.get("duplicate_fraction").and_then(serde_json::Value::as_f64),
            Some(1.0 / 3.0)
        );
        assert_eq!(
            duplication_summary
                .get("insufficient_library_size_reason")
                .and_then(serde_json::Value::as_str),
            Some("tool_report_did_not_provide_library_size_estimate")
        );

        let mut markdup_plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Markdup);
        markdup_plan.params = serde_json::json!({
            "library_type": "ssdna",
            "optical_duplicates": "remove",
            "umi_policy": "use_tag",
            "duplicate_action": "remove"
        });
        bijux_dna_infra::atomic_write_bytes(
            &markdup_dir.join("flagstat.before.txt"),
            b"10 + 0 in total (QC-passed reads + QC-failed reads)\n8 + 0 mapped (80.00% : N/A)\n2 + 0 duplicates\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(
            &markdup_dir.join("flagstat.after.txt"),
            b"8 + 0 in total (QC-passed reads + QC-failed reads)\n7 + 0 mapped (87.50% : N/A)\n1 + 0 duplicates\n",
        )?;
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Markdup,
            &markdup_dir,
            &markdup_plan,
        )?;
        let markdup_policy: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            markdup_dir.join("markdup.policy.json"),
        )?)?;
        assert_eq!(
            markdup_policy.get("library_type").and_then(serde_json::Value::as_str),
            Some("ssdna")
        );
        assert_eq!(
            markdup_policy.get("policy_scope").and_then(serde_json::Value::as_str),
            Some("pcr_vs_optical")
        );
        let markdup_summary: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            markdup_dir.join("markdup.summary.json"),
        )?)?;
        assert_eq!(
            markdup_summary.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.markdup.v1")
        );
        assert_eq!(
            markdup_summary.get("duplicate_action").and_then(serde_json::Value::as_str),
            Some("remove")
        );
        assert_eq!(markdup_summary.get("removed_reads"), Some(&serde_json::json!(2)));
        assert_eq!(markdup_summary.get("duplicate_reads_removed"), Some(&serde_json::json!(2)));
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
            None,
        ) else {
            panic!("align must require reference");
        };
        assert!(err.to_string().contains("requires resolved reference fasta"));

        let ref_fa = temp.path().join("ref.fa");
        let ref_fai = temp.path().join("ref.fa.fai");
        std::fs::write(&ref_fa, b">1\nACGT\n").unwrap_or_else(|err| panic!("write ref: {err}"));
        std::fs::write(&ref_fai, b"1\t4\t0\t4\t5\n")
            .unwrap_or_else(|err| panic!("write fai: {err}"));
        let Err(err) = enforce_stage_refusal_rules(
            bijux_dna_planner_bam::stage_api::BamStage::Sex,
            &bam,
            Some(&temp.path().join("x.bam.bai")),
            Some(&ref_fa),
            None,
        ) else {
            panic!("sex must require X/Y contigs");
        };
        assert!(err.to_string().contains("lacks required X/Y contigs"));
    }

    #[test]
    fn refusal_rules_enforce_mt_reference_for_mt_aware_stages() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let bam = temp.path().join("x.bam");
        let bam_index = temp.path().join("x.bam.bai");
        std::fs::write(&bam, b"bam")?;
        std::fs::write(&bam_index, b"bai")?;
        let ref_fa = temp.path().join("ref.fa");
        let ref_fai = temp.path().join("ref.fa.fai");
        std::fs::write(&ref_fa, b">1\nACGT\n")?;
        std::fs::write(&ref_fai, b"1\t4\t0\t4\t5\n")?;
        let Err(err) = enforce_stage_refusal_rules(
            bijux_dna_planner_bam::stage_api::BamStage::Contamination,
            &bam,
            Some(&bam_index),
            Some(&ref_fa),
            None,
        ) else {
            panic!("contamination must fail when mt contig is absent");
        };
        assert!(err.to_string().contains("lacks MT/chrMT contig"));
        Ok(())
    }

    #[test]
    fn refusal_rules_allow_missing_read_groups_when_policy_override_set() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let bam = temp.path().join("x.bam");
        let bam_index = temp.path().join("x.bam.bai");
        let ref_fa = temp.path().join("ref.fa");
        let ref_fai = temp.path().join("ref.fa.fai");
        std::fs::write(&bam, b"@HD\tVN:1.6\tSO:coordinate\n")?;
        std::fs::write(&bam_index, b"bai")?;
        std::fs::write(&ref_fa, b">chr1\nACGT\n>chrX\nACGT\n>chrY\nACGT\n>chrM\nACGT\n")?;
        std::fs::write(
            &ref_fai,
            b"chr1\t4\t0\t4\t5\nchrX\t4\t10\t4\t5\nchrY\t4\t20\t4\t5\nchrM\t4\t30\t4\t5\n",
        )?;

        enforce_stage_refusal_rules(
            bijux_dna_planner_bam::stage_api::BamStage::Validate,
            &bam,
            Some(&bam_index),
            Some(&ref_fa),
            Some("allow_missing"),
        )?;
        Ok(())
    }

    #[test]
    fn refusal_rules_reject_incomplete_read_group_fields() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let bam = temp.path().join("x.bam");
        let bam_index = temp.path().join("x.bam.bai");
        std::fs::write(&bam, b"@HD\tVN:1.6\tSO:coordinate\n@RG\tID:rg-s1\tSM:s1\n")?;
        std::fs::write(&bam_index, b"bai")?;
        let Err(err) = enforce_stage_refusal_rules(
            bijux_dna_planner_bam::stage_api::BamStage::Validate,
            &bam,
            Some(&bam_index),
            None,
            None,
        ) else {
            panic!("validate should reject incomplete read-group fields");
        };
        assert!(err.to_string().contains("missing required fields"));
        Ok(())
    }

    #[test]
    fn bam_sample_identity_manifest_prefers_declared_read_group_contract() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let bam = temp.path().join("align.bam");
        std::fs::write(&bam, b"@HD\tVN:1.6\tSO:coordinate\n@RG\tID:rg1\tSM:sample-a\tPL:ILLUMINA\tLB:lib-a\tPU:pu-a\n")?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Align);
        plan.params = serde_json::json!({
            "sample_id": "sample-a",
            "subject_id": "subject-a",
            "cohort_id": "cohort-a",
            "read_group": {
                "id": "rg1",
                "sample": "sample-a",
                "platform": "ILLUMINA",
                "library": "lib-a",
                "platform_unit": "pu-a",
                "lane_id": "L001",
                "run_id": "run-a"
            }
        });
        let identity =
            write_bam_sample_identity_manifest(temp.path(), &plan, Some(&bam), Some("preserve"))?;
        assert_eq!(identity.sample_id, "sample-a");
        assert_eq!(identity.lane_id.as_deref(), Some("L001"));
        assert_eq!(identity.subject_id.as_deref(), Some("subject-a"));
        assert!(temp.path().join("sample_identity.json").exists());
        Ok(())
    }

    #[test]
    fn bam_reference_preflight_records_required_assets_for_bwa() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let reference = temp.path().join("ref.fa");
        std::fs::write(&reference, b">chr1\nACGT\n")?;
        std::fs::write(temp.path().join("ref.fa.fai"), b"chr1\t4\t0\t4\t5\n")?;
        std::fs::write(temp.path().join("ref.fa.dict"), b"@SQ\tSN:chr1\tLN:4\n")?;
        std::fs::write(temp.path().join("ref.fa.bwt"), b"bwt")?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Align);
        plan.tool_id = ToolId::new("bwa");
        plan.params = serde_json::json!({
            "reference_digest": "sha256:ref"
        });
        write_bam_reference_preflight(
            temp.path(),
            bijux_dna_planner_bam::stage_api::BamStage::Align,
            &plan,
            &reference,
        )?;
        let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            temp.path().join("reference_preflight.json"),
        )?)?;
        assert_eq!(payload.get("passes").and_then(serde_json::Value::as_bool), Some(true));
        assert!(payload.get("required_assets").and_then(serde_json::Value::as_array).is_some_and(
            |assets| assets.iter().any(|asset| {
                asset.get("asset_kind").and_then(serde_json::Value::as_str) == Some("bwa_index")
            })
        ));
        Ok(())
    }

    #[test]
    fn bam_alignment_provenance_captures_seed_and_identity() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Align);
        plan.params = serde_json::json!({
            "sample_id": "sample-a",
            "reference": "ref.fa",
            "reference_digest": "sha256:ref",
            "preset": "modern_default",
            "sensitivity_profile": "very_sensitive",
            "seed_length": 21,
            "read_group": {
                "id": "rg1",
                "sample": "sample-a",
                "platform": "ILLUMINA",
                "library": "lib-a",
                "platform_unit": "pu-a",
                "lane_id": "L001",
                "run_id": "run-a"
            }
        });
        let identity =
            write_bam_sample_identity_manifest(temp.path(), &plan, None, Some("regenerate"))?;
        write_bam_alignment_provenance(temp.path(), &plan, identity)?;
        let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            temp.path().join("alignment_provenance.json"),
        )?)?;
        assert_eq!(payload.get("seed_length").and_then(serde_json::Value::as_u64), Some(21));
        assert_eq!(
            payload.pointer("/sample_identity/sample_id").and_then(serde_json::Value::as_str),
            Some("sample-a")
        );
        Ok(())
    }

    #[test]
    fn validate_stage_hard_fails_without_bam_index() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("validate");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        std::fs::write(
            stage_dir.join("flagstat.txt"),
            b"10 + 0 in total (QC-passed reads + QC-failed reads)\n",
        )?;
        let bam = temp.path().join("input.bam");
        std::fs::write(&bam, b"@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:4\n")?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Validate);
        plan.io.inputs[0].path = bam.clone();
        let Err(err) = validate_stage_hard_failures(&stage_dir, &plan) else {
            panic!("validate should fail when .bai is missing");
        };
        assert!(err.to_string().contains("missing BAM index"));
        Ok(())
    }

    #[test]
    fn bam_invariants_and_wrapper_contracts_are_emitted() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("validate");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        let bam = temp.path().join("input.bam");
        let bam_index = temp.path().join("input.bam.bai");
        std::fs::write(&bam, b"@HD\tVN:1.6\tSO:coordinate\n@RG\tID:rg1\tSM:s1\n")?;
        std::fs::write(&bam_index, b"index")?;

        let stage = bijux_dna_planner_bam::stage_api::BamStage::Validate;
        let plan = mock_plan(stage);
        let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&plan);
        write_bam_invariants(&stage_dir, stage, &bam, Some(&bam_index), None)?;
        write_tool_wrapper_contract(&stage_dir, stage, &plan, &step)?;
        let invariants: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(stage_dir.join("bam_invariants.json"))?)?;
        assert_eq!(
            invariants.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.invariants.v1")
        );
        assert_eq!(
            invariants.get("sort_order").and_then(serde_json::Value::as_str),
            Some("coordinate")
        );
        assert_eq!(
            invariants.pointer("/read_groups/status").and_then(serde_json::Value::as_str),
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
        let bam = stage_dir.join("align.bam");
        let bam_index = stage_dir.join("align.bam.bai");
        std::fs::write(&bam, b"bam")?;
        std::fs::write(&bam_index, b"bai")?;
        let bam_hash = bijux_dna_infra::hash_file_sha256(&bam)?;
        let index_hash = bijux_dna_infra::hash_file_sha256(&bam_index)?;
        let accounting = serde_json::json!({
            "stage_id": "bam.align",
            "output_checksums": [
                {"path": bam, "sha256": bam_hash},
                {"path": bam_index, "sha256": index_hash}
            ]
        });
        bijux_dna_infra::atomic_write_json(
            &stage_dir.join("stage_loss_accounting.json"),
            &accounting,
        )?;
        let plan = mock_plan(stage);
        write_bam_output_contract(stage, &plan, &stage_dir)?;

        let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&mock_plan(stage));
        let Some(resumed) = maybe_resume_bam_stage(stage, &stage_dir, &step)? else {
            panic!("resume should trigger with complete artifacts");
        };
        assert_eq!(resumed.result.command, "resume-skip");
        assert!(stage_dir.join("stage_resume.json").exists());
        Ok(())
    }

    #[test]
    fn bam_output_contract_enforcement_rejects_missing_index() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("align");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        let stage = bijux_dna_planner_bam::stage_api::BamStage::Align;
        std::fs::write(stage_dir.join("align.bam"), b"bam")?;
        let plan = mock_plan(stage);
        write_bam_output_contract(stage, &plan, &stage_dir)?;
        let Err(err) = enforce_bam_output_contract(stage, &stage_dir) else {
            panic!("enforcement must fail when .bai is missing");
        };
        assert!(err.to_string().contains("output contract violation"));
        Ok(())
    }

    #[test]
    fn alignment_regime_validation_accepts_adna_bwa_command() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let mut step = bijux_dna_stage_contract::execution_step_from_stage_plan(&mock_plan(
            bijux_dna_planner_bam::stage_api::BamStage::Align,
        ));
        step.command = CommandSpecV1 {
            template: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "bwa aln -l 1024 -n 0.01 ref.fa r1.fq".to_string(),
            ],
        };
        write_alignment_regime_validation(temp.path(), AlignmentRegime::Adna, "bwa", &step)?;
        assert!(temp.path().join("alignment_regime_validation.json").exists());
        Ok(())
    }

    #[test]
    fn duplicate_policy_split_codifies_collapse_refusal_path() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Markdup);
        plan.params = serde_json::json!({
            "umi_policy": "collapse",
            "duplicate_action": "mark",
            "optical_duplicates": "mark_only",
            "library_type": "ssdna"
        });
        write_duplicate_policy_split(temp.path(), &plan)?;
        let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            temp.path().join("duplicate_policy_split.json"),
        )?)?;
        assert_eq!(
            payload.get("selected_executor").and_then(serde_json::Value::as_str),
            Some("bam.collapse")
        );
        assert_eq!(
            payload.pointer("/modes/bam.collapse/supported").and_then(serde_json::Value::as_bool),
            Some(false)
        );
        assert_eq!(payload.get("library_type").and_then(serde_json::Value::as_str), Some("ssdna"));
        Ok(())
    }

    #[test]
    fn contamination_postprocess_emits_stratified_summary() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("contamination");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        bijux_dna_infra::atomic_write_json(
            &stage_dir.join("contamination.summary.json"),
            &serde_json::json!({
                "method":"verifybamid2",
                "estimate":0.07,
                "ci_low":0.03,
                "ci_high":0.10,
                "assumptions":[]
            }),
        )?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Contamination);
        plan.params = serde_json::json!({
            "scope": "both",
            "tool_scope": "both",
            "sex_specific": false
        });
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Contamination,
            &stage_dir,
            &plan,
        )?;
        let stratified: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            stage_dir.join("contamination.stratified.json"),
        )?)?;
        assert_eq!(
            stratified.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.bam.contamination_stratified.v1")
        );
        assert_eq!(
            stratified.get("global_estimate").and_then(serde_json::Value::as_f64),
            Some(0.07)
        );
        Ok(())
    }

    #[test]
    fn contamination_postprocess_refuses_verifybamid2_without_af_reference() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("contamination");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Contamination);
        plan.tool_id = ToolId::new("verifybamid2");
        plan.params = serde_json::json!({
            "scope": "both",
            "tool_scope": "both"
        });
        let Err(err) = stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::Contamination,
            &stage_dir,
            &plan,
        ) else {
            panic!("verifybamid2 should fail without AF reference");
        };
        assert!(err.to_string().contains("requires population AF reference panel"));
        Ok(())
    }

    #[test]
    fn mapping_summary_fails_when_mapq_regime_is_below_floor() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("mapping_summary");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("flagstat.txt"),
            b"20 + 0 in total (QC-passed reads + QC-failed reads)\n15 + 0 mapped (75.00% : N/A)\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(&stage_dir.join("samtools_stats.txt"), b"MQ\t0\t20\n")?;
        let Err(err) = stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::MappingSummary,
            &stage_dir,
            &mock_plan(bijux_dna_planner_bam::stage_api::BamStage::MappingSummary),
        ) else {
            panic!("mapq regime should fail for zero MAPQ");
        };
        assert!(err.to_string().contains("mapQ mean"));
        Ok(())
    }

    #[test]
    fn overlap_and_endogenous_postprocess_emit_explicit_artifacts() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let overlap_dir = temp.path().join("overlap_correction");
        bijux_dna_infra::ensure_dir(&overlap_dir)?;
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection,
            &overlap_dir,
            &mock_plan(bijux_dna_planner_bam::stage_api::BamStage::OverlapCorrection),
        )?;
        assert!(overlap_dir.join("overlap_correction.outputs.json").exists());

        let endogenous_dir = temp.path().join("endogenous_content");
        bijux_dna_infra::ensure_dir(&endogenous_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &endogenous_dir.join("flagstat.txt"),
            b"10 + 0 in total (QC-passed reads + QC-failed reads)\n6 + 0 mapped (60.00% : N/A)\n",
        )?;
        let mut plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::EndogenousContent);
        plan.params = serde_json::json!({ "competitive_mapping": true });
        stage_postprocess(
            bijux_dna_planner_bam::stage_api::BamStage::EndogenousContent,
            &endogenous_dir,
            &plan,
        )?;
        let endogenous: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            endogenous_dir.join("endogenous.content.json"),
        )?)?;
        assert_eq!(
            endogenous.get("competitive_mapping_enabled").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        Ok(())
    }

    #[test]
    fn bam_qc_aggregator_emits_tsv_with_stage_rows() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let bam_root = temp.path().join("bam");
        let stage_dir = bam_root.join("mapping_summary");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("flagstat.txt"),
            b"10 + 0 in total (QC-passed reads + QC-failed reads)\n8 + 0 mapped (80.00% : N/A)\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(
            &stage_dir.join("samtools_stats.txt"),
            b"MQ\t30\t10\n",
        )?;
        write_bam_qc_aggregator_tsv(&bam_root)?;
        let raw = std::fs::read_to_string(bam_root.join("bam_qc.tsv"))?;
        assert!(raw.lines().next().is_some_and(|line| line.contains("stage")));
        assert!(raw.contains("mapping_summary"));
        Ok(())
    }

    #[test]
    fn coverage_regime_classifier_uses_governed_contracts() {
        let sparse = bijux_dna_domain_bam::classify_bam_coverage_regime(0.5, 0.2);
        assert_eq!(sparse.regime_id, "sparse");
        let low_pass = bijux_dna_domain_bam::classify_bam_coverage_regime(2.0, 0.5);
        assert_eq!(low_pass.regime_id, "low_pass");
        let target_like = bijux_dna_domain_bam::classify_bam_coverage_regime(8.0, 0.6);
        assert_eq!(target_like.regime_id, "target_like");
        let whole_genome_like = bijux_dna_domain_bam::classify_bam_coverage_regime(20.0, 0.9);
        assert_eq!(whole_genome_like.regime_id, "whole_genome_like");
    }

    #[test]
    fn bam_stage_contract_suite_emits_normalized_metrics_for_all_stages() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let result = bijux_dna_runner::step_runner::StageResultV1 {
            run_id: "test".to_string(),
            exit_code: 0,
            runtime_s: 1.0,
            memory_mb: 64.0,
            outputs: Vec::new(),
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "tool".to_string(),
        };
        for stage in bijux_dna_domain_bam::BamStage::all() {
            let runtime_stage =
                bijux_dna_planner_bam::stage_api::BamStage::try_from(stage.as_str())?;
            let stage_dir = temp.path().join(stage.as_str().trim_start_matches("bam."));
            bijux_dna_infra::ensure_dir(&stage_dir)?;
            let plan = mock_plan(runtime_stage);
            write_normalized_bam_metrics(runtime_stage, &stage_dir, &plan, &result)?;
            let metrics_path = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&stage_dir)
                .join("metrics.json");
            let payload: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(metrics_path)?)?;
            let stage_metrics_path = stage_dir.join("metrics.json");
            let stage_payload: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(stage_metrics_path)?)?;
            assert_eq!(
                payload.pointer("/metrics/schema_version").and_then(serde_json::Value::as_str),
                Some("bijux.bam.metrics.normalized.v1")
            );
            assert_eq!(
                stage_payload.get("schema_version").and_then(serde_json::Value::as_str),
                Some("bijux.bam.metrics.normalized.v1")
            );
            assert_eq!(
                payload.pointer("/metrics/stage_id").and_then(serde_json::Value::as_str),
                Some(stage.as_str())
            );
            assert_eq!(
                stage_payload.get("stage_id").and_then(serde_json::Value::as_str),
                Some(stage.as_str())
            );
            assert!(payload.pointer("/metrics/normalized_keys").is_some());
            assert!(stage_payload.get("normalized_keys").is_some());
        }
        Ok(())
    }

    #[test]
    fn bam_smoke_runner_minimal_pipeline_validates_report_section_presence() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let bam_root = temp.path().join("bam");
        let mapping = bam_root.join("mapping_summary");
        let coverage = bam_root.join("coverage");
        let contamination = bam_root.join("contamination");
        for dir in [&mapping, &coverage, &contamination] {
            bijux_dna_infra::ensure_dir(dir)?;
        }
        bijux_dna_infra::atomic_write_bytes(
            &mapping.join("flagstat.txt"),
            b"10 + 0 in total (QC-passed reads + QC-failed reads)\n8 + 0 mapped (80.00% : N/A)\n",
        )?;
        bijux_dna_infra::atomic_write_bytes(&mapping.join("samtools_stats.txt"), b"MQ\t30\t10\n")?;
        bijux_dna_infra::atomic_write_bytes(
            &coverage.join("coverage.depth.txt"),
            b"chr1\t1\t2\nchr1\t2\t3\n",
        )?;
        bijux_dna_infra::atomic_write_json(
            &contamination.join("contamination.summary.json"),
            &serde_json::json!({"estimate": 0.02}),
        )?;
        write_bam_qc_aggregator_tsv(&bam_root)?;
        let qc = std::fs::read_to_string(bam_root.join("bam_qc.tsv"))?;
        let header = qc.lines().next().unwrap_or_default();
        assert!(header.contains("stage"));
        assert!(header.contains("mapped_fraction"));
        assert!(header.contains("contamination_estimate"));
        assert!(qc.contains("mapping_summary"));
        Ok(())
    }

    #[test]
    fn bam_stage_contract_suite_uses_golden_toy_sam_and_bam_inputs() -> Result<()> {
        let workspace = crate::support::workspace::resolve_repo_root()?;
        let toy_sam = workspace.join("assets/golden/smoke-inputs-v1/bam/toy.sam");
        let sample_bam = workspace
            .join("crates/bijux-dna-planner-bam/tests/fixtures/plan_inputs/default/sample.bam");
        assert!(toy_sam.exists(), "missing {}", toy_sam.display());
        assert!(sample_bam.exists(), "missing {}", sample_bam.display());

        let temp = tempfile::tempdir()?;
        let stage_dir = temp.path().join("validate");
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        write_bam_invariants(
            &stage_dir,
            bijux_dna_planner_bam::stage_api::BamStage::Validate,
            &toy_sam,
            None,
            None,
        )?;
        let plan = mock_plan(bijux_dna_planner_bam::stage_api::BamStage::Validate);
        let result = bijux_dna_runner::step_runner::StageResultV1 {
            run_id: "test".to_string(),
            exit_code: 0,
            runtime_s: 0.5,
            memory_mb: 32.0,
            outputs: vec![sample_bam],
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: "samtools".to_string(),
        };
        write_normalized_bam_metrics(
            bijux_dna_planner_bam::stage_api::BamStage::Validate,
            &stage_dir,
            &plan,
            &result,
        )?;
        assert!(stage_dir.join("bam_invariants.json").exists());
        assert!(bijux_dna_runtime::recording::run_artifacts_dir_for_out(&stage_dir)
            .join("metrics.json")
            .exists());
        Ok(())
    }
}
