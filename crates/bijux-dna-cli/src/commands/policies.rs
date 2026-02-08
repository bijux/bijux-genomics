/// # Errors
/// Returns an error if the workspace audit fails.
pub fn workspace_audit(out_dir: &Path) -> Result<()> {
    let edges = bijux_dna_api::v1::api::workspace_edges().context("load workspace edges")?;

    let allowed = parse_boundary_contract()?;
    let mut violations = Vec::new();
    for (from, to) in &edges {
        let Some(allowed_deps) = allowed.get(from) else {
            continue;
        };
        if !allowed_deps.contains(to) {
            violations.push(format!("{from} -> {to}"));
        }
    }
    println!("allowed edges (workspace): {}", edges.len());
    for (from, to) in &edges {
        println!("  {from} -> {to}");
    }
    if violations.is_empty() {
        println!("violations: none");
    } else {
        println!("violations: {}", violations.len());
        for violation in &violations {
            println!("  {violation}");
        }
    }

    let mut dot = String::from("digraph workspace {\n");
    for (from, to) in edges {
        let _ = std::fmt::Write::write_fmt(
            &mut dot,
            format_args!("  \"{from}\" -> \"{to}\";\n"),
        );
    }
    dot.push_str("}\n");
    let dot_path = bijux_dna_api::v1::api::write_workspace_audit(out_dir, &dot)?;
    println!("graph: {}", dot_path.display());
    if !violations.is_empty() {
        return Err(anyhow!("workspace boundary violations detected"));
    }
    Ok(())
}

fn parse_boundary_contract() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let root = std::env::current_dir().context("resolve workspace root")?;
    let path = root
        .join("docs")
        .join("10-architecture")
        .join("BOUNDARY_MAP.md");
    let content = std::fs::read_to_string(&path).context("read boundaries.md")?;
    let mut lines: Vec<&str> = Vec::new();
    let mut in_block = false;
    for line in content.lines() {
        if line.trim() == "```boundaries" {
            in_block = true;
            continue;
        }
        if in_block && line.trim() == "```" {
            break;
        }
        if in_block {
            lines.push(line);
        }
    }
    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((from, rest)) = line.split_once(':') else {
            continue;
        };
        let from = from.trim().to_string();
        let mut deps = BTreeSet::new();
        for dep in rest.split_whitespace() {
            deps.insert(dep.trim().to_string());
        }
        edges.insert(from, deps);
    }
    Ok(edges)
}
