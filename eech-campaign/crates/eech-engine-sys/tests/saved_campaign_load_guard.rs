//! SOURCE GUARD of the saved-campaign loader (EEDCS-49, D3, ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1). It runs in every profile and reads only text.
//!
//! It is NOT the load: a saved campaign cannot be loaded here (no retail data, and the open question is the Windows C runtime's text-mode and partial-element behaviour, which only a native run of the real Core save and the real fresh Core
//! boot of its generated script can show). It pins what the loader must keep doing so that a short or truncated delivery of the saved data can never again become a silently ZEROED world: the first real native load of the generated script
//! stated zero for all 1748 sectors although 1027 saved words were non-zero, read the session's force root as entity index 0 (the sector 0 entity) and stopped at `en_list.c:356` (docs: reference/d3-entity-collision/sector-height-native-core-load).

use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let bytes = std::fs::read(crate_dir().join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"));
    bytes.iter().map(|&b| b as char).collect()
}

fn strip_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        rest = &rest[rest.find("*/").map_or(rest.len(), |end| end + 2)..];
    }
    out.push_str(rest);
    out.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n")
}

/// the code of the `FILE_TAG_SAVED_CAMPAIGN` case of `parse_campaign_script`, up to its `break`
fn saved_campaign_case() -> String {
    let source = strip_comments(&read("../../../aphavoc/source/ai/parser/parsgen.c"));
    let start = source.find("case FILE_TAG_SAVED_CAMPAIGN:").expect("the saved campaign case");
    let rest = &source[start..];
    let end = rest.find("break;").expect("the case ends");
    rest[..end].to_string()
}

fn at(code: &str, needle: &str) -> usize {
    code.find(needle).unwrap_or_else(|| panic!("the loader no longer contains `{needle}`:\n{code}"))
}

#[test]
fn the_saved_data_is_sized_from_its_own_binary_handle_not_from_a_second_text_mode_open() {
    let code = saved_campaign_case();
    // file_size () opens the file again in TEXT mode: what a text stream counts or stops at is the C runtime's decision (the saved data holds Ctrl-Z and CRLF bytes)
    assert!(!code.contains("file_size ("), "the size must not come from the text-mode helper:\n{code}");
    let open = at(&code, "saved_file_ptr = safe_fopen (filename, \"rb\");");
    let seek_end = at(&code, "fseek (saved_file_ptr, 0, SEEK_END);");
    let tell = at(&code, "campaign_size = (int) ftell (saved_file_ptr);");
    let rewind = at(&code, "fseek (saved_file_ptr, 0, SEEK_SET);");
    assert!(
        open < seek_end && seek_end < tell && tell < rewind,
        "the length is read from the handle opened for the data, then rewound"
    );
    // `fopen (` also matches inside `safe_fopen (`: one open of any kind, and it is the safe one
    assert_eq!(
        (code.matches("fopen (").count(), code.matches("safe_fopen (").count()),
        (1, 1),
        "the data file is opened exactly once"
    );
}

#[test]
fn every_byte_must_arrive_and_the_unpack_must_not_overflow_or_the_loader_stops_by_name() {
    let code = saved_campaign_case();
    let read_call = at(&code, "bytes_read = (int) fread (campaign_buffer, 1, campaign_size, saved_file_ptr);");
    assert!(
        !code.contains("campaign_size + 1, 1,"),
        "the one-element read of size + 1 returns 0 on a short read and was never checked:\n{code}"
    );
    let short = at(&code, "if (bytes_read != campaign_size)");
    assert!(
        code[short..].contains("debug_fatal (\"PARSGEN: saved campaign data %s: read %d of its %d bytes\""),
        "a short read is a named fatal"
    );
    let version = at(&code, "server_version_number = get_list_item (load_ptr, int);");
    let unpack = at(&code, "if (unpack_session (load_ptr, campaign_size, PACK_MODE_SERVER_SESSION))");
    assert!(
        read_call < short && short < version && version < unpack,
        "full read, then the version number, then the unpack"
    );
    // pack_session sizes the data to the last partly used byte, so an unpack of what was packed never overflows: the overflow flag is a fatal, never ignored
    assert!(
        code[unpack..].contains("debug_fatal (\"PARSGEN: saved campaign data %s ended before its unpack finished"),
        "an unpack overflow is a named fatal"
    );
    assert_eq!(code.matches("unpack_session (").count(), 1, "{code}");
    // the loader still hands the data to the Core's own unpack (never a bypass, never a hand-made restore)
    assert!(
        code.contains("PACK_MODE_SERVER_SESSION") && !code.contains("campaign_model_sector_height_restore") && !code.contains("unpack_local_"),
        "{code}"
    );
}

#[test]
fn the_unpack_side_still_returns_its_overflow_and_the_pack_side_still_sizes_to_the_last_used_byte() {
    // the premise of the fatal above, from the Core's own sources: unpack_session returns the overflow flag, close_pack_buffer counts the partly used last byte
    let session = strip_comments(&read("../../../aphavoc/source/entity/system/en_comms/en_sessn.c"));
    let unpack = &session[session.find("int unpack_session (").expect("unpack_session")..];
    assert!(unpack.contains("unpack_buffer_overflow = get_unpack_buffer_overflow ();") && unpack.contains("return (unpack_buffer_overflow);"));
    let pack = strip_comments(&read("../../../aphavoc/source/entity/system/en_funcs/en_pack.c"));
    let close = &pack[pack.find("int close_pack_buffer (void)").expect("close_pack_buffer")..];
    assert!(
        close.contains("buffer_size = current_pack_buffer->ptr - current_pack_buffer->buffer + 1;"),
        "the packed size includes the partly used last byte"
    );
    // and a buffer that is exhausted answers 0 for every further value: the silent-zeros mechanism the fatal replaces
    let unsigned = &pack[pack.find("unsigned int unpack_unsigned_data (").expect("unpack_unsigned_data")..];
    assert!(unsigned.contains("if (current_unpack_buffer->overflow)") && unsigned.contains("return (0);"));
}
