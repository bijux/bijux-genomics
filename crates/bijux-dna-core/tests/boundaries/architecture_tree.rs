use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn core_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries([
            "BOUNDARY.md",
            "Cargo.toml",
            "PUBLIC_API.md",
            "README.md",
            "docs/",
            "src/",
            "tests/",
        ]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "contract/",
            "foundation/",
            "id_catalog/",
            "ids/",
            "lib.rs",
            "metrics/",
            "prelude/",
            "public_api/",
        ]),
        "src tree must match the documented core layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract/execution")),
        entries([
            "OWNER.toml",
            "contract.rs",
            "graph.rs",
            "io.rs",
            "manifest.rs",
            "mod.rs",
            "policy.rs",
            "record.rs",
        ]),
        "execution contracts must stay partitioned by execution concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract/run")),
        entries([
            "OWNER.toml",
            "domain.rs",
            "index.rs",
            "metadata.rs",
            "mod.rs",
            "provenance.rs",
            "spec.rs",
        ]),
        "run contracts must stay partitioned by run concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/id_catalog")),
        entries(["OWNER.toml", "mod.rs", "pipeline/", "stage/", "tool/"]),
        "identifier catalog must stay partitioned by catalog concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/ids")),
        entries([
            "OWNER.toml",
            "domain_model.rs",
            "mod.rs",
            "parsing/",
            "typed/"
        ]),
        "ids tree must keep typed ids, parsing, and semantic models separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/prelude")),
        entries([
            "OWNER.toml",
            "catalog_surface.rs",
            "contract_surface.rs",
            "foundation_surface.rs",
            "identity_surface.rs",
            "metric_surface.rs",
            "mod.rs",
        ]),
        "prelude tree must stay grouped by source area"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["OWNER.toml", "mod.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/id_catalog/pipeline")),
        entries([
            "OWNER.toml",
            "bam.rs",
            "fastq.rs",
            "fastq_to_bam.rs",
            "mod.rs",
            "vcf.rs"
        ]),
        "pipeline catalog must stay partitioned by graph concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/id_catalog/stage")),
        entries([
            "OWNER.toml",
            "bam.rs",
            "core.rs",
            "fastq.rs",
            "mod.rs",
            "prefixes.rs",
            "report.rs",
            "vcf.rs",
        ]),
        "stage catalog must stay partitioned by domain and shared concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/id_catalog/tool")),
        entries([
            "OWNER.toml",
            "bam.rs",
            "fastq.rs",
            "mod.rs",
            "shared.rs",
            "vcf.rs"
        ]),
        "tool catalog must stay partitioned by workflow concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/ids/parsing")),
        entries([
            "OWNER.toml",
            "mod.rs",
            "pipeline.rs",
            "stage.rs",
            "symbolic.rs",
            "tool.rs"
        ]),
        "parsing tree must stay partitioned by identifier family"
    );

    assert_eq!(
        dir_entries(&root.join("src/ids/typed")),
        entries([
            "OWNER.toml",
            "artifact.rs",
            "mod.rs",
            "pipeline.rs",
            "run.rs",
            "stage.rs",
            "tool.rs",
        ]),
        "typed id tree must stay partitioned by identifier family"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "README.md",
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "fixtures/",
            "guardrails.rs",
            "schemas/",
            "schemas.rs",
            "semantics/",
            "semantics.rs",
            "snapshots/",
        ]),
        "test tree must stay organized by enduring intent"
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
