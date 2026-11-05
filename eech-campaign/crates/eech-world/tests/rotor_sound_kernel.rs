//! The host's rotor acoustics kernel (`eech_world::env::rotor_sound`, EEDCS-51, D3) against the MEASURED words of the ORIGINAL `update_local_helicopter_rotor_sounds`. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The oracle is `tests/data/rotor_sound_oracle.tsv`: the inputs and the written output words, and the ordered pause and resume requests, that the original rotor probe recorded in the independently closed native
//! e54423b5c / driver 06de242ae runs -- the 12 AI and 15 flight model controlled sweep rows (including the five rows whose unassigned locals are zero by the canonical build's `-ftrivial-auto-var-init=zero`, F1) and
//! every distinct input of the natural 120 s record (208800 calls, 261 distinct AI inputs). The file is data copied from those retained records by a one-off extraction (header: provenance); NO second solver is written
//! here, and the kernel is not compared with itself. The natural record holds AI helicopters without a flight model only; every other shape is covered by the sweeps alone, which are world-input variation.
//!
//! What the kernel adds to the original is only its transport (32-bit words, the canonical arithmetic environment, the caller's floating-point state restored); the numbers must be the original's, bit for bit.
//!
//! CHARACTERISATION BEFORE THE FIX (ENV-HOST-ROTOR-CI-1; UNMEASURED until a build prints it). The oracle is the Windows x64 original; the CI jobs on i686 Linux (gcc-12, x87) and on Windows x64 under Wine (MinGW) each
//! differ from it by one ulp in one row. Before every oracle assertion below this file therefore prints (straight to the process's stderr, which the test harness does not capture, so a passing job shows it too) a
//! CONTROL: the arithmetic statements of the routine, token-identical to the original after three declared changes (tests/csrc/rotor_control_body.h), compiled by the compiler and with the flags that built the kernel for
//! this target, in a SURROGATE of the engine's compilation context (collaborators in the engine's shape with stub bodies, reduced layouts: NOT the engine's code generation), words in and words out. Each of four
//! variants (unmodified or STAGED -- diagnostic changed code that stores each intermediate to an addressable float and reloads it -- and in the default or the requested canonical floating-point environment) starts from
//! the same captured baseline, has its own save and restore, and is reported with the floating-point states measured before, at entry, after the arithmetic and after the restoration; they are set next to the kernel and
//! the oracle for every sweep row, with the compiler, the full command, the capture status, the target and profile, the source hashes and the untouched assembly (as emitted, with the clones and local callees) of the
//! kernel and of the control. The control is test-only reference data: it is not a second runtime, the host and the module do not reference it, and no oracle assertion in this file depends on it; a capture that is
//! not complete is not evidence for any correction. It says what a target computes, not which answer is right.

use eech_world::env::{rotor_sound, RotorPlayer, RotorSoundInput};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;

extern "C" {
    fn eech_rotor_control_run(staged: i32, player: i32, have_flight_model: i32, in_w: *const u32, out_w: *mut u32, stage_w: *mut u32);
    fn eech_rotor_control_facts() -> *const std::os::raw::c_char;
}

/// the untouched assembly of the kernel and of the control, captured by build.rs with the compiler that built them (embedded: a test run under Wine cannot read the Unix build directory)
const ASSEMBLY: &str = include_str!(concat!(env!("OUT_DIR"), "/rotor_control_assembly.txt"));

const ORACLE: &str = include_str!("data/rotor_sound_oracle.tsv");

fn word(text: &str) -> u32 {
    u32::from_str_radix(text, 16).unwrap_or_else(|e| panic!("{text:?} is not a word: {e}"))
}

fn value(text: &str) -> f32 {
    f32::from_bits(word(text))
}

/// One retained row of the oracle: the inputs as words, and the written amplification and pitch words (`None` where the original had no child to write).
struct Row {
    source: String,
    index: String,
    player: i32,
    have_flight_model: bool,
    inputs: [u32; 7],
    amplification: Vec<Option<u32>>,
    pitch: Vec<Option<u32>>,
}

fn oracle_rows() -> Vec<Row> {
    let optional = |text: &str| (text != "-").then(|| word(text));
    ORACLE
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let row = line.split('\t').collect::<Vec<_>>();
            let inputs = row[4].split(',').map(word).collect::<Vec<u32>>();
            Row {
                source: row[0].to_string(),
                index: row[1].to_string(),
                player: row[2].parse().expect("a player"),
                have_flight_model: row[3] == "1",
                inputs: inputs.try_into().expect("seven input words"),
                amplification: row[5].split(',').map(optional).collect(),
                pitch: row[6].split(',').map(optional).collect(),
            }
        })
        .collect()
}

/// Where words differ from the oracle's, as text.
fn differences(row: &Row, amplification: &[u32], pitch: &[u32]) -> Vec<String> {
    let mut found = Vec::new();
    for (channel, want) in row.amplification.iter().enumerate() {
        if let (Some(want), Some(got)) = (want, amplification.get(channel)) {
            if want != got {
                found.push(format!("amp{channel} want {want:08x} got {got:08x}"));
            }
        }
    }
    for (channel, want) in row.pitch.iter().enumerate() {
        if let (Some(want), Some(got)) = (want, pitch.get(channel)) {
            if want != got {
                found.push(format!("pitch{channel} want {want:08x} got {got:08x}"));
            }
        }
    }
    found
}

fn kernel_words(row: &Row) -> Result<([u32; 6], [u32; 6]), String> {
    let player = match row.player {
        0 => RotorPlayer::Ai,
        1 => RotorPlayer::Local,
        _ => RotorPlayer::Remote,
    };
    let inputs = row.inputs.map(f32::from_bits);
    let sound = rotor_sound(&RotorSoundInput {
        player,
        rpm: inputs[0],
        acceleration: row.have_flight_model.then(|| [inputs[1], inputs[2], inputs[3]]),
        engines: [inputs[4], inputs[5], inputs[6]],
    })?;
    Ok((sound.amplification.map(f32::to_bits), sound.pitch.map(f32::to_bits)))
}

/// The floating-point state a diagnostic variant is bracketed by: the x87 environment image (control, status and tag words and the pointers) and the complete MXCSR. The x87 data registers, the XMM data and every other
/// thread are NOT part of it, and nothing is inferred from it about the engine's startup state or the public caller's.
#[derive(Clone, Copy, Debug)]
struct FpState {
    x87: [u8; 28],
    mxcsr: u32,
}

impl FpState {
    fn word(&self, at: usize) -> u16 {
        self.x87.get(at..at + 2).map_or(0, |bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// the control, status and tag words and the MXCSR, which is what a restoration is judged by
    fn identity(&self) -> (u16, u16, u16, u32) {
        (self.word(0), self.word(4), self.word(8), self.mxcsr)
    }

    fn text(&self) -> String {
        let (control, status, tag, mxcsr) = self.identity();
        format!("x87 cw={control:04x} sw={status:04x} tw={tag:04x} mxcsr={mxcsr:08x}")
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod fp {
    use super::FpState;
    use std::arch::asm;

    /// The caller's state as it is now (fnstenv masks every x87 exception, so the image is loaded straight back).
    pub fn capture() -> FpState {
        let mut x87 = [0u8; 28];
        let mut mxcsr = 0u32;
        // SAFETY: stores the x87 environment and MXCSR to locals and loads the same x87 image back.
        unsafe {
            asm!("fnstenv [{}]", in(reg) x87.as_mut_ptr(), options(nostack));
            asm!("fldenv [{}]", in(reg) x87.as_ptr(), options(nostack, readonly));
            asm!("stmxcsr dword ptr [{}]", in(reg) &mut mxcsr as *mut u32, options(nostack, preserves_flags));
        }
        FpState { x87, mxcsr }
    }

    pub fn restore(state: &FpState) {
        // SAFETY: loads an environment image and MXCSR that were captured from this thread.
        unsafe {
            asm!("fldenv [{}]", in(reg) state.x87.as_ptr(), options(nostack, readonly));
            asm!("ldmxcsr dword ptr [{}]", in(reg) &state.mxcsr as *const u32, options(nostack, readonly));
        }
    }

    /// The canonical arithmetic environment the engine's kernels enter (the x87 control word 0x0f7f and MXCSR 0x7f80, round toward zero, flags clear): REQUESTED here, then read back by the caller.
    pub fn enter_canonical() {
        let (x87, mxcsr) = (0x0f7fu16, 0x7f80u32);
        // SAFETY: clears the x87 exception flags and loads the two control words.
        unsafe {
            asm!("fnclex", options(nostack));
            asm!("fldcw word ptr [{}]", in(reg) &x87 as *const u16, options(nostack, readonly));
            asm!("ldmxcsr dword ptr [{}]", in(reg) &mxcsr as *const u32, options(nostack, readonly));
        }
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod fp {
    use super::FpState;

    pub fn capture() -> FpState {
        FpState { x87: [0; 28], mxcsr: 0 }
    }

    pub fn restore(_: &FpState) {}

    pub fn enter_canonical() {}
}

/// Puts the captured baseline back on every exit of a variant, a panic included.
struct RestoreBaseline<'a>(&'a FpState);

impl Drop for RestoreBaseline<'_> {
    fn drop(&mut self) {
        fp::restore(self.0);
    }
}

/// The measured states around one variant: where it started (after the baseline was put back), what it entered, what it left after the arithmetic, and what was read back after the baseline was restored again.
struct Boundary {
    before: FpState,
    entered: FpState,
    after: FpState,
    restored: FpState,
}

static BOUNDARY_RUNS: AtomicUsize = AtomicUsize::new(0);
static RESTORE_FAILURES: AtomicUsize = AtomicUsize::new(0);

/// The control's words for one row: the written amplification and pitch (twelve words, 0xdededede where nothing was written), the staged values (sixteen words) and the measured floating-point boundary. Every
/// variant starts from the same captured `baseline` (so none inherits the status of an earlier one), has its own balanced save and restore, and is judged by the readback after the restoration.
fn control_words(staged: bool, canonical: bool, row: &Row, baseline: &FpState) -> ([u32; 12], [u32; 16], Boundary) {
    let mut written = [0u32; 12];
    let mut stages = [0u32; 16];
    let _guard = RestoreBaseline(baseline);
    fp::restore(baseline);
    let before = fp::capture();
    if canonical {
        fp::enter_canonical();
    }
    let entered = fp::capture();
    // SAFETY: the arrays have the lengths the control reads and writes; the ingress and egress are integer words.
    unsafe {
        eech_rotor_control_run(
            i32::from(staged),
            row.player,
            i32::from(row.have_flight_model),
            row.inputs.as_ptr(),
            written.as_mut_ptr(),
            stages.as_mut_ptr(),
        )
    };
    let after = fp::capture();
    fp::restore(baseline);
    let restored = fp::capture();
    BOUNDARY_RUNS.fetch_add(1, Ordering::SeqCst);
    if restored.identity() != baseline.identity() {
        RESTORE_FAILURES.fetch_add(1, Ordering::SeqCst);
    }
    (
        written,
        stages,
        Boundary {
            before,
            entered,
            after,
            restored,
        },
    )
}

/// One variant of the control run on one row.
struct Variant {
    label: &'static str,
    staged: bool,
    found: Vec<String>,
    stages: [u32; 16],
    boundary: Boundary,
}

fn describe(found: &[String]) -> String {
    if found.is_empty() {
        "matches".to_string()
    } else {
        found.join(", ")
    }
}

/// The characterisation, printed once, before any oracle assertion of this file.
fn diagnostics() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let baseline = fp::capture();
        let result = std::panic::catch_unwind(|| {
            let mut text = String::new();
            text.push_str("==== ROTOR-CI-CONTROL (ENV-HOST-ROTOR-CI-1): characterisation only, UNMEASURED until a build prints it; it says what this target computes, not which answer is right\n");
            text.push_str(&format!("capture status: {}\n", env!("EECH_ROTOR_CONTROL_CAPTURE_STATUS")));
            text.push_str(&format!(
                "target {} host {} profile {} | build-time environment of the cargo invocation (NOT the World EECH_ENGINE_OPT unless it is the same variable): EECH_ENGINE_OPT {} EECH_C_OPT {}; the C optimisation actually used is the -O of the command below (opt-level {}); EECH_C_OPT now {} | rust target_arch {} pointer_width {} os {}\n",
                env!("EECH_ROTOR_CONTROL_TARGET"),
                env!("EECH_ROTOR_CONTROL_HOST"),
                env!("EECH_ROTOR_CONTROL_PROFILE"),
                env!("EECH_ROTOR_CONTROL_EECH_ENGINE_OPT"),
                env!("EECH_ROTOR_CONTROL_EECH_C_OPT"),
                env!("EECH_ROTOR_CONTROL_OPT_LEVEL"),
                std::env::var("EECH_C_OPT").unwrap_or_else(|_| "(unset)".into()),
                std::env::consts::ARCH,
                usize::BITS,
                std::env::consts::OS
            ));
            text.push_str(&format!("compiler version: {}\n", env!("EECH_ROTOR_CONTROL_CC_VERSION")));
            text.push_str(&format!("compiler machine: {}\n", env!("EECH_ROTOR_CONTROL_CC_MACHINE")));
            text.push_str(&format!(
                "full compile command (the steps archive with env_ref.c uses the same settings): {}\n",
                env!("EECH_ROTOR_CONTROL_CC_COMMAND")
            ));
            // SAFETY: the function returns a pointer to a static NUL-terminated string.
            let facts = unsafe { std::ffi::CStr::from_ptr(eech_rotor_control_facts()) }.to_string_lossy().into_owned();
            text.push_str(&format!("control build facts: {facts}\n"));
            text.push_str(&format!("source sha256: {}\n", env!("EECH_ROTOR_CONTROL_SOURCE_HASHES")));
            text.push_str("variants: U = the arithmetic statements of the original, token-identical, in a SURROGATE compilation context (stub collaborators, reduced layouts), not the engine's code generation; S = STAGED, diagnostic CHANGED code (each intermediate stored to an addressable volatile float and reloaded; one extra magnitude call, tag 0); d = no environment change; c = the canonical environment REQUESTED (x87 0x0f7f, MXCSR 0x7f80). Every variant starts from the same captured baseline, is bracketed by its own save and restore, and is reported with the states MEASURED before, at entry, after the arithmetic and after the restoration\n");
            text.push_str(&format!("floating-point baseline captured at the start of the diagnostics: {}\n", baseline.text()));
            text.push_str("observed and restored: the x87 control, status and tag words (and the environment pointers, restored but not compared) and the complete MXCSR of this thread; NOT the x87 data registers, the XMM data or other threads; nothing is inferred about the engine's startup state or about the public caller's floating-point equivalence\n");

            let rows = oracle_rows();
            let mut natural = [0usize; 6];
            let mut seen: Vec<(String, String)> = Vec::new();
            for row in &rows {
                let kernel = kernel_words(row);
                let kernel_found = match &kernel {
                    Ok((amplification, pitch)) => differences(row, amplification, pitch),
                    Err(error) => vec![format!("refused: {error}")],
                };
                let mut variants = Vec::new();
                for (label, staged, canonical) in [("U/d", false, false), ("U/c", false, true), ("S/d", true, false), ("S/c", true, true)] {
                    let (written, stages, boundary) = control_words(staged, canonical, row, &baseline);
                    let found = differences(row, &written[..6], &written[6..]);
                    for (kind, state) in [("before", &boundary.before), ("entered", &boundary.entered), ("after", &boundary.after), ("restored", &boundary.restored)] {
                        let entry = (format!("{label} {kind}"), state.text());
                        if !seen.contains(&entry) {
                            seen.push(entry);
                        }
                    }
                    variants.push(Variant { label, staged, found, stages, boundary });
                }
                if row.source == "natural" {
                    natural[0] += 1;
                    natural[1] += usize::from(!kernel_found.is_empty());
                    for (position, variant) in variants.iter().enumerate() {
                        natural[2 + position] += usize::from(!variant.found.is_empty());
                    }
                    continue;
                }
                let any = !kernel_found.is_empty() || variants.iter().any(|variant| !variant.found.is_empty());
                let named = row.source == "fd-sweep" && (row.index == "1" || row.index == "7");
                text.push_str(&format!(
                    "{} {} (player {} flight model {}): in {} | kernel: {}",
                    row.source,
                    row.index,
                    row.player,
                    row.have_flight_model,
                    row.inputs.iter().map(|word| format!("{word:08x}")).collect::<Vec<_>>().join(","),
                    describe(&kernel_found)
                ));
                for variant in &variants {
                    text.push_str(&format!(" | {}: {}", variant.label, describe(&variant.found)));
                }
                text.push('\n');
                if any || named {
                    for variant in &variants {
                        let boundary = &variant.boundary;
                        text.push_str(&format!(
                            "    {} measured: before [{}] entered [{}] after [{}] restored [{}]\n",
                            variant.label,
                            boundary.before.text(),
                            boundary.entered.text(),
                            boundary.after.text(),
                            boundary.restored.text()
                        ));
                        if variant.staged {
                            text.push_str(&format!(
                                "    {} staged floats [0 EXTRA magnitude call, 1 acc_force, 2 slap_amp, 3 slap_amp bounded, 4 looping_amp, 5 looping_pitch, 6 apu_pitch, 7 apu_amp, 8 left_pitch, 9 left_amp, 10 left_amp x1.2, 11 right_pitch, 12 right_amp] = {}\n",
                                variant.label,
                                variant.stages.iter().take(13).map(|word| format!("{word:08x}")).collect::<Vec<_>>().join(" ")
                            ));
                        }
                    }
                    if let Ok((amplification, pitch)) = &kernel {
                        text.push_str(&format!(
                            "    kernel words: amplification {} pitch {}\n",
                            amplification.iter().map(|word| format!("{word:08x}")).collect::<Vec<_>>().join(" "),
                            pitch.iter().map(|word| format!("{word:08x}")).collect::<Vec<_>>().join(" ")
                        ));
                    }
                }
            }
            text.push_str(&format!(
                "natural distinct inputs {}: rows differing from the oracle: kernel {} | U/d {} U/c {} S/d {} S/c {} (AI helicopters without a flight model only)\n",
                natural[0], natural[1], natural[2], natural[3], natural[4], natural[5]
            ));
            text.push_str("distinct measured states over every variant run (label and moment):\n");
            for (label, state) in &seen {
                text.push_str(&format!("    {label}: {state}\n"));
            }
            text.push_str(&format!(
                "floating-point boundaries: {} variant runs, {} whose readback after the restoration differed from the baseline{}\n",
                BOUNDARY_RUNS.load(Ordering::SeqCst),
                RESTORE_FAILURES.load(Ordering::SeqCst),
                if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
                    ""
                } else {
                    " (NOT OBSERVED on this architecture: the states above are zeros)"
                }
            ));
            text.push_str("---- untouched assembly: the kernel env_ref_rotor_sound and its callees, the control's entry eech_rotor_control_run and the local functions it calls (the unmodified and the STAGED variant as emitted), and the stub control_magnitude, with their constant data; each section says if its label was not found\n");
            text.push_str(ASSEMBLY);
            text.push_str("==== end of ROTOR-CI-CONTROL\n");
            text
        });
        // whatever happened, this thread's floating-point state is the baseline again before any assertion
        fp::restore(&baseline);
        let text = match result {
            Ok(text) => text,
            Err(_) => {
                RESTORE_FAILURES.fetch_add(1, Ordering::SeqCst);
                "==== ROTOR-CI-CONTROL: the characterisation itself PANICKED (reported, not hidden; no oracle assertion depends on it)\n".to_string()
            }
        };
        // straight to stderr: the harness captures print! and eprint! of passing tests, not this
        let mut stderr = std::io::stderr().lock();
        let _ = stderr.write_all(text.as_bytes());
        let _ = stderr.flush();
    });
}

#[test]
fn the_kernel_gives_the_measured_words_of_the_original_for_every_retained_input() {
    diagnostics();
    let mut checked = (0usize, 0usize, 0usize);
    for line in ORACLE.lines().filter(|line| !line.starts_with('#')) {
        let row = line.split('\t').collect::<Vec<_>>();
        let (source, index) = (row[0], row[1]);
        let what = format!("{source} {index}: {line}");
        let player = match row[2] {
            "0" => RotorPlayer::Ai,
            "1" => RotorPlayer::Local,
            "2" => RotorPlayer::Remote,
            other => panic!("{what}: player {other}"),
        };
        let have_flight_model = row[3] == "1";
        let inputs = row[4].split(',').map(value).collect::<Vec<f32>>();
        let sound = rotor_sound(&RotorSoundInput {
            player,
            rpm: inputs[0],
            acceleration: have_flight_model.then(|| [inputs[1], inputs[2], inputs[3]]),
            engines: [inputs[4], inputs[5], inputs[6]],
        })
        .unwrap_or_else(|e| panic!("{what}: the kernel refused: {e}"));

        // the written levels and pitches: a channel the helicopter had no child for ('-') wrote nothing in the original and is not compared
        for (channel, expected) in row[5].split(',').enumerate() {
            if expected != "-" {
                assert_eq!(
                    sound.amplification[channel].to_bits(),
                    word(expected),
                    "{what}: amplification of channel {channel}"
                );
            }
        }
        for (channel, expected) in row[6].split(',').enumerate() {
            if expected != "-" {
                assert_eq!(sound.pitch[channel].to_bits(), word(expected), "{what}: pitch of channel {channel}");
            }
        }
        assert_eq!(sound.write, [3, 3, 3, 3, 3, 1], "{what}: the write masks");
        assert_eq!(sound.pitch[5].to_bits(), 0, "{what}: the winding channel carries no pitch");

        // the ordered requests, with the delay words
        let actions = if row.get(7).copied().unwrap_or("").is_empty() {
            Vec::new()
        } else {
            row[7]
                .split(',')
                .map(|item| {
                    let part = item.split(':').collect::<Vec<_>>();
                    (part[0] == "1", part[1].parse::<usize>().expect("a channel"), word(part[2]))
                })
                .collect::<Vec<_>>()
        };
        let got = sound
            .actions
            .iter()
            .map(|action| (action.resume, action.channel, action.delay.to_bits()))
            .collect::<Vec<_>>();
        assert_eq!(got, actions, "{what}: the ordered requests");

        match source {
            "natural" => checked.0 += 1,
            "ai-sweep" => checked.1 += 1,
            _ => checked.2 += 1,
        }
    }
    // the retained record, not a sample: 261 distinct natural inputs, 12 AI and 15 flight model sweep rows
    assert_eq!(checked, (261, 12, 15));
}

#[test]
fn a_sweep_row_the_oracle_marks_as_the_canonical_builds_zero_has_both_unassigned_levels_zero_in_the_kernel() {
    diagnostics();
    // F1: rows 10 to 14 of the flight model sweep (the two engines within 1.0 of each other, or no APU): the original leaves right_turbine_amp and / or apu_amp unassigned and the canonical build's zero is the measured value
    let marked = ORACLE.lines().filter(|line| line.contains("# compiler_zero_init")).count();
    assert_eq!(marked, 5);
    for line in ORACLE.lines().filter(|line| line.contains("# compiler_zero_init")) {
        let row = line.split('\t').collect::<Vec<_>>();
        let inputs = row[4].split(',').map(value).collect::<Vec<f32>>();
        let sound = rotor_sound(&RotorSoundInput {
            player: RotorPlayer::Local,
            rpm: inputs[0],
            acceleration: Some([inputs[1], inputs[2], inputs[3]]),
            engines: [inputs[4], inputs[5], inputs[6]],
        })
        .unwrap();
        let single_sound = (inputs[4] - inputs[5]).abs() <= 1.0;
        let no_apu = inputs[6] <= 0.0;
        assert!(single_sound || no_apu, "{line}");
        if single_sound {
            assert_eq!(sound.amplification[3].to_bits(), 0, "{line}: right_turbine_amp");
        }
        if no_apu {
            assert_eq!(sound.amplification[4].to_bits(), 0, "{line}: apu_amp");
        }
    }
}

#[test]
fn the_diagnostic_variants_each_restored_the_captured_floating_point_baseline() {
    diagnostics();
    let runs = BOUNDARY_RUNS.load(Ordering::SeqCst);
    // a diagnostic that could not bracket or restore the floating-point state is a failure of the diagnostic, reported apart from the oracle assertions
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
        assert!(runs > 0, "no diagnostic variant ran");
    }
    assert_eq!(
        RESTORE_FAILURES.load(Ordering::SeqCst),
        0,
        "{runs} variant runs, some read back a state other than the captured baseline (or the diagnostics panicked)"
    );
}
