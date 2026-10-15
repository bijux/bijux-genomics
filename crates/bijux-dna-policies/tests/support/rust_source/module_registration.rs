#![cfg(test)]

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Context as _, Result};

use crate::cargo_metadata_authority::CargoPackageSnapshot;
use crate::rust_source_tokens::{source_tokens, SourceToken};

#[path = "../rust_source_intake.rs"]
mod source_intake;

pub(crate) fn registered_private_helper_bucket(
    owner: &Path,
    bucket: &str,
    authority: &CargoPackageSnapshot,
) -> Result<bool> {
    let owner = owner.canonicalize().context("resolve test helper owner")?;
    let tests = owner.join("tests");
    let directory = tests.join(bucket);
    let mut helpers = BTreeSet::new();
    for path in source_intake::collect_rs_files(&directory)? {
        let path = owned_regular_file(&path, &tests)?;
        let source = std::fs::read_to_string(&path)
            .with_context(|| format!("read private test helper {}", path.display()))?;
        if source.lines().find(|line| !line.trim().is_empty()).map(str::trim)
            != Some("#![cfg(test)]")
        {
            return Ok(false);
        }
        helpers.insert(path);
    }
    let mut registered = BTreeSet::new();
    for target in authority
        .package(&owner.join("Cargo.toml"))
        .targets
        .iter()
        .filter(|target| target.kind.iter().any(|kind| kind == "test"))
    {
        let target = owned_regular_file(target.src_path.as_std_path(), &tests)?;
        let source = std::fs::read_to_string(&target)
            .with_context(|| format!("read declared test target {}", target.display()))?;
        let parent = target.parent().context("declared test target has no parent")?;
        for relative in direct_private_modules(&source)? {
            let relative = Path::new(&relative);
            ensure!(
                relative.components().all(|component| matches!(component, Component::Normal(_))),
                "direct test helper path must stay inside its declared owner"
            );
            registered.insert(owned_regular_file(&parent.join(relative), &tests)?);
        }
    }
    Ok(helpers.is_subset(&registered))
}

fn owned_regular_file(path: &Path, tests: &Path) -> Result<PathBuf> {
    let relative = path.strip_prefix(tests).with_context(|| {
        format!("declared test source escapes {}: {}", tests.display(), path.display())
    })?;
    let mut current = tests.to_path_buf();
    ensure!(
        !std::fs::symlink_metadata(&current)?.file_type().is_symlink(),
        "test source root must not redirect: {}",
        current.display()
    );
    for component in relative.components() {
        ensure!(matches!(component, Component::Normal(_)), "test source path must be direct");
        current.push(component);
        ensure!(
            !std::fs::symlink_metadata(&current)?.file_type().is_symlink(),
            "test source must not redirect: {}",
            current.display()
        );
    }
    ensure!(current.is_file(), "test source must be a regular file: {}", current.display());
    current.canonicalize().with_context(|| format!("resolve test source {}", current.display()))
}

fn token_is(tokens: &[SourceToken<'_>], index: usize, expected: &str) -> bool {
    tokens.get(index).is_some_and(|token| !token.literal && token.text == expected)
}

fn closing(opening: &str) -> Option<&'static str> {
    match opening {
        "(" => Some(")"),
        "[" => Some("]"),
        "{" => Some("}"),
        _ => None,
    }
}

fn balanced_end(tokens: &[SourceToken<'_>], start: usize) -> Result<usize> {
    let mut expected = vec![closing(tokens[start].text).context("expected source delimiter")?];
    for (index, token) in tokens.iter().enumerate().skip(start + 1) {
        if token.literal {
            continue;
        }
        if let Some(end) = closing(token.text) {
            expected.push(end);
        } else if matches!(token.text, ")" | "]" | "}") {
            ensure!(expected.pop() == Some(token.text), "mismatched test source delimiter");
            if expected.is_empty() {
                return Ok(index);
            }
        }
    }
    anyhow::bail!("unterminated test source delimiter")
}

// Only unconditional private external modules establish this fallback's source authority.
// Cargo owns target discovery; this bounded reader does not evaluate Rust cfg expressions.
fn direct_private_modules(source: &str) -> Result<Vec<String>> {
    let tokens = source_tokens(source).context("malformed declared test target source")?;
    let mut modules = Vec::new();
    let mut path = None;
    let mut blocked = false;
    let mut index = 0;
    while index < tokens.len() {
        if token_is(&tokens, index, "#") {
            let inner = token_is(&tokens, index + 1, "!");
            let opening = index + if inner { 2 } else { 1 };
            ensure!(token_is(&tokens, opening, "["), "malformed test module attribute");
            let end = balanced_end(&tokens, opening)?;
            let attribute = &tokens[opening + 1..end];
            if inner {
                if !attribute.first().is_some_and(|token| {
                    !token.literal && matches!(token.text, "allow" | "warn" | "deny" | "forbid")
                }) {
                    return Ok(Vec::new());
                }
            } else if attribute.len() == 3
                && token_is(attribute, 0, "path")
                && token_is(attribute, 1, "=")
                && attribute[2].literal
            {
                let literal =
                    attribute[2].text.strip_prefix('"').and_then(|text| text.strip_suffix('"'));
                if let Some(literal) =
                    literal.filter(|text| !text.is_empty() && !text.contains('\\'))
                {
                    blocked |= path.is_some();
                    path = Some(literal.to_string());
                } else {
                    blocked = true;
                }
            } else {
                blocked = true;
            }
            index = end + 1;
            continue;
        }
        if token_is(&tokens, index, "mod")
            && tokens.get(index + 1).is_some_and(|token| !token.literal)
            && token_is(&tokens, index + 2, ";")
            && !blocked
        {
            if let Some(relative) = path.take() {
                modules.push(relative);
            }
        }
        path = None;
        blocked = false;
        if !tokens[index].literal {
            if closing(tokens[index].text).is_some() {
                index = balanced_end(&tokens, index)? + 1;
                continue;
            }
            ensure!(
                !matches!(tokens[index].text, ")" | "]" | "}"),
                "unexpected closing test source delimiter"
            );
        }
        index += 1;
    }
    Ok(modules)
}
