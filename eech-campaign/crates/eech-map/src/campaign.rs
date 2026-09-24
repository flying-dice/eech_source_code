//! The campaign side of the map: the side file (`.sid`, a Photoshop image of
//! the AI sectors), the population placement file (binary: templates, cities,
//! airfields, SAM/AAA sites), the campaign script (`.chc`) and the per-map
//! text files (`route\popname.dat`, `route\bridge.pop`, `mapinfo.txt`).

use anyhow::Result;
use std::fmt::Write as _;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Blue,
    Red,
}

/// a sector side map (AI sectors), z-major, row 0 = south
pub struct Sides {
    pub width: usize,
    pub height: usize,
    pub side: Vec<Side>,
}

impl Sides {
    /// `.sid`: a 3-channel 8-bit RGB Photoshop file, big-endian, planar,
    /// row 0 = north (popread.c :: read_sector_side_file: blue > 240 or red > 240)
    pub fn write_psd(&self, path: &Path) -> Result<()> {
        let mut o = Vec::new();
        o.extend_from_slice(b"8BPS");
        o.extend_from_slice(&1u16.to_be_bytes());
        o.extend_from_slice(&[0; 6]);
        o.extend_from_slice(&3u16.to_be_bytes());
        o.extend_from_slice(&(self.height as u32).to_be_bytes());
        o.extend_from_slice(&(self.width as u32).to_be_bytes());
        o.extend_from_slice(&8u16.to_be_bytes());
        o.extend_from_slice(&3u16.to_be_bytes());
        o.extend_from_slice(&0u32.to_be_bytes()); // colour mode data
        o.extend_from_slice(&0u32.to_be_bytes()); // image resources
        o.extend_from_slice(&0u32.to_be_bytes()); // layers and masks
        o.extend_from_slice(&0u16.to_be_bytes()); // raw
        for channel in 0..3 {
            for row in 0..self.height {
                let z = self.height - 1 - row;
                for x in 0..self.width {
                    let s = self.side[z * self.width + x];
                    let v = match (channel, s) {
                        (0, Side::Red) | (2, Side::Blue) => 255u8,
                        _ => 0,
                    };
                    o.push(v);
                }
            }
        }
        std::fs::write(path, o)?;
        Ok(())
    }
}

pub struct Airfield {
    pub x: f64,
    pub z: f64,
    pub scene: String,
}

pub struct Sam {
    pub x: f64,
    pub z: f64,
}

/// the binary population placement file (popread.c). z is stored flipped
/// against the map's maximum z rounded down to 100 m.
pub fn write_population(path: &Path, max_map_z: f64, airfields: &[Airfield], sams: &[Sam]) -> Result<()> {
    let flip = ((max_map_z / 100.0).floor() * 100.0) as f32;
    let mut o = Vec::new();
    o.extend_from_slice(&0i32.to_le_bytes()); // templates
    o.extend_from_slice(&0i32.to_le_bytes()); // city placements
    o.extend_from_slice(&(airfields.len() as i32).to_le_bytes());
    for a in airfields {
        o.extend_from_slice(&(a.x as f32).to_le_bytes());
        o.extend_from_slice(&(flip - a.z as f32).to_le_bytes());
        let mut name = a.scene.clone().into_bytes();
        name.push(0);
        o.extend_from_slice(&(name.len() as i32).to_le_bytes());
        o.extend_from_slice(&name);
    }
    o.extend_from_slice(&(sams.len() as i32).to_le_bytes());
    for s in sams {
        o.extend_from_slice(&(s.x as f32).to_le_bytes());
        o.extend_from_slice(&(flip - s.z as f32).to_le_bytes());
    }
    std::fs::write(path, o)?;
    Ok(())
}

pub struct PopName {
    pub name: String,
    pub keysite: bool,
    pub x: f64,
    pub z: f64,
    pub zoom_km: f64,
}

pub fn write_popnames(path: &Path, names: &[PopName]) -> Result<()> {
    let mut s = String::from(":START\n");
    for n in names {
        let _ = writeln!(s, ":NAME {}", ascii(&n.name));
        let _ = writeln!(s, ":POPULATION_TYPE {}", if n.keysite { "KEYSITE" } else { "TOWN" });
        let _ = writeln!(s, ":POSITION {:.0} {:.0}", n.x, n.z);
        let _ = writeln!(s, ":ZOOM {:.1}", n.zoom_km);
    }
    s.push_str(":END\n");
    std::fs::write(path, s)?;
    Ok(())
}

pub fn write_bridge_types(path: &Path) -> Result<()> {
    std::fs::write(
        path,
        ":START\n:BRIDGE\n:TYPE BRIDGE_GIRDER\n:TYPE BRIDGE_CONCRETE_ARCH\n:TYPE BRIDGE_STONE\n:END\n",
    )?;
    Ok(())
}

pub fn write_mapinfo(path: &Path, latitude: f64, longitude: f64) -> Result<()> {
    std::fs::write(
        path,
        format!("season=1#summer\ncoordinate={latitude:.6},{longitude:.6}#map origin (x=0, z=0)\n"),
    )?;
    Ok(())
}

/// EECH's text files are Latin-1 read as bytes; names are written in ASCII
pub fn ascii(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'û' | 'ü' | 'ù' => 'u',
            'ç' => 'c',
            'É' => 'E',
            'Ä' => 'A',
            'Ö' => 'O',
            'Ü' => 'U',
            ':' | '\\' => ' ',
            c if c.is_ascii() => c,
            _ => '_',
        })
        .collect()
}
