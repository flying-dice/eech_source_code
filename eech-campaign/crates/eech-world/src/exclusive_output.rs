//! `host.create_exclusive (path)`: a new output file that is created atomically and exclusively, for evidence that must never overwrite anything (EEDCS-50, D3).
//!
//! Lua 5.1's `io.open` cannot do this: `"w"` truncates, and checking with `"r"` first is a race that also misses a file that exists but is unreadable. The standard `x` open
//! mode is not part of Lua 5.1 or of every C runtime this host is built for, so none is assumed. This uses the platform's own create-new open (`O_CREAT | O_EXCL` on Linux,
//! `CREATE_NEW` on Windows), which refuses ANY existing target atomically: a regular file whatever its permissions, a directory, a symbolic link (also a dangling one) and
//! any other alias. The refusal and every later failure are reported the way Lua's file methods report them (`nil` and a reason), so a writer such as
//! `json-lines.lua`'s `wrap` can check each `write`, `flush` and `close`. `json-lines.open` and every existing client are unchanged.

use mlua::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[derive(Debug)]
pub struct ExclusiveFile {
    path: String,
    /// None once closed
    file: Option<File>,
}

fn failure(lua: &Lua, reason: String) -> LuaResult<LuaMultiValue> {
    (LuaNil, reason).into_lua_multi(lua)
}

impl LuaUserData for ExclusiveFile {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("write", |lua, this, text: LuaString| {
            let Some(file) = this.file.as_mut() else {
                return failure(lua, format!("{} is closed", this.path));
            };
            match file.write_all(&text.as_bytes()) {
                Ok(()) => true.into_lua_multi(lua),
                Err(e) => failure(lua, format!("cannot write {}: {e}", this.path)),
            }
        });
        methods.add_method_mut("flush", |lua, this, ()| {
            let Some(file) = this.file.as_mut() else {
                return failure(lua, format!("{} is closed", this.path));
            };
            match file.flush() {
                Ok(()) => true.into_lua_multi(lua),
                Err(e) => failure(lua, format!("cannot flush {}: {e}", this.path)),
            }
        });
        // closing flushes the data to the device and reports a failure; closing twice is harmless
        methods.add_method_mut("close", |lua, this, ()| match this.file.take() {
            None => true.into_lua_multi(lua),
            Some(file) => match file.sync_all() {
                Ok(()) => true.into_lua_multi(lua),
                Err(e) => failure(lua, format!("cannot close {}: {e}", this.path)),
            },
        });
    }
}

/// Creates `path` if and only if nothing exists there; otherwise the reason it could not.
pub fn create_exclusive(path: &str) -> Result<ExclusiveFile, String> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => Ok(ExclusiveFile {
            path: path.to_string(),
            file: Some(file),
        }),
        Err(e) => Err(format!("cannot create {path} exclusively (nothing may already exist there): {e}")),
    }
}

pub fn register(lua: &Lua, host: &LuaTable) -> LuaResult<()> {
    host.set(
        "create_exclusive",
        lua.create_function(|lua, path: String| match create_exclusive(&path) {
            Ok(file) => lua.create_userdata(file)?.into_lua_multi(lua),
            Err(reason) => failure(lua, reason),
        })?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn scratch(name: &str) -> std::path::PathBuf {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "eech-exclusive-{}-{}-{name}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn lua_with_helper() -> Lua {
        let lua = Lua::new();
        let host = lua.create_table().unwrap();
        register(&lua, &host).unwrap();
        lua.globals().set("host", host).unwrap();
        lua
    }

    #[test]
    fn a_new_file_is_created_written_flushed_and_closed_with_every_step_checked() {
        let dir = scratch("new");
        let path = dir.join("out.jsonl");
        let lua = lua_with_helper();
        lua.globals().set("path", path.to_string_lossy().into_owned()).unwrap();
        let ok: bool = lua
            .load(
                r#"
                local file, why = host.create_exclusive (path)
                assert (file, why)
                assert (file:write ("one\n") == true)
                assert (file:write ("two\n") == true)
                assert (file:flush () == true)
                assert (file:close () == true)
                assert (file:close () == true, "a second close is harmless")
                local ok, reason = file:write ("late")
                assert (ok == nil and string.find (reason, "closed", 1, true), "a write after the close is a reported failure")
                ok, reason = file:flush ()
                assert (ok == nil and string.find (reason, "closed", 1, true), "a flush after the close is a reported failure")
                return true
                "#,
            )
            .eval()
            .unwrap();
        assert!(ok);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "one\ntwo\n");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn any_existing_target_is_refused_and_left_exactly_as_it_was() {
        let dir = scratch("existing");
        // a regular file with content, an empty one and a directory
        let full = dir.join("full");
        std::fs::write(&full, "earlier evidence\n").unwrap();
        let empty = dir.join("empty");
        std::fs::write(&empty, "").unwrap();
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();
        for target in [&full, &empty, &sub] {
            let refusal = create_exclusive(&target.to_string_lossy()).expect_err("an existing target is refused");
            assert!(refusal.contains("exclusively"), "{refusal}");
        }
        assert_eq!(std::fs::read_to_string(&full).unwrap(), "earlier evidence\n");
        assert_eq!(std::fs::read_to_string(&empty).unwrap(), "");
        assert!(sub.is_dir());
        // through Lua the refusal is `nil` and a reason, and nothing was created or truncated
        let lua = lua_with_helper();
        lua.globals().set("path", full.to_string_lossy().into_owned()).unwrap();
        let reason: String = lua
            .load(r#"local file, why = host.create_exclusive (path); assert (file == nil); return why"#)
            .eval()
            .unwrap();
        assert!(reason.contains("exclusively"), "{reason}");
        assert_eq!(std::fs::read_to_string(&full).unwrap(), "earlier evidence\n");
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// permissions do not matter to an exclusive create: a file nobody may read or write is still refused, never truncated or replaced (the check a read-open cannot make)
    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_and_symbolic_links_are_refused_too() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let dir = scratch("unix");
        let locked = dir.join("locked");
        std::fs::write(&locked, "locked evidence\n").unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
        assert!(create_exclusive(&locked.to_string_lossy()).is_err());
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(std::fs::read_to_string(&locked).unwrap(), "locked evidence\n");
        // an alias of an existing file, and a dangling one: neither is followed, and the dangling target is not created
        let alias = dir.join("alias");
        symlink(&locked, &alias).unwrap();
        assert!(create_exclusive(&alias.to_string_lossy()).is_err());
        assert_eq!(std::fs::read_to_string(&locked).unwrap(), "locked evidence\n");
        let missing = dir.join("missing");
        let dangling = dir.join("dangling");
        symlink(&missing, &dangling).unwrap();
        assert!(create_exclusive(&dangling.to_string_lossy()).is_err());
        assert!(!missing.exists(), "the dangling link's target must not be created");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn an_unwritable_location_is_a_reported_failure_not_a_panic() {
        let dir = scratch("absent");
        let inside_missing_directory = dir.join("no-such-directory").join("out.jsonl");
        let refusal = create_exclusive(&inside_missing_directory.to_string_lossy()).expect_err("a path under a missing directory cannot be created");
        assert!(refusal.contains("cannot create"), "{refusal}");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
