//! eech-map: builds an EECH map and dynamic campaign from an OpenStreetMap
//! extract and SRTM elevation.
//!
//! Output (an EECH installation's common\ tree):
//!   maps\mapN\terrain\   terrain.ffp, default.sec, default.rgb
//!   maps\mapN\route\     ROADS.dat/.nde/.wp, popname.dat, bridge.pop
//!   maps\mapN\camp01\    <name>.chc, <name>.sid, <name>.pop
//!   maps\mapN\mapinfo.txt

mod campaign;
mod geo;
mod osm;
mod roads;
mod srtm;
mod terrain;

use anyhow::{Context, Result};
use campaign::{Airfield, PopName, Side, Sides};
use std::fmt::Write as _;
use std::path::PathBuf;
use terrain::{types, Terrain, CELL, CELLS, SECTOR};

struct Spec {
    name: &'static str,
    title: &'static str,
    map_number: u32,
    origin: (f64, f64),
    width_sectors: usize,
    height_sectors: usize,
    ai_sector_size: u32,
    /// the front line: blue west of this longitude, red east
    front_longitude: f64,
}

const LUXEMBOURG: Spec = Spec {
    name: "luxembourg",
    title: "Luxembourg",
    map_number: 15,
    origin: (49.40, 5.70),
    width_sectors: 30,
    height_sectors: 44,
    ai_sector_size: 4096,
    front_longitude: 6.07,
};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        anyhow::bail!("usage: eech-map <extract.osm.pbf> <srtm dir> <installation root (contains common/)>");
    }
    let (pbf, srtm_dir, root) = (PathBuf::from(&args[1]), PathBuf::from(&args[2]), PathBuf::from(&args[3]));
    let spec = &LUXEMBOURG;
    let geo = geo::Geo::new(spec.origin.0, spec.origin.1);
    let extent = (spec.width_sectors as f64 * SECTOR, spec.height_sectors as f64 * SECTOR);
    let map_dir = root.join("common").join("maps").join(format!("map{}", spec.map_number));
    install_repository_data(&root)?;

    let t = std::time::Instant::now();
    let osm = osm::read(&pbf).context("reading the OSM extract")?;
    eprintln!(
        "osm: {} areas, {} roads, {} places, {} sites ({:?})",
        osm.areas.len(),
        osm.roads.len(),
        osm.places.len(),
        osm.sites.len(),
        t.elapsed()
    );
    let srtm = srtm::Srtm::load(&srtm_dir)?;

    // terrain heights at every grid point
    let px = spec.width_sectors * CELLS + 1;
    let pz = spec.height_sectors * CELLS + 1;
    let mut heights = Vec::with_capacity(px * pz);
    for j in 0..pz {
        for i in 0..px {
            let (lat, lon) = geo.to_geo(i as f64 * CELL, j as f64 * CELL);
            heights.push(srtm.elevation(lat, lon).unwrap_or(0.0) as f32);
        }
    }
    // land cover raster (cell centres), by priority
    let (cx, cz) = (spec.width_sectors * CELLS, spec.height_sectors * CELLS);
    let mut cover = vec![0u8; cx * cz];
    let rank = |c: osm::Cover| match c {
        osm::Cover::Field => 1,
        osm::Cover::Forest => 2,
        osm::Cover::Military => 3,
        osm::Cover::Industrial => 4,
        osm::Cover::Urban => 5,
        osm::Cover::Water => 6,
    };
    for area in &osm.areas {
        let ring: Vec<(f64, f64)> = area
            .ring
            .iter()
            .map(|(lat, lon)| geo.to_map(*lat, *lon))
            .map(|(x, z)| (x / CELL, z / CELL))
            .collect();
        fill(&ring, cx, cz, |i, j| {
            let r = rank(area.cover);
            if cover[j * cx + i] < r {
                cover[j * cx + i] = r;
            }
        });
    }
    let cells: Vec<u8> = cover
        .iter()
        .enumerate()
        .map(|(k, r)| match r {
            1 => types::FIELD1 + (((k * 2654435761) >> 7) % 11) as u8,
            2 => types::FOREST_FLOOR,
            3 => types::ALTERED_LAND1,
            4 => types::BUILT_UP_AREA3,
            5 => types::BUILT_UP_AREA1,
            6 => types::RESERVOIR,
            _ => types::LAND,
        })
        .collect();
    let terrain = Terrain {
        width: spec.width_sectors,
        height: spec.height_sectors,
        heights,
        cells,
    };
    terrain.write(&map_dir.join("terrain"))?;
    eprintln!("terrain: {} x {} sectors", spec.width_sectors, spec.height_sectors);

    // roads
    let net = roads::Network::build(&osm.roads, osm::RoadClass::Secondary, &geo, extent)?;
    net.write(&map_dir.join("route"), &terrain)?;
    eprintln!("roads: {} nodes, {} links", net.nodes.len(), net.links.len());

    // sides
    let (ax, az) = (
        (extent.0 / spec.ai_sector_size as f64) as usize,
        (extent.1 / spec.ai_sector_size as f64) as usize,
    );
    let mut sides = Sides {
        width: ax,
        height: az,
        side: Vec::with_capacity(ax * az),
    };
    for j in 0..az {
        for i in 0..ax {
            let (_, lon) = geo.to_geo((i as f64 + 0.5) * spec.ai_sector_size as f64, (j as f64 + 0.5) * spec.ai_sector_size as f64);
            sides.side.push(if lon < spec.front_longitude { Side::Blue } else { Side::Red });
        }
    }
    let side_at =
        |x: f64, z: f64| sides.side[((z / spec.ai_sector_size as f64) as usize).min(az - 1) * ax + ((x / spec.ai_sector_size as f64) as usize).min(ax - 1)];

    // airfields: the named aerodromes, largest first; each side's two largest
    // (at least 10 km apart) get its side's airport scenes. A side needs a second
    // airbase: a REPAIR or SUPPLY task never starts from the keysite it serves
    // (taskgen.c), so a lone airbase put out of action stays out of action.
    let mut aerodromes: Vec<&osm::Site> = osm.sites.iter().filter(|s| s.kind == osm::SiteKind::Aerodrome && !s.name.is_empty()).collect();
    aerodromes.sort_by(|a, b| b.size.total_cmp(&a.size));
    let mut airfields = Vec::new();
    let mut popnames = Vec::new();
    for a in aerodromes {
        let (x, z) = geo.to_map(a.position.0, a.position.1);
        if x < 2000.0 || z < 2000.0 || x > extent.0 - 2000.0 || z > extent.1 - 2000.0 {
            continue;
        }
        let side = side_at(x, z);
        let same_side = airfields.iter().filter(|f: &&(Side, Airfield)| f.0 == side).count();
        if same_side >= 2 || airfields.iter().any(|(_, f)| ((f.x - x).powi(2) + (f.z - z).powi(2)).sqrt() < 10_000.0) {
            continue;
        }
        eprintln!("airbase: {} ({side:?})", a.name);
        let scene = match (side, same_side) {
            (Side::Blue, 0) => "AMERICAN_AIRPORT01",
            (Side::Blue, _) => "AMERICAN_AIRPORT02",
            (Side::Red, 0) => "RUSSIAN_AIRPORT01",
            (Side::Red, _) => "RUSSIAN_AIRPORT02",
        };
        popnames.push(PopName {
            name: a.name.clone(),
            keysite: true,
            x,
            z,
            zoom_km: 5.0,
        });
        airfields.push((
            side,
            Airfield {
                x,
                z,
                scene: scene.to_string(),
            },
        ));
    }
    // a side short of aerodromes gets an airbase at its town farthest from its
    // other airbases, at least 8 km behind the front and inside the map
    for side in [Side::Blue, Side::Red] {
        while airfields.iter().filter(|f| f.0 == side).count() < 2 {
            let best = osm
                .places
                .iter()
                .filter(|p| p.kind != osm::PlaceKind::Village)
                .filter_map(|p| {
                    let (x, z) = geo.to_map(p.position.0, p.position.1);
                    let inside = x > 4000.0 && z > 4000.0 && x < extent.0 - 4000.0 && z < extent.1 - 4000.0;
                    let behind = (p.position.1 - spec.front_longitude).abs() > 0.11;
                    (inside && behind && side_at(x, z) == side).then(|| {
                        let d = airfields.iter().map(|(_, f)| ((f.x - x).powi(2) + (f.z - z).powi(2)).sqrt()).fold(f64::MAX, f64::min);
                        (p, x, z, d)
                    })
                })
                .filter(|c| c.3 > 10_000.0)
                .max_by(|a, b| a.3.total_cmp(&b.3));
            let Some((place, tx, tz, _)) = best else { break };
            // off water: EECH makes a keysite on water terrain an anchorage (popread.c)
            let dry = |x: f64, z: f64| {
                let r = (800.0 / CELL) as i64;
                let (i0, j0) = ((x / CELL) as i64, (z / CELL) as i64);
                (-r..=r).all(|dj| {
                    (-r..=r).all(|di| {
                        let (i, j) = (i0 + di, j0 + dj);
                        i < 0 || j < 0 || i >= cx as i64 || j >= cz as i64 || cover[j as usize * cx + i as usize] != 6
                    })
                })
            };
            let spot = (0..=8)
                .flat_map(|ring| (0..16).map(move |k| (ring as f64 * 500.0, k as f64 * std::f64::consts::PI / 8.0)))
                .map(|(d, a)| (tx + d * a.cos(), tz + d * a.sin()))
                .find(|&(x, z)| dry(x, z) && side_at(x, z) == side);
            let Some((x, z)) = spot else { break };
            let name = format!("{} Airbase", campaign::ascii(&place.name));
            eprintln!("airbase: {name} ({side:?}, synthesised)");
            let n = airfields.iter().filter(|f| f.0 == side).count();
            let scene = match (side, n) {
                (Side::Blue, 0) => "AMERICAN_AIRPORT01",
                (Side::Blue, _) => "AMERICAN_AIRPORT02",
                (Side::Red, 0) => "RUSSIAN_AIRPORT01",
                (Side::Red, _) => "RUSSIAN_AIRPORT02",
            };
            popnames.push(PopName {
                name,
                keysite: true,
                x,
                z,
                zoom_km: 5.0,
            });
            airfields.push((
                side,
                Airfield {
                    x,
                    z,
                    scene: scene.to_string(),
                },
            ));
        }
    }
    for p in &osm.places {
        if p.kind == osm::PlaceKind::Village {
            continue;
        }
        let (x, z) = geo.to_map(p.position.0, p.position.1);
        if x > 0.0 && z > 0.0 && x < extent.0 && z < extent.1 {
            popnames.push(PopName {
                name: p.name.clone(),
                keysite: false,
                x,
                z,
                zoom_km: if p.kind == osm::PlaceKind::City { 10.0 } else { 5.0 },
            });
        }
    }
    // FARPs: two per side, 6 km behind the front, north and south
    let mut farps: Vec<(Side, Airfield)> = Vec::new();
    for (side, scene, dlon) in [(Side::Blue, "AMERICAN_FARP01", -0.085), (Side::Red, "RUSSIAN_FARP01", 0.085)] {
        for lat in [49.62, 49.90] {
            let (x, z) = geo.to_map(lat, spec.front_longitude + dlon);
            farps.push((
                side,
                Airfield {
                    x,
                    z,
                    scene: scene.to_string(),
                },
            ));
        }
    }
    // SAM/AAA sites: two per side, around each airbase
    let mut sams = Vec::new();
    for (_, a) in &airfields {
        sams.push(campaign::Sam {
            x: a.x + 1500.0,
            z: a.z + 1500.0,
        });
        sams.push(campaign::Sam {
            x: a.x - 1500.0,
            z: a.z - 1500.0,
        });
    }
    let route = map_dir.join("route");
    campaign::write_popnames(&route.join("popname.dat"), &popnames)?;
    campaign::write_bridge_types(&route.join("bridge.pop"))?;
    campaign::write_mapinfo(&map_dir.join("mapinfo.txt"), spec.origin.0, spec.origin.1)?;

    let camp = map_dir.join("camp01");
    std::fs::create_dir_all(&camp)?;
    sides.write_psd(&camp.join(format!("{}.sid", spec.name)))?;
    // placement order names the keysites: airbases by popname.dat, FARPs "FARP n" in order
    let airbase_names: Vec<(Side, String)> = airfields.iter().map(|(side, a)| (*side, keysite_name(&popnames, a.x, a.z))).collect();
    let farp_names: Vec<(Side, String)> = farps.iter().enumerate().map(|(i, (side, _))| (*side, format!("FARP {}", i + 1))).collect();
    let mut placements: Vec<Airfield> = airfields.into_iter().map(|(_, a)| a).collect();
    placements.extend(farps.into_iter().map(|(_, a)| a));
    campaign::write_population(&camp.join(format!("{}.pop", spec.name)), extent.1 - 1.0, &placements, &sams)?;

    // the campaign script
    let game_path = format!("..\\common\\maps\\map{}\\camp01", spec.map_number);
    let mut chc = String::new();
    let _ = writeln!(chc, ":START");
    let _ = writeln!(chc, ":TITLE {}", spec.title);
    let _ = writeln!(chc, ":CAMPAIGN_DATA");
    let _ = writeln!(chc, ":FILENAME {game_path}\\{}.sid", spec.name);
    let _ = writeln!(chc, ":FILENAME {game_path}\\{}.pop", spec.name);
    let _ = writeln!(chc, ":MAP_X_SIZE {ax}");
    let _ = writeln!(chc, ":MAP_Z_SIZE {az}");
    let _ = writeln!(chc, ":MAP_SECTOR_SIZE {}", spec.ai_sector_size);
    let _ = writeln!(chc, ":FACTION\n:SIDE SIDE_BLUE_FORCE\n:COLOUR COL_BLUE");
    let _ = writeln!(chc, ":FACTION\n:SIDE SIDE_RED_FORCE\n:COLOUR COL_RED");
    let _ = writeln!(chc, ":END");
    for side in [Side::Blue, Side::Red] {
        campaign::write_force(&mut chc, side, &airbase_names, &farp_names);
    }
    let _ = writeln!(chc, ":END");
    std::fs::write(camp.join(format!("{}.chc", spec.name)), chc)?;
    eprintln!("campaign: {} keysites, {} names -> {}", placements.len(), popnames.len(), map_dir.display());
    Ok(())
}

/// the EECH data files the repository carries (setup/common/data): formation
/// databases, the language database and the suspension tables. The
/// generated rest of an installation (3D database, texture names, briefing
/// texts) comes from the engine (eech_dc.prepare_installation).
fn install_repository_data(root: &std::path::Path) -> Result<()> {
    let setup = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../setup/common/data");
    let data = root.join("common").join("data");
    std::fs::create_dir_all(root.join("cohokum"))?;
    for sub in ["", "language", "suspension"] {
        let from = setup.join(sub);
        let to = data.join(sub);
        std::fs::create_dir_all(&to)?;
        for entry in std::fs::read_dir(&from).with_context(|| format!("reading {}", from.display()))? {
            let path = entry?.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    std::fs::copy(&path, to.join(name))?;
                }
            }
        }
    }
    // the game directory's tables, as an installation ships them (setup/cohokum):
    // the weapon and unit tuning table (eechini.c DEFAULT_GWUT_FILE) and the
    // explosion and smoke databases. Weapon weights, drag and motor power come
    // only from the GWUT table (the compiled weapon database leaves them zero,
    // and a missile launched without it flies with a NaN velocity). Without
    // EXPLOS.CSV, EECH exports its compiled explosion database and runs on it,
    // and that database declares components it never initialises
    // (XSMALL_HE_META_EXPLOSION: 5 declared, 3 set).
    let cohokum = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../setup/cohokum");
    for name in ["GWUT1162.CSV", "EXPLOS.CSV", "METASMOK.CSV", "SMOKES.CSV"] {
        let from = cohokum.join(name);
        std::fs::copy(&from, root.join("cohokum").join(name)).with_context(|| format!("copying {}", from.display()))?;
    }
    Ok(())
}

/// get_keysite_name (popread.c): the KEYSITE name in popname.dat within 5 km
fn keysite_name(names: &[PopName], x: f64, z: f64) -> String {
    names
        .iter()
        .filter(|n| n.keysite && ((n.x - x).powi(2) + (n.z - z).powi(2)).sqrt() < 5000.0)
        .map(|n| campaign::ascii(&n.name))
        .next()
        .unwrap_or_default()
}

/// scanline fill of a polygon (cell units) over the cells whose centres are inside
fn fill(ring: &[(f64, f64)], width: usize, height: usize, mut f: impl FnMut(usize, usize)) {
    if ring.len() < 3 {
        return;
    }
    let (zmin, zmax) = ring.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.1), b.max(p.1)));
    let j0 = (zmin - 0.5).ceil().max(0.0) as usize;
    let j1 = ((zmax - 0.5).floor() as i64).min(height as i64 - 1);
    if j1 < 0 {
        return;
    }
    let mut xs = Vec::new();
    for j in j0..=(j1 as usize) {
        let zc = j as f64 + 0.5;
        xs.clear();
        for k in 0..ring.len() {
            let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
            if (a.1 <= zc) != (b.1 <= zc) {
                xs.push(a.0 + (zc - a.1) / (b.1 - a.1) * (b.0 - a.0));
            }
        }
        xs.sort_by(f64::total_cmp);
        for pair in xs.chunks_exact(2) {
            let i0 = (pair[0] - 0.5).ceil().max(0.0) as usize;
            let i1 = ((pair[1] - 0.5).floor() as i64).min(width as i64 - 1);
            if i1 >= 0 {
                for i in i0..=(i1 as usize) {
                    f(i, j);
                }
            }
        }
    }
}
