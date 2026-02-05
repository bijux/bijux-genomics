pub mod validate {
    use std::path::Path;

    use bijux_core::{
        CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_domain_bam::params::ValidateEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Validate.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        bam_index: Option<&Path>,
        reference: Option<&Path>,
        out_dir: &Path,
    ) -> anyhow::Result<StagePlanV1> {
        let effective_params = ValidateEffectiveParams { strict: true };
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Validate, out_dir);
        let flagstat = out_dir.join("flagstat.txt");
        let report = out_dir.join("validation.json");
        let mut inputs = vec![bijux_core::plan::stage_plan::ArtifactRef {
            name: "bam".to_string(),
            path: bam.to_path_buf(),
        }];
        if let Some(reference) = reference {
            inputs.push(bijux_core::plan::stage_plan::ArtifactRef {
                name: "reference".to_string(),
                path: reference.to_path_buf(),
            });
        }
        if let Some(bam_index) = bam_index {
            inputs.push(bijux_core::plan::stage_plan::ArtifactRef {
                name: "bam_bai".to_string(),
                path: bam_index.to_path_buf(),
            });
        }
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: crate::tools::samtools::validate_args(
                    bam,
                    &flagstat,
                    &report,
                    &effective_params,
                ),
            },
            resources: tool.resources.clone(),
            io: StageIO { inputs, outputs },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "bai": bam_index,
                "reference": reference,
                "strict": effective_params.strict,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(&effective_params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["validation_report", "flagstat", "stage_metrics"],
        )
    }
}

pub mod align {
    use std::path::Path;

    use bijux_core::{
        CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_domain_bam::params::{AlignEffectiveParams, ReadGroupSpec};

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Align.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        r1: &Path,
        r2: Option<&Path>,
        reference: &Path,
        sample_id: &str,
        params: &AlignEffectiveParams,
        out_dir: &Path,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Align, out_dir);
        let _assets =
            crate::stages_support::ensure_reference_assets(reference, params.build_indices, true)?;
        let read_group = if params.read_group.id.is_empty() {
            ReadGroupSpec::with_defaults(sample_id)
        } else {
            params.read_group.clone()
        };
        let mut effective = params.clone();
        effective.read_group = read_group;

        let command = match tool.tool_id.0.as_str() {
            "bwa" => crate::tools::bwa::align_args(reference, r1, r2, out_dir, &effective),
            "bowtie2" => crate::tools::bowtie2::align_args(reference, r1, r2, out_dir, &effective),
            other => {
                return Err(anyhow::anyhow!("unsupported align tool: {other}"));
            }
        };

        let inputs = {
            let mut items = vec![
                bijux_core::plan::stage_plan::ArtifactRef {
                    name: "fastq_r1".to_string(),
                    path: r1.to_path_buf(),
                },
                bijux_core::plan::stage_plan::ArtifactRef {
                    name: "reference".to_string(),
                    path: reference.to_path_buf(),
                },
            ];
            if let Some(r2) = r2 {
                items.push(bijux_core::plan::stage_plan::ArtifactRef {
                    name: "fastq_r2".to_string(),
                    path: r2.to_path_buf(),
                });
            }
            items
        };

        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
            resources: tool.resources.clone(),
            io: StageIO { inputs, outputs },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "r1": r1,
                "r2": r2,
                "reference": reference,
                "sample_id": sample_id,
                "aligner": effective.aligner,
                "preset": effective.preset,
                "threads": effective.threads,
                "build_indices": effective.build_indices,
                "read_group": {
                    "id": effective.read_group.id,
                    "sample": effective.read_group.sample,
                    "platform": effective.read_group.platform,
                    "library": effective.read_group.library,
                }
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(&effective).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &[
                "align_bam",
                "align_bai",
                "flagstat",
                "idxstats",
                "stats",
                "align_metrics",
                "stage_metrics",
            ],
        )
    }
}

pub mod qc_pre {
    use std::path::Path;

    use bijux_core::{
        CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_domain_bam::params::QcPreEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::QcPre.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
    ) -> anyhow::Result<StagePlanV1> {
        let effective_params = QcPreEffectiveParams { regions: None };
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::QcPre, out_dir);
        let flagstat = out_dir.join("flagstat.txt");
        let idxstats = out_dir.join("idxstats.txt");
        let stats = out_dir.join("samtools_stats.txt");
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: crate::tools::samtools::qc_pre_args(
                    bam,
                    &flagstat,
                    &idxstats,
                    &stats,
                    &effective_params,
                ),
            },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_core::plan::stage_plan::ArtifactRef {
                    name: "bam".to_string(),
                    path: bam.to_path_buf(),
                }],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "regions": effective_params.regions,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(&effective_params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &["flagstat", "idxstats", "stats", "stage_metrics"],
        )
    }
}

pub mod filter {
    use std::path::Path;

    use bijux_core::{
        CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_domain_bam::params::FilterEffectiveParams;

    pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Filter.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &FilterEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs =
            crate::stages_support::audit_outputs(bijux_domain_bam::BamStage::Filter, out_dir);
        let out_bam = out_dir.join("filtered.bam");
        let flagstat_before = out_dir.join("flagstat.before.txt");
        let flagstat_after = out_dir.join("flagstat.after.txt");
        let idxstats_before = out_dir.join("idxstats.before.txt");
        let idxstats_after = out_dir.join("idxstats.after.txt");
        let summary = out_dir.join("filter.summary.json");
        let plan = StagePlanV1 {
            stage_id: StageId(STAGE_ID.to_string()),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: crate::tools::samtools::filter_args_with_audit(
                    bam,
                    params,
                    &out_bam,
                    &flagstat_before,
                    &flagstat_after,
                    &idxstats_before,
                    &idxstats_after,
                    &summary,
                ),
            },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_core::plan::stage_plan::ArtifactRef {
                    name: "bam".to_string(),
                    path: bam.to_path_buf(),
                }],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "mapq_threshold": params.mapq_threshold,
                "include_flags": params.include_flags,
                "exclude_flags": params.exclude_flags,
                "min_length": params.min_length,
                "remove_duplicates": params.remove_duplicates,
                "base_quality_threshold": params.base_quality_threshold,
            }),
            effective_params: crate::stages_support::ensure_effective_params(
                serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
        };
        crate::stages_support::ensure_required_outputs(
            plan,
            &[
                "filtered_bam",
                "filtered_bai",
                "flagstat_before",
                "flagstat_after",
                "idxstats_before",
                "idxstats_after",
                "summary",
                "stage_metrics",
            ],
        )
    }
}
