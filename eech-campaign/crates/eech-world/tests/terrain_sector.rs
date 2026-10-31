//! Local contracts of the reference host's sector surface enumeration (EEDCS-51, D3): `host_terrain_types_in_sector`, the host's answer to the campaign's `get_terrain_3d_types_in_sector`. SOURCE
//! PREPARED: UNBUILT, UNRUN, UNREVIEWED.
//!
//! The maps are one-sector maps built here from bytes (the layout of `terrain_backend.rs`), so no retail asset is needed. They go through the real loader, the real kernel and the real Lua
//! provider of `eech_world::terrain`; nothing here re-implements the enumeration. They are bookkeeping of a LOCAL contract (the return value is 1 plus the surface changes and is not the number of
//! flags; the first surface is always flagged; the domain), not conformance with the original on a real map: that is the three-root collection of `lua/env-sector-characterization.lua`, which
//! this change makes comparable after the cutover.

use eech_world::terrain::{provider, Terrain};
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
