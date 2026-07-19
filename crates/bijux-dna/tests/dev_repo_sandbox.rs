#![allow(clippy::expect_used)]

use std::fs;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn repo_sandboxes_keep_governed_writes_independent() {
    let source_root = support::repo_root().expect("repo root");
    let relative_path = "benchmarks/readiness/missing-result-report-test.json";
    let source_path = source_root.join(relative_path);
    let source_contents = fs::read(&source_path).expect("read source report");

    let producer =
        support::RepoSandbox::new("readiness-isolation-producer-").expect("producer sandbox");
    let auditor =
        support::RepoSandbox::new("readiness-isolation-auditor-").expect("auditor sandbox");
    assert_ne!(producer.path(), auditor.path());

    fs::write(producer.path().join(relative_path), b"{\"sandbox\":\"producer\"}\n")
        .expect("write producer sandbox report");
    fs::write(auditor.path().join(relative_path), b"{\"sandbox\":\"auditor\"}\n")
        .expect("write auditor sandbox report");

    assert_eq!(
        fs::read(producer.path().join(relative_path)).expect("read producer sandbox report"),
        b"{\"sandbox\":\"producer\"}\n"
    );
    assert_eq!(
        fs::read(auditor.path().join(relative_path)).expect("read auditor sandbox report"),
        b"{\"sandbox\":\"auditor\"}\n"
    );
    assert_eq!(fs::read(source_path).expect("reread source report"), source_contents);
}
