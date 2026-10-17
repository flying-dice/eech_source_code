//! SRTM elevation (the 1 arc-second .hgt tiles, gzipped: NxxEyyy.hgt.gz).

use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

pub struct Srtm {
    tiles: HashMap<(i32, i32), Tile>,
}

struct Tile {
    size: usize,
    samples: Vec<i16>,
}

impl Srtm {
    /// loads every N??E???.hgt.gz / .hgt in the directory
    pub fn load(dir: &Path) -> Result<Self> {
        let mut tiles = HashMap::new();
        for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
            let path = entry?.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
            let Some(stem) = name.strip_suffix(".hgt.gz").or_else(|| name.strip_suffix(".hgt")) else {
                continue;
            };
            if stem.len() != 7 {
                continue;
            }
            let lat: i32 = stem[1..3].parse()?;
            let lon: i32 = stem[4..7].parse()?;
            let lat = if &stem[0..1] == "S" { -lat } else { lat };
            let lon = if &stem[3..4] == "W" { -lon } else { lon };
            let mut bytes = Vec::new();
            if name.ends_with(".gz") {
                GzDecoder::new(std::fs::File::open(&path)?).read_to_end(&mut bytes)?;
            } else {
                bytes = std::fs::read(&path)?;
            }
            let n = bytes.len() / 2;
            let size = (n as f64).sqrt() as usize;
            if size * size != n {
                bail!("{}: not a square SRTM tile", path.display());
            }
            let samples = bytes.chunks_exact(2).map(|b| i16::from_be_bytes([b[0], b[1]])).collect();
            tiles.insert((lat, lon), Tile { size, samples });
        }
        if tiles.is_empty() {
            bail!("no SRTM tiles in {}", dir.display());
        }
        Ok(Srtm { tiles })
    }

    fn sample(&self, lat_i: i64, lon_i: i64, per_degree: i64) -> Option<f64> {
        // tile rows run north to south
        let tile_lat = lat_i.div_euclid(per_degree) as i32;
        let tile_lon = lon_i.div_euclid(per_degree) as i32;
        let tile = self.tiles.get(&(tile_lat, tile_lon))?;
        let row = (per_degree - lat_i.rem_euclid(per_degree)) as usize;
        let col = lon_i.rem_euclid(per_degree) as usize;
        let v = *tile.samples.get(row.min(tile.size - 1) * tile.size + col.min(tile.size - 1))?;
        (v != i16::MIN).then_some(v as f64)
    }

    /// metres above sea level, bilinear; None outside the tiles
    pub fn elevation(&self, latitude: f64, longitude: f64) -> Option<f64> {
        let size = self.tiles.values().next()?.size as i64;
        let per_degree = size - 1;
        let fy = latitude * per_degree as f64;
        let fx = longitude * per_degree as f64;
        let (y0, x0) = (fy.floor() as i64, fx.floor() as i64);
        let (ty, tx) = (fy - y0 as f64, fx - x0 as f64);
        let s00 = self.sample(y0, x0, per_degree)?;
        let s01 = self.sample(y0, x0 + 1, per_degree).unwrap_or(s00);
        let s10 = self.sample(y0 + 1, x0, per_degree).unwrap_or(s00);
        let s11 = self.sample(y0 + 1, x0 + 1, per_degree).unwrap_or(s00);
        Some((s00 * (1.0 - tx) + s01 * tx) * (1.0 - ty) + (s10 * (1.0 - tx) + s11 * tx) * ty)
    }
}
