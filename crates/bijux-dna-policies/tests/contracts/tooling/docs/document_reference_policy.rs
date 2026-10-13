#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

#[test]
fn policy__contracts__document_reference_policy__repository_reference_spellings_preserve_the_same_existing_authority(
) {
    let scratch = bijux_dna_testkit::tempdir_for("documentation-reference-identities");
    let root = scratch.path();
    std::fs::create_dir_all(root.join("docs/guide")).expect("create document context");
    std::fs::create_dir(root.join("configs")).expect("create authority directory");
    std::fs::write(root.join("configs/settings.toml"), "enabled = true\n")
        .expect("write authority");
    let document = "docs/guide/index.md";
    for target in [
        "../../configs/settings.toml",
        "/configs/settings.toml",
        "https://github.com/bijux/bijux-genomics/blob/main/configs/settings.toml",
    ] {
        assert_eq!(
            crate::document_references::repository_target(root, document, target)
                .expect("admit same authority"),
            "configs/settings.toml"
        );
    }
    assert_eq!(
        crate::document_references::repository_target(
            root,
            document,
            "https://github.com/bijux/bijux-genomics/tree/main/configs/",
        )
        .expect("admit directory authority"),
        "configs"
    );
}

#[test]
fn policy__contracts__document_reference_policy__repository_references_reject_foreign_refs_escapes_and_missing_authorities(
) {
    let scratch = bijux_dna_testkit::tempdir_for("documentation-reference-refusals");
    let root = scratch.path();
    std::fs::create_dir(root.join("configs")).expect("create authority directory");
    std::fs::write(root.join("configs/settings.toml"), "enabled = true\n")
        .expect("write authority");
    for target in [
        "https://example.org/bijux/bijux-genomics/blob/main/configs/settings.toml",
        "https://github.com/foreign/bijux-genomics/blob/main/configs/settings.toml",
        "https://github.com/bijux/foreign/blob/main/configs/settings.toml",
        "https://github.com/bijux/bijux-genomics/blob/unrecognized/configs/settings.toml",
        "https://github.com/bijux/bijux-genomics/blob/main/%2e%2e/configs/settings.toml",
        "https://github.com/bijux/bijux-genomics/blob/main/configs/settings.toml?raw=true",
        "https://github.com/bijux/bijux-genomics/tree/main/configs/settings.toml",
        "https://github.com/bijux/bijux-genomics/blob/main/configs",
        "https://github.com/bijux/bijux-genomics/blob/main/configs/missing.toml",
        "//github.com/bijux/bijux-genomics/blob/main/configs/settings.toml",
        "../../../outside.toml",
        "../../%2e%2e/outside.toml",
        "../../configs/missing.toml",
        "../../configs/%GG.toml",
    ] {
        assert!(
            crate::document_references::repository_target(root, "docs/guide/index.md", target)
                .is_err(),
            "unsupported authority was accepted: {target}"
        );
    }
}

#[cfg(unix)]
#[test]
fn policy__contracts__document_reference_policy__repository_reference_containment_rejects_symlinked_external_files(
) {
    let scratch = bijux_dna_testkit::tempdir_for("documentation-reference-symlinks");
    let outside = bijux_dna_testkit::tempdir_for("documentation-external-authority");
    std::fs::write(outside.path().join("authority.toml"), "enabled = true\n")
        .expect("write external authority");
    std::os::unix::fs::symlink(
        outside.path().join("authority.toml"),
        scratch.path().join("authority.toml"),
    )
    .expect("create external symlink");
    assert!(crate::document_references::repository_target(
        scratch.path(),
        "docs/index.md",
        "/authority.toml",
    )
    .is_err());
}

#[test]
fn policy__contracts__document_reference_policy__documented_references_cannot_replace_required_authorities_with_external_citations(
) {
    let scratch = bijux_dna_testkit::tempdir_for("documentation-required-authorities");
    let root = scratch.path();
    std::fs::create_dir(root.join("docs")).expect("create document directory");
    std::fs::create_dir(root.join("configs")).expect("create authority directory");
    std::fs::write(root.join("configs/settings.toml"), "enabled = true\n")
        .expect("write authority");
    let document = "docs/index.md";
    let expected = crate::document_references::expected_targets(
        root,
        document,
        &BTreeSet::from(["../configs/settings.toml".to_string()]),
    )
    .expect("admit required authority");
    std::fs::write(
        root.join(document),
        "[authority](https://github.com/bijux/bijux-genomics/blob/main/configs/settings.toml)\n",
    )
    .expect("write published citation");
    assert_eq!(
        crate::document_references::markdown_targets(root, document).expect("read citations"),
        expected
    );
    std::fs::write(
        root.join(document),
        "[authority](https://github.com/foreign/repo/blob/main/configs/settings.toml)\n\
```markdown\n[example](../configs/settings.toml)\n```\n",
    )
    .expect("write substituted external citation");
    let substituted = crate::document_references::markdown_targets(root, document)
        .expect("read external-resource citation");
    assert!(!expected.is_subset(&substituted));
    std::fs::write(root.join(document), "[missing](../configs/missing.toml)\n")
        .expect("write missing citation");
    assert!(crate::document_references::markdown_targets(root, document).is_err());
}

#[test]
fn policy__contracts__document_reference_policy__required_reference_fragments_cannot_be_removed_or_substituted(
) {
    let scratch = bijux_dna_testkit::tempdir_for("documentation-reference-sections");
    let root = scratch.path();
    std::fs::create_dir(root.join("docs")).expect("create document directory");
    std::fs::write(root.join("authority.md"), "# Rules\n\n# Other\n")
        .expect("write section authority");
    let document = "docs/index.md";
    let expected = crate::document_references::expected_targets(
        root,
        document,
        &BTreeSet::from(["../authority.md#rules".to_string()]),
    )
    .expect("admit required section identity");
    for target in [
        "../authority.md#rules",
        "/authority.md#rules",
        "https://github.com/bijux/bijux-genomics/blob/main/authority.md#%72ules",
    ] {
        std::fs::write(root.join(document), format!("[required section]({target})\n"))
            .expect("write equivalent section citation");
        assert_eq!(
            crate::document_references::markdown_targets(root, document).expect("read section"),
            expected
        );
    }
    for target in ["../authority.md", "../authority.md#other"] {
        std::fs::write(root.join(document), format!("[other section]({target})\n"))
            .expect("write mismatched section citation");
        let documented = crate::document_references::markdown_targets(root, document)
            .expect("read mismatched section");
        assert!(!expected.is_subset(&documented));
    }
    assert!(crate::document_references::repository_target(root, document, "../authority.md#%00",)
        .is_err());
}
