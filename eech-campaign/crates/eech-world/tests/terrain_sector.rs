//! Local contracts of the reference host's sector surface enumeration (EEDCS-51, D3): `host_terrain_types_in_sector`, the host's answer to the campaign's `get_terrain_3d_types_in_sector`. SOURCE
//! PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! The maps are one-sector maps built here from bytes (the layout of `terrain_backend.rs`), so no retail asset is needed. They go through the real loader, the real kernel and the real Lua
//! provider of `eech_world::terrain`; nothing here re-implements the enumeration. They are bookkeeping of a LOCAL contract (the return value is 1 plus the surface changes and is not the number of
//! flags; the first surface is always flagged; the domain), not conformance with the original on a real map: that is the three-root collection of `lua/env-sector-characterization.lua`, which
//! this change makes comparable after the cutover.
//!
//! The last test is the CALLER'S-FLOATING-POINT-STATE contract of the same public entry (`Terrain::types_in_sector`): the complete x87 and SSE state, seeded and read back with no mask, across
//! accepted and refused queries. It was written BEFORE the kernel's guard (a red test against the unguarded commit 6711d5f11); see the test for what it covers and what it does not.

use eech_world::terrain::{provider, AmbientChild, Terrain, AMBIENT_MAX};
use mlua::Lua;
use std::rc::Rc;

const A: i16 = 16384;

fn le16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn le32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// A one-sector map of `faces.len()` quads (the same four corners each), the face at index `i` changing surface when `faces[i]` is true, over the surface table `surfaces`
/// (the loader needs `1 + the number of changes`; a test that wants a short table or a long one passes it).
fn map(faces: &[bool], surfaces: &[u8]) -> Terrain {
    let mut geometry: Vec<u8> = Vec::new();
    let points = [(-A, -A), (-A, A), (A, A), (A, -A)];
    for (x, z) in points {
        le16(&mut geometry, x as u16);
        le16(&mut geometry, z as u16);
    }
    let off_y = geometry.len() as u32;
    for _ in points {
        le16(&mut geometry, 1000);
    }
    let off_ref = geometry.len() as u32;
    for _ in faces {
        geometry.extend_from_slice(&[0, 1, 2, 3]);
    }
    let off_poly = geometry.len() as u32;
    for change in faces {
        geometry.push(4 | (u8::from(*change) << 7));
    }
    let off_surface = geometry.len() as u32;
    geometry.extend_from_slice(surfaces);

    let mut sector = Vec::new();
    le32(&mut sector, 1);
    le16(&mut sector, 1);
    le16(&mut sector, 1);
    sector.extend_from_slice(&0.0f32.to_le_bytes());
    sector.extend_from_slice(&32767.0f32.to_le_bytes());
    le16(&mut sector, 4);
    le16(&mut sector, 0);
    le16(&mut sector, faces.len() as u16);
    for _ in 0..5 {
        le16(&mut sector, 0);
    }
    le32(&mut sector, off_y);
    le32(&mut sector, 0);
    le32(&mut sector, off_ref);
    le32(&mut sector, off_poly);
    le32(&mut sector, off_surface);
    for _ in 0..3 {
        le32(&mut sector, 0);
    }
    le32(&mut sector, 0);
    Terrain::from_bytes(&sector, &geometry, None, 5).expect("the synthetic map loads")
}

fn set(flags: &[bool]) -> Vec<usize> {
    flags.iter().enumerate().filter(|(_, flag)| **flag).map(|(id, _)| id).collect()
}

#[test]
fn the_return_value_is_one_plus_the_surface_changes_and_not_the_number_of_surfaces_flagged() {
    // faces: land (the first surface), a change to 7, land again under the same surface, a change back to 9, a change to 3: three changes, surfaces 9 / 7 / 9 / 3
    let terrain = map(&[false, true, false, true, true], &[9, 7, 9, 3]);
    let (returned, flags) = terrain.types_in_sector(512.0, 1536.0).expect("the sector is in the map");
    assert_eq!(flags.len(), 62);
    assert_eq!(
        set(&flags),
        vec![3, 7, 9],
        "the first surface and the surface of EVERY change, each flagged once"
    );
    assert_eq!(
        returned, 4,
        "1 + three changes: surface 9 was met twice and is flagged once, so the return value is not the three flags"
    );
    // the first face changing surface: the first entry is flagged although no face is typed by it, and it is counted
    let first_changes = map(&[true, false], &[5, 6]);
    let (returned, flags) = first_changes.types_in_sector(1.0, 1.0).unwrap();
    assert_eq!((returned, set(&flags)), (2, vec![5, 6]));
    // no polygon at all: the sector's first surface alone, and the return value 1
    let empty = map(&[], &[9]);
    let (returned, flags) = empty.types_in_sector(1.0, 1.0).unwrap();
    assert_eq!((returned, set(&flags)), (1, vec![9]));
    // the answer is a function of the sector: the same sector, another position, the same answer, and a repeat does not accumulate
    assert_eq!(terrain.types_in_sector(2000.0, 20.0).unwrap(), terrain.types_in_sector(512.0, 1536.0).unwrap());
    assert_eq!(terrain.types_in_sector(512.0, 1536.0).unwrap(), terrain.types_in_sector(512.0, 1536.0).unwrap());
}

#[test]
fn the_sector_is_chosen_as_the_original_chose_it_and_a_position_outside_the_map_is_refused_not_indexed() {
    let terrain = map(&[false, true], &[9, 7]);
    // the position over 2048, truncated toward zero: a small negative position is sector 0, not -1
    for (x, z) in [(0.0, 0.0), (-0.0, -0.0), (-0.5, -0.5), (2047.9995, 2047.9995), (1.0e-30, 1.0e-30)] {
        assert!(terrain.types_in_sector(x, z).is_ok(), "({x}, {z}) is in sector 0");
    }
    // the next sector, the one before, a position far away, and numbers that are not positions
    for (x, z, fragment) in [
        (2048.0, 0.0, "outside the map"),
        (0.0, 2048.0, "outside the map"),
        (-2048.0, 0.0, "outside the map"),
        (0.0, -2049.0, "outside the map"),
        (3.0e30, 0.0, "outside the map"),
        (f32::NAN, 0.0, "finite"),
        (0.0, f32::INFINITY, "finite"),
        (f32::NEG_INFINITY, 0.0, "finite"),
    ] {
        let error = terrain.types_in_sector(x, z).unwrap_err();
        assert!(error.contains(fragment), "({x}, {z}): {error}");
    }
}

#[test]
fn a_surface_id_the_original_would_use_outside_its_scratch_is_refused() {
    // the first face changes surface, so the first table entry types no face and the loader never reads it as a type: the enumeration does flag it, and the id is not a terrain type
    let terrain = map(&[true, false], &[200, 9]);
    let error = terrain.types_in_sector(1.0, 1.0).unwrap_err();
    assert!(error.contains("first surface of type 200"), "{error}");
}

#[test]
fn the_lua_provider_answers_sector_with_the_return_value_and_the_flags_and_refuses_what_it_cannot() {
    let lua = Lua::new();
    let terrain = Rc::new(map(&[false, true, false, true, true], &[9, 7, 9, 3]));
    let function = provider(&lua, terrain).expect("the provider");
    let ask = |x: f64, z: f64| -> mlua::Result<mlua::Table> {
        let request = lua.create_table()?;
        request.set("op", "sector")?;
        request.set("x", x)?;
        request.set("z", z)?;
        request.set("winter", false)?;
        function.call(request)
    };
    let answer = ask(512.0, 1536.0).expect("an answer");
    assert_eq!(answer.get::<i64>("returned").unwrap(), 4);
    let flags: String = answer.get("flags").unwrap();
    assert_eq!(flags.len(), 62);
    let expected: String = (0..62).map(|id| if [3, 7, 9].contains(&id) { '1' } else { '0' }).collect();
    assert_eq!(flags, expected);
    // a position outside the map and a coordinate that is not an exact single are errors with the host's message, not guesses
    let error = ask(2048.0, 0.0).unwrap_err().to_string();
    assert!(error.contains("outside the map"), "{error}");
    let error = ask(0.1, 0.0).unwrap_err().to_string();
    assert!(error.contains("exact single-precision"), "{error}");
}

/// The processor's floating-point state of this thread, read and written directly -- the x87 control, status AND tag words and the whole of MXCSR -- so that the sector query is
/// checked against the REAL registers, in full, with no mask. The same technique as the field's guard test (`env.rs`, `fp`), which is a private module of the library's own unit tests and
/// so is repeated here for this integration test; MXCSR's mode bits are added (FTZ and DAZ, as far as this processor supports them). Only where those registers exist.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod caller_fp {
    use std::arch::asm;

    /// The complete state a guard promises to give back (the x87 words are the full words).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Env {
        pub mxcsr: u32,
        pub control: u16,
        pub status: u16,
        pub tag: u16,
    }

    pub fn mxcsr() -> u32 {
        let mut value = 0u32;
        // SAFETY: stores the 32-bit MXCSR to a local.
        unsafe { asm!("stmxcsr dword ptr [{}]", in(reg) &mut value as *mut u32, options(nostack, preserves_flags)) };
        value
    }

    pub fn set_mxcsr(value: u32) {
        // SAFETY: loads MXCSR from a local; the values written are masked with the bits this processor accepts (see `mxcsr_mask`).
        unsafe { asm!("ldmxcsr dword ptr [{}]", in(reg) &value as *const u32, options(nostack, readonly)) };
    }

    #[repr(align(16))]
    struct Image512([u8; 512]);

    /// The MXCSR bits this processor accepts (MXCSR_MASK of fxsave; zero means the architectural default 0xffbf, which has no DAZ).
    pub fn mxcsr_mask() -> u32 {
        let mut image = Image512([0u8; 512]);
        // SAFETY: stores the 512-byte FXSAVE image to a 16-byte aligned local; fxsave changes no register state.
        unsafe { asm!("fxsave [{}]", in(reg) image.0.as_mut_ptr(), options(nostack)) };
        let mask = u32::from_le_bytes([image.0[28], image.0[29], image.0[30], image.0[31]]);
        if mask == 0 {
            0xffbf
        } else {
            mask
        }
    }

    /// The x87 environment image (fnstenv: control, status and tag words, the exception pointers). fnstenv masks every exception, so the image is loaded straight back.
    pub fn x87_image() -> [u8; 28] {
        let mut image = [0u8; 28];
        // SAFETY: stores the 28-byte environment to a local and loads the same image back.
        unsafe {
            asm!("fnstenv [{}]", in(reg) image.as_mut_ptr(), options(nostack));
            asm!("fldenv [{}]", in(reg) image.as_ptr(), options(nostack, readonly));
        }
        image
    }

    pub fn set_x87_image(image: &[u8; 28]) {
        // SAFETY: loads an environment image built from a prior read with valid words.
        unsafe { asm!("fldenv [{}]", in(reg) image.as_ptr(), options(nostack, readonly)) };
    }

    pub fn now() -> Env {
        let image = x87_image();
        Env {
            mxcsr: mxcsr(),
            control: u16::from_le_bytes([image[0], image[1]]),
            status: u16::from_le_bytes([image[4], image[5]]),
            tag: u16::from_le_bytes([image[8], image[9]]),
        }
    }

    /// One deliberately arranged caller state.
    #[derive(Clone, Copy, Debug)]
    pub struct Seed {
        /// x87 rounding control (0..4) and precision control (0 = 24 bit, 2 = 53 bit, 3 = 64 bit).
        pub x87_rounding: u16,
        pub x87_precision: u16,
        /// A full x87 status word (stack top, condition codes C0-C3 including C1, exception flags).
        pub status: u16,
        /// MXCSR rounding control (0..4), sticky exception flags and the FTZ / DAZ mode bits (0x8000, 0x0040; dropped where the processor lacks them).
        pub sse_rounding: u32,
        pub sticky: u32,
        pub modes: u32,
    }

    /// Arranges the caller state: the x87 words with an all-empty tag word, and MXCSR with every exception masked (a seeded state must never trap).
    pub fn seed(seed: Seed) {
        let mut image = x87_image();
        let control = (u16::from_le_bytes([image[0], image[1]]) & !0x0f00) | (seed.x87_rounding << 10) | (seed.x87_precision << 8);
        image[0..2].copy_from_slice(&control.to_le_bytes());
        image[4..6].copy_from_slice(&seed.status.to_le_bytes());
        image[8..10].copy_from_slice(&0xffffu16.to_le_bytes());
        set_x87_image(&image);
        set_mxcsr((0x1f80 | (seed.sse_rounding << 13) | seed.sticky | seed.modes) & mxcsr_mask());
    }

    /// Which words differ, in hex, for a failure report.
    pub fn difference(expected: Env, actual: Env) -> String {
        let mut parts = Vec::new();
        if expected.control != actual.control {
            parts.push(format!("x87 control {:#06x} -> {:#06x}", expected.control, actual.control));
        }
        if expected.status != actual.status {
            parts.push(format!("x87 status {:#06x} -> {:#06x}", expected.status, actual.status));
        }
        if expected.tag != actual.tag {
            parts.push(format!("x87 tag {:#06x} -> {:#06x}", expected.tag, actual.tag));
        }
        if expected.mxcsr != actual.mxcsr {
            parts.push(format!("MXCSR {:#010x} -> {:#010x}", expected.mxcsr, actual.mxcsr));
        }
        parts.join(", ")
    }
}

/// Puts the thread's COMPLETE floating-point environment back whatever a test did to it, even when an assertion fails.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
struct RestoreFp {
    image: [u8; 28],
    mxcsr: u32,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Drop for RestoreFp {
    fn drop(&mut self) {
        caller_fp::set_x87_image(&self.image);
        caller_fp::set_mxcsr(self.mxcsr);
    }
}

/// The public host sector query (`Terrain::types_in_sector`, the library entry the Lua provider and the campaign's answer both use), accepted and refused, under a caller's COMPLETE floating-point
/// state: every pair of x87 and SSE rounding controls (the mixed pairs too), two x87 precision controls, MXCSR sticky flags clear and set, FTZ/DAZ clear and set (where the processor has them),
/// and three deliberately arranged x87 status words (clear; the condition code C3 and every exception flag; the stack top 4 and the codes C0, C1 and C2). After every call the caller's
/// x87 control, status (C1 included) and tag words and the whole of MXCSR must be EXACTLY what was seeded, no mask. The positions are ordinary fractional coordinates (whose quotient by the
/// 2048-metre sector is inexact and whose truncation is inexact), the admitted subnormals (the smallest, the largest, a negative one, a subnormal beside a normal), signed zero, the minimum
/// normal, and the refusals (outside the map; not finite; a quotient that does not fit an integer; a surface id outside the terrain types). And whatever the caller's state, the answers are the
/// same (the same return value and flags, the same refusal text).
///
/// The mismatches are COLLECTED, not stopped at the first, so a failing run reports every call and every word that leaked once. Only the entry's own effect is claimed: the surrounding Rust is the
/// test's own and runs in the seeded state too (on an x87-only target it can itself touch the status word, which this test would then report). It goes through the library entry, so it survives a
/// later correction of the private C ABI. Source only: Root runs it on the actual targets, against the production under test.
#[test]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn the_callers_complete_floating_point_state_survives_every_sector_query_in_every_mode() {
    use caller_fp::{difference, now, seed, Env, Seed};
    use std::collections::BTreeMap;

    let restore = RestoreFp {
        image: caller_fp::x87_image(),
        mxcsr: caller_fp::mxcsr(),
    };
    let terrain = map(&[false, true, false, true, true], &[9, 7, 9, 3]);
    // the first table entry types no face, so the loader never validated it, and the enumeration finds a surface id that is not a terrain type
    let unlisted = map(&[true, false], &[200, 9]);

    let subnormal = f32::from_bits(0x0000_0001);
    let largest_subnormal = f32::from_bits(0x007f_ffff);
    let negative_subnormal = f32::from_bits(0x8000_0001);
    let minimum_normal = f32::from_bits(0x0080_0000);
    // (what, which map, x, z); a call labelled "refused" must be refused and every other call accepted
    let calls: Vec<(&str, &Terrain, f32, f32)> = vec![
        ("fractional (1, 1)", &terrain, 1.0, 1.0),
        ("fractional (1234.567, 2000.123)", &terrain, 1234.567, 2000.123),
        ("fractional just below the next sector", &terrain, 2047.9995, 2047.9995),
        ("smallest subnormal", &terrain, subnormal, subnormal),
        ("largest subnormal", &terrain, largest_subnormal, largest_subnormal),
        ("negative subnormal", &terrain, negative_subnormal, negative_subnormal),
        ("a subnormal beside a normal", &terrain, subnormal, 1500.5),
        ("minimum normal", &terrain, minimum_normal, minimum_normal),
        ("signed zero", &terrain, -0.0, 0.0),
        ("negative fraction (truncates to sector 0)", &terrain, -0.5, -1.5),
        ("refused: next sector, outside the map", &terrain, 2048.0, 0.0),
        ("refused: sector before the map", &terrain, 0.0, -2049.0),
        ("refused: NaN", &terrain, f32::NAN, 0.0),
        ("refused: infinite", &terrain, 0.0, f32::INFINITY),
        ("refused: quotient does not fit an integer", &terrain, 3.0e30, 0.0),
        ("refused: surface id outside the terrain types", &unlisted, 1.0, 1.0),
    ];

    let mask = caller_fp::mxcsr_mask();
    let mode_sets: Vec<u32> = if mask & 0x8040 == 0x8040 { vec![0, 0x8040] } else { vec![0, mask & 0x8000] };
    // the answer of one call, as the public entry gives it: the return value with the flags, or the refusal text
    type SectorAnswer = Result<(i32, Vec<bool>), String>;
    let mut reference: Vec<Option<SectorAnswer>> = vec![None; calls.len()];
    // what leaked: "call: words" -> (times, the first seed it happened under)
    let mut leaks: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut answer_changes: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for x87_rounding in 0u16..4 {
        for x87_precision in [0u16, 3] {
            for sse_rounding in 0u32..4 {
                for sticky in [0u32, 0x3f] {
                    for &modes in &mode_sets {
                        for status in [0x0000u16, 0x403f, 0x2700] {
                            let state = Seed {
                                x87_rounding,
                                x87_precision,
                                status,
                                sse_rounding,
                                sticky,
                                modes,
                            };
                            for (index, (what, terrain, x, z)) in calls.iter().enumerate() {
                                seed(state);
                                let expected: Env = now();
                                let answer = terrain.types_in_sector(*x, *z);
                                let actual = now();
                                checked += 1;
                                if actual != expected {
                                    let key = format!("{what}: {}", difference(expected, actual));
                                    leaks.entry(key).or_insert((0, format!("{state:?}"))).0 += 1;
                                }
                                if reference[index].is_none() {
                                    reference[index] = Some(answer);
                                } else if reference[index].as_ref() != Some(&answer) {
                                    answer_changes.push(format!("{what} under {state:?}: {:?} -> {answer:?}", reference[index]));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    drop(restore);

    // the calls really were accepted or refused as labelled (a refusal is exercised as a refusal, a success as a success)
    for (index, (what, _, _, _)) in calls.iter().enumerate() {
        let answer = reference[index].as_ref().expect("every call ran");
        assert_eq!(answer.is_err(), what.starts_with("refused"), "{what}: {answer:?}");
    }
    assert!(
        answer_changes.is_empty(),
        "the caller's state reached the kernel's answers ({} of {checked} calls):\n{}",
        answer_changes.len(),
        answer_changes.join("\n")
    );
    assert!(
        leaks.is_empty(),
        "the sector query changed the caller's floating-point state in {} kinds of call ({checked} calls checked, no mask):\n{}",
        leaks.len(),
        leaks
            .iter()
            .map(|(key, (times, first))| format!("  {key}  x{times}, first under {first}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the physical amplitude of the terrain ambient sound children (EEDCS-51, D3): the host's own sector surfaces and the original's smoothing

fn child(class: i32, previous: f32) -> AmbientChild {
    AmbientChild { class, previous }
}

#[test]
fn the_host_decides_the_terrain_ambient_amplitude_from_its_own_sector_surfaces_with_the_originals_smoothing() {
    // the six classes follow sea (7), forest top (46), river (1), reservoir (2), altered land 1 (21) and altered land 2 (22): a child rises by half the delta when its surface is flagged in the
    // sector and falls by it when not. Previous 0.125 and delta 0.25 are dyadic: the results are exact, at altitude 0 outdoors (the altitude factor is 1).
    type World = (&'static [bool], &'static [u8], [bool; 6]);
    let worlds: [World; 4] = [
        (&[false, true, false, true, true], &[9, 7, 9, 3], [true, false, false, false, false, false]),
        (&[false, true], &[46, 21], [false, true, false, false, true, false]),
        (&[false, true], &[1, 2], [false, false, true, true, false, false]),
        (&[false, true], &[7, 22], [true, false, false, false, false, true]),
    ];
    for (faces, surfaces, flagged) in worlds {
        let terrain = map(faces, surfaces);
        let children: Vec<AmbientChild> = (0..6).map(|class| child(class, 0.125)).collect();
        let amplitudes = terrain.ambient(512.0, 1536.0, false, 0.0, 0.25, &children).expect("the sector is in the map");
        let expected: Vec<f32> = flagged.iter().map(|on| if *on { 0.25 } else { 0.0 }).collect();
        assert_eq!(amplitudes, expected, "surfaces {surfaces:?}");
    }
    let terrain = map(&[false, true, false, true, true], &[9, 7, 9, 3]);
    let ask = |alt: f32, delta: f32, cockpit: bool, children: &[AmbientChild]| terrain.ambient(1.0, 1.0, cockpit, alt, delta, children);
    // the upper bound of the smoothing is the double 0.3 stored as a float under round-toward-zero, the engine's rounding mode (0x3e999999, one float below the nearest, 0.3f32), the lower 0
    let upper = f32::from_bits(0x3e99_9999);
    assert_eq!(ask(0.0, 0.25, false, &[child(0, 0.25), child(1, 0.0)]).unwrap(), vec![upper, 0.0]);
    // the altitude factor 1 - alt / 280 (0.5 at 140 m) multiplies the bounded value, and the cockpit factor 0.75 follows it
    assert_eq!(ask(140.0, 0.25, false, &[child(0, 0.125)]).unwrap(), vec![0.125f32]);
    assert_eq!(ask(0.0, 0.25, true, &[child(0, 0.125)]).unwrap(), vec![0.1875f32]);
    // at or above 280 m the amplitude is 0, and the previous amplitude is not used at all (a NaN there is not an error): the branch-dependent input stays unused
    for alt in [280.0f32, 400.0] {
        assert_eq!(
            ask(alt, 0.25, false, &[child(0, f32::NAN), child(5, 0.25)]).unwrap(),
            vec![0.0f32, 0.0],
            "alt {alt}"
        );
    }
    // just below 280 m the factor is a hair above zero
    let tiny = ask(f32::from_bits(280.0f32.to_bits() - 1), 0.25, false, &[child(0, 0.125)]).unwrap();
    assert!(tiny[0] > 0.0 && tiny[0] < 1.0e-6, "{tiny:?}");
    // each child is smoothed from ITS OWN previous amplitude, in the order given (two children of one class): 0.125 + 0.125 and 0.25 + 0.125 bounded
    assert_eq!(ask(0.0, 0.25, false, &[child(0, 0.125), child(0, 0.25)]).unwrap(), vec![0.25f32, upper]);
    // no child at all is a question with no answers
    assert_eq!(ask(0.0, 0.25, false, &[]).unwrap(), Vec::<f32>::new());
}

#[test]
fn a_terrain_ambient_question_that_the_host_cannot_answer_is_refused_with_a_message() {
    let terrain = map(&[false, true], &[9, 7]);
    let refused = |result: Result<Vec<f32>, String>, fragment: &str| {
        let error = result.unwrap_err();
        assert!(error.contains(fragment), "{fragment}: {error}");
    };
    // the position: outside the map, not finite (the same domain as the sector enumeration the amplitude is read from)
    refused(terrain.ambient(2048.0, 0.0, false, 0.0, 0.25, &[child(0, 0.0)]), "outside the map");
    refused(terrain.ambient(f32::NAN, 0.0, false, 0.0, 0.25, &[child(0, 0.0)]), "finite");
    // a class that is not one of the six, a count above the limit
    refused(terrain.ambient(1.0, 1.0, false, 0.0, 0.25, &[child(0, 0.0), child(6, 0.0)]), "ambient class 6");
    refused(terrain.ambient(1.0, 1.0, false, 0.0, 0.25, &[child(-1, 0.0)]), "ambient class -1");
    let too_many: Vec<AmbientChild> = (0..=AMBIENT_MAX).map(|_| child(0, 0.0)).collect();
    refused(terrain.ambient(1.0, 1.0, false, 0.0, 0.25, &too_many), "not 0 to 16");
    let at_the_limit: Vec<AmbientChild> = (0..AMBIENT_MAX).map(|_| child(0, 0.0)).collect();
    assert_eq!(terrain.ambient(1.0, 1.0, false, 0.0, 0.25, &at_the_limit).unwrap().len(), AMBIENT_MAX);
}

#[test]
fn the_lua_provider_answers_ambient_with_one_amplitude_per_child_and_refuses_a_question_it_cannot_read() {
    let lua = Lua::new();
    let terrain = Rc::new(map(&[false, true], &[9, 7]));
    let function = provider(&lua, terrain).expect("the provider");
    let question = |children: &str| -> mlua::Result<mlua::Table> {
        let request: mlua::Table = lua
            .load(format!(
                "return {{ op = 'ambient', x = 1.0, z = 1.0, winter = false, alt = 0.0, delta = 0.25, in_cockpit = false, children = {children} }}"
            ))
            .eval()?;
        function.call(request)
    };
    let answer = question("{ { class = 0, previous = 0.125 }, { class = 1, previous = 0.125 } }").expect("an answer");
    let amplitudes: Vec<f64> = answer.get("amplitudes").unwrap();
    assert_eq!(amplitudes, vec![0.25, 0.0], "the sea (7) is flagged in this sector, the forest top (46) is not");
    for (children, fragment) in [
        ("{ { class = 9, previous = 0.0 } }", "ambient class"),
        ("{ { class = 0.5, previous = 0.0 } }", "ambient class"),
        ("{ { class = 0, previous = 0.1 } }", "exact single-precision"),
        ("nil", "children must be a list"),
    ] {
        let error = question(children).unwrap_err().to_string();
        assert!(error.contains(fragment), "{children}: {error}");
    }
    let request: mlua::Table = lua
        .load("return { op = 'ambient', x = 1.0, z = 1.0, winter = false, alt = 0.0, delta = 0.25, in_cockpit = 1, children = {} }")
        .eval()
        .unwrap();
    assert!(function
        .call::<mlua::Table>(request)
        .unwrap_err()
        .to_string()
        .contains("in_cockpit must be a boolean"));
}

/// The caller's complete floating-point state around the terrain AMBIENT amplitude, in the manner of the sector enumeration's test above (the same seeded states, the same registers, no mask): accepted
/// questions (the smoothing both ways, the bounds, the altitude factor, the cockpit, the zero-volume branch with an input that must stay unused, a subnormal previous amplitude and delta) and refusals
/// (outside the map, not finite, a class that is not one of the six), and the answers are the same whatever the state. It goes through the public library entry, and the position and every float cross
/// the C boundary as bits or inside arrays read within the kernel's guard. Only the entry's own effect is claimed. Source only: Root runs it on the actual targets.
#[test]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn the_callers_complete_floating_point_state_survives_every_terrain_ambient_question_in_every_mode() {
    use caller_fp::{difference, now, seed, Env, Seed};
    use std::collections::BTreeMap;

    let restore = RestoreFp {
        image: caller_fp::x87_image(),
        mxcsr: caller_fp::mxcsr(),
    };
    let terrain = map(&[false, true, false, true, true], &[9, 7, 9, 3]);
    let subnormal = f32::from_bits(1);
    type Question = (&'static str, f32, f32, bool, f32, f32, Vec<AmbientChild>);
    let all_six: Vec<AmbientChild> = (0..6).map(|class| child(class, 0.125)).collect();
    let questions: Vec<Question> = vec![
        ("six classes outdoors", 512.0, 1536.0, false, 10.0, 0.1, all_six.clone()),
        ("six classes in the cockpit", 512.0, 1536.0, true, 0.0, 0.25, all_six),
        (
            "fractional altitude and delta",
            1.0,
            1.0,
            false,
            123.456,
            0.0166,
            vec![child(0, 0.1), child(1, 0.2)],
        ),
        ("the upper bound", 1.0, 1.0, false, 0.0, 0.25, vec![child(0, 0.25)]),
        ("the lower bound", 1.0, 1.0, false, 0.0, 0.25, vec![child(1, 0.0)]),
        (
            "just below the zero-volume altitude",
            1.0,
            1.0,
            false,
            f32::from_bits(280.0f32.to_bits() - 1),
            0.1,
            vec![child(0, 0.1)],
        ),
        (
            "at the zero-volume altitude with an unused NaN",
            1.0,
            1.0,
            false,
            280.0,
            0.1,
            vec![child(0, f32::NAN)],
        ),
        (
            "subnormal previous and delta",
            1.0,
            1.0,
            false,
            5.0,
            subnormal,
            vec![child(0, subnormal), child(2, subnormal)],
        ),
        ("subnormal altitude", 1.0, 1.0, false, subnormal, 0.1, vec![child(0, 0.1)]),
        ("signed zero position and altitude", -0.0, -0.0, false, -0.0, -0.0, vec![child(0, 0.1)]),
        ("no children", 1.0, 1.0, false, 5.0, 0.1, Vec::new()),
        ("refused: outside the map", 2048.0, 0.0, false, 5.0, 0.1, vec![child(0, 0.1)]),
        ("refused: not finite", f32::NAN, 0.0, false, 5.0, 0.1, vec![child(0, 0.1)]),
        (
            "refused: a class outside the six",
            1.0,
            1.0,
            false,
            5.0,
            0.1,
            vec![child(0, 0.1), child(6, 0.1)],
        ),
    ];

    let mask = caller_fp::mxcsr_mask();
    let mode_sets: Vec<u32> = if mask & 0x8040 == 0x8040 { vec![0, 0x8040] } else { vec![0, mask & 0x8000] };
    type Answer = Result<Vec<f32>, String>;
    let mut reference: Vec<Option<Answer>> = vec![None; questions.len()];
    let mut leaks: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut answer_changes: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for x87_rounding in 0u16..4 {
        for x87_precision in [0u16, 3] {
            for sse_rounding in 0u32..4 {
                for sticky in [0u32, 0x3f] {
                    for &modes in &mode_sets {
                        for status in [0x0000u16, 0x403f, 0x2700] {
                            let state = Seed {
                                x87_rounding,
                                x87_precision,
                                status,
                                sse_rounding,
                                sticky,
                                modes,
                            };
                            for (index, (what, x, z, cockpit, alt, delta, children)) in questions.iter().enumerate() {
                                seed(state);
                                let expected: Env = now();
                                let answer = terrain.ambient(*x, *z, *cockpit, *alt, *delta, children);
                                let actual = now();
                                checked += 1;
                                if actual != expected {
                                    let key = format!("{what}: {}", difference(expected, actual));
                                    leaks.entry(key).or_insert((0, format!("{state:?}"))).0 += 1;
                                }
                                if reference[index].is_none() {
                                    reference[index] = Some(answer);
                                } else if reference[index].as_ref() != Some(&answer) {
                                    answer_changes.push(format!("{what} under {state:?}: {:?} -> {answer:?}", reference[index]));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    drop(restore);

    for (index, (what, ..)) in questions.iter().enumerate() {
        let answer = reference[index].as_ref().expect("every question ran");
        assert_eq!(answer.is_err(), what.starts_with("refused"), "{what}: {answer:?}");
    }
    assert!(
        answer_changes.is_empty(),
        "the caller's state reached the kernel's answers ({} of {checked} calls):\n{}",
        answer_changes.len(),
        answer_changes.join("\n")
    );
    assert!(
        leaks.is_empty(),
        "the terrain ambient question changed the caller's floating-point state in {} kinds of call ({checked} calls checked, no mask):\n{}",
        leaks.len(),
        leaks
            .iter()
            .map(|(key, (times, first))| format!("  {key}  x{times}, first under {first}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
