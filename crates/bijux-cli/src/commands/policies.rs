use cargo_metadata::MetadataCommand;

use bijux_infra::{ensure_dir, write_string};

/// # Errors
/// Returns an error if cargo metadata cannot be parsed or the DOT file cannot be written.
pub fn workspace_audit(out_dir: &Path) -> Result<()> {
    ensure_dir(out_dir).context("create audit output dir")?;
    let metadata = MetadataCommand::default()
        .exec()
        .context("run cargo metadata")?;
    let workspace_members: HashSet<String> = metadata
        .workspace_members
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    let mut id_to_name = HashMap::new();
    for pkg in &metadata.packages {
        id_to_name.insert(pkg.id.to_string(), pkg.name.clone());
    }
    let mut edges = BTreeSet::new();
    if let Some(resolve) = metadata.resolve.as_ref() {
        for node in &resolve.nodes {
            let id = node.id.to_string();
            if !workspace_members.contains(&id) {
                continue;
            }
            for dep in &node.deps {
                let dep_id = dep.pkg.to_string();
                if !workspace_members.contains(&dep_id) {
                    continue;
                }
                let from = id_to_name.get(&id).cloned().unwrap_or(id.clone());
                let to = id_to_name
                    .get(&dep_id)
                    .cloned()
                    .unwrap_or_else(|| dep_id.clone());
                edges.insert((from, to));
            }
        }
    }

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

    let dot_path = out_dir.join("graph.dot");
    let mut dot = String::from("digraph workspace {\n");
    for (from, to) in edges {
        let _ = std::fmt::Write::write_fmt(
            &mut dot,
            format_args!("  \"{from}\" -> \"{to}\";\n"),
        );
    }
    dot.push_str("}\n");
    write_string(&dot_path, &dot).context("write graph.dot")?;
    println!("graph: {}", dot_path.display());
    if !violations.is_empty() {
        return Err(anyhow!("workspace boundary violations detected"));
    }
    Ok(())
}

fn parse_boundary_contract() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let root = std::env::current_dir().context("resolve workspace root")?;
    let path = root
        .join("crates")
        .join("bijux-core")
        .join("src")
        .join("boundaries.md");
    let content = std::fs::read_to_string(&path).context("read boundaries.md")?;
    let mut lines = Vec::new();
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
            lines.push(line.trim().to_string());
        }
    }
    if !in_block || lines.is_empty() {
        return Err(anyhow!(
            "missing executable boundaries block in {}",
            path.display()
        ));
    }
    let mut map = BTreeMap::new();
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, deps) = line
            .split_once(':')
            .ok_or_else(|| anyhow!("invalid boundaries line: {line}"))?;
        let deps = deps
            .split_whitespace()
            .filter(|dep| !dep.is_empty())
            .map(std::string::ToString::to_string)
            .collect::<BTreeSet<_>>();
        map.insert(name.trim().to_string(), deps);
    }
    Ok(map)
}
