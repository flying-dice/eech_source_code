//! eech-harness: the headless harness of the native campaign module.
//!
//!   eech-harness run <scenario.json>   run a semantic scenario through the
//!                                      public Campaign API; JSON result on stdout
//!   eech-harness replay                replay one eech-core-ts C reference
//!                                      scenario (stdin); the C reference
//!                                      harness's output on stdout

use std::io::{Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("replay") => replay(),
        Some("run") if args.len() == 2 => run(&args[1]),
        _ => {
            eprintln!("usage: eech-harness run <scenario.json> | eech-harness replay < scenario.txt");
            ExitCode::from(64)
        }
    }
}

fn replay() -> ExitCode {
    let mut scenario = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut scenario) {
        eprintln!("eech-harness: cannot read the scenario: {e}");
        return ExitCode::from(2);
    }
    // a dedicated process: an unguarded NULL dereference is the C reference's reported outcome
    if let Err(e) = eech_campaign::conformance::install_null_dereference_reporter() {
        eprintln!("eech-harness: {e}");
    }
    let mut stdout = std::io::stdout();
    let mut sink = |text: &str| {
        // unbuffered: nothing written before a fault is lost
        let _ = stdout.write_all(text.as_bytes());
        let _ = stdout.flush();
    };
    match eech_campaign::conformance::replay_streaming(&scenario, &mut sink) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("eech-harness: replay failed: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(path: &str) -> ExitCode {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("eech-harness: cannot read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    match eech_harness::run_json(&text) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("eech-harness: {e}");
            ExitCode::from(2)
        }
    }
}
