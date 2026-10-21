#!/usr/bin/env python3
# Shows that tools/world-boundary-check.py detects what it claims to. Every case runs the real CLI.
#
# SYNTHETIC cases build a tiny PE/COFF image and a tiny labelled source tree. They prove the parser, the fail-closed
# input handling and the stub detection, nothing about EECH. The one green case is labelled SYNTHETIC-GREEN by the tool
# itself and cannot be produced for a revision: a green here says the parser can say yes, not that any DLL is clean.
#
# RETAINED cases run the real retained eech_dc.dll with its BUILD-INFO (and the repository, which must contain the
# BUILD-INFO commit). They are skipped, loudly, when the two paths are not given.
#
# Usage: tools/world-boundary-check-selftest.py [<eech_dc.dll> <BUILD-INFO.txt>]
import hashlib, importlib.util, os, shutil, struct, subprocess, sys, tempfile
from pathlib import Path

here = Path(__file__).resolve().parent
tool = here / "world-boundary-check.py"
SYS = "eech-campaign/crates/eech-engine-sys/"
COMMIT = "a" * 40

# ---- the synthetic source tree: classes C (campaign), W and E (physical), F (shared), H (host layer)
STEP = ("int %s(int *unit, int heading, int speed)\n{\n  int step = unit[0] + speed * heading;\n"
        "  unit[1] = step * 3 + heading - speed;\n  unit[2] = unit[1] ^ step;\n  return unit[2] + 17;\n}\n")
SOURCES = {
    "aphavoc/source/ai/campaign.c": "int camp_a(int x)\n{\n  return x + 1;\n}\nint camp_b(int x)\n{\n  return x * 2;\n}\n"
                                    "int camp_c(int x)\n{\n  return x - 3;\n}\n",
    "aphavoc/source/entity/mobile/mobile.c": STEP % "mobile_step",
    "modules/3d/terrain.c": "int terrain_height(int x, int z)\n{\n  return x * z + 5;\n}\n",
    "modules/misc/util.c": "int util_f(int x)\n{\n  return x;\n}\n",
    SYS + "csrc/boundary.c": "int luaopen_eech_dc(void *L)\n{\n  return 0;\n}\n",
}
UNITS = ["aphavoc/source/ai/campaign.c", "aphavoc/source/entity/mobile/mobile.c", "modules/3d/terrain.c",
         "modules/misc/util.c"]
CAMPAIGN = ["camp_a", "camp_b", "camp_c"]
BASE = CAMPAIGN + ["util_f", "luaopen_eech_dc"]
OBJ_NO_WORLD = ["aphavoc/source/ai/campaign.o", "modules/misc/util.o"]
OBJ_WORLD = ["aphavoc/source/entity/mobile/mobile.o", "modules/3d/terrain.o"]


def write_tree(root, edit=None):
    root = Path(root)
    files = dict(SOURCES)
    units = list(UNITS)
    if edit:
        edit(files, units)
    for p, t in files.items():
        (root / p).parent.mkdir(parents=True, exist_ok=True)
        (root / p).write_text(t)
    (root / SYS).mkdir(parents=True, exist_ok=True)
    (root / (SYS + "sources.txt")).write_text("# synthetic\n" + "\n".join(units) + "\n")
    return root


# ---- a minimal PE32+ with a COFF symbol table and an optional import directory
def make_pe(symbols, imports=None, symptr_nsym=None, drop_symbols=False):
    """symbols [(name, kind)] with kind 'def', 'static' or 'undef'; imports {dll: [names]}"""
    sec_va, sec_raw = 0x1000, 0x200
    idata = bytearray()
    irva = isize = 0
    if imports:
        n = len(imports)
        desc = 20 * (n + 1)
        ilts, tail = [], bytearray()
        # layout: descriptors, then per dll: ILT, name, hint/name entries
        pos = desc
        body = bytearray()
        descriptors = bytearray()
        for dll, names in imports.items():
            ilt_off = pos
            hn = bytearray()
            hn_off = ilt_off + 8 * (len(names) + 1)
            name_off = hn_off
            dll_name = dll.encode() + b"\0"
            hn_start = name_off + len(dll_name)
            ilt = bytearray()
            cur = hn_start
            entries = bytearray()
            for nm in names:
                ilt += struct.pack("<Q", sec_va + cur)
                ent = b"\0\0" + nm.encode() + b"\0"
                entries += ent
                cur += len(ent)
            ilt += struct.pack("<Q", 0)
            body += ilt + dll_name + entries
            descriptors += struct.pack("<IIIII", sec_va + ilt_off, 0, 0, sec_va + name_off, sec_va + ilt_off)
            pos = ilt_off + len(ilt) + len(dll_name) + len(entries)
        descriptors += b"\0" * 20
        idata = bytearray(descriptors) + body
        irva, isize = sec_va, len(idata)
    raw = bytes(idata).ljust(0x200, b"\0")
    symptr = sec_raw + len(raw)
    strtab = bytearray(4)
    table = bytearray()
    for name, kind in symbols:
        off = len(strtab)
        strtab += name.encode() + b"\0"
        sec, scl = {"def": (1, 2), "static": (1, 3), "undef": (0, 2)}[kind]
        table += struct.pack("<8sIhHBB", b"\0\0\0\0" + struct.pack("<I", off), 0, sec, 0, scl, 0)
    struct.pack_into("<I", strtab, 0, len(strtab))
    nsym = len(symbols)
    if drop_symbols:
        symptr = nsym = 0
        table, strtab = bytearray(), bytearray()
    if symptr_nsym is not None:
        nsym = symptr_nsym
    dos = bytearray(0x40)
    dos[0:2] = b"MZ"
    struct.pack_into("<I", dos, 0x3C, 0x40)
    coff = b"PE\0\0" + struct.pack("<HHIIIHH", 0x8664, 1, 0, symptr, nsym, 240, 0x2022)
    opt = bytearray(240)
    struct.pack_into("<H", opt, 0, 0x20B)
    struct.pack_into("<II", opt, 112 + 8, irva, isize)
    section = struct.pack("<8sIIIIIIHHI", b".idata", max(len(idata), 1), sec_va, len(raw), sec_raw, 0, 0, 0, 0, 0x40000040)
    head = (bytes(dos) + coff + bytes(opt) + section).ljust(sec_raw, b"\0")
    return head + raw + bytes(table) + bytes(strtab)


def syms(names, static=()):
    return [(n, "def") for n in names] + [(n, "static") for n in static]


def build_info(path, dll, commit=COMMIT, dirty=0, sha=None):
    sha = sha or hashlib.sha256(Path(dll).read_bytes()).hexdigest()
    Path(path).write_text("commit: %s\nuncommitted changes in the build inputs: %d files\nsha256:\n  %s  eech_dc.dll\n"
                          % (commit, dirty, sha))


def manifest(path, dll, objects, commit=COMMIT, sha=None, prefix="build/"):
    sha = sha or hashlib.sha256(Path(dll).read_bytes()).hexdigest()
    Path(path).write_text("dll-sha256: %s\nsource-commit: %s\n" % (sha, commit)
                          + "".join("object: %s%s\n" % (prefix, o) for o in objects))


def run(*args):
    r = subprocess.run([sys.executable, str(tool), *map(str, args)], capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


results = []


def case(label, name, want_rc, want_text, rc_out):
    rc, out = rc_out
    ok = rc == want_rc and want_text in out
    results.append(ok)
    first = out.strip().splitlines()[0] if out.strip() else ""
    print("%-9s %-34s exit %d (want %d) %s | %s" % (label, name, rc, want_rc, "ok" if ok else "NOT AS EXPECTED", first[:100]))
    if not ok:
        print("          want text %r; output:\n%s" % (want_text, out))


def synthetic(tmp):
    base = Path(tmp)

    def fixture(name, symbols=None, edit=None, objects=OBJ_NO_WORLD, manifest_edit=None, info_edit=None, pe_args=None):
        d = base / name
        d.mkdir()
        tree = write_tree(d / "src", edit)
        dll = d / "eech_dc.dll"
        dll.write_bytes(make_pe(symbols or syms(BASE), **(pe_args or {})))
        info = d / "BUILD-INFO.txt"
        build_info(info, dll, **(info_edit or {}))
        man = None
        if objects is not None:
            man = d / "objects.txt"
            manifest(man, dll, objects, **(manifest_edit or {}))
        return d, tree, dll, info, man

    def go(d, tree, dll, info, man, *extra):
        args = ["check", "--dll", dll, "--build-info", info, "--synthetic-source-dir", tree, "--root", d,
                "--report", d / "report.json"]
        if man:
            args += ["--objects", man]
        return run(*args, *extra)

    # the only green: parser logic, labelled by the tool as a fixture
    f = fixture("green")
    case("SYNTHETIC", "green-fixture-only", 0, "SYNTHETIC FIXTURE, NOT ACCEPTANCE EVIDENCE: SYNTHETIC-GREEN", go(*f))
    # a clean DLL with no object manifest is never green
    f = fixture("no-manifest", objects=None)
    case("SYNTHETIC", "clean-dll-no-object-manifest", 2, "INCOMPLETE", go(*f))
    # known physical definitions fail, with or without a manifest, in a doubled table, or with a clean-looking manifest
    f = fixture("physical", syms(BASE + ["mobile_step"]))
    case("SYNTHETIC", "physical-function-defined", 1, "FAIL", go(*f))
    f = fixture("physical-nomanifest", syms(BASE + ["terrain_height"]), objects=None)
    case("SYNTHETIC", "physical-no-manifest-still-fail", 1, "FAIL", go(*f))
    f = fixture("doubled", syms(BASE + ["mobile_step"]) * 2)
    case("SYNTHETIC", "doubled-symbol-table", 1, "FAIL", go(*f))

    def drop_world(files, units):  # the world file is still in the tree but no longer in sources.txt
        units.remove("aphavoc/source/entity/mobile/mobile.c")
    f = fixture("dropped", syms(BASE + ["mobile_step"]), edit=drop_world)
    case("SYNTHETIC", "world-file-dropped-from-sources", 1, "FAIL", go(*f))
    # a renamed stand-in for a physical function inside the campaign unit (its original is not defined)
    def renamed(files, units):
        files["aphavoc/source/ai/campaign.c"] += STEP % "camp_pseudo_step"
    f = fixture("renamed", syms(BASE + ["camp_pseudo_step"]), edit=renamed)
    case("SYNTHETIC", "renamed-in-dll-physical-stub", 1, "copies of a physical function body", go(*f))
    # the same stub when the world original is dropped from the build: it stays a stub
    def renamed_and_dropped(files, units):
        renamed(files, units); drop_world(files, units)
    f = fixture("renamed2", syms(BASE + ["camp_pseudo_step"]), edit=renamed_and_dropped)
    case("SYNTHETIC", "renamed-stub-world-file-dropped", 1, "FAIL", go(*f))
    # world objects in the link map fail even when no physical symbol is defined
    f = fixture("world-objs", objects=OBJ_NO_WORLD + OBJ_WORLD)
    case("SYNTHETIC", "world-objects-in-link-map", 1, "world/environment objects are linked", go(*f))
    f = fixture("world-objs-doubled", objects=(OBJ_NO_WORLD + OBJ_WORLD) * 2)
    case("SYNTHETIC", "doubled-object-manifest", 1, "FAIL", go(*f))

    # fail closed on the DLL
    f = fixture("blank", pe_args={"drop_symbols": True})
    case("SYNTHETIC", "blank-symbol-table", 2, "stripped", go(*f))
    f = fixture("truncated")
    d, tree, dll, info, man = f
    data = dll.read_bytes()
    dll.write_bytes(data[:len(data) - 120])
    build_info(info, dll)
    manifest(man, dll, OBJ_NO_WORLD)
    case("SYNTHETIC", "truncated-symbol-table", 2, "INCOMPLETE", go(*f))
    f = fixture("overclaims", pe_args={"symptr_nsym": 5000})
    case("SYNTHETIC", "symbol-count-beyond-file", 2, "INCOMPLETE", go(*f))
    f = fixture("notpe")
    f[2].write_bytes(b"not a PE file at all" * 20)
    build_info(f[3], f[2]); manifest(f[4], f[2], OBJ_NO_WORLD)
    case("SYNTHETIC", "non-pe-input", 2, "not a PE", go(*f))
    f = fixture("nolua", syms(CAMPAIGN + ["util_f"]))
    case("SYNTHETIC", "no-lua-entry-point", 2, "luaopen_eech_dc", go(*f))
    f = fixture("foreign", syms(["foo", "bar", "luaopen_eech_dc"]))
    case("SYNTHETIC", "symbols-not-of-this-source", 2, "do not correspond", go(*f))
    f = fixture("imports", pe_args={"imports": {"kernel32.dll": ["Sleep", "GetTickCount"], "lua.dll": ["lua_pushnumber"]}})
    case("SYNTHETIC", "platform-imports-recorded", 0, "kernel32.dll 2, lua.dll 1", go(*f))

    # fail closed on BUILD-INFO
    f = fixture("badsha", info_edit={"sha": "0" * 64})
    case("SYNTHETIC", "dll-sha-mismatch", 2, "is not the BUILD-INFO", go(*f))
    f = fixture("dirty", info_edit={"dirty": 3})
    case("SYNTHETIC", "dirty-build-info", 2, "uncommitted", go(*f))
    f = fixture("noinfo")
    f[3].write_text("commit: nothing\n")
    case("SYNTHETIC", "malformed-build-info", 2, "BUILD-INFO", go(*f))
    f = fixture("wrongcommit")
    case("SYNTHETIC", "wrong-expected-commit", 2, "not the expected", go(*f, "--expect-commit", "b" * 40))
    d, tree, dll, info, man = fixture("unresolvable", info_edit={"commit": "f" * 40})
    case("SYNTHETIC", "unresolvable-commit-no-fixture", 2, "git ls-tree", run(
        "check", "--dll", dll, "--build-info", info, "--root", here.parents[1], "--objects", man))

    # fail closed on the object manifest
    f = fixture("stale-sha", manifest_edit={"sha": "1" * 64})
    case("SYNTHETIC", "stale-manifest-other-dll", 2, "stale", go(*f))
    f = fixture("stale-commit", manifest_edit={"commit": "c" * 40})
    case("SYNTHETIC", "stale-manifest-other-commit", 2, "another source commit", go(*f))
    f = fixture("partial", objects=OBJ_NO_WORLD[1:])
    case("SYNTHETIC", "incomplete-manifest-missing-campaign", 2, "incomplete", go(*f))
    f = fixture("unmapped", objects=OBJ_NO_WORLD + ["mystery/other.o"])
    case("SYNTHETIC", "manifest-object-maps-to-no-unit", 2, "incomplete", go(*f))
    f = fixture("emptyman", objects=[])
    case("SYNTHETIC", "empty-manifest", 2, "lists no objects", go(*f))
    f = fixture("noman")
    case("SYNTHETIC", "missing-manifest-file", 2, "cannot read the object manifest", go(f[0], f[1], f[2], f[3], f[0] / "nope.txt"))

    # fail closed on the source
    def empty_sources(files, units):
        units.clear()
    f = fixture("nounits", edit=empty_sources)
    case("SYNTHETIC", "empty-sources-txt", 2, "match no compiled", go(*f))
    def stale_rules(files, units):
        units.remove("aphavoc/source/entity/mobile/mobile.c"); units.remove("modules/3d/terrain.c")
    f = fixture("stale-rules", edit=stale_rules)
    case("SYNTHETIC", "path-classes-match-no-world-file", 2, "rules are stale", go(*f))
    def missing_unit(files, units):
        units.append("modules/misc/missing.c")
    f = fixture("missing-unit", edit=missing_unit)
    case("SYNTHETIC", "compiled-unit-not-in-tree", 2, "not in the source tree", go(*f))


def load_tool():
    spec = importlib.util.spec_from_file_location("world_boundary_check", tool)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def retained(tmp, dll, info):
    base = Path(tmp)
    repo = here.parents[1]
    args = ["check", "--dll", dll, "--build-info", info, "--root", repo]
    case("RETAINED", "retained-dll-no-manifest", 1, "FAIL", run(*args, "--report", base / "r.json"))
    # a clean-looking manifest for the real DLL cannot launder the 1378 physical definitions
    m = load_tool()
    sha = hashlib.sha256(Path(dll).read_bytes()).hexdigest()
    commit = m.build_info(info)["commit"]
    src = m.Source(repo, commit)
    try:
        derived = m.Derived(src)
    finally:
        src.close()
    man = base / "objects.txt"
    manifest(man, dll, [u[:-2] + ".o" for u in derived.units if derived.cls[u] not in m.PHYSICAL], commit=commit, sha=sha)
    case("RETAINED", "retained-dll-clean-looking-manifest", 1, "physical functions are defined", run(*args, "--objects", man))
    man_w = base / "objects-world.txt"
    manifest(man_w, dll, [u[:-2] + ".o" for u in derived.units], commit=commit, sha=sha)
    case("RETAINED", "retained-dll-world-objects-listed", 1, "world/environment objects are linked", run(*args, "--objects", man_w))
    # the retained DLL with one byte changed no longer matches its BUILD-INFO
    bad = base / "eech_dc.dll"
    data = bytearray(Path(dll).read_bytes()); data[len(data) // 2] ^= 1
    bad.write_bytes(bytes(data))
    case("RETAINED", "retained-dll-one-byte-changed", 2, "is not the BUILD-INFO", run(
        "check", "--dll", bad, "--build-info", info, "--root", repo))
    # truncated DLL with a matching hash is malformed, not clean
    short = base / "short.dll"
    short.write_bytes(Path(dll).read_bytes()[:1 << 20])
    info2 = base / "BUILD-INFO.txt"
    build_info(info2, short, commit=commit)
    case("RETAINED", "retained-dll-truncated", 2, "INCOMPLETE", run("check", "--dll", short, "--build-info", info2, "--root", repo))
    # right DLL, other revision named in BUILD-INFO: its source cannot corroborate these symbols
    info3 = base / "BUILD-INFO-wrong.txt"
    build_info(info3, dll, commit=subprocess.run(["git", "-C", str(repo), "rev-parse", "HEAD"], capture_output=True,
                                                 text=True).stdout.strip(), sha=sha)
    # the commit is an ancestor with identical executable inputs (see the docs), so the verdict is still FAIL
    case("RETAINED", "retained-dll-head-as-build-commit", 1, "FAIL", run("check", "--dll", dll, "--build-info", info3, "--root", repo))


if __name__ == "__main__":
    if len(sys.argv) not in (1, 3):
        sys.exit(__doc__)
    with tempfile.TemporaryDirectory() as tmp:
        (Path(tmp) / "syn").mkdir()
        synthetic(Path(tmp) / "syn")
        if len(sys.argv) == 3:
            (Path(tmp) / "real").mkdir()
            retained(Path(tmp) / "real", sys.argv[1], sys.argv[2])
        else:
            print("SKIPPED   retained-DLL cases: pass <eech_dc.dll> <BUILD-INFO.txt> (not a pass of them)")
    print("%d/%d cases as expected" % (sum(results), len(results)))
    sys.exit(0 if all(results) else 1)
