"""Capture native Python case identities and outcomes for complete local tests."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import sys
import unittest


def flatten(suite):
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            yield from flatten(item)
        else:
            yield item


class CaseResult(unittest.TextTestResult):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.outcomes = []

    def record(self, test, outcome):
        self.outcomes.append({"id": test.id(), "outcome": outcome})

    def addSuccess(self, test):
        super().addSuccess(test)
        self.record(test, "passed")

    def addFailure(self, test, error):
        super().addFailure(test, error)
        self.record(test, "failed")

    def addError(self, test, error):
        super().addError(test, error)
        self.record(test, "error")

    def addSkip(self, test, reason):
        super().addSkip(test, reason)
        self.record(test, "skipped")

    def addExpectedFailure(self, test, error):
        super().addExpectedFailure(test, error)
        self.record(test, "expected_failure")

    def addUnexpectedSuccess(self, test):
        super().addUnexpectedSuccess(test)
        self.record(test, "unexpected_success")

    def addSubTest(self, test, subtest, error):
        super().addSubTest(test, subtest, error)
        if error is not None:
            self.record(test, "failed_subtest")


class PytestCases:
    def __init__(self):
        self.expected = []
        self.states = {}
        self.starts = {}
        self.collection_errors = []

    def pytest_collection_finish(self, session):
        self.expected = [item.nodeid for item in session.items]

    def pytest_collectreport(self, report):
        if report.failed:
            self.collection_errors.append(report.nodeid)

    def pytest_deselected(self, items):
        self.collection_errors.extend(f"deselected:{item.nodeid}" for item in items)

    def pytest_runtest_logreport(self, report):
        if report.when == "setup":
            self.starts[report.nodeid] = self.starts.get(report.nodeid, 0) + 1
        if hasattr(report, "wasxfail"):
            self.states[report.nodeid] = "expected_failure" if report.skipped else "unexpected_success"
        elif report.failed:
            self.states[report.nodeid] = "failed"
        elif report.skipped:
            self.states[report.nodeid] = "skipped"
        elif report.when == "call" and report.passed:
            self.states.setdefault(report.nodeid, "passed")


def membership(expected, outcomes):
    actual = [record["id"] for record in outcomes]
    duplicate = sorted({identity for identity in actual if actual.count(identity) > 1})
    expected_duplicates = len(expected) != len(set(expected))
    missing = sorted(set(expected) - set(actual))
    unexpected = sorted(set(actual) - set(expected))
    passed = (
        bool(expected)
        and not expected_duplicates
        and not duplicate
        and not missing
        and not unexpected
        and all(record["outcome"] == "passed" for record in outcomes)
    )
    return {"missing": missing, "unexpected": unexpected, "duplicate": duplicate,
            "expected_duplicates": expected_duplicates, "all_passed": passed}


def run(args):
    root = Path(args.source_root).resolve()
    reads = set()
    external_reads = set()
    calls = []

    def audit(event, values):
        if event == "open" and values and isinstance(values[0], (str, bytes, os.PathLike)):
            path = Path(os.fsdecode(values[0]))
            try:
                path = path.resolve()
                if path.is_relative_to(root / "artifacts") or path.is_relative_to(Path(args.output).resolve().parent):
                    return
                if path.is_relative_to(root):
                    reads.add(path.relative_to(root).as_posix())
                elif path.is_file():
                    external_reads.add(str(path))
            except (OSError, ValueError):
                pass
        elif event == "subprocess.Popen":
            program = str(values[0])
            environment = values[3] or os.environ
            calls.append(shutil.which(program, path=environment.get("PATH")) or program)

    sys.addaudithook(audit)
    sys.path.insert(0, str(root))
    os.chdir(root)
    collection_errors = []
    if args.harness == "unittest":
        loader = unittest.TestLoader()
        suite = loader.loadTestsFromNames(args.selections)
        expected = [test.id() for test in flatten(suite)]
        collection_errors = list(loader.errors)
        if args.discover:
            outcomes = []
            exit_code = 0 if not collection_errors else 1
        else:
            result = unittest.TextTestRunner(verbosity=2, resultclass=CaseResult).run(suite)
            outcomes = result.outcomes
            exit_code = 0 if result.wasSuccessful() else 1
    else:
        import pytest
        plugin = PytestCases()
        options = ["-q", *args.selections, "-o", "addopts=", "-o", f"cache_dir={args.scratch}/pytest-cache",
                   f"--basetemp={args.scratch}/pytest-scratch"]
        if args.discover:
            options.append("--collect-only")
        exit_code = int(pytest.main(options, plugins=[plugin]))
        expected = plugin.expected
        outcomes = [{"id": identity, "outcome": outcome} for identity, outcome in sorted(plugin.states.items())]
        outcomes.extend({"id": identity, "outcome": "duplicate_started"}
                        for identity, starts in plugin.starts.items() if starts != 1)
        collection_errors = plugin.collection_errors
    if args.expected:
        admitted = json.loads(Path(args.expected).read_text())["expected"]
        if set(admitted) != set(expected) or len(admitted) != len(expected):
            collection_errors.append("execution discovery differs from admitted case inventory")
        expected = admitted
    facts = membership(expected, outcomes) if not args.discover else {"all_passed": False}
    record = {"schema_version": "bijux.ci.python_cases.v1", "source_root": str(root), "harness": args.harness,
              "discovery_only": args.discover, "expected": expected, "outcomes": outcomes,
              "collection_errors": collection_errors, "source_reads": sorted(reads),
              "external_reads": sorted(external_reads),
              "subprocess_programs": sorted(set(calls)), "exit_code": exit_code,
              "membership": facts}
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(record, indent=2) + "\n")
    if args.discover:
        return exit_code if expected and not collection_errors else 1
    return 0 if exit_code == 0 and not collection_errors and facts["all_passed"] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--scratch", required=True)
    parser.add_argument("--harness", choices=["pytest", "unittest"], required=True)
    parser.add_argument("--discover", action="store_true")
    parser.add_argument("--expected")
    parser.add_argument("selections", nargs="+")
    return run(parser.parse_args())


if __name__ == "__main__":
    raise SystemExit(main())
