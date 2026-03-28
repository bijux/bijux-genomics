#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


BENCHMARK_TOOLING_GLOBS = (
    "makes/bin/*.py",
    "makes/*.mk",
)

EXCLUDED_TOOLING_PATHS = {
    Path("makes/bin/test_corpus_01_fastq_benchmarks.py"),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fail when benchmark tooling reintroduces forbidden hardcoded literals."
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root to scan.",
    )
    parser.add_argument(
        "--json-out",
        default=None,
        help="Optional machine-readable report path.",
    )
    return parser.parse_args()


def benchmark_tooling_paths(repo_root: Path) -> list[Path]:
    paths: set[Path] = set()
    for pattern in BENCHMARK_TOOLING_GLOBS:
        paths.update(path for path in repo_root.glob(pattern) if path.is_file())
    return sorted(
        path
        for path in paths
        if path.relative_to(repo_root) not in EXCLUDED_TOOLING_PATHS
    )


def literal_matches(
    repo_root: Path,
    *,
    literal: str,
    issue_id: str,
) -> list[dict[str, str | int]]:
    matches: list[dict[str, str | int]] = []
    for path in benchmark_tooling_paths(repo_root):
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if literal not in line:
                continue
            matches.append(
                {
                    "issue_id": issue_id,
                    "path": str(path.relative_to(repo_root)),
                    "line": line_number,
                    "literal": literal,
                    "content": line.strip(),
                }
            )
    return matches


def audit_repo_checks(repo_root: Path) -> dict:
    violations = literal_matches(
        repo_root,
        literal="/Users/bijan/",
        issue_id="hardcoded-local-operator-path",
    )
    return {
        "check_count": 1,
        "violation_count": len(violations),
        "violations": violations,
    }


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    report = audit_repo_checks(repo_root)

    if args.json_out:
        json_out = Path(args.json_out).resolve()
        json_out.parent.mkdir(parents=True, exist_ok=True)
        json_out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    if report["violations"]:
        for violation in report["violations"]:
            print(
                f"{violation['path']}:{violation['line']}: "
                f"{violation['issue_id']} {violation['content']}"
            )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
