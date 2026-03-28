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
BENCHMARK_CONFIG_ENV = "BIJUX_BENCHMARK_CONFIG"
LEGACY_BENCHMARK_FASTQ_CORPUS_CONFIG_ENV = "BIJUX_FASTQ_CORPUS_CONFIG"
BENCHMARK_FASTQ_CORPUS_CONFIG_ENV = LEGACY_BENCHMARK_FASTQ_CORPUS_CONFIG_ENV
DEFAULT_BENCHMARK_CONFIG_PATH = REPO_ROOT / "configs" / "bench" / "benchmark.toml"
DEFAULT_WORKSPACE_CONFIG_PATH = REPO_ROOT / "configs" / "bench" / "workspace.toml"
DEFAULT_PUBLICATION_CONFIG_PATH = REPO_ROOT / "configs" / "bench" / "publication.toml"

_workspace_config_override: Path | None = None


def expand_env_placeholders(raw: str) -> str:
    expanded: list[str] = []
    index = 0
    while index < len(raw):
        if raw[index : index + 2] == "${":
            end = raw.find("}", index + 2)
            if end == -1:
                expanded.append(raw[index:])
                break
            name = raw[index + 2 : end]
            expanded.append(os.environ.get(name, ""))
            index = end + 1
            continue
        expanded.append(raw[index])
        index += 1
    return "".join(expanded)


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
    path = current_workspace_config_path()
    if not path.is_file() or toml_loader is None:
        return {}
    raw = path.read_text(encoding="utf-8")
    return toml_loader.loads(expand_env_placeholders(raw))


@lru_cache(maxsize=1)
def load_workspace_config() -> dict:
    path = current_workspace_config_path()
    payload = load_benchmark_config()
    if "workspace" in payload:
        workspace = payload.get("workspace", {})
        return workspace if isinstance(workspace, dict) else {}
    if path == DEFAULT_BENCHMARK_CONFIG_PATH:
        legacy_path = DEFAULT_WORKSPACE_CONFIG_PATH
        if not legacy_path.is_file() or toml_loader is None:
            return {}
        raw = legacy_path.read_text(encoding="utf-8")
        return toml_loader.loads(expand_env_placeholders(raw))
    return payload


@lru_cache(maxsize=1)
def load_publication_config() -> dict:
    payload = load_benchmark_config()
    if "publication" in payload:
        publication = payload.get("publication", {})
        return publication if isinstance(publication, dict) else {}
    path = DEFAULT_PUBLICATION_CONFIG_PATH
    if not path.is_file() or toml_loader is None:
        return {}
    raw = path.read_text(encoding="utf-8")
    return toml_loader.loads(expand_env_placeholders(raw))


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
