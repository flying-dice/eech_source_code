// The host embeds a real Lua 5.1 (PUC liblua5.1), and the DC DLL it loads
// resolves its lua_* symbols against that same library, as a module DLL does
// against its host's Lua.
fn main() {
    // The convoy, ship, person and fixed-wing steps are EECH's own arithmetic: it takes the engine's numeric flags
    // (eech-engine-sys build) so the host and the original reference produce the same bits.
    println!("cargo::rerun-if-env-changed=EECH_ENGINE_OPT");
    for file in [
        "eech_arith.h",
        "convoy_step.c",
        "ship_step.c",
        "person_step.c",
        "fw_step.c",
        "hc_step.c",
        "fw_navigator.h",
        "hc_guidance.h",
        "death_step.c",
        "place_step.c",
        "dependent_step.c",
        "expire_step.c",
        "los_step.c",
        "sense_step.c",
        "aim_step.c",
        "wn_flight.c",
        "cmb_step.c",
        "terrain_ref.c",
        "terrain_ref.h",
        "env_ref.c",
        "env_ref.h",
    ] {
        println!("cargo::rerun-if-changed=csrc/{file}");
    }
    let opt = std::env::var("EECH_ENGINE_OPT").unwrap_or_else(|_| "1".into());
    cc::Build::new()
        .file("csrc/convoy_step.c")
        .file("csrc/ship_step.c")
        .file("csrc/person_step.c")
        .file("csrc/fw_step.c")
        .file("csrc/hc_step.c")
        .file("csrc/death_step.c")
        .file("csrc/place_step.c")
        .file("csrc/dependent_step.c")
        .file("csrc/expire_step.c")
        .file("csrc/los_step.c")
        .file("csrc/sense_step.c")
        .file("csrc/aim_step.c")
        .file("csrc/wn_flight.c")
        .file("csrc/cmb_step.c")
        .file("csrc/terrain_ref.c")
        .file("csrc/env_ref.c")
        .opt_level_str(&opt)
        .debug(true)
        .pic(true)
        .warnings(false)
        .flag("-std=gnu99")
        .flag("-w")
        .flag("-fno-strict-aliasing")
        .flag("-fwrapv")
        .flag("-ftrivial-auto-var-init=zero")
        .compile("eech_steps");
    rotor_control(&opt);
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        // a link argument (at the end of every link line of this package: the
        // binary and the tests), after mlua-sys, which references lua_*
        println!("cargo::rustc-link-arg=-llua5.1");
    } else if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        // EECH runs on the main thread: Linux gives it 8 MB of stack, Windows 1 MB
        println!("cargo::rustc-link-arg-bins=-Wl,--stack,16777216");
    }
}

// ---- the TEST-ONLY numeric control of the original rotor sound routine (EEDCS-51, D3; ENV-HOST-ROTOR-CI-1). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//
// tests/csrc/rotor_control*.{c,h} hold the arithmetic statements of update_local_helicopter_rotor_sounds in a surrogate of the engine's compilation context. They are compiled here with the SAME settings as the steps
// archive above (keep the flags in step with it), so that the control and the env_ref.c kernel are built by one compiler with one set of flags for one target. The archive is built whenever the package is built and
// is referenced by no host or module code, so a binary that does not name its symbols does not take it; only the integration test references it. What the compiler is, the full command, the target and profile, the
// numeric environment variables, the source hashes and the untouched assembly (the functions, their clones and the local functions they call, with the constant data) are recorded here, at compile time, into the
// environment of the test build and a text file the test embeds (a test run under Wine cannot read a Unix OUT_DIR). Recording never fails the build for a tool problem, but NOTHING is hidden: every command's exit
// status and output is kept, and a capture that is not complete and non-empty says so in EECH_ROTOR_CONTROL_CAPTURE_STATUS. An incomplete capture is not evidence for any numerical correction.
fn rotor_control(opt: &str) {
    use std::path::PathBuf;
    use std::process::Command;

    for file in ["rotor_control.c", "rotor_control.h", "rotor_control_body.h", "rotor_control_stubs.c"] {
        println!("cargo::rerun-if-changed=tests/csrc/{file}");
    }
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=EECH_C_OPT");
    let mut build = cc::Build::new();
    build
        .file("tests/csrc/rotor_control.c")
        .file("tests/csrc/rotor_control_stubs.c")
        .opt_level_str(opt)
        .debug(true)
        .pic(true)
        .warnings(false)
        .flag("-std=gnu99")
        .flag("-w")
        .flag("-fno-strict-aliasing")
        .flag("-fwrapv")
        .flag("-ftrivial-auto-var-init=zero");
    build.compile("eech_rotor_control");

    let tool = build.get_compiler();
    let mut problems: Vec<String> = Vec::new();
    let one_line = |text: &str| text.replace('\r', "").replace('\n', " | ");
    ran_selfcheck();
    let version = Ran::of(Command::new(tool.path()).arg("--version"));
    if let Some(problem) = version.identity_problem("the compiler version", 12) {
        problems.push(problem);
    }
    let machine = Ran::of(Command::new(tool.path()).arg("-dumpmachine"));
    if let Some(problem) = machine.identity_problem("the compiler machine", 8) {
        problems.push(problem);
    }
    let (version, machine) = (version.record(), machine.record());
    let args: Vec<String> = tool.args().iter().map(|arg| arg.to_string_lossy().into_owned()).collect();
    if args.is_empty() {
        problems.push("the compiler command has no arguments".to_string());
    }
    let command = format!("{} {} -c -o <object> <source>", tool.path().display(), args.join(" "));
    let environment = |name: &str| std::env::var(name).unwrap_or_else(|_| "(unset)".into());
    let hashes = [
        "csrc/env_ref.c",
        "csrc/env_ref.h",
        "tests/csrc/rotor_control.c",
        "tests/csrc/rotor_control.h",
        "tests/csrc/rotor_control_body.h",
        "tests/csrc/rotor_control_stubs.c",
        "build.rs",
    ]
    .iter()
    .map(|file| match std::fs::read(file) {
        Ok(bytes) if !bytes.is_empty() => format!("{file}={}", sha256_hex(&bytes)),
        Ok(_) => {
            problems.push(format!("{file} is empty"));
            format!("{file}=empty")
        }
        Err(error) => {
            problems.push(format!("{file} was not read: {error}"));
            format!("{file}=unreadable ({error})")
        }
    })
    .collect::<Vec<_>>()
    .join("; ");

    // the untouched assembly: the same command as the object, with -S (and -g0 for the debug lines only, which change no code)
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_else(|_| ".".into()));
    let mut text = String::new();
    for (source, functions) in [
        ("csrc/env_ref.c", &["env_ref_rotor_sound"][..]),
        ("tests/csrc/rotor_control.c", &["eech_rotor_control_run"][..]),
        ("tests/csrc/rotor_control_stubs.c", &["control_magnitude"][..]),
    ] {
        let target = out_dir.join(format!("{}.s", source.replace('/', "_")));
        let mut command = tool.to_command();
        command.arg("-S").arg("-g0").arg("-o").arg(&target).arg(source);
        let ran = Ran::of(&mut command);
        let (ok, output) = (ran.success, ran.record());
        text.push_str(&format!(
            "==== {source}: assembly of the unmodified source, compiled with -S -g0 added to the command above ({})\n",
            one_line(output.trim())
        ));
        match std::fs::read_to_string(&target) {
            Ok(assembly) if ok && !assembly.trim().is_empty() => {
                for function in functions {
                    let (extracted, found) = extract_symbols(&assembly, function);
                    text.push_str(&extracted);
                    if !found {
                        problems.push(format!("{function} was not found in the assembly of {source}"));
                    }
                }
            }
            Ok(_) => problems.push(format!("the assembly of {source} was not produced ({})", one_line(output.trim()))),
            Err(error) => problems.push(format!("the assembly of {source} was not read: {error}")),
        }
    }
    let status = if problems.is_empty() {
        "complete".to_string()
    } else {
        format!("INCOMPLETE: {}", problems.join(" ; "))
    };
    text.insert_str(0, &format!("capture status: {status}\n"));
    for (name, value) in [
        ("EECH_ROTOR_CONTROL_CAPTURE_STATUS", one_line(&status)),
        ("EECH_ROTOR_CONTROL_CC_VERSION", one_line(version.trim())),
        ("EECH_ROTOR_CONTROL_CC_MACHINE", one_line(machine.trim())),
        ("EECH_ROTOR_CONTROL_CC_COMMAND", one_line(&command)),
        ("EECH_ROTOR_CONTROL_TARGET", environment("TARGET")),
        ("EECH_ROTOR_CONTROL_HOST", environment("HOST")),
        ("EECH_ROTOR_CONTROL_PROFILE", environment("PROFILE")),
        ("EECH_ROTOR_CONTROL_OPT_LEVEL", opt.to_string()),
        ("EECH_ROTOR_CONTROL_EECH_ENGINE_OPT", environment("EECH_ENGINE_OPT")),
        ("EECH_ROTOR_CONTROL_EECH_C_OPT", environment("EECH_C_OPT")),
        ("EECH_ROTOR_CONTROL_SOURCE_HASHES", one_line(&hashes)),
    ] {
        println!("cargo::rustc-env={name}={value}");
    }
    let path = out_dir.join("rotor_control_assembly.txt");
    if let Err(error) = std::fs::write(&path, text) {
        panic!("the rotor control assembly record {} could not be written: {error}", path.display());
    }
}

/// One command's result with its exit status, standard output and standard error kept apart (a command that could not be run has no output and says so in its status).
struct Ran {
    success: bool,
    status: String,
    stdout: String,
    stderr: String,
}

impl Ran {
    fn of(command: &mut std::process::Command) -> Ran {
        match command.output() {
            Ok(output) => Ran {
                success: output.status.success(),
                status: format!("exit {}", output.status),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            },
            Err(error) => Ran {
                success: false,
                status: format!("could not run: {error}"),
                stdout: String::new(),
                stderr: String::new(),
            },
        }
    }

    /// The record kept: the actual status and the actual output.
    fn record(&self) -> String {
        format!("{}: {}{}", self.status, self.stdout, self.stderr)
    }

    /// Why this command's output is not an adequate identity (a successful exit AND at least `shortest` characters of trimmed standard output, judged before any status text is added), or None.
    fn identity_problem(&self, what: &str, shortest: usize) -> Option<String> {
        let text = self.stdout.trim();
        if !self.success {
            Some(format!(
                "{what} was not read (the command failed): {}",
                self.record().replace('\r', "").replace('\n', " | ")
            ))
        } else if text.len() < shortest {
            Some(format!(
                "{what} was not read (the command succeeded but its output has {} characters, fewer than {shortest}): {}",
                text.len(),
                self.record().replace('\r', "").replace('\n', " | ")
            ))
        } else {
            None
        }
    }
}

/// A finite check of the judgement above, made on every build: a zero exit with empty or blank output and a non-zero exit with plenty of text are incomplete, a genuine identity is not. A wrong judgement stops the build.
fn ran_selfcheck() {
    let made = |success: bool, stdout: &str| Ran {
        success,
        status: if success {
            "exit exit status: 0".into()
        } else {
            "exit exit status: 1".into()
        },
        stdout: stdout.to_string(),
        stderr: String::new(),
    };
    assert!(
        made(true, "").identity_problem("x", 12).is_some(),
        "an empty successful output was accepted as an identity"
    );
    assert!(
        made(true, " \n\t ").identity_problem("x", 12).is_some(),
        "a blank successful output was accepted as an identity"
    );
    assert!(
        made(false, "gcc (Ubuntu 12.4.0-2ubuntu1~24.04.1) 12.4.0").identity_problem("x", 12).is_some(),
        "a failed command with output was accepted as an identity"
    );
    assert!(
        made(true, "gcc (Ubuntu 12.4.0-2ubuntu1~24.04.1) 12.4.0\n").identity_problem("x", 12).is_none(),
        "a genuine identity was refused"
    );
    assert!(
        made(true, "x86_64-w64-mingw32\n").identity_problem("x", 8).is_none(),
        "a genuine machine was refused"
    );
}

/// The text of one label of a GNU assembly listing, from the label to its end directive, with the data of the `.LC` constants it names.
fn extract_body(lines: &[&str], label: &str, role: &str) -> String {
    let head = format!("{label}:");
    let Some(start) = lines.iter().position(|line| *line == head) else {
        return format!("---- {role} {label}: label not found\n");
    };
    let mut end = start;
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        end = index;
        let line = line.trim();
        if (line.starts_with(".size") && line.contains(label)) || line == ".seh_endproc" || line == ".cfi_endproc" {
            break;
        }
    }
    let body = lines.get(start..=end).unwrap_or(&[]).join("\n");
    let mut text = format!("---- {role} {label}\n{body}\n");
    let mut constants: Vec<&str> = Vec::new();
    for token in body.split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_')) {
        if token.starts_with(".LC") && !constants.contains(&token) {
            constants.push(token);
        }
    }
    for constant in constants {
        let head = format!("{constant}:");
        if let Some(at) = lines.iter().position(|line| *line == head) {
            text.push_str(&format!("---- constant {constant}\n{head}\n"));
            for line in lines.iter().skip(at + 1) {
                let data = [
                    ".long", ".quad", ".int", ".float", ".double", ".string", ".ascii", ".zero", ".align", ".p2align", ".byte", ".value", ".word",
                ];
                if data.iter().any(|d| line.trim_start().starts_with(d)) {
                    text.push_str(line);
                    text.push('\n');
                } else {
                    break;
                }
            }
        }
    }
    text
}

/// The function `name` of a GNU assembly listing as the compiler actually emitted it: the function itself, every clone of it (`name.constprop.0`, `name.isra.0`, `name.part.0`, `name.cold`, ...), and the functions of
/// this translation unit that these call (one level, which is how an executed callee such as a specialised helper is found), each with the data of the constants it names; the calls to symbols this translation unit does
/// not define are listed. When the name is not a label it was inlined, renamed or removed, and that is said (and returned as not found). Whether a function is the one executed is read from the calls, not assumed.
fn extract_symbols(assembly: &str, name: &str) -> (String, bool) {
    let lines: Vec<&str> = assembly.lines().collect();
    let labels: Vec<&str> = lines
        .iter()
        .filter(|line| line.ends_with(':') && !line.starts_with(char::is_whitespace) && !line.starts_with(".L"))
        .map(|line| line.trim_end_matches(':'))
        .collect();
    let clone_prefix = format!("{name}.");
    let family: Vec<&str> = labels
        .iter()
        .copied()
        .filter(|label| *label == name || label.starts_with(&clone_prefix))
        .collect();
    if family.is_empty() {
        let near: Vec<&str> = labels.iter().copied().filter(|label| label.contains(name)).collect();
        return (
            format!("---- function {name}: NO LABEL (inlined, renamed or removed); labels containing the name: {near:?}\n"),
            false,
        );
    }
    let mut text = String::new();
    let mut shown: Vec<String> = Vec::new();
    let mut external: Vec<String> = Vec::new();
    for label in &family {
        let body = extract_body(&lines, label, if *label == name { "function" } else { "clone of the function" });
        for line in body.lines() {
            let mut words = line.split_whitespace();
            if let (Some(op), Some(target)) = (words.next(), words.next()) {
                if op.starts_with("call") || op.starts_with("jmp") {
                    let target = target.trim_start_matches('*').split("@PLT").next().unwrap_or(target);
                    if labels.contains(&target) && !family.contains(&target) {
                        if !shown.iter().any(|known| known == target) {
                            shown.push(target.to_string());
                        }
                    } else if !labels.contains(&target) && !target.starts_with(".L") && !external.iter().any(|known| known == target) {
                        external.push(target.to_string());
                    }
                }
            }
        }
        text.push_str(&body);
    }
    for callee in &shown {
        text.push_str(&extract_body(&lines, callee, "local callee (executed form as emitted)"));
    }
    text.push_str(&format!("---- calls to symbols this translation unit does not define: {external:?}\n"));
    (text, true)
}

/// SHA-256 (FIPS 180-4), for the source hashes the control records; the package has no hashing dependency.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut state: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for block in message.chunks(64) {
        let mut w = [0u32; 64];
        for (slot, word) in w.iter_mut().zip(block.chunks(4)) {
            *slot = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for (constant, word) in K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let t1 = h.wrapping_add(s1).wrapping_add(choice).wrapping_add(*constant).wrapping_add(*word);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}
