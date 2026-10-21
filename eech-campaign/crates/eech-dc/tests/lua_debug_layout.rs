//! The lua_Debug that mlua allocates in this DLL must hold the record DCS World's Lua 5.1 writes into it.
//!
//! lua_getstack and lua_getinfo write into the lua_Debug the caller supplies, and mlua allocates
//! that record itself on callback error paths that request a traceback. DCS World's lua.dll is
//! built with LUA_IDSIZE 256: on x86_64 its short_src starts at 0x38 and i_ci, its last field, is
//! at 0x138. One DCS lua.dll was measured writing 316 bytes through the end of i_ci; the allocated
//! record is 320 bytes because of trailing alignment padding. The measured PUC Lua 5.1 DLL writes
//! 120 bytes. A record smaller than the host writes is a stack overrun
//! (docs/m5-dcs-lua-abi.md).
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
