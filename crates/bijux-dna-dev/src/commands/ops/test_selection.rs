use std::collections::BTreeSet;

use anyhow::Result;
use regex::escape as regex_escape;

use crate::runtime::workspace::Workspace;

const SLOW_TEST_ROSTER_PATH: &str = "configs/rust/nextest-slow-roster.txt";

pub(super) fn governed_fast_expression(workspace: &Workspace) -> Result<String> {
    let slow_expr = governed_slow_expression(workspace)?;
    Ok(if slow_expr == legacy_slow_expression() {
        format!("not {slow_expr}")
    } else {
        format!("not ({slow_expr})")
    })
}

pub(super) fn governed_slow_expression(workspace: &Workspace) -> Result<String> {
    let roster = load_governed_slow_roster(workspace)?;
    Ok(compose_slow_expression(&roster))
}

pub(super) fn load_governed_slow_roster(workspace: &Workspace) -> Result<Vec<String>> {
    let path = workspace.path(SLOW_TEST_ROSTER_PATH);
    let content = std::fs::read_to_string(&path)?;
    Ok(parse_governed_slow_roster(&content))
}

fn parse_governed_slow_roster(content: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            names.push(trimmed.to_string());
        }
    }
    names
}

fn compose_slow_expression(roster: &[String]) -> String {
    match roster_regex_alternation(roster) {
        Some(roster_regex) => {
            format!("{legacy} or test(/^(?:{roster_regex})$/)", legacy = legacy_slow_expression())
        }
        None => legacy_slow_expression().to_string(),
    }
}

fn roster_regex_alternation(roster: &[String]) -> Option<String> {
    let escaped = roster.iter().map(|name| regex_escape(name)).collect::<Vec<_>>();
    (!escaped.is_empty()).then(|| escaped.join("|"))
}

fn legacy_slow_expression() -> &'static str {
    "test(/::slow__/)"
}

#[cfg(test)]
mod tests {
    use super::{compose_slow_expression, governed_fast_expression, parse_governed_slow_roster};
    use crate::runtime::workspace::Workspace;

    #[test]
    fn parse_governed_slow_roster_skips_comments_blanks_and_duplicates() {
        let roster = parse_governed_slow_roster(
            "\n# comment\nslow_a\nslow_b\nslow_a\n   \n# comment 2\nslow_c\n",
        );
        assert_eq!(roster, vec!["slow_a", "slow_b", "slow_c"]);
    }

    #[test]
    fn compose_slow_expression_keeps_named_and_rostered_slow_tests() {
        let roster = vec!["suite::tests::slow_case".to_string(), "bench_slow_case".to_string()];
        assert_eq!(
            compose_slow_expression(&roster),
            "test(/::slow__/) or test(/^(?:suite::tests::slow_case|bench_slow_case)$/)"
        );
    }

    #[test]
    fn governed_fast_expression_negates_both_slow_sources() {
        let workspace = Workspace::resolve().expect("workspace");
        let expression = governed_fast_expression(&workspace).expect("fast expression");
        assert!(expression.starts_with("not (test(/::slow__/)"));
        assert!(expression.contains(" or test(/^(?:"));
    }
}
