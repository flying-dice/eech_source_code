# DCS Lua 5.1 link inputs

Copied from flying-dice/dcs-studio `bridge/lua5.1` (MIT), the reference DCS
native-module crate. The headers are Lua 5.1's (see `include/LICENCE.txt`).
No `lua.lib` is retained in this directory. The Windows build needs an import
library supplied by its build environment; the M5 risk register records the
current Docker override and this unresolved source-tree input. An import library
resolves `lua_*` for the linker; mlua `module` mode calls into the Lua already
loaded by the host rather than embedding another Lua VM. The retained DCS probe
used `bin-mt/lua.dll`; this repository does not retain a `bin/lua.dll` hash.
These stock headers declare `LUA_IDSIZE=60` (`include/luaconf.h`); DCS Lua uses
256. The [M5 risk register](../../../docs/m5-dcs-risks.md) records the resulting
`lua_Debug` mismatch and the separate product correction required before another
DCS campaign probe.
