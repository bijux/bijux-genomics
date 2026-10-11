#!/usr/bin/env python3
"""Delegate FASTQ corpus materialization to the stable Rust CLI."""

from __future__ import annotations

import os
import sys
from collections.abc import Mapping, Sequence


def delegated_command(
    arguments: Sequence[str], environment: Mapping[str, str] | None = None
) -> list[str]:
    """Build the argument-only compatibility delegation without invoking a shell."""
    values = os.environ if environment is None else environment
    executable = values.get("BIJUX_DNA_BIN") or "bijux-dna"
    return [executable, "bench", "materialize-fastq-corpus", *arguments]


def main(arguments: Sequence[str] | None = None) -> int:
    command = delegated_command(sys.argv[1:] if arguments is None else arguments)
    os.execvp(command[0], command)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
