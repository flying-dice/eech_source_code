# mlua-sys 0.6.8, patched for DCS World's Lua 5.1

This directory is the published crate `mlua-sys` 0.6.8 (crates.io package SHA-256
`380c1f7e2099cafcf40e51d3a9f20a346977587aa4d012eae1f043149a728a93`, the checksum in
`../../Cargo.lock` before this patch; upstream mlua-rs/mlua `31efd0cf727c3aee0b733a8dedac024589e1700c`,
`mlua-sys/`; MIT licence, Aleksandr Orlenko) with one change. The crate's `.cargo-checksum.json`
is not kept; every other file is as published. `../../Cargo.toml` selects it with
`[patch.crates-io]`.

The change, in `src/lua51/lua.rs`:

```diff
-const LUA_IDSIZE: usize = 60;
+const LUA_IDSIZE: usize = 256;
```

The reason, the safety argument and the measured scope are in
[docs/m5-dcs-lua-abi.md](../../docs/m5-dcs-lua-abi.md). In short, DCS World's `lua.dll` is built
with `LUA_IDSIZE` 256. Its `lua_getstack`/`lua_getinfo` therefore write a 320-byte `lua_Debug` on
x86_64, into records that mlua allocates at 120 bytes. The first failing native callback overruns
the stack.

To check the copy against the published crate, run `cargo vendor` on a checkout without this
patch (the integration head `6f222b1b` has the stock lockfile) and compare with `diff -r`. The only differences are `.cargo-checksum.json` and the hunk above, with its
comment.

Remove this directory and the `[patch.crates-io]` entry when the mlua-sys in use lets the host
Lua's `LUA_IDSIZE` be configured.
