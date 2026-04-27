use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn readme_links_governed_science_surfaces_exactly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected = entries([
        "../../science/specs/evidence/README.md",
        "../../science/specs/releases/README.md",
        "../../science/generated/README.md",
        "../../science/generated/current/evidence/README.md",
        "../../science/generated/indexes/README.md",
    ]);
    let documented = markdown_link_targets(root.join("README.md"))
        .into_iter()
        .filter(|target| target.starts_with("../../science/"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "README.md must link the governed authored and generated science surfaces exactly"
    );
}

#[test]
fn boundary_doc_links_owned_science_surfaces_exactly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected = entries([
        "../../../science/specs/evidence/README.md",
        "../../../science/specs/releases/README.md",
        "../../../science/generated/README.md",
        "../../../science/generated/current/README.md",
        "../../../science/generated/current/evidence/README.md",
        "../../../science/generated/indexes/README.md",
    ]);
    let documented = markdown_link_targets(root.join("docs/BOUNDARY.md"))
        .into_iter()
        .filter(|target| target.starts_with("../../../science/"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "docs/BOUNDARY.md must link the owned authored and generated science surfaces exactly"
    );
}

fn markdown_link_targets(path: impl AsRef<Path>) -> BTreeSet<String> {
    let path = path.as_ref();
    let raw =
        fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let mut targets = BTreeSet::new();
    for line in raw.lines() {
        let mut rest = line;
        while let Some((_, suffix)) = rest.split_once("](") {
            if let Some((target, tail)) = suffix.split_once(')') {
                targets.insert(target.to_string());
                rest = tail;
            } else {
                break;
            }
        }
    }
    targets
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
