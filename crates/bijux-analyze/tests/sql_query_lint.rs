use std::fs;
use std::path::Path;

fn extract_sql_literals(contents: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let bytes = contents.as_bytes();
    let mut idx = 0;
    while let Some(start) = contents[idx..].find("\"SELECT") {
        let start = idx + start;
        let mut i = start + 1;
        let mut escaped = false;
        while i < bytes.len() {
            let ch = bytes[i];
            if ch == b'\"' && !escaped {
                let literal = &contents[start + 1..i];
                literals.push(literal.to_string());
                idx = i + 1;
                break;
            }
            escaped = ch == b'\\' && !escaped;
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
    }
    literals
}

fn is_deterministic_select(query: &str) -> bool {
    let upper = query.to_uppercase();
    if !upper.contains("SELECT") || !upper.contains("FROM") {
        return true;
    }
    if upper.contains("ORDER BY") {
        return true;
    }
    let aggregate = [
        "COUNT(", "DISTINCT", "MAX(", "MIN(", "SUM(", "AVG(", "GROUP BY", "EXISTS",
    ];
    aggregate.iter().any(|needle| upper.contains(needle))
}

#[test]
fn sql_selects_are_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_path = manifest_dir.join("src").join("lib.rs");
    let contents = fs::read_to_string(&src_path)?;
    let queries = extract_sql_literals(&contents);
    let mut offenders = Vec::new();
    for query in queries {
        if !is_deterministic_select(&query) {
            offenders.push(query.replace('\\', ""));
        }
    }
    assert!(
        offenders.is_empty(),
        "non-deterministic SELECT queries detected (missing ORDER BY): {offenders:?}"
    );
    Ok(())
}
