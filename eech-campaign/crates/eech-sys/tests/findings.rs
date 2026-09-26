//! Evidence for the findings in docs/64-bit.md and docs/global-state.md.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

/// B1: en_creat.c's `pargs_buffer = (char *) pargs` only reads the variadic
/// attribute list on i386, where a va_list points into the argument stack.
#[test]
fn b1_va_list_reinterpretation_is_an_i386_only_idiom() {
    let (ok, read) = eech_sys::probe::va_list_reinterpretation();
    if cfg!(target_arch = "x86") {
        assert!(ok, "on i386 the original reads the list: {read:?}");
        assert_eq!(read, [11, 1001, 12, 1002, 13, 1003]);
    } else if cfg!(all(target_arch = "x86_64", not(windows))) {
        // System V: a va_list is a register-save descriptor, whose first words
        // are gp_offset (16: two named integer arguments) and fp_offset (48)
        assert!(!ok);
        assert_eq!(&read[..2], &[16, 48], "the original reads the va_list descriptor, not the attributes");
    } else {
        assert!(!ok, "no other ABI lays variadic ints out as the i386 stack: {read:?}");
    }
    // patch P1's marshaller reads the same list correctly everywhere
    assert_eq!(eech_sys::probe::marshalled(), (true, [11, 1001, 12, 1002, 13, 1003]));
}

fn writable_symbols() -> Option<BTreeMap<String, BTreeSet<String>>> {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let mut symbols: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for object in eech_sys::build_info::KERNEL_OBJECTS.split(separator) {
        let out = Command::new("nm").arg("--defined-only").arg(object).output().ok()?;
        assert!(out.status.success(), "nm {object}");
        let unit = std::path::Path::new(object)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .split_once('-')
            .map(|(_, u)| u.to_string())
            .unwrap_or_default();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() != 3 || !matches!(fields[1], "B" | "b" | "D" | "d" | "C" | "G" | "g" | "S" | "s") {
                continue;
            }
            // function-local statics: gcc `name.N`, clang `function.name`
            let raw = fields[2];
            let name = match raw.rsplit_once('.') {
                Some((base, suffix)) if suffix.chars().all(|c| c.is_ascii_digit()) => base,
                Some((_, local)) => local,
                None => raw,
            };
            symbols.entry(name.to_string()).or_default().insert(unit.clone());
        }
    }
    Some(symbols)
}

/// Every writable global of the kernel is classified in global-state.txt.
#[test]
fn every_writable_global_is_classified() {
    let Some(found) = writable_symbols() else {
        eprintln!("nm is not available: global state inventory skipped");
        return;
    };
    let manifest = include_str!("../global-state.txt");
    let classified: BTreeSet<String> = manifest
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.split_whitespace().next().unwrap().to_string())
        .collect();
    let unclassified: Vec<String> = found
        .iter()
        .filter(|(s, _)| !classified.contains(*s))
        .map(|(s, u)| format!("{s} ({})", u.iter().cloned().collect::<Vec<_>>().join(", ")))
        .collect();
    let gone: Vec<&String> = classified.iter().filter(|s| !found.contains_key(*s)).collect();
    assert!(
        unclassified.is_empty(),
        "writable globals missing from global-state.txt:\n{}",
        unclassified.join("\n")
    );
    assert!(gone.is_empty(), "global-state.txt lists symbols the kernel no longer has: {gone:?}");
}
