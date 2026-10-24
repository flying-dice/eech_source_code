#!/usr/bin/env python3
"""Exercise the retained host-identity hash verifier through its CLI boundary."""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path


HERE = Path(__file__).resolve().parent
VERIFIER = "verify-hashes.py"
README = "README.md"
SECTION = "## Retained file hashes"
BOOT_ROW_PREFIX = "| [boot.summary.txt](boot.summary.txt) |"


def run_verifier(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "-I", str(root / VERIFIER)],
        capture_output=True,
        check=False,
        text=True,
    )


def rewrite_readme(root: Path, transform: Callable[[str], str]) -> None:
    path = root / README
    path.write_text(transform(path.read_text(encoding="utf-8")), encoding="utf-8")


def duplicate_boot_row(text: str) -> str:
    row = next(line for line in text.splitlines() if line.startswith(BOOT_ROW_PREFIX))
    return text.replace(row, row + "\n" + row, 1)


def is_distinct_hash_mismatch(stderr: str) -> bool:
    match = re.fullmatch(
        r"retained artifact hash mismatch: accepted-lifecycle\.summary\.txt: "
        r"([0-9a-f]{64}), expected ([0-9a-f]{64})",
        stderr,
    )
    return match is not None and match.group(1) != match.group(2)


def expect_failure(
    pristine: Path,
    name: str,
    mutate: Callable[[Path], None],
    diagnostic: Callable[[str], bool],
) -> None:
    case = pristine.parent / name
    shutil.copytree(pristine, case)
    mutate(case)
    result = run_verifier(case)
    stderr = result.stderr.strip()
    if result.returncode == 0 or not diagnostic(stderr):
        raise AssertionError(
            f"{name}: expected diagnosed failure, got exit={result.returncode}, "
            f"stdout={result.stdout!r}, stderr={result.stderr!r}"
        )
    print(f"PASS: {name}")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="eech-identity-hashes-") as directory:
        temp = Path(directory)
        pristine = temp / "pristine"
        shutil.copytree(HERE, pristine)

        baseline = run_verifier(pristine)
        if baseline.returncode != 0 or baseline.stdout.strip() != (
            "PASS: 11 retained host-identity artifact hashes"
        ):
            raise AssertionError(
                f"baseline: exit={baseline.returncode}, stdout={baseline.stdout!r}, "
                f"stderr={baseline.stderr!r}"
            )
        print("PASS: baseline")

        expect_failure(
            pristine,
            "malformed-duplicate",
            lambda root: rewrite_readme(
                root,
                lambda text: text.replace(
                    SECTION,
                    SECTION + "\n\n| [boot.summary.txt](boot.summary.txt) | `bogus` |",
                    1,
                ),
            ),
            lambda stderr: stderr.startswith("malformed retained-hash row: ")
            and "boot.summary.txt" in stderr,
        )
        expect_failure(
            pristine,
            "duplicate",
            lambda root: rewrite_readme(root, duplicate_boot_row),
            lambda stderr: stderr == "duplicate retained-hash row: boot.summary.txt",
        )
        expect_failure(
            pristine,
            "missing-artifact",
            lambda root: (root / "BUILD-INFO.txt").unlink(),
            lambda stderr: stderr == "missing retained artifact: BUILD-INFO.txt",
        )
        expect_failure(
            pristine,
            "artifact-mismatch",
            lambda root: (root / "accepted-lifecycle.summary.txt").write_bytes(
                (root / "accepted-lifecycle.summary.txt").read_bytes() + b"unexpected"
            ),
            is_distinct_hash_mismatch,
        )
        expect_failure(
            pristine,
            "path-mismatch",
            lambda root: rewrite_readme(
                root,
                lambda text: text.replace(
                    "[boot.summary.txt](boot.summary.txt)",
                    "[boot.summary.txt](../boot.summary.txt)",
                    1,
                ),
            ),
            lambda stderr: stderr
            == "invalid retained-hash path: 'boot.summary.txt' -> '../boot.summary.txt'",
        )

    print("PASS: retained host-identity hash verifier self-test")
    return 0


if __name__ == "__main__":
    sys.exit(main())
