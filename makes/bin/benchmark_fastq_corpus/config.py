from __future__ import annotations

import argparse
import json
import os
import subprocess
from functools import lru_cache
from pathlib import Path

try:
    import tomllib as toml_loader  # type: ignore[attr-defined]
except ModuleNotFoundError:
    try:
        import tomli as toml_loader  # type: ignore[no-redef]
    except ModuleNotFoundError:
        toml_loader = None


REPO_ROOT = Path(__file__).resolve().parents[3]
BENCHMARK_CONFIG_ENV = "BIJUX_BENCHMARK_CONFIG"
BENCHMARK_CONFIG_JSON_ENV = "BIJUX_BENCHMARK_CONFIG_JSON"
LEGACY_BENCHMARK_FASTQ_CORPUS_CONFIG_ENV = "BIJUX_FASTQ_CORPUS_CONFIG"
BENCHMARK_FASTQ_CORPUS_CONFIG_ENV = LEGACY_BENCHMARK_FASTQ_CORPUS_CONFIG_ENV
DEFAULT_BENCHMARK_CONFIG_PATH = REPO_ROOT / "configs" / "bench" / "benchmark.toml"
DEFAULT_WORKSPACE_CONFIG_PATH = REPO_ROOT / "configs" / "bench" / "workspace.toml"
DEFAULT_PUBLICATION_CONFIG_PATH = REPO_ROOT / "configs" / "bench" / "publication.toml"

_workspace_config_override: Path | None = None


def _normalize_config_path(
    raw_path: str | os.PathLike[str] | None,
    *,
    repo_root: Path | None = None,
) -> Path:
    root = repo_root.resolve() if repo_root is not None else REPO_ROOT
    if raw_path is None or str(raw_path).strip() == "":
        for env_name in (
            BENCHMARK_CONFIG_ENV,
            LEGACY_BENCHMARK_FASTQ_CORPUS_CONFIG_ENV,
        ):
            env_value = os.environ.get(env_name, "").strip()
            if env_value:
                raw_path = env_value
                break
        else:
            return DEFAULT_BENCHMARK_CONFIG_PATH
    path = Path(raw_path).expanduser()
    if not path.is_absolute():
        path = root / path
    return path.resolve()


def clear_config_caches() -> None:
    load_benchmark_config.cache_clear()
    load_workspace_config.cache_clear()
    load_publication_config.cache_clear()


def configure_workspace_config_path(
    raw_path: str | os.PathLike[str] | None,
    *,
    repo_root: Path | None = None,
) -> Path:
    global _workspace_config_override
    _workspace_config_override = _normalize_config_path(raw_path, repo_root=repo_root)
    clear_config_caches()
    return _workspace_config_override


def current_workspace_config_path(*, repo_root: Path | None = None) -> Path:
    if _workspace_config_override is not None:
        return _workspace_config_override
    return _normalize_config_path(None, repo_root=repo_root)


@lru_cache(maxsize=1)
def load_benchmark_config() -> dict:
    inline_json = os.environ.get(BENCHMARK_CONFIG_JSON_ENV, "").strip()
    if inline_json:
        return json.loads(inline_json)
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "bijux-dna",
        "--",
        "bench",
        "config-json",
        "--config",
        str(current_workspace_config_path()),
        "--section",
        "full",
    ]
    completed = subprocess.run(
        command,
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(completed.stdout)


@lru_cache(maxsize=1)
def load_workspace_config() -> dict:
    payload = load_benchmark_config()
    workspace = payload.get("workspace", {})
    return workspace if isinstance(workspace, dict) else {}


@lru_cache(maxsize=1)
def load_publication_config() -> dict:
    payload = load_benchmark_config()
    publication = payload.get("publication", {})
    return publication if isinstance(publication, dict) else {}


def add_workspace_config_argument(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--config",
        default="",
        help=(
            "Benchmark workspace config path. Defaults to the "
            f"`{BENCHMARK_CONFIG_ENV}` or `{LEGACY_BENCHMARK_FASTQ_CORPUS_CONFIG_ENV}` "
            "environment variable or configs/bench/benchmark.toml."
        ),
    )
