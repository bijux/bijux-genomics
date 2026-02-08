#[allow(clippy::too_many_arguments, clippy::uninlined_format_args)]
#[allow(clippy::too_many_lines)]
pub(super) fn build_html_template(
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
