#!/usr/bin/env python3
# The M5 criterion 2 mechanical world-boundary guard (docs/m5-world-boundary-guard.md).
#
#   check    Level B, the zero-world acceptance guard. Reads the built eech_dc.dll's own COFF symbol table and import
#            table, derives the physical (world/environment) definitions from the build's source classification at
#            the commit its BUILD-INFO.txt names, and FAILs if the DLL defines any of them, or a renamed copy of a
#            physical function body. PASS additionally needs an object manifest from the link map that proves no
#            physical object was linked; without one the answer is INCOMPLETE, never PASS. Missing, malformed,
#            wrong-revision or stale inputs are INCOMPLETE. There is no allowlist.
#   staging  Level A, a staging report and ratchet: per campaign source file, its named references to physical
#            functions and its generic entity-attribute calls. It may only shrink; it is NOT acceptance.
#
# Usage: tools/world-boundary-check.py check --dll <eech_dc.dll> --build-info <BUILD-INFO.txt> [--objects <manifest>]
#            [--expect-commit <sha>] [--root <repo>] [--report <file.json>] [--symbols-out <file.txt>]
#        tools/world-boundary-check.py staging --build-info <BUILD-INFO.txt> [--baseline <file.json>]
#            [--write-baseline <file.json>] [--root <repo>]
# Exit status 0 only for PASS (check) or a held ratchet (staging); 1 for FAIL; 2 for INCOMPLETE or invalid input.
import argparse, collections, hashlib, json, re, struct, subprocess, sys
from pathlib import Path

SYS = "eech-campaign/crates/eech-engine-sys/"
APHAVOC = "aphavoc/source/"
CAMPAIGN_DIRS = ["force", "group", "task", "guide", "keysite", "waypoint", "division", "regen", "landing", "sector"]
# The path classes are this tool's own, versioned, conservative classification (C campaign, W world, E environment,
# X presentation, H headless host layer, F shared infrastructure); first match wins. They are not a review of each
# function's semantics (E includes pure RNG, time and timer helpers) and decide nothing about who should own them.
# They were drafted from the M5 call map, which is under review and is not their authority; change them only with
# a bump of CLASS_RULES_VERSION, recorded in the report.
CLASS_RULES_VERSION = 1
CLASS_RULES = [
    ("H", [SYS + "csrc/"]),
    ("C", [APHAVOC + "ai/", APHAVOC + "ai_extrn.c", APHAVOC + "entity/mobile/cargo/"]
          + [APHAVOC + "entity/special/" + d + "/" for d in CAMPAIGN_DIRS]),
    ("W", [APHAVOC + "entity/mobile/", APHAVOC + "entity/fixed/", APHAVOC + "entity/special/bridge/",
           APHAVOC + "entity/special/effect/"]),
    ("E", ["modules/3d/", APHAVOC + "misc/terr_db", APHAVOC + "misc/tod", APHAVOC + "misc/timeaccl",
           APHAVOC + "entity/special/session/", "modules/maths/random.c", "modules/system/time.c",
           "modules/system/timer.c"]),
    ("X", [APHAVOC + "ui_menu/", "modules/userint2/", "modules/graphics/", "modules/2d/", "modules/sound/",
           APHAVOC + "appsound/", APHAVOC + "app3d/", APHAVOC + "gunships/", APHAVOC + "entity/special/camera/",
           APHAVOC + "cdmusic.c"]),
    ("F", [""]),
]
PHYSICAL = ("W", "E")
LUA_ENTRY = "luaopen_eech_dc"
SCOPE = ("known-source-definition and object check only; static symbols, data definitions, indirect dispatch, "
         "semantic map/type/state and public host checks remain; a PASS alone is not criterion 2 acceptance")
HEX40, HEX64 = re.compile(r"[0-9a-f]{40}"), re.compile(r"[0-9a-f]{64}")
MIN_BODY = 60  # shorter normalised bodies are too generic to call a copy

_COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
_STRING = re.compile(r'"(?:\\.|[^"\\\n])*"|\'(?:\\.|[^\'\\\n])*\'')
_BRACE = re.compile(r"[{}]")
KEYWORDS = frozenset("if for while switch return sizeof else do case break continue goto default typedef struct "
                     "union enum static extern const volatile register unsigned signed int char float double long "
                     "short void".split())
FUNCTION_DEFINITION = re.compile(
    r"^(?:static\s+|extern\s+|inline\s+|__inline\s+)*[A-Za-z_][\w \t\*]*?\b([A-Za-z_]\w*)\s*"
    r"\(([^;{}()]|\((?:[^()]|\([^()]*\))*\))*\)\s*(?:#[^\n]*\n\s*)*\{", re.M)
IDENT = re.compile(r"\b[A-Za-z_]\w*\b")
ATTRIBUTE = re.compile(r"\b(?:get|set)_(?:local|client_server)_entity_\w+?\s*\(\s*[^,()]+(?:\([^()]*\))?\s*,\s*"
                       r"([A-Za-z0-9_]+)")


class Incomplete(Exception):
    """an input is missing, malformed, stale or for another revision; the verdict is INCOMPLETE, never PASS"""


def path_class(path):
    return next(k for k, prefixes in CLASS_RULES if any(path.startswith(p) for p in prefixes))


# ---- sources: a git revision (the real check) or a labelled synthetic directory (selftest fixtures only)

class Source:
    def __init__(self, root, rev=None, directory=None):
        self.root, self.rev, self.directory = Path(root), rev, directory
        self._cat = None
        if directory is None:
            out = self._git("ls-tree", "-r", "--name-only", rev)
            self.paths = out.decode("utf-8", "replace").splitlines()
        else:
            d = Path(directory)
            self.paths = sorted(p.relative_to(d).as_posix() for p in d.rglob("*") if p.is_file())
        self.pathset = set(self.paths)

    def _git(self, *args):
        r = subprocess.run(["git", "-C", str(self.root), *args], capture_output=True)
        if r.returncode:
            raise Incomplete("git %s failed: %s" % (" ".join(args), r.stderr.decode("utf-8", "replace").strip()))
        return r.stdout

    def read(self, path):
        if self.directory is not None:
            return (Path(self.directory) / path).read_text(encoding="latin-1")
        if self._cat is None:
            self._cat = subprocess.Popen(["git", "-C", str(self.root), "cat-file", "--batch"],
                                         stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self._cat.stdin.write(("%s:%s\n" % (self.rev, path)).encode()); self._cat.stdin.flush()
        header = self._cat.stdout.readline().split()
        if len(header) != 3 or header[1] != b"blob":
            raise Incomplete("%s is not a file at %s" % (path, self.rev))
        data = self._cat.stdout.read(int(header[2])); self._cat.stdout.read(1)
        return data.decode("latin-1")

    def close(self):
        if self._cat:
            self._cat.stdin.close(); self._cat.wait()


def compiled_units(src):
    """sources.txt plus the headless host layer csrc/*.c, as build/main.rs reads them"""
    if SYS + "sources.txt" not in src.pathset:
        raise Incomplete("no %ssources.txt at the build revision" % SYS)
    units = [l.strip() for l in src.read(SYS + "sources.txt").splitlines()]
    units = [l for l in units if l and not l.startswith("#")]
    if not units:
        raise Incomplete("sources.txt lists no compiled units")
    units += sorted(p for p in src.paths if p.startswith(SYS + "csrc/") and p.endswith(".c"))
    missing = [u for u in units if u not in src.pathset]
    if missing:
        raise Incomplete("%d compiled units are not in the source tree, e.g. %s" % (len(missing), missing[0]))
    return units


# ---- source parsing

def _blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))


def strip_comments(text):
    return _COMMENT.sub(_blank, text)


def strip_code(text):
    """comments and string/character literals become spaces, so offsets and line numbers survive"""
    return _STRING.sub(_blank, strip_comments(text))


def body_end(text, brace):
    depth = 0
    for m in _BRACE.finditer(text, brace):
        depth += 1 if m.group() == "{" else -1
        if depth == 0:
            return m.start()
    return len(text)


def functions(text):
    """[(name, is_static, normalised body)] of the function definitions at file scope"""
    t, c = strip_code(text), strip_comments(text)
    out, end = [], -1
    for m in FUNCTION_DEFINITION.finditer(t):
        name = m.group(1)
        if name in KEYWORDS or m.start() < end:
            continue
        brace = m.end() - 1
        end = body_end(t, brace)
        static = bool(re.match(r"(?:inline\s+|__inline\s+)*static\b", t[m.start():m.start(1)].strip()))
        body = re.sub(r"\s+", "", c[brace:end + 1])
        out.append((name, static, re.sub(r"\b%s\b" % re.escape(name), "@", body)))
    return out


class Derived:
    """what the source classification says about the build at one revision"""

    def __init__(self, src):
        self.units = compiled_units(src)
        self.cls = {u: path_class(u) for u in self.units}
        # tree-only physical files count too: a world file dropped from sources.txt must not hide a stub of it
        self.tree_world = sorted(p for p in src.paths if p.endswith(".c") and p not in self.cls
                                 and path_class(p) in PHYSICAL and (p.startswith(APHAVOC) or p.startswith("modules/")))
        self.defs = collections.defaultdict(list)  # class -> [(file, name, static, body)]
        for f in self.units + self.tree_world:
            k = self.cls.get(f) or path_class(f)
            for name, static, body in functions(src.read(f)):
                self.defs[k].append((f, name, static, body))
        self.physical = {}  # non-static name -> defining file
        for k in PHYSICAL:
            for f, name, static, body in self.defs[k]:
                if not static:
                    self.physical.setdefault(name, f)
        # a build with no compiled world file is a legitimate target, but only against a reference tree that still
        # holds the world sources: an empty classification is stale rules, not proof, and fails closed
        if not self.physical:
            raise Incomplete("the path classes find no world/environment definition in the compiled units or the "
                             "reference tree; the rules are stale or the reference is unavailable")
        self.physical_bodies = {}
        for k in PHYSICAL:
            for f, name, static, body in self.defs[k]:
                if len(body) >= MIN_BODY and not static:
                    self.physical_bodies.setdefault(body, (f, name))
        self.campaign = {n for f, n, s, b in self.defs["C"] if not s}
        if not self.campaign:
            raise Incomplete("no campaign function is defined in the compiled units; the rules are stale")


# ---- the built DLL

def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def build_info(path):
    """the identity records of BUILD-INFO.txt (commit, uncommitted count, sha256 block header and rows), each exactly
    once and well formed; toolchain and other metadata lines are free, not a closed vocabulary"""
    try:
        lines = Path(path).read_text().splitlines()
    except (OSError, UnicodeDecodeError) as e:
        raise Incomplete("cannot read BUILD-INFO: %s" % e)

    def once(prefix, pattern, what):
        found = [l for l in lines if l.startswith(prefix)]
        if len(found) != 1:
            raise Incomplete("BUILD-INFO has %d '%s' records, need exactly one" % (len(found), prefix))
        m = re.fullmatch(pattern, found[0])
        if not m:
            raise Incomplete("BUILD-INFO %s is malformed: %r" % (what, found[0][:80]))
        return m.group(1)

    commit = once("commit:", r"commit:\s*([0-9a-f]{40})\s*", "commit")
    dirty = once("uncommitted changes", r"uncommitted changes in the build inputs:\s*(\d+) files?\s*",
                 "uncommitted count")
    # a line that opens with a sha256 token is a hash-block record, wherever it sits: the one block header must be
    # exactly "sha256:" and anything else (a truncated or malformed header) is not free metadata. Mid-line text such
    # as "image: ...@sha256:..." and the file rows (which open with a hash) are not records of this kind.
    stray = [l for l in lines if l.split(None, 1)[:1] and l.split(None, 1)[0].lower().startswith("sha256")
             and l.rstrip() != "sha256:"]
    if stray:
        raise Incomplete("BUILD-INFO has a malformed sha256 header record: %r" % stray[0][:80])
    blocks = [l for l in lines if l.rstrip() == "sha256:"]
    if len(blocks) != 1:
        raise Incomplete("BUILD-INFO has %d 'sha256:' blocks, need exactly one" % len(blocks))
    shas = {}
    for l in lines[lines.index(blocks[0]) + 1:]:
        if not l.strip():
            continue
        m = re.fullmatch(r"\s+([0-9a-f]{64})\s+(\S+)\s*", l)
        if not m:
            raise Incomplete("BUILD-INFO has a malformed sha256 row: %r" % l[:80])
        if m.group(2) in shas:
            raise Incomplete("BUILD-INFO lists %s twice" % m.group(2))
        shas[m.group(2)] = m.group(1)
    if "eech_dc.dll" not in shas:
        raise Incomplete("BUILD-INFO lacks the eech_dc.dll sha256")
    if int(dirty):
        raise Incomplete("the DLL was built with %s uncommitted input files; its source is not a revision" % dirty)
    return {"commit": commit, "dll_sha256": shas["eech_dc.dll"]}


def read_pe(path):
    """(defined symbols {name: storage class}, undefined count, imports {dll: [names]}) from the PE file itself"""
    d = Path(path).read_bytes()
    try:
        pe = struct.unpack_from("<I", d, 0x3C)[0]
        if d[:2] != b"MZ" or d[pe:pe + 4] != b"PE\0\0":
            raise Incomplete("not a PE file")
        nsec, _, symptr, nsym, optsz = struct.unpack_from("<HIIIH", d, pe + 6)
        opt = pe + 24
        if struct.unpack_from("<H", d, opt)[0] != 0x20B:
            raise Incomplete("not a PE32+ (x86-64) image")
        sections = [struct.unpack_from("<8sIIII", d, opt + optsz + 40 * i) for i in range(nsec)]
        if not symptr or not nsym:
            raise Incomplete("the DLL has no COFF symbol table (stripped); nothing can be corroborated")
        strtab = symptr + 18 * nsym
        defined, undefined, i = {}, 0, 0
        while i < nsym:
            raw, value, sec, typ, scl, aux = struct.unpack_from("<8sIhHBB", d, symptr + 18 * i)
            if raw[:4] == b"\0\0\0\0":
                off = strtab + struct.unpack("<I", raw[4:])[0]
                name = d[off:d.index(b"\0", off)].decode("latin-1")
            else:
                name = raw.rstrip(b"\0").decode("latin-1")
            if sec > 0 and scl in (2, 3):
                if scl == 2:
                    defined[name] = scl
            elif sec == 0 and scl == 2:
                undefined += 1
            i += 1 + aux

        def offset(rva):
            for _, vsize, vaddr, rawsize, rawptr in sections:
                if vaddr <= rva < vaddr + max(vsize, rawsize):
                    return rawptr + rva - vaddr
            raise Incomplete("an import RVA is outside every section")

        def cstr(off):
            return d[off:d.index(b"\0", off)].decode("latin-1")

        imports, (irva, isize) = {}, struct.unpack_from("<II", d, opt + 112 + 8)
        pos = offset(irva) if irva else None
        while pos is not None:
            oft, _, _, name_rva, ft = struct.unpack_from("<IIIII", d, pos)
            if not name_rva:
                break
            names, t = [], offset(oft or ft)
            while True:
                (thunk,) = struct.unpack_from("<Q", d, t)
                if not thunk:
                    break
                names.append("#%d" % (thunk & 0xFFFF) if thunk >> 63 else cstr(offset(thunk & 0x7FFFFFFF) + 2))
                t += 8
            imports[cstr(offset(name_rva)).lower()] = names
            pos += 20
        return defined, undefined, imports
    except (struct.error, ValueError, IndexError) as e:
        raise Incomplete("malformed PE image: %s" % e)


def read_objects(path, dll_sha, commit):
    """the object manifest from the link map: 'dll-sha256: h', 'source-commit: c', then 'object: <path>' lines.
    A small closed vocabulary: any other record, a repeated header or a repeated object is INCOMPLETE."""
    try:
        lines = Path(path).read_text().splitlines()
    except (OSError, UnicodeDecodeError) as e:
        raise Incomplete("cannot read the object manifest: %s" % e)
    head, objs = {}, []
    for n, l in enumerate(lines, 1):
        if not l.strip():
            continue
        key, sep, value = l.partition(": ")
        value = value.strip()
        if not sep or key not in ("dll-sha256", "source-commit", "object") or not value:
            raise Incomplete("the object manifest has an unknown or malformed record at line %d: %r" % (n, l[:80]))
        if key == "object":
            objs.append(value.replace("\\", "/"))
        elif key in head:
            raise Incomplete("the object manifest repeats %s at line %d" % (key, n))
        else:
            head[key] = value
    for key, pattern in (("dll-sha256", HEX64), ("source-commit", HEX40)):
        if key not in head:
            raise Incomplete("the object manifest has no %s record" % key)
        if not pattern.fullmatch(head[key]):
            raise Incomplete("the object manifest %s is malformed: %r" % (key, head[key][:80]))
    if head["dll-sha256"] != dll_sha:
        raise Incomplete("the object manifest is for another DLL (stale): dll-sha256 %s" % head["dll-sha256"])
    if head["source-commit"] != commit:
        raise Incomplete("the object manifest is for another source commit: %s" % head["source-commit"])
    if not objs:
        raise Incomplete("the object manifest lists no objects")
    dup = sorted(o for o, c in collections.Counter(objs).items() if c > 1)
    if dup:
        raise Incomplete("the object manifest lists %d objects more than once, e.g. %s" % (len(dup), dup[0]))
    return objs


def map_objects(objs, derived):
    """object path -> compiled unit, by the unit path with its .c replaced by .o as the path's tail"""
    by_tail = {u[:-2] + ".o": u for u in derived.units if u.endswith(".c")}
    hit, unmapped = {}, []
    for o in objs:
        o = o.replace("\\", "/")
        m = next((t for t in by_tail if o.endswith("/" + t) or o == t), None)
        if m:
            if by_tail[m] in hit.values():
                raise Incomplete("the object manifest lists compiled unit %s more than once" % by_tail[m])
            hit[o] = by_tail[m]
        else:
            unmapped.append(o)
    return hit, unmapped


# ---- Level B

def check(a):
    root = Path(a.root)
    info = build_info(a.build_info)
    if a.expect_commit and a.expect_commit != info["commit"]:
        raise Incomplete("BUILD-INFO commit %s is not the expected %s" % (info["commit"], a.expect_commit))
    if not Path(a.dll).is_file():
        raise Incomplete("no DLL at %s" % a.dll)
    dll_sha = sha256(a.dll)
    if dll_sha != info["dll_sha256"]:
        raise Incomplete("the DLL sha256 %s is not the BUILD-INFO's %s" % (dll_sha, info["dll_sha256"]))
    synthetic = a.synthetic_source_dir is not None
    src = Source(root, info["commit"], a.synthetic_source_dir)
    try:
        derived = Derived(src)
    finally:
        src.close()
    defined, undefined, imports = read_pe(a.dll)
    if LUA_ENTRY not in defined:
        raise Incomplete("the DLL does not define %s; it is not the EECH_DC module" % LUA_ENTRY)
    covered = len(derived.campaign & set(defined))
    if derived.campaign and covered * 2 < len(derived.campaign):
        raise Incomplete("only %d of %d campaign functions from source are defined in the DLL; the symbol table does "
                         "not look like this source (a sanity heuristic, not provenance)" % (covered, len(derived.campaign)))

    hits = {n: derived.physical[n] for n in defined if n in derived.physical}
    compiled = set(derived.units)
    stubs = sorted(n for n, f in hits.items() if f not in compiled)  # defined, but its world file is not compiled
    copies, siblings = [], 0
    # only campaign (C) and Rust-boundary (H) code is held to this: F-class entity template code legitimately repeats
    # generic bodies, so a body match there says nothing about a world stub
    for k in ("C", "H"):
        for f, name, static, body in derived.defs[k]:
            if len(body) >= MIN_BODY and body in derived.physical_bodies and (static or name in defined):
                of = derived.physical_bodies[body]
                if of[1] == name or of[1] in defined:
                    # same name, or the world original is itself still defined (so nothing was replaced): a
                    # generated entity-template sibling, not a stand-in for a removed world function
                    siblings += 1
                else:
                    copies.append({"copy": f + ":" + name, "of": "%s:%s" % of})
    by_class = collections.Counter(path_class(f) for f in hits.values())
    by_dir = collections.Counter("/".join(f.split("/")[:4]) for f in hits.values())

    objects = None
    if a.objects:
        objs = read_objects(a.objects, dll_sha, info["commit"])
        mapped, unmapped = map_objects(objs, derived)
        listed = set(mapped.values())
        absent = [u for u in derived.units if derived.cls[u] == "C" and u not in listed]
        world_objs = sorted(u for u in listed if derived.cls[u] in PHYSICAL)
        objects = {"objects": len(objs), "unmapped": unmapped[:10], "unmapped_count": len(unmapped),
                   "campaign_units_missing": len(absent), "world_objects": world_objs}
        if unmapped or absent:
            raise Incomplete("the object manifest is incomplete: %d objects map to no compiled unit, %d campaign "
                             "units are absent" % (len(unmapped), len(absent)))

    failures = []
    if hits:
        failures.append("%d world/environment-class (W/E) functions are defined in the DLL (%s)" % (
            len(hits), ", ".join("%s %d" % kv for kv in sorted(by_class.items()))))
    if copies:
        failures.append("%d non-world functions are copies of a physical function body" % len(copies))
    if objects and objects["world_objects"]:
        failures.append("%d world/environment objects are linked" % len(objects["world_objects"]))
    if failures:
        verdict = "FAIL"
    elif objects is None:
        verdict = "INCOMPLETE"
        failures.append("no object manifest: link/source provenance is not proven")
    else:
        verdict = "SYNTHETIC-GREEN" if synthetic else "PASS"
    report = {
        "verdict": verdict, "reasons": failures, "synthetic_fixture": synthetic,
        "dll_sha256": dll_sha, "build_commit": info["commit"],
        "compiled_units": len(derived.units), "units_by_class": dict(collections.Counter(derived.cls.values())),
        "tree_only_world_files": len(derived.tree_world),
        "physical_functions_in_source": len(derived.physical),
        "dll_defined_external_symbols": len(defined), "dll_undefined_external_symbols": undefined,
        "campaign_symbol_sanity_check": "%d/%d campaign functions defined; at least half is required. An arbitrary "
                                        "sanity heuristic, not provenance or completeness" % (covered, len(derived.campaign)),
        "classification": "CLASS_RULES v%d: conservative path classes, not a semantic review (E includes pure RNG "
                          "and time helpers); decides nothing about ownership" % CLASS_RULES_VERSION,
        "compiled_world_units": sum(1 for u in derived.units if derived.cls[u] in PHYSICAL),
        "scope": SCOPE,
        "physical_defined": len(hits), "physical_defined_by_class": dict(by_class),
        "physical_defined_by_dir": dict(by_dir.most_common()),
        "physical_defined_sha256": hashlib.sha256("\n".join(sorted(hits)).encode()).hexdigest(),
        "physical_defined_sample": sorted(hits)[:25],
        "physical_defined_without_compiled_world_file": stubs[:25],
        "renamed_or_stub_body_copies": copies[:25], "renamed_or_stub_body_copy_count": len(copies),
        "same_name_template_siblings_ignored": siblings,
        "object_manifest": objects,
        "platform_imports": {k: len(v) for k, v in sorted(imports.items())},
        "platform_imports_note": "recorded, not enforced: no import whitelist is applied",
    }
    if a.symbols_out:
        Path(a.symbols_out).write_text("".join("%s\t%s\n" % (n, hits[n]) for n in sorted(hits)))
    return report, sorted(hits), imports


# ---- Level A

def staging(a):
    info = build_info(a.build_info)
    src = Source(a.root, info["commit"])
    try:
        derived = Derived(src)
        files = {}
        for u in derived.units:
            if derived.cls[u] != "C":
                continue
            own = {n for f, n, s, b in derived.defs["C"] if f == u}
            t = strip_code(src.read(u))
            refs = collections.Counter(n for n in IDENT.findall(t) if n in derived.physical and n not in own)
            attrs = ATTRIBUTE.findall(t)
            files[u] = {"physical_references": sum(refs.values()), "physical_names": dict(sorted(refs.items())),
                        "entity_attribute_calls": len(attrs),
                        "entity_attribute_calls_nonliteral": sum(1 for x in attrs if not re.search(r"_TYPE_", x))}
    finally:
        src.close()
    return {"level": "A staging report, NOT acceptance", "build_commit": info["commit"], "files": files,
            "unresolved": ["entity attribute and list dispatch: the entity type behind an `en` argument is not known "
                           "statically, so calls counted here may reach physical state",
                           "file-scope data and function-pointer tables",
                           "calls hidden behind macros", "static functions of physical files"]}


def ratchet(report, baseline):
    worse = []
    for f, now in report["files"].items():
        was = baseline["files"].get(f)
        if was is None:
            worse.append("%s is new to the staging report" % f)
            continue
        for key in ("physical_references", "entity_attribute_calls", "entity_attribute_calls_nonliteral"):
            if now[key] > was[key]:
                worse.append("%s %s grew %d -> %d" % (f, key, was[key], now[key]))
        for n, c in now["physical_names"].items():
            if c > was["physical_names"].get(n, 0):
                worse.append("%s now references %s %d times (was %d)" % (f, n, c, was["physical_names"].get(n, 0)))
    return worse


def totals(report):
    f = report["files"].values()
    return (sum(x["physical_references"] for x in f), sum(x["entity_attribute_calls"] for x in f),
            sum(x["entity_attribute_calls_nonliteral"] for x in f))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest="mode", required=True)
    root = str(Path(__file__).resolve().parents[2])
    c = sub.add_parser("check")
    c.add_argument("--dll", required=True)
    c.add_argument("--build-info", required=True)
    c.add_argument("--objects")
    c.add_argument("--expect-commit")
    c.add_argument("--root", default=root)
    c.add_argument("--report")
    c.add_argument("--symbols-out")
    c.add_argument("--synthetic-source-dir", help="selftest fixtures only: a labelled directory instead of a revision")
    s = sub.add_parser("staging")
    s.add_argument("--build-info", required=True)
    s.add_argument("--root", default=root)
    s.add_argument("--baseline")
    s.add_argument("--write-baseline")
    a = p.parse_args()
    try:
        if a.mode == "check":
            report, names, imports = check(a)
            if a.report:
                Path(a.report).write_text(json.dumps(report, indent=1, sort_keys=True) + "\n")
            tag = "SYNTHETIC FIXTURE, NOT ACCEPTANCE EVIDENCE: " if report["synthetic_fixture"] else ""
            print("%s%s eech_dc.dll %s at %s" % (tag, report["verdict"], report["dll_sha256"][:12],
                                                 report["build_commit"][:8]))
            for r in report["reasons"]:
                print("  - " + r)
            print("  W/E-class definitions in the DLL: %d of %d in source (%s)" % (
                report["physical_defined"], report["physical_functions_in_source"],
                ", ".join("%s %d" % kv for kv in sorted(report["physical_defined_by_class"].items())) or "none"))
            print("  scope: " + SCOPE)
            print("  platform imports (recorded, not enforced): " + ", ".join(
                "%s %d" % kv for kv in report["platform_imports"].items()))
            return {"PASS": 0, "SYNTHETIC-GREEN": 0, "FAIL": 1}.get(report["verdict"], 2)
        report = staging(a)
        if a.write_baseline:
            Path(a.write_baseline).write_text(json.dumps(report, indent=1, sort_keys=True) + "\n")
        refs, attrs, nonlit = totals(report)
        print("STAGING (Level A, not acceptance) at %s: %d campaign files, %d named physical references, "
              "%d entity attribute calls (%d non-literal)" % (report["build_commit"][:8], len(report["files"]),
                                                              refs, attrs, nonlit))
        if a.baseline:
            worse = ratchet(report, json.loads(Path(a.baseline).read_text()))
            for w in worse:
                print("  RATCHET WORSE: " + w)
            return 1 if worse else 0
        return 0
    except Incomplete as e:
        print("INCOMPLETE: %s" % e)
        return 2


if __name__ == "__main__":
    sys.exit(main())
