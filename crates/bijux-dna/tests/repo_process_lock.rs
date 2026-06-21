#![allow(clippy::expect_used)]

use std::fs;
use std::path::Path;
use std::time::Duration;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

const OWNER_PID_FILE: &str = "owner.pid";

fn remove_lock_dir(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path).expect("remove lock dir");
    }
}

#[test]
fn repo_process_lock_reclaims_stale_empty_directory() {
    let repo_root = support::repo_root().expect("repo root");
    let lock_path = repo_root.join("artifacts/test-locks/repo-process-lock-stale-empty");
    remove_lock_dir(&lock_path);

    fs::create_dir_all(&lock_path).expect("create stale lock dir");
    std::thread::sleep(Duration::from_secs(2));

    let lock = support::RepoProcessLock::acquire("repo-process-lock-stale-empty")
        .expect("reclaim empty stale lock");
    assert!(lock_path.exists(), "lock path should exist while held");
    drop(lock);

    assert!(!lock_path.exists(), "lock path should be removed after drop");
}

#[test]
fn repo_process_lock_reclaims_dead_owner_directory() {
    let repo_root = support::repo_root().expect("repo root");
    let lock_path = repo_root.join("artifacts/test-locks/repo-process-lock-dead-owner");
    remove_lock_dir(&lock_path);

    fs::create_dir_all(&lock_path).expect("create stale lock dir");
    fs::write(lock_path.join(OWNER_PID_FILE), "999999").expect("write dead owner pid");

    let lock = support::RepoProcessLock::acquire("repo-process-lock-dead-owner")
        .expect("reclaim dead owner lock");
    assert!(lock_path.exists(), "lock path should exist while held");
    drop(lock);

    assert!(!lock_path.exists(), "lock path should be removed after drop");
}
