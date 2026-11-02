//! The host's diagnostic output primitive: `host.output_create_new (path)` (EEDCS-51, D3; TEST-ONLY drivers). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! Characterization drivers write evidence files that must never overwrite, truncate or share an existing file. Portable Lua cannot promise that: `io.open (path, "w")` and `"wb"` truncate, a read-open followed by a
//! write-open is not atomic, and a Windows Lua 5.1 host measured by Root (`sound-init-exclusive-output-runtime-20261101/receipt.json`, actual 0) does NOT create a file for the mode `"wbx"` at all, so nothing here relies on it.
//! This primitive is the operating system's create-new open (`CREATE_NEW` on Windows, `O_CREAT | O_EXCL` elsewhere, through `OpenOptions::create_new`): in ONE atomic step it creates the file, or fails when anything of that
//! name exists, whether it is readable or not, a file, a directory or a link, and whatever spelling of the path names it. A second spelling of a file this run has just created therefore fails, which is how a driver
//! that creates its output and its sidecar through it refuses two names for one file BEFORE anything is booted. Nothing is truncated, nothing is retried and there is no fall back.
//!
//! The handle it returns has `write (text)`, `flush ()` and `close ()`; each returns `true`, or `nil` and a message, and a closed handle refuses them. `flush` makes the data durable (`sync_data`) and `close` makes the
//! file durable (`sync_all`) before it is released. It adds no campaign, world or physical protocol; it is a host-side file helper for scripts, like `host.log`.

use mlua::prelude::*;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::Write;

/// Creates `path` if and only if nothing of that name exists, atomically (the operating system's create-new open).
pub fn create_new(path: &str) -> std::io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

/// The Lua handle of a file created by [`create_new`].
pub struct Output {
    file: RefCell<Option<File>>,
}

fn failure(message: impl std::fmt::Display) -> LuaResult<(Option<bool>, Option<String>)> {
    Ok((None, Some(message.to_string())))
}

impl LuaUserData for Output {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("write", |_, this, text: LuaString| {
            let mut slot = this.file.borrow_mut();
            let Some(file) = slot.as_mut() else {
                return failure("the output is closed");
            };
            match file.write_all(&text.as_bytes()) {
                Ok(()) => Ok((Some(true), None)),
                Err(error) => failure(error),
            }
        });
        methods.add_method("flush", |_, this, ()| {
            let mut slot = this.file.borrow_mut();
            let Some(file) = slot.as_mut() else {
                return failure("the output is closed");
            };
            match file.flush().and_then(|()| file.sync_data()) {
                Ok(()) => Ok((Some(true), None)),
                Err(error) => failure(error),
            }
        });
        methods.add_method("close", |_, this, ()| {
            let mut slot = this.file.borrow_mut();
            let Some(file) = slot.take() else {
                return failure("the output is closed");
            };
            match file.sync_all() {
                Ok(()) => Ok((Some(true), None)),
                Err(error) => failure(error),
            }
        });
    }
}

/// Installs `host.output_create_new (path)`: a handle, or `nil` and the operating system's message (for a name that exists: "already exists").
pub fn install(lua: &Lua, host: &LuaTable) -> LuaResult<()> {
    host.set(
        "output_create_new",
        lua.create_function(|_, path: String| -> LuaResult<(Option<Output>, Option<String>)> {
            match create_new(&path) {
                Ok(file) => Ok((
                    Some(Output {
                        file: RefCell::new(Some(file)),
                    }),
                    None,
                )),
                Err(error) => Ok((None, Some(format!("{error} ({:?})", error.kind())))),
            }
        })?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A private scratch directory on a REAL filesystem, removed when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("eech-output-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
            std::fs::create_dir_all(&dir).expect("a scratch directory");
            Scratch(dir)
        }

        fn path(&self, name: &str) -> String {
            self.0.join(name).to_string_lossy().into_owned()
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(entries) = std::fs::read_dir(&self.0) {
                    for entry in entries.flatten() {
                        let _ = std::fs::set_permissions(entry.path(), std::fs::Permissions::from_mode(0o600));
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn kind(result: std::io::Result<File>) -> Option<std::io::ErrorKind> {
        result.err().map(|e| e.kind())
    }

    #[test]
    fn a_fresh_name_is_created_and_every_second_spelling_or_existing_file_is_refused_and_left_alone() {
        let scratch = Scratch::new();
        let first = scratch.path("capture.jsonl");
        let mut file = create_new(&first).expect("a fresh name is created");
        file.write_all(b"evidence").expect("written");
        drop(file);
        // the same name again, another spelling of it (a `.` component, a doubled separator), a hard link to it and a symbolic link to it: all refuse, and the evidence is untouched
        let dot = format!("{}{}./capture.jsonl", scratch.0.to_string_lossy(), std::path::MAIN_SEPARATOR);
        let doubled = format!(
            "{}{}{}capture.jsonl",
            scratch.0.to_string_lossy(),
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        );
        assert_eq!(kind(create_new(&first)), Some(std::io::ErrorKind::AlreadyExists));
        assert_eq!(kind(create_new(&dot)), Some(std::io::ErrorKind::AlreadyExists), "{dot}");
        assert_eq!(kind(create_new(&doubled)), Some(std::io::ErrorKind::AlreadyExists), "{doubled}");
        let link = scratch.path("hard.jsonl");
        std::fs::hard_link(&first, &link).expect("a hard link");
        assert_eq!(
            kind(create_new(&link)),
            Some(std::io::ErrorKind::AlreadyExists),
            "a second NAME of the same file"
        );
        #[cfg(unix)]
        {
            let symbolic = scratch.path("symbolic.jsonl");
            std::os::unix::fs::symlink(&first, &symbolic).expect("a symbolic link");
            assert_eq!(kind(create_new(&symbolic)), Some(std::io::ErrorKind::AlreadyExists));
        }
        assert_eq!(
            std::fs::read(&first).expect("readable"),
            b"evidence",
            "an existing file was changed by a refused create"
        );
    }

    #[test]
    fn an_existing_file_that_cannot_be_read_is_refused_and_not_truncated() {
        let scratch = Scratch::new();
        let path = scratch.path("write-only.raw");
        std::fs::write(&path, b"precious").expect("an existing file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o200)).expect("write-only permissions");
        }
        assert_eq!(kind(create_new(&path)), Some(std::io::ErrorKind::AlreadyExists));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("restored");
        }
        assert_eq!(
            std::fs::read(&path).expect("readable again"),
            b"precious",
            "an existing file that could not be read was truncated"
        );
    }

    #[test]
    fn a_directory_a_dangling_link_and_a_missing_parent_are_refused() {
        let scratch = Scratch::new();
        std::fs::create_dir(scratch.0.join("dir")).expect("a directory");
        assert!(create_new(&scratch.path("dir")).is_err());
        assert!(create_new(&scratch.path("missing/child.raw")).is_err());
        #[cfg(unix)]
        {
            use std::path::Path;
            let dangling = scratch.path("dangling.raw");
            std::os::unix::fs::symlink(scratch.path("nowhere"), &dangling).expect("a dangling link");
            assert_eq!(kind(create_new(&dangling)), Some(std::io::ErrorKind::AlreadyExists));
            assert!(
                !Path::new(&scratch.path("nowhere")).exists(),
                "a dangling link was followed and its target created"
            );
        }
    }

    fn host_with_the_primitive() -> (Lua, LuaTable) {
        let lua = Lua::new();
        let host = lua.create_table().expect("a table");
        install(&lua, &host).expect("installed");
        lua.globals().set("host", host.clone()).expect("a global");
        (lua, host)
    }

    #[test]
    fn the_lua_handle_writes_flushes_and_closes_with_checked_results_and_a_closed_handle_refuses() {
        let scratch = Scratch::new();
        let path = scratch.path("out.jsonl");
        let (lua, _host) = host_with_the_primitive();
        lua.globals().set("path", path.clone()).expect("a global");
        let outcome: Vec<String> = lua
            .load(
                r#"
                local out, err = host.output_create_new (path)
                assert (out ~= nil and err == nil, tostring (err))
                local a = out:write ("one ")
                local b = out:write ("two\0three")
                local c = out:flush ()
                local d = out:close ()
                local e, e_err = out:write ("late")
                local f, f_err = out:close ()
                local second, second_err = host.output_create_new (path)
                return { tostring (a), tostring (b), tostring (c), tostring (d), tostring (e), tostring (e_err), tostring (f), tostring (f_err), tostring (second), tostring (second_err) }
                "#,
            )
            .eval()
            .expect("the script runs");
        assert_eq!(&outcome[0..4], ["true", "true", "true", "true"]);
        assert_eq!(&outcome[4..8], ["nil", "the output is closed", "nil", "the output is closed"]);
        assert_eq!(outcome[8], "nil");
        assert!(outcome[9].contains("AlreadyExists"), "{}", outcome[9]);
        assert_eq!(
            std::fs::read(&path).expect("the file"),
            b"one two\0three",
            "the data, including a NUL, is on disk after close"
        );
    }

    #[test]
    fn two_names_for_one_file_cannot_both_be_created_through_the_host_primitive() {
        let scratch = Scratch::new();
        let (lua, _host) = host_with_the_primitive();
        lua.globals().set("plain", scratch.path("capture.jsonl")).expect("a global");
        lua.globals()
            .set(
                "alias",
                format!(
                    "{}{}.{}capture.jsonl",
                    scratch.0.to_string_lossy(),
                    std::path::MAIN_SEPARATOR,
                    std::path::MAIN_SEPARATOR
                ),
            )
            .expect("a global");
        let outcome: (bool, bool, String) = lua
            .load(
                r#"
                local first = host.output_create_new (plain)
                local second, err = host.output_create_new (alias)
                return first ~= nil, second ~= nil, tostring (err)
                "#,
            )
            .eval()
            .expect("the script runs");
        assert!(outcome.0 && !outcome.1, "{outcome:?}");
        assert!(outcome.2.contains("AlreadyExists"), "{}", outcome.2);
    }
}
