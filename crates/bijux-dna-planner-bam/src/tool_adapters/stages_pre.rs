//! Stage adapters for pre-alignment validation and preprocessing stages.

pub mod validate {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::ValidateEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Validate.as_str();
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
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Validate,
            out_dir,
        );
        let flagstat = out_dir.join("flagstat.txt");
        let report = out_dir.join("validation.json");
        let mut inputs = vec![bijux_dna_stage_contract::ArtifactRef::required(
            ArtifactId::from_static("bam"),
            bam.to_path_buf(),
            ArtifactRole::Bam,
        )];
        if let Some(reference) = reference {
            inputs.push(bijux_dna_stage_contract::ArtifactRef::required(
                ArtifactId::from_static("reference"),
                reference.to_path_buf(),
                ArtifactRole::Index,
            ));
        }
        if let Some(bam_index) = bam_index {
            inputs.push(bijux_dna_stage_contract::ArtifactRef::required(
                ArtifactId::from_static("bam_bai"),
                bam_index.to_path_buf(),
                ArtifactRole::Index,
            ));
        }
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: match tool.tool_id.as_str() {
                    "bamtools" => crate::tool_adapters::tools::samtools::validate_args(
                        bam,
                        &flagstat,
                        &report,
                        &effective_params,
                    ),
                    "bedtools" => crate::tool_adapters::tools::samtools::validate_args(
                        bam,
                        &flagstat,
                        &report,
                        &effective_params,
                    ),
                    _ => crate::tool_adapters::tools::samtools::validate_args(
                        bam,
                        &flagstat,
                        &report,
                        &effective_params,
                    ),
                },
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
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(&effective_params)
                    .expect("BAM stage effective params must serialize"),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["validation_report", "flagstat", "stage_metrics"],
        )
    }
}

pub mod align {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::{AlignEffectiveParams, ReadGroupSpec};
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Align.as_str();
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
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Align,
            out_dir,
        );
        let _assets = crate::tool_adapters::stages_support::ensure_reference_assets(
            reference,
            params.build_indices,
            true,
        )?;
        let read_group = if params.read_group.id.is_empty() {
            ReadGroupSpec::with_defaults(sample_id)
        } else {
            params.read_group.clone()
        };
        let mut effective = params.clone();
        effective.read_group = read_group;

        let command = match tool.tool_id.as_str() {
            "bwa" => {
                crate::tool_adapters::tools::bwa::align_args(reference, r1, r2, out_dir, &effective)
            }
            "bowtie2" => crate::tool_adapters::tools::bowtie2::align_args(
                reference, r1, r2, out_dir, &effective,
            ),
            other => {
                return Err(anyhow::anyhow!("unsupported align tool: {other}"));
            }
        };

        let inputs = {
            let mut items = vec![
                bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("fastq_r1"),
                    r1.to_path_buf(),
                    ArtifactRole::Reads,
                ),
                bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("reference"),
                    reference.to_path_buf(),
                    ArtifactRole::Index,
                ),
            ];
            if let Some(r2) = r2 {
                items.push(bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("fastq_r2"),
                    r2.to_path_buf(),
                    ArtifactRole::Reads,
                ));
            }
            items
        };

        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
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
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(&effective)
                    .expect("BAM stage effective params must serialize"),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
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

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::QcPreEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::QcPre.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
    ) -> anyhow::Result<StagePlanV1> {
        let effective_params = QcPreEffectiveParams { regions: None };
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::QcPre,
            out_dir,
        );
        let flagstat = out_dir.join("flagstat.txt");
        let idxstats = out_dir.join("idxstats.txt");
        let stats = out_dir.join("samtools_stats.txt");
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: crate::tool_adapters::tools::samtools::qc_pre_args(
                    bam,
                    &flagstat,
                    &idxstats,
                    &stats,
                    &effective_params,
                ),
            },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("bam"),
                    bam.to_path_buf(),
                    ArtifactRole::Bam,
                )],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "regions": effective_params.regions,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(&effective_params)
                    .expect("BAM stage effective params must serialize"),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["flagstat", "idxstats", "stats", "stage_metrics"],
        )
    }
}

pub mod filter {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::FilterEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Filter.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &FilterEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Filter,
            out_dir,
        );
        let out_bam = out_dir.join("filtered.bam");
        let flagstat_before = out_dir.join("flagstat.before.txt");
        let flagstat_after = out_dir.join("flagstat.after.txt");
        let idxstats_before = out_dir.join("idxstats.before.txt");
        let idxstats_after = out_dir.join("idxstats.after.txt");
        let summary = out_dir.join("filter.summary.json");
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: match tool.tool_id.as_str() {
                    "bamtools" => crate::tool_adapters::tools::bamtools::filter_args_with_audit(
                        bam,
                        params,
                        &out_bam,
                        &flagstat_before,
                        &flagstat_after,
                        &idxstats_before,
                        &idxstats_after,
                        &summary,
                    ),
                    "bedtools" => crate::tool_adapters::tools::bedtools::filter_args_with_audit(
                        bam,
                        params,
                        &out_bam,
                        &flagstat_before,
                        &flagstat_after,
                        &idxstats_before,
                        &idxstats_after,
                        &summary,
                    ),
                    _ => crate::tool_adapters::tools::samtools::filter_args_with_audit(
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
            },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("bam"),
                    bam.to_path_buf(),
                    ArtifactRole::Bam,
                )],
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
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).expect("BAM stage effective params must serialize"),
            )?,
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
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
