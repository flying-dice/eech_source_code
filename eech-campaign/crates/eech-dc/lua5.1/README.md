# DCS Lua 5.1 link inputs

Copied from flying-dice/dcs-studio `bridge/lua5.1` (MIT), the reference DCS
native-module crate. `lua.lib` is an import library for DCS World's own
`bin/lua.dll`; it only lets the Windows linker resolve `lua_*` in the cdylib,
which calls into the Lua that DCS has already loaded (mlua `module` mode never
links Lua into the DLL). The headers are Lua 5.1's (see `include/LICENCE.txt`).
