"""Exercise the slice 1a proof against the named base in an isolated Git tree."""

from pathlib import Path
from typing import NamedTuple
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
DEBUG_ALTERNATIVE = "#if DEBUG_MODULE\n#else\n#endif"


class Mutation(NamedTuple):
    """One edit at a single site, and the one complete output line that must report it."""

    label: str
    name: str
    before: str
    after: str
    expected: str
    stream: str = "stdout"


# M1 to M7 correspond to the seven independently retained manual cases. M8, M9 and M11 each reach a
# guard that no other case reaches. M10 covers the missing-helper side of M7's helper-set guard. M12
# reaches M5's shared function-list/file-scope line through an added function, and does not separate
# the two conditions. M8 is the only guard that exits through SystemExit, so its message is the whole
# of stderr; every other diagnostic is printed to stdout, with nothing on stderr.
MUTATIONS = tuple(Mutation(*case) for case in (
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
        f"FAIL {PERSON}: DEBUG_MODULE is not 0, so removing its blocks is not equivalent",
    ),
    (
        "M7 unlisted helper",
        GROUP,
        "void unlink_member_from_takeoff_queue (entity *member)",
        "static void unlisted_helper (entity *member)\n{\n\t(void) member;\n}\n\nvoid unlink_member_from_takeoff_queue (entity *member)",
        "FAIL gp_loss.c helpers ['unlisted_helper']",
    ),
    (
        "M8 debug alternative branch",
        PERSON,
        "#define DEBUG_MODULE 0",
        f"#define DEBUG_MODULE 0\n{DEBUG_ALTERNATIVE}",
        f"a DEBUG_MODULE block has an alternative branch: {DEBUG_ALTERNATIVE!r}",
        "stderr",
    ),
    (
        "M9 conditional compilation in helpers",
        GROUP,
        "void unlink_member_from_takeoff_queue (entity *member)",
        "#if 1\n#endif\n\nvoid unlink_member_from_takeoff_queue (entity *member)",
        "FAIL gp_loss.c has conditional compilation, which the comparison would not see",
    ),
    (
        "M10 missing helper",
        GROUP,
        "void unlink_member_from_takeoff_queue (entity *member)\n{\n"
        "\tdelete_local_entity_from_parents_child_list (member, LIST_TYPE_TAKEOFF_QUEUE);\n}",
        "",
        "FAIL gp_loss.c helpers ['unlink_member_from_takeoff_queue']",
    ),
    (
        "M11 helper parameter count",
        GROUP,
        "void unlink_member_task_dependents (entity *member)",
        "void unlink_member_task_dependents (entity *member, entity *unused)",
        "DIFF gp_loss.c unlink_member_task_dependents",
    ),
    (
        "M12 function list",
        PERSON,
        "static void destroy_remote (entity *en)",
        "static void added_destroy (entity *en)\n{\n}\n\nstatic void destroy_remote (entity *en)",
        "DIFF ps_dstry.c outside its functions, or its function list",
    ),
))


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


def line_present(expected: str, output: str) -> bool:
    """True only when expected is a whole line of output, not the start of a longer one."""
    return expected in output.splitlines()


def main() -> None:
    longer = "DIFF aa_dstry.c kill_local_restored_anti_aircraft_entity"
    if line_present("DIFF aa_dstry.c kill_local", longer + "\n") or not line_present(longer, longer + "\n"):
        raise AssertionError("diagnostic matching accepts the start of a longer line or rejects a whole line")
    print("Diagnostic matching: whole lines only")

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
        candidate = {name: (ROOT / name).read_bytes() for name in (*FILES, GROUP)}
        for name, content in candidate.items():
            target = isolated / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(content)

        clean = proof(base, isolated)
        if clean.returncode or not clean.stdout.rstrip().endswith("PASS"):
            raise AssertionError(f"unmodified candidate failed:\n{clean.stdout}{clean.stderr}")
        print("Unmodified candidate: PASS")

        for label, name, before, after, expected, stream in MUTATIONS:
            target = isolated / name
            original = target.read_bytes()
            current = original.decode("latin-1").replace("\r\n", "\n")
            if current.count(before) != 1:
                raise AssertionError(f"{label}: expected one mutation site, got {current.count(before)}")
            try:
                target.write_bytes(current.replace(before, after, 1).encode("latin-1"))
                result = proof(base, isolated)
                diagnostic_only = (
                    result.stderr.splitlines() == [expected] if stream == "stderr" else result.stderr == ""
                )
                if (
                    result.returncode != 1
                    or not line_present(expected, getattr(result, stream))
                    or not diagnostic_only
                    or "PASS" in result.stdout.splitlines()
                ):
                    raise AssertionError(
                        f"{label}: expected the line {expected!r} on {stream}, got exit {result.returncode}:\n"
                        f"{result.stdout}{result.stderr}"
                    )
                print(f"{label}: detected")
            finally:
                target.write_bytes(original)
            if target.read_bytes() != candidate[name]:
                raise AssertionError(f"{label}: isolated source was not restored")

        restored = proof(base, isolated)
        if restored.returncode or not restored.stdout.rstrip().endswith("PASS"):
            raise AssertionError(f"restored candidate failed:\n{restored.stdout}{restored.stderr}")
        print(f"PASS: restored candidate and {len(MUTATIONS)} independent mutations")


if __name__ == "__main__":
    main()
