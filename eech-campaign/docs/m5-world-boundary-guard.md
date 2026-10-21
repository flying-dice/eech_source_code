# M5: the world-boundary guard

**Status.** A mechanical check for [M5 criterion 2](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(no physical world inside the campaign module), version 1 of the goal. It does
not accept criterion 2 or M5, and a tool PASS alone would not either. It is a
known-source-definition and object check: static symbols, data definitions,
indirect dispatch, the semantic map/type/state of the boundary and the public
host checks remain obligations it does not discharge. Its job today is to be
unable to call the current whole-engine `eech_dc.dll` compliant, and to say so
reproducibly. It changes no production C, Rust or build file.

| Tool | Level | Meaning |
| --- | --- | --- |
| `tools/world-boundary-check.py check` | B, acceptance | Zero known W/E-class definitions in the built DLL |
| `tools/world-boundary-check.py staging` | A, ratchet | Per campaign file references to physical code; may only shrink |
| `tools/world-boundary-check-selftest.py` | | 74 synthetic cases and 6 cases against a real DLL |

Level A progress never satisfies Level B. They share source classification
only.

## Level B: `check`

Inputs: `--dll`, `--build-info` (the `BUILD-INFO.txt` beside it), optionally
`--objects` (see below), `--expect-commit`, `--report`.

1. **Derive the world/environment set from source.** At the commit
   `BUILD-INFO.txt` names, read the maintained `sources.txt` and apply the
   tool's own path classes, `CLASS_RULES` version 1 (first match wins): C
   campaign (`ai/`, `ai_extrn.c`, `entity/mobile/cargo/`, ten
   `entity/special/*` campaign directories), W world (`entity/mobile/`,
   `entity/fixed/`, `entity/special/{bridge,effect}/`), E environment
   (`modules/3d/`, `terr_db`, `tod`, `timeaccl`, `session/`, random, time,
   timer), X presentation, H headless host layer, F shared. The rules were
   drafted from the M5 call map, which is under review and is not their
   authority; they change only with a bump of the version, which the report
   records. W and E are conservative path classes, not a review of each
   function's semantics: E includes pure RNG, time and timer helpers, and the
   classes decide nothing about who should own them (no RNG ownership decision
   is made here). A W/E-class function is each non-static function defined in a
   W/E file, whether or not that file is compiled: the reference tree supplies
   the definitions, so a build that compiles no World file is still judged
   against them. There is no hand-kept list or allowlist.
2. **Read the DLL itself.** The tool parses the PE/COFF symbol table and import
   directory. It does not trust `nm` or other text output.
3. **FAIL** if the DLL defines any W/E-class function, or a C/H function whose
   normalised body (at least 60 characters) equals a W/E-class body whose
   original is not defined in the DLL under that or another name. A renamed
   in-DLL stub of a known obligation therefore cannot turn green. Same-name
   or still-defined originals are template siblings and are only counted.
4. **PASS** additionally needs an object manifest proving no world object was
   linked. Without one, a DLL with no W/E-class definition is `INCOMPLETE`, not
   `PASS`.
5. **INCOMPLETE (exit 2), never PASS,** for a missing, unreadable, truncated,
   blank or malformed DLL or symbol table; no `luaopen_eech_dc` export; a DLL
   hash that differs from `BUILD-INFO.txt`; an unresolvable or wrong-expected
   commit; a symbol table that does not look like this source (fewer than half
   the campaign functions defined: an arbitrary sanity heuristic, not
   provenance or completeness); an empty `sources.txt`; classes that find no
   W/E definition or no campaign function in the compiled units and the
   reference tree (stale rules, or an absent or empty reference world); a
   compiled unit absent from the tree; and a stale or incomplete manifest
   (below).

   The identity records are strict. `BUILD-INFO.txt` must hold exactly one
   40-hex `commit:`, exactly one `uncommitted changes in the build inputs: N
   files` (N above zero is INCOMPLETE), and exactly one `sha256:` block of
   well-formed rows without a repeated name and with `eech_dc.dll`; toolchain
   and other lines are free. Any line that opens with a `sha256` token other
   than that one exact `sha256:` header (truncated, bare, uppercase, indented
   or followed by a value) is INCOMPLETE wherever it sits; text inside a line,
   such as `image: …@sha256:…`, and hash-first file rows are not such records. The object manifest has a closed vocabulary
   (`dll-sha256`, `source-commit`, `object`); a repeated header (wrong then
   right, right then wrong, identical or truncated), an unknown, bare or
   malformed record, or a repeated object (also after `\` to `/`
   normalisation, or two objects for one compiled unit) is INCOMPLETE, not
   resolved by taking the first or the last.

Exit status: 0 PASS, 1 FAIL, 2 INCOMPLETE. With `--synthetic-source-dir` the
tool reads a fixture tree, prints `SYNTHETIC FIXTURE, NOT ACCEPTANCE EVIDENCE`
and may print `SYNTHETIC-GREEN` with exit 0. That proves parser logic only. A
legitimate fixture that compiles no World file reaches it only when its
reference tree still holds the physical C files; a known W/E stub in it fails.

### What final acceptance still needs: artifact provenance

A linked PE has no per-object provenance. The retained build recorded a
`BUILD-INFO.txt` and hashes, but no linker map or object list, so **this guard
cannot verify which objects went into the DLL and does not claim to.** The
source commit and `sources.txt` say what was meant to be compiled, not what was
linked. Acceptance requires the build to emit one, from the real link step
(for example `-Wl,-Map` or the linker's object argument list), converted to:

```
dll-sha256: <64 hex of the DLL under test>
source-commit: <40 hex, equal to BUILD-INFO.txt commit>
object: <path>.o
...
```

The guard rejects a manifest for another DLL hash or commit, an empty
manifest, an object that maps to no compiled unit, a manifest missing any
compiled campaign unit, duplicates, and any object that maps to a W/E-class
unit. Producing the manifest needs a build and is not done here; a full pack
was deliberately not rebuilt for it.

## Result on the retained build

DLL SHA-256 `60cff8d96b97dfa820043db6b4aca136105e5514e5e70aa3de23a876e75f2057`,
`BUILD-INFO.txt` commit `5a3b0139ba6d66e4be6b778290025a3bff3dd6e8`
(from `m5-host-identities/.../host-identities-abi-final`). Its 0 uncommitted
input files are recorded by that file; this tool only reads that record.

```
FAIL eech_dc.dll 60cff8d96b97 at 5a3b0139
  - 1378 world/environment-class (W/E) functions are defined in the DLL (E 707, W 671)
```

- 1,378 of 1,735 W/E-class functions in source are defined; 1,230 compiled
  units; 13,283 defined external symbols (read identically by `objdump`).
- 501 of 602 campaign functions are defined in the DLL. The sanity check
  requires at least half; that threshold is arbitrary and is neither provenance
  nor completeness. The rest are presumably static or unreferenced and dropped
  by the linker; the guard cannot distinguish.
- Renamed-stub copies 0; 60 same-name template siblings ignored.
- Provenance: `5a3b0139` is an ancestor of `a1bb7231`. Their executable inputs
  are the same; only `eech-campaign/crates/eech-dc/lua5.1/README.md` differs.
  The tool reports FAIL at both when `a1bb7231` is supplied as the build
  commit (a case in the selftest), so the verdict does not depend on this.

### Platform imports are recorded, not enforced

The DLL imports `api-ms-win-core-synch-l1-2-0` 3, `bcryptprimitives` 1,
`kernel32` 146, `lua` 83, `msvcrt` 118, `ntdll` 6, `userenv` 1 and `ws2_32` 26
functions. These come from the Rust runtime, CRT and the Windows build, so they
are listed in the report and have no verdict. There is no Lua/CRT-only
whitelist, which would be fictional. Physical authority is identified
semantically (above), not by a DLL name. Reviewed, explicit supported
dependencies belong in a later review, not in this tool.

## Level A: `staging`

For every campaign (C) file at the build commit it reports named references to
physical functions and counts the generic entity attribute calls
(`get/set_(local|client_server)_entity_*`), noting non-literal attribute
arguments. On the retained commit: 161 campaign files, 178 named physical
references, 2,794 attribute calls (85 non-literal). `--write-baseline` retains
the per-file numbers (`reference/m5-world-boundary-guard/staging-baseline.json`);
`--baseline` fails (exit 1) if any file got worse. It reports honestly that it
cannot resolve these, so none may be called "contract-only" from it:

- entity attribute and list dispatch (the type behind `en` is not known
  statically, so counted calls may reach physical state);
- file-scope data and function-pointer tables;
- calls hidden behind macros;
- static functions of physical files.

A named-reference count of zero for a file, such as `assign.c`, therefore does
not make it contract-only.

## Limits

- Level B checks known source definitions and objects only. Static symbols are
  not attributable from a DLL, data definitions are not derived, and indirect
  dispatch is not followed; all are outside its FAIL set.
- The final boundary still needs the semantic map, type and indirect-state
  review and the public host checks. A tool PASS alone is not acceptance of
  criterion 2, and none of the six criteria is accepted by this guard.
- W/E are conservative path classes (E includes pure RNG, time and timers).
  X and H files are informational. This is a classification, not a review of
  every file.
- Source is read from Git at the BUILD-INFO commit, not from the working tree.
- A synthetic or fixture result proves parser logic only, never a build.
- This guard cannot show that campaign decisions use host observations. That
  needs the product path (host Lua, public module, world) and the no-host boot
  negative controls, which this tool does not replace.

## Evidence

`reference/m5-world-boundary-guard/` retains `retained-dll-check-report.json`,
`selftest-output.txt` and `staging-baseline.json`, with hashes in its README.
`checkpoint-cdffc799/` keeps the earlier measurement (before the strict
records) unchanged, and `checkpoint-69110bcc/` keeps the previous selftest
output (72 cases, before malformed sha256 headers were rejected). The
real-DLL verdict and counts are the same in all three; only the reason wording
and added report keys differ in the first.

```
python tools/world-boundary-check-selftest.py <eech_dc.dll> <BUILD-INFO.txt>
python tools/world-boundary-check.py check --dll <eech_dc.dll> --build-info <BUILD-INFO.txt> --report <out.json>
python tools/world-boundary-check.py staging --build-info <BUILD-INFO.txt> [--baseline <json>]
```

Run the selftest with no arguments and the six retained cases are skipped
loudly: the summary says `SKIPPED, not run`, never passed. The portable
synthetic cases run in the `evidence-selftest` CI job. The retained red cases (all against the real DLL) are: no manifest
(FAIL), a clean-looking manifest (still FAIL, physical functions defined),
world objects listed (FAIL), one changed byte (INCOMPLETE, hash mismatch), a
truncated image (INCOMPLETE) and `a1bb7231` as the build commit (FAIL).
