//! Runs the C self-test of the sound initialisation probe's bookkeeping (`eech-engine-sys/csrc/eech_envobs_store.h`, `tests/csrc/envobs_store_selftest.c`, kept OUTSIDE the engine crate's `csrc` because the engine build compiles every top-level `.c` file there; EEDCS-51, D3; TEST-ONLY). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! The self-test is a small standalone C program, not a campaign and not a fixture case: it builds a 4-key store with a canary past the end of every array and drives it, with near-limit counts PRESET instead of read four billion
//! times, through the wrap path the independent review of `aa1bf9364` found (a use count that wrapped to "unseen" and appended a seen key to the first-seen order again, one past the array once the catalogue was full), the
//! order bound, the out-of-catalogue keys and the exact finite counts, first-seen order and digest. It compiles with the host C compiler (`CC`, else `cc`) on Unix; a missing compiler is a FAILURE of this test, never a skip.
//! It tests the same header the collector includes; that the collector (`eech_env_observer.c`) uses it correctly is the C compile and the engine's own reports, not this test.

#[cfg(unix)]
mod unix {
    use std::path::PathBuf;
    use std::process::Command;

    fn csrc() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../eech-engine-sys/csrc")
    }

    #[test]
    fn the_collector_bookkeeping_never_wraps_a_counter_never_writes_past_its_arrays_and_counts_the_finite_case_exactly() {
        let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
        let dir = std::env::temp_dir().join(format!("eech-envobs-store-selftest-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        let exe = dir.join("envobs_store_selftest");
        let built = Command::new(&compiler)
            .args(["-std=c99", "-Wall", "-Wextra", "-Werror", "-I"])
            .arg(csrc())
            .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/csrc/envobs_store_selftest.c"))
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap_or_else(|e| panic!("the C compiler `{compiler}` could not be run (set CC): {e}"));
        assert!(
            built.status.success(),
            "the self-test did not compile:\n{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new(&exe).output().expect("the self-test runs");
        let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            run.status.success() && stdout.contains("ENVOBS_STORE_SELFTEST OK"),
            "the self-test failed:\n{stdout}"
        );
    }
}
