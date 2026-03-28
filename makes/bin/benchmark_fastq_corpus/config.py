from __future__ import annotations

import argparse
import os
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
BENCHMARK_FASTQ_CORPUS_CONFIG_ENV = "BIJUX_FASTQ_CORPUS_CONFIG"
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
        env_value = os.environ.get(BENCHMARK_FASTQ_CORPUS_CONFIG_ENV, "").strip()
        if env_value:
            raw_path = env_value
        else:
            return DEFAULT_WORKSPACE_CONFIG_PATH
    path = Path(raw_path).expanduser()
    if not path.is_absolute():
        path = root / path
    return path.resolve()


def clear_config_caches() -> None:
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
def load_workspace_config() -> dict:
    path = current_workspace_config_path()
    if not path.is_file() or toml_loader is None:
        return {}
    with path.open("rb") as handle:
        return toml_loader.load(handle)


@lru_cache(maxsize=1)
def load_publication_config() -> dict:
    path = DEFAULT_PUBLICATION_CONFIG_PATH
    if not path.is_file() or toml_loader is None:
        return {}
    with path.open("rb") as handle:
        return toml_loader.load(handle)


def add_workspace_config_argument(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--config",
        default="",
        help=(
            "Benchmark workspace config path. Defaults to the "
            f"`{BENCHMARK_FASTQ_CORPUS_CONFIG_ENV}` environment variable or "
            "configs/bench/workspace.toml."
        ),
    )
