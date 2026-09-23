//
// Verbatim extraction of original EECH C: a Rust port of the extraction
// engine of eech-core-ts/c-reference/extract.mjs (same spec kinds, same
// matching rules, same #line provenance), so the native build needs no Node.
//
// Every fragment is preceded by a #line directive to its original file.
// Extraction fails the build if an original definition cannot be found.
//

use regex::Regex;
use std::path::Path;

#[derive(Clone, Debug)]
pub enum Spec {
    /// `#include "<name>"` of a file in csrc/ (ours)
    Local(&'static str),
    /// `#include "<name>"` of a whole original header (relative to aphavoc/source or modules)
    Include(&'static str),
    /// text written as is (ours)
    Raw(String),
    Struct { name: &'static str, file: &'static str },
    Enum { name: &'static str, file: &'static str },
    Define { name: &'static str, file: &'static str },
    Prototype { name: &'static str, file: &'static str },
    Regex { pattern: &'static str, file: &'static str },
    Function { name: &'static str, signature: &'static str, file: &'static str },
    Wrap { prologue: &'static str, epilogue: &'static str, parts: Vec<Spec> },
}

pub struct Extractor<'a> {
    pub root: &'a Path,
    /// every original file read, for the closure report and rerun-if-changed
    pub read: std::cell::RefCell<std::collections::BTreeSet<String>>,
}

impl<'a> Extractor<'a> {
    pub fn new(root: &'a Path) -> Self {
        Extractor { root, read: Default::default() }
    }

    pub fn read_source(&self, file: &str) -> String {
        let path = self.root.join(file);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read original EECH source {}: {e}", path.display()));
        self.read.borrow_mut().insert(file.to_string());
        // latin1, CRLF -> LF (as extract.mjs)
        let text: String = bytes.iter().map(|&b| b as char).collect();
        text.replace("\r\n", "\n")
    }

    pub fn extract(&self, spec: &Spec) -> String {
        match spec {
            Spec::Local(name) | Spec::Include(name) => format!("#include \"{name}\"\n"),
            Spec::Raw(text) => format!("{text}\n"),
            Spec::Wrap { prologue, epilogue, parts } => {
                let body: String = parts.iter().map(|p| self.extract(p)).collect();
                format!("{prologue}\n{body}{epilogue}\n")
            }
            Spec::Function { name, signature, file } => {
                let text = self.read_source(file);
                let needle = format!("\n{signature}\n");
                let start = text.find(&needle).unwrap_or_else(|| panic!("{signature} not found in {file}")) + 1;
                let end = extract_braced(&text, start, name);
                fragment(&text, start, end, file)
            }
            Spec::Enum { name, file } | Spec::Struct { name, file } => {
                let kind = if matches!(spec, Spec::Enum { .. }) { "enum" } else { "struct" };
                let text = self.read_source(file);
                let re = Regex::new(&format!(r"\n{kind} {}\s*\n", regex::escape(name))).unwrap();
                let m = re.find(&text).unwrap_or_else(|| panic!("{kind} {name} not found in {file}"));
                let start = m.start() + 1;
                let mut end = extract_braced(&text, start, name);
                assert!(text.as_bytes()[end] == b';', "{kind} {name} not terminated by ;");
                end += 1;
                fragment(&text, start, end, file)
            }
            Spec::Define { name, file } => {
                let text = self.read_source(file);
                let re = Regex::new(&format!(r"\n#define {}\b[^\n]*", regex::escape(name))).unwrap();
                let m = re.find(&text).unwrap_or_else(|| panic!("#define {name} not found in {file}"));
                fragment(&text, m.start() + 1, m.end(), file)
            }
            Spec::Prototype { name, file } => {
                let text = self.read_source(file);
                let re = Regex::new(&format!(r"\nextern [^\n(]*\b{} \([^\n]*\);", regex::escape(name))).unwrap();
                let m = re.find(&text).unwrap_or_else(|| panic!("prototype {name} not found in {file}"));
                fragment(&text, m.start() + 1, m.end(), file)
            }
            Spec::Regex { pattern, file } => {
                let text = self.read_source(file);
                let re = Regex::new(pattern).unwrap_or_else(|e| panic!("bad pattern /{pattern}/: {e}"));
                let m = re.find(&text).unwrap_or_else(|| panic!("/{pattern}/ not found in {file}"));
                let start = m.start() + usize::from(m.as_str().starts_with('\n'));
                fragment(&text, start, m.end(), file)
            }
        }
    }
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset].matches('\n').count() + 1
}

fn fragment(text: &str, start: usize, end: usize, file: &str) -> String {
    format!("#line {} \"{}\"\n{}\n", line_of(text, start), file, &text[start..end])
}

fn extract_braced(text: &str, start: usize, what: &str) -> usize {
    let bytes = text.as_bytes();
    let open = text[start..].find('{').map(|o| o + start).unwrap_or_else(|| panic!("no body extracting {what}"));
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
    }
    panic!("unbalanced braces extracting {what}")
}

pub const BANNER: &str = "/* GENERATED by eech-sys/build (port of eech-core-ts/c-reference/extract.mjs) - verbatim original EECH C. Do not edit. */\n";

pub fn generate(ex: &Extractor, specs: &[Spec]) -> String {
    let mut out = String::from(BANNER);
    out.push('\n');
    for s in specs {
        out.push_str(&ex.extract(s));
        out.push('\n');
    }
    out
}
