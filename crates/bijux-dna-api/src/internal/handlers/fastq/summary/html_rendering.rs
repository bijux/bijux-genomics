fn hash_optional(path: &Path) -> Option<String> {
    bijux_dna_infra::hash_file_sha256(path)
        .ok()
        .map(|v| format!("sha256:{v}"))
}

fn render_run_summary_html(summary: &serde_json::Value) -> String {
    let pretty = serde_json::to_string_pretty(summary).unwrap_or_else(|_| "{}".to_string());
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>bijux run summary</title>
  <style>
    body {{
      font-family: system-ui, -apple-system, sans-serif;
      margin: 2rem;
      line-height: 1.4;
      background: #f7f7f9;
      color: #111;
    }}
    pre {{
      padding: 1rem;
      background: #fff;
      border-radius: 8px;
      overflow: auto;
      box-shadow: 0 1px 4px rgba(0,0,0,0.08);
    }}
  </style>
</head>
<body>
  <h1>Run summary</h1>
  <pre>{pretty}</pre>
</body>
</html>
"#
    )
}
