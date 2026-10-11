"""Bind developer bootstrap success to Cargo's declared executable artifact."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path


def verify_executable(messages: Path, executable: Path, manifest: Path) -> None:
    """Require the selected binary and a successful terminal Cargo record."""
    artifacts: list[Path] = []
    finished: list[bool] = []
    for line in messages.read_text().splitlines():
        record = json.loads(line)
        if not isinstance(record, dict):
            raise ValueError("Cargo build record must be an object")
        if record.get("reason") == "build-finished":
            finished.append(record.get("success") is True)
        if record.get("reason") != "compiler-artifact":
            continue
        target = record.get("target")
        source = record.get("manifest_path")
        if (
            not isinstance(target, dict)
            or target.get("name") != "bijux-dna-dev"
            or target.get("kind") != ["bin"]
            or not isinstance(source, str)
            or Path(source).resolve() != manifest.resolve()
        ):
            continue
        output = record.get("executable")
        if not isinstance(output, str) or not output:
            raise ValueError("Cargo did not declare the selected executable")
        artifacts.append(Path(output).resolve())
    if finished != [True]:
        raise ValueError("Cargo did not declare one successful completed build")
    if artifacts != [executable.resolve()]:
        raise ValueError(
            "Cargo output does not match the requested developer executable"
        )
    if not executable.is_file() or not os.access(executable, os.X_OK):
        raise ValueError(
            "Cargo's requested developer executable is missing or not executable"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("messages", type=Path)
    parser.add_argument("executable", type=Path)
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    try:
        verify_executable(args.messages, args.executable, args.manifest)
    except (OSError, ValueError) as error:
        parser.exit(1, f"developer bootstrap artifact admission failed: {error}\n")


if __name__ == "__main__":
    main()
