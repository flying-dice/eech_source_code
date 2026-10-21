#!/usr/bin/env python3
"""Exercise the retained SUPPLY-arrival verifier through its CLI boundary."""

from __future__ import annotations

import json
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
BUILD_ROW_PREFIX = "| [M4-BUILD-INFO.txt](M4-BUILD-INFO.txt) |"


def run_verifier(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "-I", str(root / VERIFIER)],
        capture_output=True,
        check=False,
        text=True,
    )


def rewrite(root: Path, name: str, transform: Callable[[str], str]) -> None:
    path = root / name
    path.write_text(transform(path.read_text(encoding="utf-8")), encoding="utf-8")


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
    with tempfile.TemporaryDirectory(prefix="eech-arrival-hashes-") as directory:
        temp = Path(directory)
        pristine = temp / "pristine"
        shutil.copytree(HERE, pristine)
        baseline = run_verifier(pristine)
        if baseline.returncode != 0 or baseline.stdout.strip() != (
            "PASS: 15 retained SUPPLY-arrival artifacts and identities"
        ):
            raise AssertionError(
                f"baseline: exit={baseline.returncode}, stdout={baseline.stdout!r}, "
                f"stderr={baseline.stderr!r}"
            )
        print("PASS: baseline")

        expect_failure(
            pristine,
            "malformed-duplicate",
            lambda root: rewrite(
                root,
                README,
                lambda text: text.replace(
                    SECTION,
                    SECTION + "\n\n| [M4-BUILD-INFO.txt](M4-BUILD-INFO.txt) | `bogus` |",
                    1,
                ),
            ),
            lambda stderr: stderr.startswith("malformed retained-hash row: "),
        )
        expect_failure(
            pristine,
            "duplicate",
            lambda root: rewrite(
                root,
                README,
                lambda text: text.replace(
                    next(line for line in text.splitlines() if line.startswith(BUILD_ROW_PREFIX)),
                    "\n".join(
                        [line for line in text.splitlines() if line.startswith(BUILD_ROW_PREFIX)]
                        * 2
                    ),
                    1,
                ),
            ),
            lambda stderr: stderr == "duplicate retained-hash row: M4-BUILD-INFO.txt",
        )
        expect_failure(
            pristine,
            "missing-artifact",
            lambda root: (root / "M4-BUILD-INFO.txt").unlink(),
            lambda stderr: stderr == "missing retained artifact: M4-BUILD-INFO.txt",
        )
        expect_failure(
            pristine,
            "artifact-mismatch",
            lambda root: (root / "causal-reference.out").write_bytes(
                (root / "causal-reference.out").read_bytes() + b"unexpected"
            ),
            lambda stderr: stderr.startswith(
                "retained artifact hash mismatch: causal-reference.out:"
            ),
        )
        expect_failure(
            pristine,
            "path-mismatch",
            lambda root: rewrite(
                root,
                README,
                lambda text: text.replace(
                    next(
                        line
                        for line in text.splitlines()
                        if line.startswith(BUILD_ROW_PREFIX)
                    ),
                    next(
                        line
                        for line in text.splitlines()
                        if line.startswith(BUILD_ROW_PREFIX)
                    ).replace(
                        "[M4-BUILD-INFO.txt](M4-BUILD-INFO.txt)",
                        "[M4-BUILD-INFO.txt](../M4-BUILD-INFO.txt)",
                    ),
                    1,
                ),
            ),
            lambda stderr: stderr
            == "invalid retained-hash path: 'M4-BUILD-INFO.txt' -> '../M4-BUILD-INFO.txt'",
        )

        def truncate_host(root: Path) -> None:
            path = root / "causal.summary.json"
            data = json.loads(path.read_text(encoding="utf-8"))
            data["eech_world_sha256"] = data["eech_world_sha256"][:-1]
            path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
            readme = (root / README).read_text(encoding="utf-8")
            digest = __import__("hashlib").sha256(path.read_bytes()).hexdigest()
            readme = re.sub(
                r"(\| \[causal\.summary\.json]\(causal\.summary\.json\) \| `)[0-9a-f]{64}(` \|)",
                rf"\g<1>{digest}\2",
                readme,
            )
            (root / README).write_text(readme, encoding="utf-8")

        expect_failure(
            pristine,
            "truncated-host-digest",
            truncate_host,
            lambda stderr: stderr.startswith("invalid SHA-256 for causal host:"),
        )

    print("PASS: retained SUPPLY-arrival verifier self-test")
    return 0


if __name__ == "__main__":
    sys.exit(main())
