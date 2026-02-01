//! Owner: bijux-analyze
//! Minimal HTML renderer for report models.

use anyhow::Result;
use std::fmt::Write;

use bijux_core::ReportStageSummaryV1;

use crate::report::model::ReportModel;

#[allow(dead_code)]
pub fn render_report_html(model: &ReportModel) -> Result<String> {
    let report = &model.report;
    let report_json = serde_json::to_string_pretty(report)?;
    let sections = report.sections.as_object().cloned().unwrap_or_default();
    let mut section_keys: Vec<String> = sections.keys().cloned().collect();
    section_keys.sort();

    let nav_items = build_nav_items(&section_keys);
    let section_blocks = build_section_blocks(&sections, &section_keys)?;
    let stage_tabs = build_stage_tabs(&report.stages);
    let stage_panels = build_stage_panels(&report.stages);

    let stage_plots = sections
        .get("stage_plots")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let stage_plots_json = serde_json::to_string(&stage_plots)?;
    let reproducibility = sections
        .get("reproducibility")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let repro_json = serde_json::to_string_pretty(&reproducibility)?;
    let command = reproducibility
        .get("command")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");

    Ok(build_html_template(
        &report_json,
        &nav_items,
        &section_blocks,
        &stage_tabs,
        &stage_panels,
        &stage_plots_json,
        &repro_json,
        command,
    ))
}

fn build_nav_items(section_keys: &[String]) -> String {
    let mut nav_items = String::new();
    for key in section_keys {
        let _ = write!(nav_items, "<li><a href=\"#section-{key}\">{key}</a></li>");
    }
    nav_items
}

fn build_section_blocks(
    sections: &serde_json::Map<String, serde_json::Value>,
    section_keys: &[String],
) -> Result<String> {
    let mut section_blocks = String::new();
    for key in section_keys {
        if let Some(value) = sections.get(key) {
            let json = serde_json::to_string_pretty(value)?;
            let _ = write!(
                section_blocks,
                r#"<details id="section-{key}" class="section"><summary>{key}</summary><pre>{json}</pre></details>"#
            );
        }
    }
    Ok(section_blocks)
}

fn build_stage_tabs(stages: &[ReportStageSummaryV1]) -> String {
    let mut stage_tabs = String::new();
    for (idx, stage) in stages.iter().enumerate() {
        let active = if idx == 0 { "active" } else { "" };
        let _ = write!(
            stage_tabs,
            r#"<button class="stage-tab {active}" data-stage="{id}">{id}</button>"#,
            id = stage.stage_id
        );
    }
    stage_tabs
}

fn build_stage_panels(stages: &[ReportStageSummaryV1]) -> String {
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
                let _ = write!(artifact_links, r#"<li><a href="{path}">{label}</a></li>"#);
            }
        }
        let _ = write!(
            stage_panels,
            r#"<div class="stage-panel {active}" data-stage="{id}">
  <h3>{id}</h3>
  <div class="stage-meta">
    <span>tool: {tool}</span>
    <span>version: {version}</span>
    <span>params: {params}</span>
  </div>
  <div class="stage-meta">
    <span>exit_code: {exit_code}</span>
    <span>runtime_s: {runtime_s:.2}</span>
    <span>memory_mb: {memory_mb:.1}</span>
  </div>
  <div class="stage-artifacts">
    <h4>Artifacts</h4>
    <ul>{artifact_links}</ul>
  </div>
  <div class="stage-plots" data-stage="{id}"></div>
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

#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments, clippy::uninlined_format_args)]
fn build_html_template(
    report_json: &str,
    nav_items: &str,
    section_blocks: &str,
    stage_tabs: &str,
    stage_panels: &str,
    stage_plots_json: &str,
    repro_json: &str,
    command: &str,
) -> String {
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>bijux analyze report</title>
  <style>
    body {{
      font-family: system-ui, -apple-system, sans-serif;
      margin: 0;
      line-height: 1.4;
      background: #f7f7f9;
      color: #111;
    }}
    .layout {{
      display: grid;
      grid-template-columns: 220px 1fr;
      min-height: 100vh;
    }}
    nav {{
      background: #111827;
      color: #f9fafb;
      padding: 1.5rem 1rem;
    }}
    nav h2 {{
      font-size: 1rem;
      margin: 0 0 1rem;
    }}
    nav ul {{
      list-style: none;
      padding: 0;
      margin: 0;
    }}
    nav li {{
      margin: 0.5rem 0;
    }}
    nav a {{
      color: #e5e7eb;
      text-decoration: none;
      font-size: 0.9rem;
    }}
    main {{
      padding: 2rem;
    }}
    .section {{
      background: #fff;
      border-radius: 10px;
      padding: 1rem;
      margin-bottom: 1rem;
      box-shadow: 0 1px 4px rgba(0,0,0,0.08);
    }}
    .section summary {{
      font-weight: 600;
      cursor: pointer;
      margin-bottom: 0.5rem;
    }}
    pre {{
      white-space: pre-wrap;
    }}
    .stage-tabs {{
      display: flex;
      gap: 0.5rem;
      flex-wrap: wrap;
      margin-bottom: 1rem;
    }}
    .stage-tab {{
      padding: 0.4rem 0.8rem;
      border: 1px solid #d1d5db;
      background: #fff;
      border-radius: 999px;
      cursor: pointer;
    }}
    .stage-tab.active {{
      background: #111827;
      color: #fff;
      border-color: #111827;
    }}
    .stage-panel {{
      display: none;
      background: #fff;
      border-radius: 10px;
      padding: 1rem;
      box-shadow: 0 1px 4px rgba(0,0,0,0.08);
    }}
    .stage-panel.active {{
      display: block;
    }}
    .stage-meta {{
      display: flex;
      gap: 1rem;
      flex-wrap: wrap;
      margin-bottom: 0.5rem;
      color: #4b5563;
      font-size: 0.9rem;
    }}
    .stage-artifacts ul {{
      padding-left: 1.2rem;
    }}
    .plot-bar {{
      height: 8px;
      background: #e5e7eb;
      border-radius: 4px;
      overflow: hidden;
    }}
    .plot-bar span {{
      display: block;
      height: 100%;
      background: #2563eb;
    }}
    .copy-btn {{
      margin-left: 0.5rem;
      padding: 0.2rem 0.6rem;
      border-radius: 6px;
      border: 1px solid #d1d5db;
      background: #fff;
      cursor: pointer;
    }}
  </style>
</head>
<body>
  <script id="report-json" type="application/json">{report_json}</script>
  <div class="layout">
    <nav>
      <h2>bijux report</h2>
      <ul>
        <li><a href="#overview">overview</a></li>
        <li><a href="#stages">stages</a></li>
{nav_items}
      </ul>
    </nav>
    <main>
      <section id="overview" class="section">
        <h2>overview</h2>
        <div class="section">
          <h3>reproducibility</h3>
          <pre>{repro_json}</pre>
          <div>
            <code>{command}</code>
            <button class="copy-btn" data-copy="command">copy command</button>
          </div>
        </div>
      </section>
      <section id="stages" class="section">
        <h2>stages</h2>
        <div class="stage-tabs">
{stage_tabs}
        </div>
{stage_panels}
      </section>
{section_blocks}
    </main>
  </div>
  <script>
    const stagePlots = {stage_plots_json};
    const stagePanels = document.querySelectorAll('.stage-panel');
    const stageTabs = document.querySelectorAll('.stage-tab');
    function showStage(id) {{
      stageTabs.forEach(tab => tab.classList.toggle('active', tab.dataset.stage === id));
      stagePanels.forEach(panel => panel.classList.toggle('active', panel.dataset.stage === id));
    }}
    stageTabs.forEach(tab => tab.addEventListener('click', () => showStage(tab.dataset.stage)));
    if (stageTabs.length > 0) {{
      showStage(stageTabs[0].dataset.stage);
    }}
    stagePanels.forEach(panel => {{
      const stageId = panel.dataset.stage;
      const container = panel.querySelector('.stage-plots');
      const data = stagePlots.entries || [];
      const stageEntry = data.find(entry => entry.stage_id === stageId);
      if (!stageEntry || !container) return;
      const plots = stageEntry.plots || [];
      plots.forEach(plot => {{
        const wrapper = document.createElement('div');
        wrapper.className = 'section';
        wrapper.innerHTML = `<h4>${{plot.title}}</h4><div class="plot-bar"><span style="width:${{plot.value}}%"></span></div><small>${{plot.label}}</small>`;
        container.appendChild(wrapper);
      }});
    }});
    document.querySelectorAll('.copy-btn').forEach(btn => {{
      btn.addEventListener('click', () => {{
        const text = {command:?};
        navigator.clipboard.writeText(text).catch(() => {{}});
      }});
    }});
  </script>
</body>
</html>"##,
        report_json = report_json,
        nav_items = nav_items,
        section_blocks = section_blocks,
        stage_tabs = stage_tabs,
        stage_panels = stage_panels,
        stage_plots_json = stage_plots_json,
        repro_json = repro_json,
        command = command
    )
}
