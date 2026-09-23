//! Differential replay of the eech-core-ts corpus (corpus/*.jsonl.gz): the
//! native module against the output of the original 32-bit C reference,
//! with the TSTL port's recorded verdict as the third implementation.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use serde::Deserialize;

/// one campaign per process: the corpus tests take turns
static KERNEL: Mutex<()> = Mutex::new(());

#[derive(Deserialize)]
struct Entry {
    family: String,
    id: String,
    input: String,
    c: Vec<String>,
    ts: String,
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

fn load(path: &Path) -> Vec<Entry> {
    let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut text = String::new();
    flate2::read::GzDecoder::new(file).read_to_string(&mut text).unwrap();
    text.lines().filter(|l| !l.is_empty()).map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn replay_in_process(input: &str) -> Result<Vec<String>, String> {
    eech_campaign::conformance::replay(input).map(|r| r.lines).map_err(|e| e.to_string())
}

fn replay_in_subprocess(input: &str) -> Result<Vec<String>, String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_eech-harness"))
        .arg("replay")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child.stdin.take().unwrap().write_all(input.as_bytes()).map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("replay process failed: {:?} {}", out.status, String::from_utf8_lossy(&out.stderr)));
    }
    let mut lines: Vec<String> = String::from_utf8_lossy(&out.stdout).split('\n').map(str::to_string).collect();
    if lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    Ok(lines)
}

fn first_difference(native: &[String], c: &[String]) -> String {
    for (i, (a, b)) in native.iter().zip(c).enumerate() {
        if a != b {
            return format!("line {i}: native {a:?} vs C {b:?}");
        }
    }
    format!("length: native {} lines vs C {}", native.len(), c.len())
}

fn check(file: &str) {
    let _guard = KERNEL.lock().unwrap_or_else(|e| e.into_inner());
    let path = corpus_dir().join(file);
    if !path.exists() {
        eprintln!("{file}: absent (optional corpus), skipped");
        return;
    }
    let entries = load(&path);
    let databases = eech_campaign::conformance::build::database_digest();
    let mut per_family: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    let mut failures = Vec::new();
    for e in &entries {
        let subprocess = e.c.iter().any(|l| l == "result null-dereference") || std::env::var_os("EECH_CORPUS_SUBPROCESS").is_some();
        let native = if subprocess { replay_in_subprocess(&e.input) } else { replay_in_process(&e.input) };
        let f = per_family.entry(e.family.clone()).or_default();
        f.0 += 1;
        if subprocess {
            f.3 += 1;
        }
        if e.ts == "agrees" {
            f.2 += 1;
        }
        match native {
            Ok(lines) if lines == e.c => f.1 += 1,
            Ok(lines) => failures.push(format!("{} {}: {}", e.family, e.id, first_difference(&lines, &e.c))),
            Err(err) => failures.push(format!("{} {}: {err}", e.family, e.id)),
        }
    }
    assert_eq!(eech_campaign::conformance::build::database_digest(), databases, "the corpus changed an original database (docs/global-state.md)");
    eprintln!("{file}: family / scenarios / native == C / TS == C / via subprocess");
    for (family, (n, native, ts, sub)) in &per_family {
        eprintln!("  {family:28} {n:5} {native:5} {ts:5} {sub:5}");
    }
    assert!(failures.is_empty(), "{} of {} scenarios differ from the C reference:\n{}", failures.len(), entries.len(), failures.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
fn recorded_scenarios_match_the_c_reference() {
    check("scenarios.jsonl.gz");
}

#[test]
fn float_semantics_match_the_c_reference() {
    check("float32.jsonl.gz");
}

#[test]
fn fresh_scenarios_match_the_c_reference() {
    // generated on demand: EECH_CORPUS_FRESH=N tools/corpus/export.sh
    check("scenarios-fresh.jsonl.gz");
    check("float32-fresh.jsonl.gz");
}
