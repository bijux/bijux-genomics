"""Bind fresh workspace binaries before delegating to the managed Rust test gate."""

import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def artifact_paths(environment, workspace):
    def resolve(value):
        path = Path(value)
        if not path.is_absolute():
            path = workspace / path
        return path.resolve()

    boundary = resolve(environment.get("ARTIFACT_ROOT") or workspace / "artifacts")
    rust = resolve(environment.get("RS_ARTIFACT_ROOT") or boundary / "rust")
    paths = {
        "RS_ARTIFACT_ROOT": rust,
        "RS_TARGET_DIR": resolve(environment.get("RS_TARGET_DIR") or rust / "target"),
        "RS_CARGO_HOME": resolve(environment.get("RS_CARGO_HOME") or rust / "cargo/home"),
        "RS_TMP_DIR": resolve(environment.get("RS_TMP_DIR") or rust / "tmp"),
    }
    paths["RS_NEXTEST_CACHE_DIR"] = resolve(
        environment.get("RS_NEXTEST_CACHE_DIR") or paths["RS_TARGET_DIR"] / "nextest"
    )
    paths["RS_NEXTEST_CONFIG_HOME"] = resolve(
        environment.get("RS_NEXTEST_CONFIG_HOME") or rust / "nextest/config"
    )
    for name, path in paths.items():
        if not path.is_relative_to(boundary):
            raise ValueError(f"{name} must stay under {boundary}: {path}")
    return boundary, paths


def list_arguments(environment, mode, target):
    profile_variable = "NEXTEST_PROFILE_FAST" if mode == "test" else "NEXTEST_PROFILE_SLOW"
    arguments = ["cargo", "nextest", "list", "--workspace", "--all-features"]
    if (environment.get("RUST_CARGO_LOCKED") or "1") == "1":
        arguments.append("--locked")
    arguments.extend([
        "--config-file", environment["NEXTEST_CONFIG_FILE"],
        "--profile", environment.get(profile_variable) or "default",
        "--target-dir", str(target),
        "--list-type", "binaries-only", "--message-format", "json",
    ])
    return arguments


def executable_bindings(inventory, expected_target):
    metadata = inventory["rust-build-meta"]
    target = Path(metadata["target-directory"])
    if not target.is_absolute() or target.resolve() != expected_target:
        raise ValueError("compiled inventory target directory does not match the selected target")
    required = {"bijux-dna", "bijux-dna-dev"}
    selected = {}
    for entries in metadata["non-test-binaries"].values():
        for entry in entries:
            name = entry["name"]
            if name not in required:
                continue
            if name in selected or entry["kind"] != "bin-exe":
                raise ValueError(f"compiled workspace executable is ambiguous or not binary: {name}")
            relative = entry["path"]
            if (
                not isinstance(relative, str) or not relative or "\\" in relative or "\0" in relative
                or Path(relative).is_absolute()
                or any(part in ("", ".", "..") for part in relative.split("/"))
                or Path(relative).name not in (name, f"{name}.exe")
            ):
                raise ValueError(f"compiled workspace executable path is invalid: {name}")
            path = target / relative
            if not path.resolve().is_relative_to(expected_target) or not path.is_file() or not os.access(path, os.X_OK):
                raise ValueError(f"compiled workspace executable is absent or not executable: {path}")
            selected[name] = str(path)
    if set(selected) != required:
        raise ValueError("compiled inventory is missing required workspace executables")
    return {f"NEXTEST_BIN_EXE_{name}": path for name, path in selected.items()}


def main(arguments):
    if len(arguments) < 2 or arguments[1] not in ("test", "test-slow"):
        raise ValueError("expected managed gate path and test or test-slow command")
    shared_gate, mode = arguments[:2]
    environment = os.environ.copy()
    workspace = Path(environment["PROJECT_ROOT"]).resolve(strict=True)
    config = Path(environment["NEXTEST_CONFIG_FILE"])
    if not config.is_absolute():
        config = workspace / config
    if not config.is_file():
        raise ValueError(f"required file is unavailable: {config}")
    run_id = environment.get("RS_RUN_ID") or environment.get("RUN_ID") or "local"
    if re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", run_id) is None:
        raise ValueError(f"RS_RUN_ID contains unsupported characters: {run_id}")
    boundary, paths = artifact_paths(environment, workspace)
    inventories = (paths["RS_ARTIFACT_ROOT"] / "test/inventories").resolve()
    if not inventories.is_relative_to(boundary):
        raise ValueError(f"inventory logs must stay under {boundary}: {inventories}")
    for path in paths.values():
        path.mkdir(parents=True, exist_ok=True)
    environment.update({name: str(path) for name, path in paths.items()})
    environment.update({
        "ARTIFACT_ROOT": str(boundary),
        "CARGO_HOME": str(paths["RS_CARGO_HOME"]),
        "CARGO_TARGET_DIR": str(paths["RS_TARGET_DIR"]),
        "TMPDIR": str(paths["RS_TMP_DIR"]), "TMP": str(paths["RS_TMP_DIR"]),
        "TEMP": str(paths["RS_TMP_DIR"]),
        "NEXTEST_CACHE_DIR": str(paths["RS_NEXTEST_CACHE_DIR"]),
        "XDG_CONFIG_HOME": str(paths["RS_NEXTEST_CONFIG_HOME"]),
    })
    inventories.mkdir(parents=True, exist_ok=True)
    observation = Path(tempfile.mkdtemp(prefix=f"{mode}-", dir=inventories))
    metadata_path = observation / "binaries-metadata.json"
    error_path = observation / "producer.stderr"
    with metadata_path.open("wb") as output, error_path.open("wb") as errors:
        producer = subprocess.run(
            list_arguments(environment, mode, paths["RS_TARGET_DIR"]),
            cwd=workspace, env=environment, stdout=output, stderr=errors, check=False,
        )
    with error_path.open("rb") as errors:
        shutil.copyfileobj(errors, sys.stderr.buffer)
    if producer.returncode:
        return producer.returncode if producer.returncode > 0 else 128 - producer.returncode
    environment.update(executable_bindings(json.loads(metadata_path.read_bytes()), paths["RS_TARGET_DIR"]))
    os.execve(shared_gate, arguments, environment)


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv[1:]))
    except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        print(f"prepare workspace test executables: {error}", file=sys.stderr)
        sys.exit(1)
