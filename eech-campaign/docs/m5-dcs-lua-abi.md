# M5: native errors under DCS World's Lua (the `lua_Debug` size)

This record is part of M5 ([EEDCS-5](https://flying-dice.atlassian.net/browse/EEDCS-5)). The sole
scope authority is the [EECH M5 Goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(Confluence 11534337, version 1), criterion 5, DCS hosting risks. It corrects one measured defect:
**in DCS World's Lua, the first failing native callback crashed the process.** Examples are a bad
argument to `dc.boot` and `host.log ({})`. It does not make DCS hosting safe in general, and no
corrected build has run in a DCS mission.

## The defect

DCS World's `lua.dll` (Lua 5.1, x86_64, SHA-256 `8a228b54…b755`) is built with `LUA_IDSIZE` 256.
Stock Lua 5.1 uses 60. `lua_Debug` ends with `char short_src[LUA_IDSIZE]` and the private
`int i_ci`, so the two builds disagree on the record's size:

| x86_64 | `short_src` | `i_ci` | record |
|---|---|---|---|
| stock Lua 5.1 (and mlua-sys 0.6.8) | 0x38, 60 bytes | 0x74 | 120 bytes |
| DCS World `lua.dll` | 0x38, 256 bytes | 0x138 | 320 bytes (316 written, see below) |

The two numbers for DCS are different things. **316 bytes** is what `lua_getstack` and
`lua_getinfo` were measured to write on one DCS `lua.dll`: through the end of `i_ci` (0x138 + 4).
**320 bytes** is the size of the `lua_Debug` the corrected product allocates: 0x138 + 4 = 316,
rounded up to the struct's 8-byte alignment. The stock record is 0x74 + 4 = 120, already aligned,
so it is 120 both allocated and written.

`lua_getstack` and `lua_getinfo` write into a buffer the caller supplies.
mlua supplies it, sized from mlua-sys's `lua_Debug`. mlua-sys 0.6.8 hard-codes `LUA_IDSIZE` 60
(`src/lua51/lua.rs`). For the callback failures exercised here, mlua builds a traceback with
mlua-sys's compat `luaL_traceback` when Lua has enough stack space, because Lua 5.1 has none.
That traceback walks the stack with a 120-byte record on the Rust stack. DCS World's Lua then
writes up to 316 bytes into it: a 196-byte stack overrun. With the GNU build the overrun corrupts
a return address, and the process faults at address 0 (`0xc0000005`).

## The correction

`patches/mlua-sys-0.6.8` is the published crate with `LUA_IDSIZE` 256, selected by
`[patch.crates-io]` in `Cargo.toml` ([PATCH.md](../patches/mlua-sys-0.6.8/PATCH.md)). mlua-sys's
`lua_Debug` becomes 320 bytes on x86_64, which holds either host's record. Nothing else changes:
no compiler, mlua version, campaign code, baseline or tolerance. The same `eech_dc.dll` serves
both hosts, EECH World's PUC `lua.dll` and DCS World's.

### Why a larger record is safe, path by path

These are all the `lua_Debug` users in mlua 0.10.5 and mlua-sys 0.6.8 (lua51; source read at
those versions):

| Use | Record | Reached by this product | With the 320-byte record |
|---|---|---|---|
| compat `luaL_traceback` and its `compat53_countlevels` (mlua-sys `lua51/compat.rs`) | allocated by mlua-sys, zeroed | yes: the exercised failing native callbacks, in the DLL and in the host (mlua `state/util.rs`, `util/error.rs`), and error-handler paths that request a traceback | no overrun under either host |
| `Lua::inspect_stack` (`state.rs`) | allocated by mlua, zeroed, moved into `Debug` | no call in `crates/` | no overrun. The move copies all 320 bytes, so the host's `i_ci` survives at the host's own offset for later `lua_getinfo` calls |
| `Function::info` (`function.rs`) | allocated by mlua, zeroed | no call in `crates/` | no overrun |
| hook callbacks: `Debug::new` from `hook_proc` (`state/raw.rs`, `hook.rs`) | allocated **by the host Lua**, at the host's size | no: nothing in `crates/` calls `set_hook`, so `hook_proc` is never installed | **not covered** (see Limits) |

Two properties make the 320-byte allocation sufficient for the two measured hosts on the
mlua-owned paths above:
- **Each measured host wrote within 320 bytes.** The tool below measured 316 bytes written for
  DCS and 120 for PUC. Its only bound is its finite 64 KiB buffer: a write reaching the last byte
  is reported as unmeasured, while a host writing beyond the buffer is outside the tool's safe
  measurement range. It does not show that every Lua 5.1 record fits in 320 bytes.
- **The fields mlua reads (`event` through `lastlinedefined`, and `short_src`) are at the same
  offsets in both layouts.** Only the length of `short_src` differs, and it comes after them.
  - `short_src` is read as a C string. The host NUL-terminates it within its own `LUA_IDSIZE`,
    and the zeroed record ends in zeros anyway.
  - `i_ci` is written and read only by the host, at its own offset, inside the same buffer.
  - On x86_64, `crates/eech-dc/tests/lua_debug_layout.rs` holds the offset (`short_src` at 0x38) and the size.

### Scope

- **Measured and supported:** x86_64 Windows. The product is `x86_64-pc-windows-gnu`, built in the
  pinned `eech-build` image. Hosts: DCS World openbeta `2.9.29.27468` MT `lua.dll` `8a228b54…b755`
  (measured standalone, outside a mission), and EECH World's PUC Lua 5.1.5 `lua.dll`
  `d0c799fe…0350`.
- **Linux and i686 (CI, tests only):** the patched record is larger than the system Lua 5.1's
  (`LUA_IDSIZE` 60), which the same argument covers. No build flag, library or job changes.

## Coverage

| Check | Where | What it shows |
|---|---|---|
| `lua_debug_holds_the_dcs_world_record`, `lua_debug_short_src_starts_where_dcs_world_writes_it` | `crates/eech-dc/tests/lua_debug_layout.rs`, every CI test job | mlua-sys's record holds the measured DCS record, and the read fields keep their offsets. **Red against stock mlua-sys 0.6.8** (120 < 316). |
| `a_failing_dc_callback_returns_its_error_to_pcall` | `crates/eech-world/tests/load_dc.rs` | 100 failing `dc.boot` calls return the expected error and traceback, and the state collects and closes. Runs on PUC Lua (Linux, and Wine in `windows-cross`). It characterises the patched path on the normal host; it is green before and after. |
| `a_failing_host_callback_returns_its_error_to_pcall` | `crates/eech-world/tests/host_callbacks.rs` | The same for the host's own callback, `host.log ({})`, through the real `eech-world` binary |
| native-error controls, DCS and PUC `lua.dll` | `tools/native-error-controls/run-controls.ps1`, by hand on a host that has DCS World's `lua.dll` | Red and green at the real scope: the six controls, 3 runs each, for the stock and corrected product builds |
| record size of a `lua.dll` | `tools/lua-debug-layout/` (`build.sh`, `lua-debug-layout.exe`) | The measurement this correction rests on. Rerun it after a DCS update. |

DCS World's `lua.dll` is vendor binary and is not in the repository, so the DCS column cannot run
in CI.

Evidence, including the original experiment's red and green and this candidate's runs:
[reference/m5-dcs-lua-abi](../reference/m5-dcs-lua-abi/README.md).

## Limits

- **Hooks (borrowed PUC `lua_Debug`): unsupported.** Under the PUC host, a hook record is
  allocated by the host at 120 bytes, and mlua borrows it as a `lua_Debug` whose `short_src` is a
  256-byte array. That reference outruns the host's allocation: undefined behaviour in Rust,
  though the string read stops at the NUL. This record does **not** claim it safe. The product
  never calls `set_hook` (no call in `crates/` or `tools/`), so `hook_proc` is never installed and
  no check here exercises it. A future `set_hook`, or any `Debug` built from a host hook record,
  needs its own decision before it is used.
- **Standalone only.** No DCS mission has run the corrected build (the next DCS probe, EEDCS-12).
  That is a separate claim.
- **Other layouts.** Checked beyond `lua_Debug`: `luaL_Buffer` (`LUAL_BUFFERSIZE` 512), `lua_Integer`
  (64-bit) and `TValue` (16 bytes). Any other DCS `luaconf` deviation is unknown.
- **One measured DCS build.** Measured on one `lua.dll`: the extent the Lua writes (316 bytes, by
  `tools/lua-debug-layout/`) and, in its disassembly, `i_ci` at 0x138. `LUA_IDSIZE` 256 and the 0x38
  start of `short_src` follow from the stock field order and agree with both. A different DCS
  `lua.dll` must be measured first.
- **Upstream.** The patch stays until mlua-sys lets the host's `LUA_IDSIZE` be configured
  ([PATCH.md](../patches/mlua-sys-0.6.8/PATCH.md)).
