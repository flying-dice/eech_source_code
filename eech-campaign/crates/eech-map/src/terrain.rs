//! EECH terrain: `terrain\terrain.ffp` (geometry), `default.sec` (sector
//! directory) and `default.rgb` (colours), as `modules/3d/terrain/terrdata.c`
//! (`load_3d_terrain`) reads them.
//!
//! Each 2048 m sector is a regular grid of CELLS x CELLS quads, one fan face
//! per quad (SW, NW, NE, SE: clockwise from above, as the elevation lookup's
//! inside test requires), typed from the land-cover raster. Points are
//! 1/16 m offsets from the sector centre; heights are 15-bit fractions of the
//! map's height range; normals index EECH's fixed table of 254 normals.

use anyhow::Result;
use std::path::Path;

pub const SECTOR: f64 = 2048.0;
pub const CELLS: usize = 8;
pub const CELL: f64 = SECTOR / CELLS as f64;

/// terrain types (modules/3d/terrain/terrtype.h)
pub mod types {
    pub const RESERVOIR: u8 = 2;
    pub const LAND: u8 = 9;
    pub const FIELD1: u8 = 10;
    pub const ALTERED_LAND1: u8 = 21;
    pub const FOREST_FLOOR: u8 = 24;
    pub const BUILT_UP_AREA1: u8 = 37;
    pub const BUILT_UP_AREA3: u8 = 39;
}

pub struct Terrain {
    pub width: usize,
    pub height: usize,
    /// heights at grid points, (width*CELLS+1) x (height*CELLS+1), z-major
    pub heights: Vec<f32>,
    /// terrain type per cell, (width*CELLS) x (height*CELLS), z-major
    pub cells: Vec<u8>,
}

impl Terrain {
    pub fn points_x(&self) -> usize {
        self.width * CELLS + 1
    }

    pub fn height_at_point(&self, px: usize, pz: usize) -> f32 {
        self.heights[pz * self.points_x() + px]
    }

    /// bilinear height at world (x, z)
    pub fn elevation(&self, x: f64, z: f64) -> f64 {
        let fx = (x / CELL).clamp(0.0, (self.width * CELLS) as f64 - 1e-6);
        let fz = (z / CELL).clamp(0.0, (self.height * CELLS) as f64 - 1e-6);
        let (x0, z0) = (fx.floor() as usize, fz.floor() as usize);
        let (tx, tz) = (fx - x0 as f64, fz - z0 as f64);
        let h = |px, pz| self.height_at_point(px, pz) as f64;
        (h(x0, z0) * (1.0 - tx) + h(x0 + 1, z0) * tx) * (1.0 - tz) + (h(x0, z0 + 1) * (1.0 - tx) + h(x0 + 1, z0 + 1) * tx) * tz
    }

    pub fn cell_type(&self, x: f64, z: f64) -> u8 {
        let cx = ((x / CELL) as usize).min(self.width * CELLS - 1);
        let cz = ((z / CELL) as usize).min(self.height * CELLS - 1);
        self.cells[cz * self.width * CELLS + cx]
    }

    fn normal_index(&self, px: usize, pz: usize) -> u8 {
        let max_x = self.width * CELLS;
        let max_z = self.height * CELLS;
        let h = |x: usize, z: usize| self.height_at_point(x.min(max_x), z.min(max_z)) as f64;
        let dhdx = (h(px + 1, pz) - h(px.saturating_sub(1), pz)) / (CELL * (if px == 0 || px == max_x { 1.0 } else { 2.0 }));
        let dhdz = (h(px, pz + 1) - h(px, pz.saturating_sub(1))) / (CELL * (if pz == 0 || pz == max_z { 1.0 } else { 2.0 }));
        // normal (-dh/dx, 1, -dh/dz)
        let (nx, ny, nz) = (-dhdx, 1.0, -dhdz);
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        let pitch = (ny / len).asin().to_degrees();
        if pitch > 85.5 {
            return 240;
        }
        let band = ((pitch / 9.0).round() as i32).clamp(0, 9) as u8;
        let heading = nx.atan2(nz).to_degrees().rem_euclid(360.0);
        let hband = ((heading / 15.0).round() as i32).rem_euclid(24) as u8;
        band * 24 + hband
    }

    pub fn write(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let pts = CELLS + 1;
        let (min_h, max_h) = self.heights.iter().fold((f32::MAX, f32::MIN), |(a, b), &h| (a.min(h), b.max(h)));
        let (min_h, max_h) = (min_h.floor() - 1.0, max_h.ceil() + 1.0);
        let range = (max_h - min_h) as f64;

        let mut sec = Vec::new();
        let mut ffp = Vec::new();
        let mut rgb = Vec::new();
        sec.extend_from_slice(&0u32.to_le_bytes()); // version (the map number is taken from the path)
        sec.extend_from_slice(&(self.width as u16).to_le_bytes());
        sec.extend_from_slice(&(self.height as u16).to_le_bytes());
        sec.extend_from_slice(&min_h.to_le_bytes());
        sec.extend_from_slice(&max_h.to_le_bytes());

        for sz in 0..self.height {
            for sx in 0..self.width {
                let mut ys = Vec::with_capacity(pts * pts);
                let mut normals = Vec::with_capacity(pts * pts);
                let mut xz = Vec::with_capacity(pts * pts);
                let (mut lo, mut hi, mut sum) = (f32::MAX, f32::MIN, 0.0f64);
                for j in 0..pts {
                    for i in 0..pts {
                        let (px, pz) = (sx * CELLS + i, sz * CELLS + j);
                        let h = self.height_at_point(px, pz);
                        lo = lo.min(h);
                        hi = hi.max(h);
                        sum += h as f64;
                        ys.push((((h - min_h) as f64 / range) * 32767.0).round().clamp(0.0, 32767.0) as u16);
                        normals.push(self.normal_index(px, pz));
                        // 1/16 m from the sector centre
                        let ox = (i as f64 * CELL - SECTOR / 2.0) * 16.0;
                        let oz = (j as f64 * CELL - SECTOR / 2.0) * 16.0;
                        xz.push((ox.round() as i16, oz.round() as i16));
                    }
                }
                // faces: one fan per cell, SW NW NE SE
                let mut faces = Vec::new();
                let mut refs = Vec::new();
                let mut surfaces: Vec<u8> = Vec::new();
                for j in 0..CELLS {
                    for i in 0..CELLS {
                        let t = self.cells[(sz * CELLS + j) * self.width * CELLS + sx * CELLS + i];
                        let change = match surfaces.last() {
                            Some(&last) if last == t => 0u8,
                            Some(_) => {
                                surfaces.push(t);
                                1
                            }
                            None => {
                                surfaces.push(t);
                                0
                            }
                        };
                        faces.push(4u8 | (change << 7));
                        let p = |a: usize, b: usize| (b * pts + a) as u8;
                        refs.extend_from_slice(&[p(i, j), p(i, j + 1), p(i + 1, j + 1), p(i + 1, j)]);
                    }
                }
                // normal changes: one entry per change along the point order
                let mut normal_changes: Vec<u8> = Vec::new();
                let mut y_words = Vec::with_capacity(ys.len());
                for (k, (&y, &n)) in ys.iter().zip(normals.iter()).enumerate() {
                    let change = if k == 0 {
                        normal_changes.push(n);
                        0u16
                    } else if normal_changes.last() != Some(&n) {
                        normal_changes.push(n);
                        1
                    } else {
                        0
                    };
                    y_words.push(y | (change << 15));
                }
                // colours: palette by terrain type
                let mut palette: Vec<[u8; 3]> = Vec::new();
                let mut colour_indices = Vec::with_capacity(pts * pts);
                for j in 0..pts {
                    for i in 0..pts {
                        let ci = (sx * CELLS + i).min(self.width * CELLS - 1);
                        let cj = (sz * CELLS + j).min(self.height * CELLS - 1);
                        let c = colour_of(self.cells[cj * self.width * CELLS + ci]);
                        let idx = palette.iter().position(|p| *p == c).unwrap_or_else(|| {
                            palette.push(c);
                            palette.len() - 1
                        });
                        colour_indices.push(idx as u8);
                    }
                }

                let align = |v: &mut Vec<u8>| {
                    while v.len() % 4 != 0 {
                        v.push(0)
                    }
                };
                align(&mut ffp);
                let off_y = ffp.len();
                for w in &y_words {
                    ffp.extend_from_slice(&w.to_le_bytes());
                }
                align(&mut ffp);
                let off_xz = ffp.len();
                for (x, z) in &xz {
                    ffp.extend_from_slice(&x.to_le_bytes());
                    ffp.extend_from_slice(&z.to_le_bytes());
                }
                let off_refs = ffp.len();
                ffp.extend_from_slice(&refs);
                let off_faces = ffp.len();
                ffp.extend_from_slice(&faces);
                let off_surfaces = ffp.len();
                ffp.extend_from_slice(&surfaces);
                let off_normals = ffp.len();
                ffp.extend_from_slice(&normal_changes);

                let off_ci = rgb.len();
                rgb.extend_from_slice(&colour_indices);
                let off_pal = rgb.len();
                for c in &palette {
                    rgb.extend_from_slice(c);
                }

                let avg = (sum / (pts * pts) as f64) as f32;
                let radius = ((2.0 * (SECTOR / 2.0).powi(2) + (((hi - lo) / 2.0) as f64).powi(2)).sqrt()).min(65535.0) as u16;
                sec.extend_from_slice(&((pts * pts) as u16).to_le_bytes());
                sec.extend_from_slice(&0u16.to_le_bytes());
                sec.extend_from_slice(&((CELLS * CELLS) as u16).to_le_bytes());
                sec.extend_from_slice(&radius.to_le_bytes());
                sec.extend_from_slice(&(lo.floor() as i16).to_le_bytes());
                sec.extend_from_slice(&(hi.ceil() as i16).to_le_bytes());
                sec.extend_from_slice(&(avg.round() as i16).to_le_bytes());
                sec.extend_from_slice(&0i16.to_le_bytes());
                for off in [off_y, off_xz, off_refs, off_faces, off_surfaces, off_normals, off_ci, off_pal] {
                    sec.extend_from_slice(&(off as i32).to_le_bytes());
                }
            }
        }
        sec.extend_from_slice(&0u32.to_le_bytes()); // sector approximations
        std::fs::write(dir.join("default.sec"), sec)?;
        std::fs::write(dir.join("terrain.ffp"), ffp)?;
        std::fs::write(dir.join("default.rgb"), rgb)?;
        Ok(())
    }
}

fn colour_of(t: u8) -> [u8; 3] {
    match t {
        types::RESERVOIR => [60, 90, 140],
        types::FOREST_FLOOR => [40, 80, 35],
        types::BUILT_UP_AREA1 | types::BUILT_UP_AREA3 => [130, 120, 110],
        types::ALTERED_LAND1 => [120, 110, 80],
        t if (types::FIELD1..types::FIELD1 + 11).contains(&t) => [110, 140, 60],
        _ => [95, 125, 60],
    }
}
