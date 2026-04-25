#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import re
import subprocess
import sys
import tarfile
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import urlparse


SCAN_PATTERNS = (
    "configs/ci/registry/*.toml",
    "domain/**/*.yaml",
    "docs/20-science/**/*.md",
    "docs/30-operations/CONTAINER_LICENSE_INDEX.md",
    "science/**/*.yaml",
    "science-docs/README.md",
    "science-docs/TODO_DOWNLOAD.md",
    "science-docs/upstream/**/*.md",
    "science-docs/upstream/**/*.tsv",
    "mkdocs.yml",
)

GITHUB_URL_PATTERN = re.compile(r"https?://github\.com/[^\s;,)>\]\"']+")
TRAILING_URL_JUNK = "`),.;:"
IGNORED_SCAN_PATHS = {
    "science-docs/upstream/github-repos/MANIFEST.tsv",
}
IGNORED_SCAN_PREFIXES = (
    "science-docs/upstream/github-repos/archives/",
    "science-docs/upstream/github-repos/mirrors/",
    "science-docs/upstream/papers/",
)
IGNORED_SCAN_SUBSTRINGS = (
    "/repo/source.git/",
    "/repo/source/",
)


@dataclass(frozen=True)
class RepoRecord:
    owner: str
    repo: str
    sources: tuple[str, ...]

    @property
    def repo_id(self) -> str:
        return f"{self.owner}/{self.repo}"

    @property
    def clone_url(self) -> str:
        return f"https://github.com/{self.owner}/{self.repo}.git"

    def mirror_relpath(self) -> str:
        return f"science-docs/upstream/github-repos/mirrors/{self.owner}/{self.repo}.git"

    def archive_relpath(self) -> str:
        return f"science-docs/upstream/github-repos/archives/{self.owner}--{self.repo}.tar.gz"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Sync the local science-docs GitHub repository evidence archive."
    )
    repo_root_default = Path(__file__).resolve().parents[2]
    parser.add_argument("--repo-root", type=Path, default=repo_root_default)
    parser.add_argument(
        "--manifest-out",
        type=Path,
        default=Path("science-docs/upstream/github-repos/MANIFEST.tsv"),
    )
    parser.add_argument(
        "--mirror-root",
        type=Path,
        default=Path("science-docs/upstream/github-repos/mirrors"),
    )
    parser.add_argument(
        "--archive-root",
        type=Path,
        default=Path("science-docs/upstream/github-repos/archives"),
    )
    parser.add_argument(
        "--state-out",
        type=Path,
        default=Path("artifacts/science-docs/github-repos/local-state.tsv"),
    )
    parser.add_argument(
        "--archive-format",
        choices=("none", "tar.gz"),
        default="none",
        help="optionally create compressed exports alongside the local mirrors",
    )
    parser.add_argument(
        "--skip-sync",
        action="store_true",
        help="only regenerate the tracked manifest; do not clone or update local mirrors",
    )
    return parser.parse_args()


def normalize_repo_id(raw_url: str) -> tuple[str, str] | None:
    trimmed = raw_url.split(";", 1)[0].rstrip(TRAILING_URL_JUNK)
    parsed = urlparse(trimmed)
    if parsed.netloc.lower() != "github.com":
        return None
    parts = [part for part in parsed.path.split("/") if part]
    if len(parts) < 2:
        return None
    owner = parts[0].strip()
    repo = parts[1].strip().removesuffix(".git").rstrip(TRAILING_URL_JUNK)
    if not owner or not repo:
        return None
    return owner, repo


def discover_repo_records(repo_root: Path) -> list[RepoRecord]:
    discovered: dict[tuple[str, str], set[str]] = {}
    for pattern in SCAN_PATTERNS:
        for path in repo_root.glob(pattern):
            if not path.is_file():
                continue
            relpath = str(path.relative_to(repo_root))
            if relpath in IGNORED_SCAN_PATHS:
                continue
            if relpath.startswith(IGNORED_SCAN_PREFIXES):
                continue
            if any(marker in relpath for marker in IGNORED_SCAN_SUBSTRINGS):
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            for raw_url in GITHUB_URL_PATTERN.findall(text):
                repo_id = normalize_repo_id(raw_url)
                if repo_id is None:
                    continue
                discovered.setdefault(repo_id, set()).add(relpath)

    records = [
        RepoRecord(owner=owner, repo=repo, sources=tuple(sorted(paths)))
        for (owner, repo), paths in sorted(discovered.items())
    ]
    return records


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def resolve_path(repo_root: Path, path: Path) -> Path:
    return path if path.is_absolute() else repo_root / path


def write_manifest(repo_root: Path, manifest_path: Path, records: list[RepoRecord]) -> None:
    ensure_parent(manifest_path)
    with manifest_path.open("w", encoding="utf-8", newline="") as handle:
        handle.write(
            "# GENERATED FILE - DO NOT EDIT\n"
            "# Regenerate with: python3 makes/bin/sync_science_docs_github_repos.py --skip-sync\n"
        )
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(
            [
                "repo_id",
                "clone_url",
                "mirror_path",
                "archive_path",
                "reference_count",
                "sample_sources",
            ]
        )
        for record in records:
            writer.writerow(
                [
                    record.repo_id,
                    record.clone_url,
                    record.mirror_relpath(),
                    record.archive_relpath(),
                    str(len(record.sources)),
                    ";".join(record.sources[:5]),
                ]
            )


def run_git(args: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=cwd,
        check=False,
        text=True,
        capture_output=True,
    )


def sync_mirror(record: RepoRecord, mirror_dir: Path) -> tuple[str, str, str]:
    mirror_dir.parent.mkdir(parents=True, exist_ok=True)
    if mirror_dir.exists():
        fetch = run_git(
            [
                "git",
                "--git-dir",
                str(mirror_dir),
                "fetch",
                "--prune",
                "--tags",
                "origin",
                "+refs/*:refs/*",
            ]
        )
        if fetch.returncode != 0:
            raise RuntimeError(fetch.stderr.strip() or fetch.stdout.strip() or "git fetch failed")
        status = "updated"
    else:
        clone = run_git(["git", "clone", "--mirror", record.clone_url, str(mirror_dir)])
        if clone.returncode != 0:
            raise RuntimeError(clone.stderr.strip() or clone.stdout.strip() or "git clone failed")
        status = "cloned"

    head_ref = run_git(["git", "--git-dir", str(mirror_dir), "symbolic-ref", "HEAD"])
    head_commit = run_git(["git", "--git-dir", str(mirror_dir), "rev-parse", "HEAD"])
    if head_ref.returncode != 0:
        raise RuntimeError(head_ref.stderr.strip() or "git symbolic-ref HEAD failed")
    if head_commit.returncode != 0:
        raise RuntimeError(head_commit.stderr.strip() or "git rev-parse HEAD failed")
    return status, head_ref.stdout.strip(), head_commit.stdout.strip()


def write_tar_gz_archive(source_dir: Path, archive_path: Path) -> None:
    ensure_parent(archive_path)
    with tarfile.open(archive_path, "w:gz") as archive:
        archive.add(source_dir, arcname=source_dir.name)


def write_local_state(
    state_path: Path,
    rows: list[tuple[str, str, str, str, str, str]],
) -> None:
    ensure_parent(state_path)
    with state_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(
            [
                "repo_id",
                "sync_status",
                "head_ref",
                "head_commit",
                "mirror_path",
                "archive_path",
            ]
        )
        writer.writerows(rows)


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.resolve()
    manifest_path = resolve_path(repo_root, args.manifest_out)
    mirror_root = resolve_path(repo_root, args.mirror_root)
    archive_root = resolve_path(repo_root, args.archive_root)
    state_path = resolve_path(repo_root, args.state_out)

    records = discover_repo_records(repo_root)
    write_manifest(repo_root, manifest_path, records)
    print(f"wrote manifest: {manifest_path.relative_to(repo_root)}")

    if args.skip_sync:
        return 0

    state_rows: list[tuple[str, str, str, str, str, str]] = []
    failures: list[str] = []

    for record in records:
        mirror_dir = mirror_root / record.owner / f"{record.repo}.git"
        archive_path = archive_root / f"{record.owner}--{record.repo}.tar.gz"
        print(f"sync {record.repo_id}")
        try:
            sync_status, head_ref, head_commit = sync_mirror(record, mirror_dir)
            archive_value = ""
            if args.archive_format == "tar.gz":
                write_tar_gz_archive(mirror_dir, archive_path)
                archive_value = str(archive_path.relative_to(repo_root))
            state_rows.append(
                (
                    record.repo_id,
                    sync_status,
                    head_ref,
                    head_commit,
                    str(mirror_dir.relative_to(repo_root)),
                    archive_value,
                )
            )
        except Exception as exc:  # pragma: no cover - runtime/network dependent
            failures.append(f"{record.repo_id}: {exc}")

    write_local_state(state_path, state_rows)
    print(f"wrote local state: {state_path.relative_to(repo_root)}")

    if failures:
        print("github repo sync failures:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
