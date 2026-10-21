"""Exercise the slice 1a proof against the named base in an isolated Git tree."""

from pathlib import Path
import subprocess
import sys
import tempfile


BASE = "163e9158ffc5d979617e66d6c5af4def780c8227"
ROOT = Path(__file__).resolve().parents[3]
CHECK = Path(__file__).with_name("expand-check.py")
WORLD = "aphavoc/source/entity/mobile"
GROUP = "aphavoc/source/entity/special/group/gp_loss.c"
ROUTED = f"{WORLD}/vehicle/routed/rv_dstry.c"
ANTI_AIR = f"{WORLD}/vehicle/anti_air/aa_dstry.c"
PERSON = f"{WORLD}/vehicle/person/ps_dstry.c"
FILES = (
    f"{WORLD}/aircraft/helicop/hc_dstry.c",
    f"{WORLD}/aircraft/fixwing/fw_dstry.c",
    ROUTED,
    ANTI_AIR,
    PERSON,
    f"{WORLD}/vehicle/ship/sh_dstry.c",
)

# Each mutation corresponds to one independently retained manual case.
MUTATIONS = (
    (
        "M1 ordering",
        ROUTED,
        "unlink_member_from_group (en);\n\n\tdelete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);\n\n\tunlink_member_from_guide (en);",
        "unlink_member_from_guide (en);\n\n\tdelete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);\n\n\tunlink_member_from_group (en);",
        "DIFF rv_dstry.c kill_local",
    ),
    (
        "M2 per-type omission",
        ANTI_AIR,
        "increment_group_losses (group);",
        "increment_group_losses (group);\n\tincrement_group_primary_task_losses (group);",
        "DIFF aa_dstry.c kill_local",
    ),
    (
        "M3 helper body",
        GROUP,
        "TASK_TERMINATED_OBJECTIVE_MESSAGE);",
        "TASK_TERMINATED_GROUP_DESTROYED_MESSAGE);",
        "DIFF gp_loss.c notify_member_task_dependents_completed",
    ),
    (
        "M4 helper parameters",
        GROUP,
        "void notify_group_member_killed (entity *group, entity *member)",
        "void notify_group_member_killed (entity *member, entity *group)",
        "DIFF gp_loss.c notify_group_member_killed",
    ),
    (
        "M5 file scope",
        PERSON,
        "#define DEBUG_MODULE 0",
        "#define DEBUG_MODULE 0\n#define LIST_TYPE_MEMBER LIST_TYPE_FOLLOWER",
        "DIFF ps_dstry.c outside its functions, or its function list",
    ),
    (
        "M6 debug configuration",
        PERSON,
        "#define DEBUG_MODULE 0",
        "#define DEBUG_MODULE 1",
        "DEBUG_MODULE is not 0",
    ),
    (
        "M7 unlisted helper",
        GROUP,
        "void unlink_member_from_takeoff_queue (entity *member)",
        "static void unlisted_helper (entity *member)\n{\n\t(void) member;\n}\n\nvoid unlink_member_from_takeoff_queue (entity *member)",
        "FAIL gp_loss.c helpers ['unlisted_helper']",
    ),
)


def git(*args: str, cwd: Path = ROOT) -> bytes:
    return subprocess.check_output(("git", *args), cwd=cwd)


def proof(base: str, cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        (sys.executable, "-I", str(CHECK), base),
        cwd=cwd,
        capture_output=True,
        text=True,
        check=False,
    )


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="eech-loss-proof-") as directory:
        isolated = Path(directory)
        git("init", "--quiet", cwd=isolated)
        for name in FILES:
            target = isolated / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(git("show", f"{BASE}:{name}"))
        git("add", "--", *FILES, cwd=isolated)
        git(
            "-c", "user.name=Proof Selftest",
            "-c", "user.email=proof@example.invalid",
            "commit", "--quiet", "-m", "named base sources",
            cwd=isolated,
        )
        base = git("rev-parse", "HEAD", cwd=isolated).decode().strip()
        for name in (*FILES, GROUP):
            target = isolated / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((ROOT / name).read_bytes())

        clean = proof(base, isolated)
        if clean.returncode or not clean.stdout.rstrip().endswith("PASS"):
            raise AssertionError(f"unmodified candidate failed:\n{clean.stdout}{clean.stderr}")
        print("Unmodified candidate: PASS")

        for label, name, before, after, expected in MUTATIONS:
            target = isolated / name
            original = target.read_bytes()
            current = original.decode("latin-1").replace("\r\n", "\n")
            if current.count(before) != 1:
                raise AssertionError(f"{label}: expected one mutation site, got {current.count(before)}")
            try:
                target.write_bytes(current.replace(before, after, 1).encode("latin-1"))
                result = proof(base, isolated)
                if result.returncode == 0 or expected not in result.stdout:
                    raise AssertionError(
                        f"{label}: expected {expected!r}, got exit {result.returncode}:\n"
                        f"{result.stdout}{result.stderr}"
                    )
                print(f"{label}: detected")
            finally:
                target.write_bytes(original)
            if target.read_bytes() != original:
                raise AssertionError(f"{label}: isolated source was not restored")

        restored = proof(base, isolated)
        if restored.returncode or not restored.stdout.rstrip().endswith("PASS"):
            raise AssertionError(f"restored candidate failed:\n{restored.stdout}{restored.stderr}")
        print("PASS: restored candidate and seven independent mutations")


if __name__ == "__main__":
    main()
