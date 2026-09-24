#!/usr/bin/env python3
"""Behavioral tests for validate_evidence.py (stdlib unittest only, no deps)."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import validate_evidence as ve


def _base_row(**overrides) -> dict:
    row = {
        "id": "evt-0001",
        "timestamp_utc": "2026-09-24T12:00:00Z",
        "actor": "baseline-verification",
        "worktree": "integration",
        "commit_sha": "1" * 40,
        "tree_sha": "1" * 40,
        "environment": "gene / refcell, rustc 1.96.0",
        "runtime_overrides": [],
        "category": "build",
        "command": "cargo build --workspace",
        "result": "unknown",
        "evidence_refs": [],
        "notes": "test row",
    }
    row.update(overrides)
    return row


def _seed_row(**overrides) -> dict:
    row = {
        "id": "evt-0000",
        "example": True,
        "timestamp_utc": "1970-01-01T00:00:00Z",
        "actor": "docs-worker-illustrative",
        "worktree": "worktrees/factory-docs",
        "commit_sha": "0" * 40,
        "category": "other",
        "command": "echo example only - not executed",
        "result": "unknown",
        "evidence_refs": [],
        "notes": (
            "Illustrative schema example only. Not a real check. "
            "Real entries must omit `example` or set it to false."
        ),
    }
    row.update(overrides)
    return row


def _write_lines(lines: list[str]) -> Path:
    temporary = tempfile.NamedTemporaryFile(mode="w", suffix=".jsonl", delete=False)
    for line in lines:
        temporary.write(line + "\n")
    temporary.close()
    return Path(temporary.name)


def _write_jsonl(rows: list[dict]) -> Path:
    return _write_lines([json.dumps(row) for row in rows])


class ValidateRowTests(unittest.TestCase):
    def assert_has_error(self, row: dict, text: str, line_no: int = 1):
        messages = [error.message for error in ve.validate_row(row, line_no)]
        self.assertTrue(any(text in message for message in messages), messages)

    def test_valid_clean_and_dirty_rows_have_no_errors(self):
        clean = _base_row(result="pass", evidence_refs=["logs/build.txt"])
        dirty = _base_row(
            tree_sha=("1" * 40) + "+dirty:" + ("a" * 64),
            runtime_overrides=["PORT=8545"],
        )
        self.assertEqual(ve.validate_row(clean, 1), [])
        self.assertEqual(ve.validate_row(dirty, 1), [])

    def test_pass_without_evidence_refs_is_rejected(self):
        self.assert_has_error(_base_row(result="pass", evidence_refs=[]), "evidence_refs")

    def test_commit_and_tree_identity_formats_are_enforced(self):
        self.assert_has_error(_base_row(commit_sha="abc"), "exactly 40")
        self.assert_has_error(_base_row(tree_sha="1" * 64), "tree_sha")
        self.assert_has_error(
            _base_row(tree_sha=("2" * 40) + "+dirty:" + ("a" * 64)),
            "row's 'commit_sha'",
        )
        self.assert_has_error(
            _base_row(tree_sha=("1" * 40) + "+dirty:not-a-sha256"),
            "tree_sha",
        )

    def test_timestamp_must_be_valid_utc_iso8601(self):
        for timestamp in ("2026-09-24", "2026-09-24T12:00:00+00:00", "2026-02-30T12:00:00Z"):
            with self.subTest(timestamp=timestamp):
                self.assert_has_error(_base_row(timestamp_utc=timestamp), "timestamp_utc")

    def test_required_text_fields_are_typed_and_nonblank(self):
        for field_name in ("actor", "worktree", "environment", "command", "notes"):
            with self.subTest(field=field_name, value="blank"):
                self.assert_has_error(_base_row(**{field_name: "  "}), field_name)
            with self.subTest(field=field_name, value="wrong type"):
                self.assert_has_error(_base_row(**{field_name: True}), field_name)

    def test_array_fields_require_nonblank_strings(self):
        for field_name in ("evidence_refs", "runtime_overrides"):
            for value in ("not-an-array", [""], [3]):
                with self.subTest(field=field_name, value=value):
                    self.assert_has_error(_base_row(**{field_name: value}), field_name)

    def test_example_marker_must_be_boolean(self):
        row = _base_row(example="false", result="pass", evidence_refs=["ref"])
        self.assert_has_error(row, "must be a boolean")
        self.assertFalse(ve.counts_as_pass(row))
        typed_false = _base_row(example=False, result="pass", evidence_refs=["ref"])
        self.assertEqual(ve.validate_row(typed_false, 1), [])
        self.assertTrue(ve.counts_as_pass(typed_false))

    def test_only_exact_first_evt_0000_receives_legacy_example_exemption(self):
        self.assertEqual(ve.validate_row(_seed_row(), 1), [])
        self.assert_has_error(_seed_row(id="evt-0002"), "historical first-row")
        self.assert_has_error(_seed_row(), "historical first-row", line_no=2)
        self.assert_has_error(_seed_row(actor="imposter"), "historical first-row")
        later_example = _base_row(id="evt-0002", example=True)
        del later_example["tree_sha"]
        del later_example["environment"]
        self.assert_has_error(later_example, "identity/environment", line_no=2)

    def test_example_pass_is_rejected_and_never_counted(self):
        row = _seed_row(result="pass", evidence_refs=["fake"])
        self.assert_has_error(row, "example row")
        self.assertFalse(ve.counts_as_pass(row))


class ValidateFileTests(unittest.TestCase):
    def check_report(self, rows: list[dict]):
        path = _write_jsonl(rows)
        self.addCleanup(path.unlink)
        return ve.validate_file(path)

    def test_valid_future_pass_is_counted(self):
        report = self.check_report(
            [_seed_row(), _base_row(result="pass", evidence_refs=["logs/build.txt"])]
        )
        self.assertTrue(report.ok, report.errors)
        self.assertEqual(report.real_pass_count, 1)

    def test_duplicate_and_nonmonotonic_ids_are_rejected(self):
        duplicate = self.check_report([_base_row(id="evt-0001"), _base_row(id="evt-0001")])
        self.assertTrue(any("duplicate" in error.message for error in duplicate.errors))
        descending = self.check_report([_base_row(id="evt-0002"), _base_row(id="evt-0001")])
        self.assertTrue(any("strictly increasing" in error.message for error in descending.errors))

    def test_id_syntax_is_enforced(self):
        report = self.check_report([_base_row(id="evt-1")])
        self.assertTrue(any("evt-NNNN" in error.message for error in report.errors))

    def test_malformed_json_and_nonobject_lines_are_useful_errors(self):
        path = _write_lines(["{broken", "[]"])
        self.addCleanup(path.unlink)
        report = ve.validate_file(path)
        self.assertFalse(report.ok)
        self.assertIn("malformed JSON", str(report.errors[0]))
        self.assertIn("must be an object", str(report.errors[1]))

    def test_invalid_pass_does_not_inflate_count(self):
        report = self.check_report([_base_row(result="pass", evidence_refs=[])])
        self.assertFalse(report.ok)
        self.assertEqual(report.real_pass_count, 0)

    def test_actual_shipped_evidence_jsonl_validates(self):
        """The append-only ledger may acquire valid real passes over time."""
        real_path = Path(__file__).resolve().parent / "evidence.jsonl"
        if not real_path.exists():
            self.skipTest("evidence.jsonl not present in this checkout")
        report = ve.validate_file(real_path)
        self.assertTrue(report.ok, f"shipped evidence.jsonl should validate cleanly: {report.errors}")


class CliTests(unittest.TestCase):
    def run_validator(self, lines: list[str]):
        path = _write_lines(lines)
        self.addCleanup(path.unlink)
        return subprocess.run(
            [sys.executable, str(Path(ve.__file__).resolve()), str(path)],
            check=False,
            capture_output=True,
            text=True,
        )

    def test_cli_returns_zero_for_valid_file(self):
        completed = self.run_validator([json.dumps(_seed_row())])
        self.assertEqual(completed.returncode, 0, completed.stderr)

    def test_cli_returns_nonzero_and_error_for_malformed_json(self):
        completed = self.run_validator(["not json"])
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("malformed JSON", completed.stderr)


if __name__ == "__main__":
    unittest.main()
