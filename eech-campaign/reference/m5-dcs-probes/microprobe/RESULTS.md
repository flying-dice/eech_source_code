# DCS lua.dll ABI microprobe (M5 criterion 5): results

Date 2026-10-20, Windows 11 Pro 10.0.26200 (this PC, Windows retail host). Scope: a minimal
mlua 0.10.5 / mlua-sys 0.6.8 Lua 5.1 module in disposable hosts against the exact DCS `lua.dll`.
No DCS launch, no DCS mission, no product code changed, no installs; nothing published.

## Answer

**The crash is not caused by the compiler.** DCS `lua.dll` (sha256 `8a228b54…b755`) is built with
`LUA_IDSIZE = 256`, so its `lua_Debug` is 320 bytes. mlua-sys 0.6.8 hardcodes `LUA_IDSIZE = 60`
(`src/lua51/lua.rs:355`), so it allocates a 120-byte `lua_Debug`. When a callback fails, mlua builds
a traceback (mlua-sys `compat.rs` `luaL_traceback`, which calls `lua_getstack`/`lua_getinfo` on a
stack `lua_Debug`). DCS `lua.dll` then writes **196 bytes past the end** of that struct and
corrupts the module's stack.
- GNU module: the corrupted return address leads to a jump to 0x0
  (`0xc0000005 at 0x0, access [8,0]`). This is the same signature as the lead's `eech_dc` repro.
- MSVC module: the stack layout differs, so different data is corrupted. Results: a wrong error
  value (`"attempt to concatenate a userdata value"`), heap corruption (`0xc0000374`), or a later
  0xc0000005. MSVC is **not** a mitigation; it only changes the symptom.

**Mitigation that works in the microprobe:** mlua-sys 0.6.8 with only `LUA_IDSIZE` changed from 60
to 256. It passes every stage in both hosts and with both Lua DLLs, for both GNU and MSVC builds.

## How the cause was established
1. **Static, DCS `lua.dll` disassembly** (`build/dcs-lua.disasm.txt`):
   - `lua_getstack` (RVA 0x5DF0) writes `ar->i_ci` at `[r8+138h]`.
   - `lua_getinfo` (RVA 0x5BF0) reads it at `[r8+138h]`.
   - Stock PUC 5.1 x64 keeps `i_ci` at 0x74, so `short_src` here spans 0x38..0x138, i.e. 256 bytes.
   - Other layouts match stock: `luaL_Buffer` buffer is at +0x18 with its end at +0x218
     (`LUAL_BUFFERSIZE` 512), `lua_Integer` is 64-bit (`cvttsd2si rax`), and `TValue` is 16 bytes.
   - DCS `lua.dll` is MSVC-built (linker 14.39), in C mode, and uses `__intrinsic_setjmp`/`longjmp`
     from VCRUNTIME140.
2. **Dynamic, canary measurement** (`debug_footprint`, stage `footprint`): `lua_getstack` and
   `lua_getinfo("Sln")` run into a 1024-byte buffer filled with 0xA5.

   | lua.dll | bytes written | mlua-sys sizeof (stock / patched) |
   |---|---|---|
   | DCS | 316 | 120 / 320 |
   | mingw PUC 5.1.5 (candidate `d0c799fe…`) | 120 | 120 / 320 |

   The result is the same for the GNU and MSVC builds.
3. **Red/green with the same source:**
   - The stock module reproduces the crash in both hosts. This includes my MSVC C host, so the GNU
     `eech-world` host is not required for the crash.
   - The IDSIZE-256 module (one changed constant) passes everything.
   - The longjmp/SEH unwinding hypothesis is not needed. Raising Lua errors from `lua.dll`'s own C
     (`control`) passes everywhere. The stock GNU module also crosses DCS longjmp without problems
     once it no longer overruns (see the patched GNU row).

## Matrix (matrix-2 and matrix-2-msvc)
Stages:
- `control`: `pcall(error)` and `pcall(string.rep)` ×100, no module.
- `load`: require and GC.
- `footprint`
- `good`: ok() and take_table(table) ×1000.
- `gconly`: 1000 userdata with a Drop counter, collected, no errors.
- `typeerror`: `pcall(take_table, 'not a table')` ×100. This is the same mlua conversion path as
  `eech_dc.boot`.
- `explicit`: `pcall(fail)` ×100, which returns `Err(LuaError::RuntimeError)`.
- `gc`: tokens interleaved with errors.
- `inspect`: type, metatable and tostring of the error value.
- `all`: everything, then 1000 more of each error kind, with live userdata left for `lua_close`.

Hosts: `eech-world.exe` (the GNU candidate host, 65892de1, sha `13cc2a6f…`) and `luahost.exe`
(MSVC C, sha `e67ba1ad…`). Each cell ran 3 times in a fresh directory, as a hidden bounded process.

| module | DCS lua.dll | mingw lua.dll |
|---|---|---|
| gnu (stock mlua-sys) | 30/60: control/load/footprint/good/gconly pass; typeerror/explicit/gc/inspect/all crash 0xc0000005 at first module error | 60/60 |
| msvc (stock) | 36/60: same plus gc happens to survive; typeerror/explicit give wrong error value, then 0xc0000005 (eech-world) or script error (luahost); inspect 0xc0000374 | 60/60 |
| gnu-idsize256 | **60/60** | 60/60 |
| msvc-idsize256 | **60/60** | 60/60 |

Patched builds on DCS produce the correct error values, e.g.
`bad argument #1: error converting Lua string to table` and
`runtime error: explicit failure: x`, plus a traceback. Teardown (`lua_close` with live userdata)
exits 0. `HOST_CLOSED` is printed after `lua_close`.

matrix-1 is the earlier 2×2×2 run, without the footprint function, using module hashes
recorded per run in its `results.jsonl`. It shows the same red pattern for the stock builds.

## Toolchains and inputs
- MSVC side: rustc 1.97.0 (2d8144b78 2026-07-07), cargo 1.97.0, target x86_64-pc-windows-msvc, with
  VS 2022 BuildTools cl 19.44.35208 and link 14.44.35208.
- `lua.lib` was generated with `lib /def` from `dumpbin /exports` of the DCS `lua.dll`
  (130 exports, `build/importlib/lua.def`). The project's `crates/eech-dc/lua5.1/lua.lib`, which its
  README describes, is **absent**: `.gitignore` matches `*.lib`. Its vendored `luaconf.h` also says
  `LUA_IDSIZE 60`, which does not match DCS.
- GNU side: the pinned image `eech-build` `sha256:0c60c75d…` (the same ID as the pr59-candidate
  BUILD-INFO). It has rustc 1.98.1 (48a229cea) and gcc-mingw-w64-x86-64-posix
  12.2.0-14+deb12u1+25.2+b1. The build ran with `--network none`, offline and `--locked`, against
  `/opt/lua-mingw`, whose `lua.dll` equals the candidate's `d0c799fe…`.
- Both sides used identical vendored sources (`build/vendor`, `cargo vendor --locked`). The module
  pins `mlua =0.10.5` and `mlua-sys =0.6.8` with features lua51+module, the same as eech-dc minus
  `serialize`. The other dependency versions resolved fresh and differ from the product lockfile.
- Hashes of everything are in `SHA256SUMS.txt`, and each run's `results.jsonl` records the
  module/host/lua/script sha256. The `lua.lib` hash changed when it was regenerated on the rebuild
  (it embeds a timestamp); its exports are unchanged.

## Commands
- `cmd /c scripts\build-msvc.cmd` (import lib, luahost, stock MSVC module) → `build/build-msvc*.log`
- `cmd /c scripts\build-msvc-variants.cmd` → `build/build-msvc-variants.log`
- `sh scripts/build-gnu.sh` (both GNU variants in eech-build) → `build/build-gnu-2.log`
- `scripts\run-matrix.ps1 -Only gnu,gnu-idsize256 -Reps 3 -TimeoutSec 30 -Tag matrix-2`
- `scripts\run-matrix.ps1 -Only msvc,msvc-idsize256 -Reps 3 -TimeoutSec 30 -Tag matrix-2-msvc`
- Per run: `stdout.txt`, `stderr.txt`, exit code and hashes are in `runs/<tag>/<cell>/` and
  `runs/<tag>/results.jsonl`. The summary is in `runs/<tag>/summary.txt`.

## Limits
- Reproduced in a tiny module. It is **inferred**, not proven, that `eech_dc.dll` fails on the same
  overrun. That is very likely: it has the same mlua-sys 0.6.8, the same callback-error path, and
  the same crash signature and stage.
- The patched IDSIZE-256 build was not applied to the product. It has not been shown to give full
  `eech_dc` compatibility or DCS mission compatibility, and DCS was not launched.
- Other DCS luaconf deviations outside the checked layouts are unknown. Checked: lua_Debug,
  luaL_Buffer, lua_Integer, TValue size.
- The `eech-world` host itself embeds mlua with the same `lua_Debug` size. Any host-side mlua path
  that walks the stack (callback errors, `inspect_stack`, hooks, `Function::info`) carries the same
  overrun when the host runs on DCS `lua.dll`. In DCS itself only the module side matters.
- The `luahost` fault filter printed nothing for the GNU crashes: the stack was corrupt. The exit
  code is still 0xc0000005.
- Patched IDSIZE 256 on a stock IDSIZE-60 `lua.dll` is safe here: the buffer is larger than what the
  DLL writes, and the field offsets mlua reads are unchanged. The mingw column is 60/60.

## Concrete next step
Ask the lead to approve a bounded product Task for a DCS-ABI `lua_Debug` fix in eech-dc. Two ways:
- a `[patch.crates-io]` mlua-sys fork with `LUA_IDSIZE = 256`, gated to the DCS target;
- or avoiding mlua's traceback path on errors.

Then rerun the lead's existing `lua-compat.lua` / `first-error.lua` repro against the patched
`eech_dc.dll` with DCS `lua.dll`. Run the M4 pack on the mingw Lua to check that the change is
behaviour-neutral, before any DCS mission probe. Separately, report upstream that mlua-sys
hardcodes `LUA_IDSIZE` for lua51.
