//! Owner: bijux-analyze
//! Canonical analyze pipeline entrypoint.
//! Owns orchestration and cross-layer wiring only.
//! Must not be called from other layers; only pipeline crosses load/aggregate/decision/report/failure.
//! Invariants: pipeline steps are ordered and typed (load → validate → compute → report → render).

use std::path::Path;

use anyhow::{Context, Result};

use bijux_core::selection::{objective_spec, Objective};

use crate::decision::compare::compare_runs;
use crate::{AnalyzeInput, AnalyzeMode, AnalyzeOutput};

pub(crate) mod compute_step;
pub(crate) mod load_step;
pub(crate) mod render_step;
pub(crate) mod validate_step;

pub fn analyze_run_pipeline(input: &AnalyzeInput) -> Result<AnalyzeOutput> {
    let mut output = AnalyzeOutput {
        run_id: input.run_id.clone(),
        report_json: None,
        report_html: None,
        summary_json: None,
        compare_json: None,
        ranking_json: None,
        decision_trace_json: None,
    };

    if let AnalyzeMode::Compare {
        ref run_a,
        ref run_b,
    } = input.options.mode
    {
        let objective = objective_spec(Objective::Balanced);
        let comparison = compare_runs(Path::new(run_a), Path::new(run_b), &objective)?;
        let output_dir = input
            .options
            .render
            .output_dir
            .clone()
            .context("compare output_dir must be set in RenderOptions")?;
        let path = output_dir.join("compare.json");
        std::fs::write(&path, serde_json::to_vec_pretty(&comparison)?)
            .context("write compare.json")?;
        output.compare_json = Some(path);
        return Ok(output);
    }

    let loaded = load_step::load_inputs(&input.sources)?;
    let validated = validate_step::validate_inputs(loaded)?;
    let core = compute_step::compute_core(validated, &input.options)?;
    let rendered = render_step::render_outputs(&core, None, &input.options)?;
    render_step::merge_output(&mut output, rendered);

    Ok(output)
}
