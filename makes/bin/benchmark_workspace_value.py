#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys

import corpus_01_fastq_benchmark_support as support


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Print a configured benchmark workspace value."
    )
    parser.add_argument(
        "key_path",
        help="Dotted workspace key path, for example remote.corpus_root.",
    )
    return parser.parse_args()


def resolve_workspace_value(key_path: str) -> str:
    if key_path == "remote.ssh_host":
        return support.benchmark_remote_ssh_host()
    if key_path == "remote.frontend_root":
        return str(support.benchmark_remote_frontend_root())
    if key_path == "remote.corpus_root":
        return str(support.benchmark_remote_corpus_root())
    if key_path == "remote.repo_root":
        return str(support.benchmark_remote_repo_root())
    if key_path == "remote.results_root":
        return str(support.benchmark_remote_results_root())
    if key_path == "remote.cache_root":
        return str(support.benchmark_remote_cache_root())
    if key_path == "remote.extra_data_root":
        return str(support.benchmark_remote_extra_data_root())
    if key_path == "remote.reference_root":
        return str(support.benchmark_remote_reference_root())
    if key_path == "remote.containers_root":
        return str(support.benchmark_remote_containers_root())
    if key_path == "local.results_root":
        return str(support.benchmark_local_results_root())
    if key_path == "local.cache_mirror_root":
        return str(support.benchmark_local_cache_mirror_root())
    if key_path == "local.extra_data_root":
        return str(support.benchmark_local_extra_data_root())
    if key_path == "local.reference_root":
        return str(support.benchmark_local_reference_root())
    if key_path == "sync.defaults.pull_base":
        return str(support.benchmark_sync_default_pull_base())
    if key_path == "sync.defaults.pull_mode":
        return support.benchmark_sync_default_pull_mode()
    if key_path == "sync.defaults.include_profile":
        return support.benchmark_sync_default_include_profile()
    if key_path == "sync.defaults.exclude_profile":
        return support.benchmark_sync_default_exclude_profile()
    if key_path == "sync.defaults.clean_context":
        return "1" if support.benchmark_sync_default_clean_context() else "0"
    if key_path == "sync.defaults.allow_dirty":
        return "1" if support.benchmark_sync_default_allow_dirty() else "0"
    if key_path == "sync.defaults.include_containers_manifest":
        return (
            "1"
            if support.benchmark_sync_default_include_containers_manifest()
            else "0"
        )
    if key_path == "sync.defaults.data_manifest_glob":
        return support.benchmark_sync_default_data_manifest_glob()
    raise SystemExit(f"unsupported benchmark workspace key path: {key_path}")


def main() -> int:
    args = parse_args()
    sys.stdout.write(resolve_workspace_value(args.key_path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
