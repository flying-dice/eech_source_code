//! The lua_Debug that mlua allocates in this DLL must hold the record DCS World's Lua 5.1 writes into it.
//!
//! lua_getstack and lua_getinfo write the host Lua's whole lua_Debug, and mlua allocates the
//! record itself (luaL_traceback on every failing native callback). DCS World's lua.dll is built
//! with LUA_IDSIZE 256: on x86_64 its short_src starts at 0x38 and i_ci, its last field, is at
//! 0x138 (320 bytes), where stock Lua 5.1 writes 120. A record smaller than the host's is a
//! stack overrun on the first callback error (docs/m5-dcs-lua-abi.md).
use mlua::ffi::lua_Debug;
use std::ffi::c_int;
use std::mem::{offset_of, size_of};

/// LUA_IDSIZE of DCS World's lua.dll (docs/m5-dcs-lua-abi.md: from the measured 316-byte extent and i_ci at 0x138)
const DCS_LUA_IDSIZE: usize = 256;

#[test]
fn lua_debug_holds_the_dcs_world_record() {
    // the host's record: the fields before short_src, short_src, then i_ci
    let dcs_record = offset_of!(lua_Debug, short_src) + DCS_LUA_IDSIZE + size_of::<c_int>();
    assert!(
        size_of::<lua_Debug>() >= dcs_record,
        "mlua-sys lua_Debug is {} bytes, DCS World's Lua writes {dcs_record}",
        size_of::<lua_Debug>()
    );
}

/// mlua reads the fields before short_src (and short_src itself): they are at DCS World's offsets
/// only while short_src starts where DCS World's does (0x38 on x86_64: the stock field order, consistent with the measured 316 and 0x138).
#[cfg(target_pointer_width = "64")]
#[test]
fn lua_debug_short_src_starts_where_dcs_world_writes_it() {
    assert_eq!(offset_of!(lua_Debug, short_src), 0x38);
}
