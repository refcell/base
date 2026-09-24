#!/usr/bin/env python3
"""Standard-library-only validator for the append-only evidence ledger.

Usage:
    python3 validate_evidence.py [--summary] [path/to/evidence.jsonl]

The command exits zero only when every nonblank line is a valid evidence object.
"""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path

DEFAULT_PATH = Path(__file__).resolve().parent / "evidence.jsonl"

BASE_REQUIRED_FIELDS = (
    "id",
    "timestamp_utc",
    "actor",
    "worktree",
    "commit_sha",
    "category",
    "command",
    "result",
    "evidence_refs",
    "notes",
)
TEXT_FIELDS = (
    "id",
    "timestamp_utc",
    "actor",
    "worktree",
    "commit_sha",
    "category",
    "command",
    "result",
    "notes",
)
VALID_RESULTS = {"pass", "fail", "blocked", "skipped", "unknown"}
VALID_CATEGORIES = {"build", "test", "devnet", "transaction", "fault", "review", "other"}
ID_RE = re.compile(r"evt-(\d{4})\Z")
HEX40_RE = re.compile(r"[0-9a-fA-F]{40}\Z")
DIRTY_TREE_RE = re.compile(r"([0-9a-fA-F]{40})\+dirty:([0-9a-fA-F]{64})\Z")
UTC_TIMESTAMP_RE = re.compile(
    r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\Z"
)
SEED_EXAMPLE_ROW = {
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


@dataclass
class RowError:
    line_no: int
    row_id: str
    message: str

    def __str__(self) -> str:  # pragma: no cover - trivial formatting
        return f"line {self.line_no} ({self.row_id}): {self.message}"


@dataclass
class ValidationReport:
    errors: list[RowError] = field(default_factory=list)
    real_pass_count: int = 0
    example_pass_attempts: int = 0

    @property
    def ok(self) -> bool:
        return not self.errors


def is_example(row: dict) -> bool:
    """Return whether the typed example marker is exactly boolean true."""
    return row.get("example") is True


def is_seed_example(row: dict, line_no: int) -> bool:
    """Only the exact first-line historical seed receives the legacy exemption."""
    return line_no == 1 and row == SEED_EXAMPLE_ROW


def counts_as_pass(row: dict) -> bool:
    """Only a pass with an omitted or typed-false example marker can count."""
    example = row.get("example", False)
    return isinstance(example, bool) and not example and row.get("result") == "pass"


def _array_errors(row: dict, field_name: str, line_no: int, row_id: str) -> list[RowError]:
    value = row.get(field_name)
    if not isinstance(value, list):
        return [RowError(line_no, row_id, f"'{field_name}' must be an array")]

    errors = []
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            errors.append(
                RowError(
                    line_no,
                    row_id,
                    f"'{field_name}[{index}]' must be a non-blank string",
                )
            )
    return errors


def validate_row(row: dict, line_no: int) -> list[RowError]:
    errors: list[RowError] = []
    raw_id = row.get("id")
    row_id = raw_id if isinstance(raw_id, str) and raw_id else "<missing id>"

    for field_name in BASE_REQUIRED_FIELDS:
        if field_name not in row:
            errors.append(RowError(line_no, row_id, f"missing required field '{field_name}'"))

    for field_name in TEXT_FIELDS:
        if field_name not in row:
            continue
        value = row[field_name]
        if not isinstance(value, str):
            errors.append(RowError(line_no, row_id, f"'{field_name}' must be a string"))
        elif not value.strip():
            errors.append(RowError(line_no, row_id, f"'{field_name}' must be non-blank"))

    if isinstance(raw_id, str) and raw_id.strip() and not ID_RE.fullmatch(raw_id):
        errors.append(RowError(line_no, row_id, "'id' must use syntax evt-NNNN"))

    timestamp = row.get("timestamp_utc")
    if isinstance(timestamp, str) and timestamp.strip():
        if not UTC_TIMESTAMP_RE.fullmatch(timestamp):
            errors.append(
                RowError(line_no, row_id, "'timestamp_utc' must be an ISO 8601 UTC timestamp ending in Z")
            )
        else:
            try:
                datetime.fromisoformat(timestamp[:-1] + "+00:00")
            except ValueError:
                errors.append(RowError(line_no, row_id, "'timestamp_utc' is not a valid date/time"))

    commit_sha = row.get("commit_sha")
    if isinstance(commit_sha, str) and commit_sha.strip() and not HEX40_RE.fullmatch(commit_sha):
        errors.append(RowError(line_no, row_id, "'commit_sha' must be exactly 40 hexadecimal characters"))

    result = row.get("result")
    if isinstance(result, str) and result.strip() and result not in VALID_RESULTS:
        errors.append(RowError(line_no, row_id, f"invalid result '{result}'"))

    category = row.get("category")
    if isinstance(category, str) and category.strip() and category not in VALID_CATEGORIES:
        errors.append(RowError(line_no, row_id, f"invalid category '{category}'"))

    if "evidence_refs" in row:
        errors.extend(_array_errors(row, "evidence_refs", line_no, row_id))

    if "example" in row and not isinstance(row["example"], bool):
        errors.append(RowError(line_no, row_id, "'example' must be a boolean"))

    seed_example = is_seed_example(row, line_no)
    if is_example(row) and not seed_example:
        errors.append(
            RowError(
                line_no,
                row_id,
                "only the historical first-row evt-0000 may set 'example' to true",
            )
        )

    if not seed_example:
        for field_name in ("tree_sha", "environment", "runtime_overrides"):
            if field_name not in row:
                errors.append(
                    RowError(
                        line_no,
                        row_id,
                        f"missing required identity/environment field '{field_name}'",
                    )
                )

    environment = row.get("environment")
    if "environment" in row:
        if not isinstance(environment, str):
            errors.append(RowError(line_no, row_id, "'environment' must be a string"))
        elif not environment.strip():
            errors.append(RowError(line_no, row_id, "'environment' must be non-blank"))

    if "runtime_overrides" in row:
        errors.extend(_array_errors(row, "runtime_overrides", line_no, row_id))

    tree_sha = row.get("tree_sha")
    if "tree_sha" in row:
        if not isinstance(tree_sha, str):
            errors.append(RowError(line_no, row_id, "'tree_sha' must be a string"))
        elif not tree_sha.strip():
            errors.append(RowError(line_no, row_id, "'tree_sha' must be non-blank"))
        elif not HEX40_RE.fullmatch(tree_sha):
            dirty_match = DIRTY_TREE_RE.fullmatch(tree_sha)
            if dirty_match is None:
                errors.append(
                    RowError(
                        line_no,
                        row_id,
                        "'tree_sha' must be 40 hexadecimal characters or '<commit_sha>+dirty:<64-hex-sha256>'",
                    )
                )
            elif isinstance(commit_sha, str) and dirty_match.group(1).lower() != commit_sha.lower():
                errors.append(
                    RowError(line_no, row_id, "dirty 'tree_sha' must begin with the row's 'commit_sha'")
                )

    if result == "pass":
        if is_example(row):
            errors.append(RowError(line_no, row_id, "an example row must never have result 'pass'"))
        elif not isinstance(row.get("evidence_refs"), list) or not row["evidence_refs"]:
            errors.append(RowError(line_no, row_id, "'pass' result requires a non-empty 'evidence_refs'"))

    return errors


def load_rows(path: Path) -> tuple[list[tuple[int, dict]], list[RowError]]:
    rows: list[tuple[int, dict]] = []
    errors: list[RowError] = []
    try:
        with path.open("r", encoding="utf-8") as handle:
            for line_no, line in enumerate(handle, start=1):
                if not line.strip():
                    continue
                try:
                    value = json.loads(line)
                except json.JSONDecodeError as error:
                    errors.append(
                        RowError(line_no, "<invalid json>", f"malformed JSON: {error.msg}")
                    )
                    continue
                if not isinstance(value, dict):
                    errors.append(RowError(line_no, "<non-object>", "JSON line must be an object"))
                    continue
                rows.append((line_no, value))
    except (OSError, UnicodeError) as error:
        errors.append(RowError(0, "<file>", f"cannot read '{path}': {error}"))
    return rows, errors


def validate_file(path: Path) -> ValidationReport:
    report = ValidationReport()
    rows, load_errors = load_rows(path)
    report.errors.extend(load_errors)
    seen_numbers: set[int] = set()
    previous_number: int | None = None

    for line_no, row in rows:
        row_errors = validate_row(row, line_no)
        raw_id = row.get("id")
        id_match = ID_RE.fullmatch(raw_id) if isinstance(raw_id, str) else None
        if id_match:
            number = int(id_match.group(1))
            if number in seen_numbers:
                row_errors.append(RowError(line_no, raw_id, "duplicate event id"))
            if previous_number is not None and number <= previous_number:
                row_errors.append(
                    RowError(line_no, raw_id, "event id must be numerically strictly increasing")
                )
            seen_numbers.add(number)
            previous_number = number

        report.errors.extend(row_errors)
        if counts_as_pass(row) and not row_errors:
            report.real_pass_count += 1
        if is_example(row) and row.get("result") == "pass":
            report.example_pass_attempts += 1

    return report


def main(argv: list[str]) -> int:
    args = [argument for argument in argv if argument != "--summary"]
    summary = "--summary" in argv
    path = Path(args[0]) if args else DEFAULT_PATH

    report = validate_file(path)
    for error in report.errors:
        print(error, file=sys.stderr)
    if summary:
        print(f"real (non-example) pass count: {report.real_pass_count}")
    return 0 if report.ok else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
