//! Stage adapters for core BAM processing and QC stages.

pub mod markdup {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::MarkDupEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Markdup.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &MarkDupEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Markdup,
            out_dir,
        );
        let out_bam = out_dir.join("markdup.bam");
        let flagstat_before = out_dir.join("flagstat.before.txt");
        let flagstat_after = out_dir.join("flagstat.after.txt");
        let idxstats_before = out_dir.join("idxstats.before.txt");
        let idxstats_after = out_dir.join("idxstats.after.txt");
        let summary = out_dir.join("markdup.summary.json");
        let command = match tool.tool_id.as_str() {
            "samtools" => crate::tool_adapters::tools::samtools::markdup_args_with_audit(
                bam,
                &out_bam,
                &flagstat_before,
                &flagstat_after,
                &idxstats_before,
                &idxstats_after,
                &summary,
                params,
            ),
            _ => crate::tool_adapters::tools::gatk::markdup_args_with_audit(
                bam,
                &out_bam,
                &flagstat_before,
                &flagstat_after,
                &idxstats_before,
                &idxstats_after,
                &summary,
                params,
            ),
        };
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
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
                "optical_duplicates": params.optical_duplicates,
                "umi_policy": params.umi_policy,
                "duplicate_action": params.duplicate_action,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &[
                "markdup_bam",
                "markdup_bai",
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

pub mod complexity {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::ComplexityEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Complexity.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &ComplexityEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Complexity,
            out_dir,
        );
        let preseq_txt = out_dir.join("preseq.txt");
        let complexity_json = out_dir.join("complexity.json");
        let summary_json = out_dir.join("complexity.summary.json");
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: match tool.tool_id.as_str() {
                    "preseq" => crate::tool_adapters::tools::preseq::args_with_outputs(
                        bam,
                        &preseq_txt,
                        &complexity_json,
                        &summary_json,
                        params,
                    ),
                    _ => crate::tool_adapters::tools::preseq::args_with_outputs(
                        bam,
                        &preseq_txt,
                        &complexity_json,
                        &summary_json,
                        params,
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
                "min_reads": params.min_reads,
                "projection_points": params.projection_points,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["complexity_report", "preseq", "summary"],
        )
    }
}

pub mod duplication_metrics {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::MarkDupEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::DuplicationMetrics.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &MarkDupEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::DuplicationMetrics,
            out_dir,
        );
        let report = out_dir.join("duplication.metrics.json");
        let histogram = out_dir.join("duplication.histogram.txt");
        let summary = out_dir.join("duplication.summary.json");
        let command = match tool.tool_id.as_str() {
            "picard" => vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                format!(
                    "picard MarkDuplicates I={bam} O={tmp_bam} M={histogram} VALIDATION_STRINGENCY=SILENT ASSUME_SORTED=true && \
python - <<'PY' {histogram} > {report}\nimport json,sys\npath=sys.argv[1]\nmetrics={{\"method\":\"picard\",\"source\":path}}\nfor line in open(path):\n    if line.startswith(\"LIBRARY\"):\n        values=next(open(path))\n        cols=line.rstrip().split('\\t')\n        vals=values.rstrip().split('\\t')\n        if len(cols)==len(vals):\n            row=dict(zip(cols,vals))\n            metrics[\"pct_duplication\"]=float(row.get(\"PERCENT_DUPLICATION\",0.0) or 0.0)\n            metrics[\"read_pair_duplicates\"]=int(float(row.get(\"READ_PAIR_DUPLICATES\",0) or 0))\n        break\nprint(json.dumps(metrics, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"stage\": \"bam.duplication_metrics\", \"method\": \"picard\", \"optical_duplicates\": \"{optical}\", \"duplicate_action\": \"{action}\"}}, indent=2))\nPY",
                    bam = bam.display(),
                    tmp_bam = out_dir.join("duplication.tmp.bam").display(),
                    histogram = histogram.display(),
                    report = report.display(),
                    summary = summary.display(),
                    optical = format!("{:?}", params.optical_duplicates),
                    action = format!("{:?}", params.duplicate_action),
                ),
            ],
            _ => vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                format!(
                    "samtools markdup -s {bam} {tmp_bam} 2> {histogram} && \
python - <<'PY' {histogram} > {report}\nimport json,re,sys\ntext=open(sys.argv[1]).read()\npairs=re.findall(r'EXAMINED:\\s*(\\d+)', text)\ndups=re.findall(r'DUPLICATE PAIR:\\s*(\\d+)', text)\nout={{\"method\":\"samtools\",\"source\":sys.argv[1],\"examined_pairs\":int(pairs[0]) if pairs else 0,\"duplicate_pairs\":int(dups[0]) if dups else 0}}\nprint(json.dumps(out, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"stage\": \"bam.duplication_metrics\", \"method\": \"samtools\", \"optical_duplicates\": \"{optical}\", \"duplicate_action\": \"{action}\"}}, indent=2))\nPY",
                    bam = bam.display(),
                    tmp_bam = out_dir.join("duplication.tmp.bam").display(),
                    histogram = histogram.display(),
                    report = report.display(),
                    summary = summary.display(),
                    optical = format!("{:?}", params.optical_duplicates),
                    action = format!("{:?}", params.duplicate_action),
                ),
            ],
        };
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
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
                "optical_duplicates": params.optical_duplicates,
                "umi_policy": params.umi_policy,
                "duplicate_action": params.duplicate_action,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["duplication_report", "duplication_histogram", "summary", "stage_metrics"],
        )
    }
}

pub mod coverage {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::CoverageEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Coverage.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &CoverageEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let mut outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Coverage,
            out_dir,
        );
        let prefix = out_dir.join("coverage");
        let depth_path = out_dir.join("coverage.depth.txt");
        let summary_path = out_dir.join("coverage.mosdepth.summary.txt");
        let command = match tool.tool_id.as_str() {
            "samtools" => {
                outputs.push(bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("coverage_depth"),
                    depth_path.clone(),
                    ArtifactRole::ReportJson,
                ));
                crate::tool_adapters::tools::samtools::depth_args(bam, &depth_path, &summary_path)
            }
            _ => crate::tool_adapters::tools::mosdepth::args(bam, &prefix, params),
        };
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
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
                "regions": params.regions,
                "depth_thresholds": params.depth_thresholds,
                "regime_mode": params.regime_mode,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["coverage_summary", "stage_metrics"],
        )
    }
}

pub mod endogenous_content {
    use std::path::Path;

    use bijux_dna_core::prelude::{CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1};
    use bijux_dna_domain_bam::params::{CoverageEffectiveParams, EndogenousContentEffectiveParams};
    use bijux_dna_stage_contract::StagePlanV1;

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::EndogenousContent.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &EndogenousContentEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let coverage_params = CoverageEffectiveParams {
            regions: params.regions.clone(),
            depth_thresholds: params.depth_thresholds.clone(),
            regime_mode: "advisory_and_enforced".to_string(),
        };
        let mut plan = super::coverage::plan(tool, bam, out_dir, &coverage_params)?;
        plan.stage_id = StageId::from_static(STAGE_ID);
        plan.stage_version = STAGE_VERSION;
        plan.command = CommandSpecV1 { template: plan.command.template.clone() };
        plan.params = serde_json::json!({
            "bam": bam,
            "regions": params.regions,
            "depth_thresholds": params.depth_thresholds,
            "host_reference_scope": params.host_reference_scope,
            "host_reference_digest": params.host_reference_digest,
            "refuse_without_host_reference": params.refuse_without_host_reference,
        });
        plan.effective_params = crate::tool_adapters::stages_support::ensure_effective_params(
            serde_json::to_value(params).map_err(|error| {
                anyhow::anyhow!("BAM stage effective params must serialize: {error}")
            })?,
        )?;
        Ok(plan)
    }
}

pub mod insert_size {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::CoverageEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::InsertSize.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &CoverageEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::InsertSize,
            out_dir,
        );
        let report = out_dir.join("insert_size.metrics.txt");
        let histogram = out_dir.join("insert_size.histogram.pdf");
        let command = match tool.tool_id.as_str() {
            "picard" => {
                crate::tool_adapters::tools::core::picard::collect_insert_size_metrics_args(
                    bam, &report, &histogram,
                )
            }
            _ => crate::tool_adapters::tools::core::picard::collect_insert_size_metrics_args(
                bam, &report, &histogram,
            ),
        };
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
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
                "regions": params.regions,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["insert_size_report", "insert_size_histogram", "summary", "stage_metrics"],
        )
    }
}

pub mod gc_bias {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::CoverageEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::GcBias.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        reference: &Path,
        out_dir: &Path,
        params: &CoverageEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::GcBias,
            out_dir,
        );
        let report = out_dir.join("gc_bias.metrics.txt");
        let summary = out_dir.join("gc_bias.summary.json");
        let chart = out_dir.join("gc_bias.plot.pdf");
        let command = match tool.tool_id.as_str() {
            "picard" => crate::tool_adapters::tools::core::picard::collect_gc_bias_metrics_args(
                bam, reference, &report, &summary, &chart,
            ),
            _ => crate::tool_adapters::tools::core::picard::collect_gc_bias_metrics_args(
                bam, reference, &report, &summary, &chart,
            ),
        };
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 { template: command },
            resources: tool.resources.clone(),
            io: StageIO {
                inputs: vec![
                    bijux_dna_stage_contract::ArtifactRef::required(
                        ArtifactId::from_static("bam"),
                        bam.to_path_buf(),
                        ArtifactRole::Bam,
                    ),
                    bijux_dna_stage_contract::ArtifactRef::required(
                        ArtifactId::from_static("reference"),
                        reference.to_path_buf(),
                        ArtifactRole::Reference,
                    ),
                ],
                outputs,
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({
                "bam": bam,
                "reference": reference,
                "regions": params.regions,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["gc_bias_report", "gc_bias_plot", "summary", "stage_metrics"],
        )
    }
}

pub mod recalibration {
    use std::path::Path;

    use bijux_dna_core::prelude::{
        ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
    };
    use bijux_dna_domain_bam::params::BqsrEffectiveParams;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};

    pub const STAGE_ID: &str = bijux_dna_domain_bam::BamStage::Recalibration.as_str();
    pub const STAGE_VERSION: StageVersion = StageVersion(1);

    /// # Errors
    /// Returns an error if required outputs are missing from the plan.
    pub fn plan(
        tool: &ToolExecutionSpecV1,
        bam: &Path,
        out_dir: &Path,
        params: &BqsrEffectiveParams,
    ) -> anyhow::Result<StagePlanV1> {
        let outputs = crate::tool_adapters::stages_support::audit_outputs(
            bijux_dna_domain_bam::BamStage::Recalibration,
            out_dir,
        );
        let out_bam = out_dir.join("recalibrated.bam");
        let recal_report = out_dir.join("recalibration.table");
        let summary = out_dir.join("recalibration.summary.json");
        let plan = StagePlanV1 {
            stage_id: StageId::from_static(STAGE_ID),
            stage_instance_id: None,
            stage_version: STAGE_VERSION,
            tool_id: tool.tool_id.clone(),
            tool_version: tool.tool_version.clone(),
            image: tool.image.clone(),
            command: CommandSpecV1 {
                template: match tool.tool_id.as_str() {
                    "gatk" => crate::tool_adapters::tools::gatk::recalibration_args_with_outputs(
                        bam,
                        &out_bam,
                        &recal_report,
                        &summary,
                        params,
                    ),
                    _ => crate::tool_adapters::tools::gatk::recalibration_args_with_outputs(
                        bam,
                        &out_bam,
                        &recal_report,
                        &summary,
                        params,
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
                "known_sites": params.known_sites,
                "mode": params.mode,
                "skip_criteria": params.skip_criteria,
            }),
            effective_params: crate::tool_adapters::stages_support::ensure_effective_params(
                serde_json::to_value(params).map_err(|error| {
                    anyhow::anyhow!("BAM stage effective params must serialize: {error}")
                })?,
            )?,
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        crate::tool_adapters::stages_support::ensure_required_outputs(
            plan,
            &["recal_bam", "recal_bai", "recal_report", "summary"],
        )
    }
}
