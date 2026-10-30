# DCS Lua 5.1 link inputs

Copied from flying-dice/dcs-studio `bridge/lua5.1` (MIT), the reference DCS
native-module crate. The headers are stock Lua 5.1 (see `include/LICENCE.txt`):
`include/luaconf.h` declares `LUA_IDSIZE=60`, while DCS Lua uses 256.

No `lua.lib` is retained in this directory. The Windows build needs an import
library supplied by its build environment; the M5 risk register records the
current Docker override and this unresolved source-tree input. An import library
resolves `lua_*` for the linker; mlua `module` mode calls into the Lua already
loaded by the host rather than embedding another Lua VM. The retained actual-DCS
probe used `bin-mt/lua.dll`; this repository does not retain a `bin/lua.dll` hash.

The product correction for DCS's 256-byte identifier is now merged through PR
#84. It patches the vendored `mlua-sys` binding used by the product; it does not
rewrite these copied stock headers. See [M5 DCS Lua ABI](../../../docs/m5-dcs-lua-abi.md).
The actual-DCS mission evidence in the [M5 risk register](../../../docs/m5-dcs-risks.md)
predates that correction and remains evidence for the stock-header candidate.
The corrected combined product has standalone native-error controls and M4
regression evidence, but has not run a corrected DCS mission probe.
