use bijux_runtime::ReportStageSummaryV1;
use std::fmt::Write;

pub(super) fn build_nav_items(section_keys: &[String]) -> String {
    let mut nav_items = String::new();
    for key in section_keys {
        let _ = write!(nav_items, "<li><a href=\"#section-{key}\">{key}</a></li>");
    }
    nav_items
}

pub(super) fn build_section_blocks(
    sections: &serde_json::Map<String, serde_json::Value>,
    section_keys: &[String],
) -> anyhow::Result<String> {
    let mut section_blocks = String::new();
    for key in section_keys {
        if let Some(value) = sections.get(key) {
            if key == "bam_plots" {
                let plots = render_bam_plots(value);
                let _ = write!(
                    section_blocks,
                    r#"<section id=\"section-{key}\" class=\"section\"><h3>{key}</h3>{plots}</section>"#
                );
            } else {
                let json = serde_json::to_string_pretty(value)?;
                let _ = write!(
                    section_blocks,
                    r#"<details id=\"section-{key}\" class=\"section\"><summary>{key}</summary><pre>{json}</pre></details>"#
                );
            }
        }
    }
    Ok(section_blocks)
}

fn render_bam_plots(value: &serde_json::Value) -> String {
    let mut html = String::new();
    let entries = value
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    for entry in entries {
        let stage_id = entry
            .get("stage_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("bam");
        let damage = entry
            .get("damage")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let frag = entry
            .get("fragment_length")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let coverage = entry
            .get("coverage")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let dup = entry
            .get("dup_vs_complexity")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let c_to_t = damage
            .get("c_to_t_5p")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let g_to_a = damage
            .get("g_to_a_3p")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let mean_len = frag
            .get("mean")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let short_frac = frag
            .get("short_fraction")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let cov_mean = coverage
            .get("mean")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let breadth = coverage
            .get("breadth_1x")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let dup_fraction = dup
            .get("dup_fraction")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let short_pct = short_frac * 100.0;
        let breadth_pct = breadth * 100.0;
        let dup_pct = dup_fraction * 100.0;
        let _ = write!(
            html,
            r#"<div class=\"section\">
<h4>{stage_id}</h4>
<div class=\"plot\">
<div>damage decay (5' C→T vs 3' G→A)</div>
<div class=\"plot-bar\"><span style=\"width:{c_to_t:.0}%\"></span></div>
<div class=\"plot-bar\"><span style=\"width:{g_to_a:.0}%\"></span></div>
</div>
<div class=\"plot\">
<div>fragment length (mean {mean_len:.1}bp, short {short_frac:.2})</div>
<div class=\"plot-bar\"><span style=\"width:{short_pct:.0}%\"></span></div>
</div>
<div class=\"plot\">
<div>coverage (mean {cov_mean:.2}x, breadth@1x {breadth:.2})</div>
<div class=\"plot-bar\"><span style=\"width:{breadth_pct:.0}%\"></span></div>
</div>
<div class=\"plot\">
<div>duplication vs complexity (dup fraction {dup_fraction:.2})</div>
<div class=\"plot-bar\"><span style=\"width:{dup_pct:.0}%\"></span></div>
</div>
</div>"#
        );
    }
    html
}

pub(super) fn build_stage_tabs(stages: &[ReportStageSummaryV1]) -> String {
    let mut stage_tabs = String::new();
    for (idx, stage) in stages.iter().enumerate() {
        let active = if idx == 0 { "active" } else { "" };
        let _ = write!(
            stage_tabs,
            r#"<button class=\"stage-tab {active}\" data-stage=\"{id}\">{id}</button>"#,
            id = stage.stage_id
        );
    }
    stage_tabs
}

pub(super) fn build_stage_panels(stages: &[ReportStageSummaryV1]) -> String {
    let mut stage_panels = String::new();
    for (idx, stage) in stages.iter().enumerate() {
        let active = if idx == 0 { "active" } else { "" };
        let artifacts: [(&str, Option<&String>); 5] = [
            ("stage_report", Some(&stage.stage_report_path)),
            ("metrics", Some(&stage.metrics_path)),
            ("retention_report", stage.retention_report_path.as_ref()),
            ("bank_report", stage.bank_report_path.as_ref()),
            ("tool_invocation", Some(&stage.tool_invocation_path)),
        ];
        let mut artifact_links = String::new();
        for (label, path) in artifacts {
            if let Some(path) = path {
                let _ = write!(artifact_links, r#"<li><a href=\"{path}\">{label}</a></li>"#);
            }
        }
        let _ = write!(
            stage_panels,
            r#"<div class=\"stage-panel {active}\" data-stage=\"{id}\">
  <h3>{id}</h3>
  <div class=\"stage-meta\">
    <span>tool: {tool}</span>
    <span>version: {version}</span>
    <span>params: {params}</span>
  </div>
  <div class=\"stage-meta\">
    <span>exit_code: {exit_code}</span>
    <span>runtime_s: {runtime_s:.2}</span>
    <span>memory_mb: {memory_mb:.1}</span>
  </div>
  <div class=\"stage-artifacts\">
    <h4>Artifacts</h4>
    <ul>{artifact_links}</ul>
  </div>
  <div class=\"stage-plots\" data-stage=\"{id}\"></div>
</div>"#,
            id = stage.stage_id,
            tool = stage.tool_id,
            version = stage.tool_version,
            params = stage.params_hash,
            exit_code = stage.exit_code,
            runtime_s = stage.runtime_s,
            memory_mb = stage.memory_mb,
            artifact_links = artifact_links
        );
    }
    stage_panels
}
