//! The road network the ground AI routes over: `route\ROADS.dat` (nodes and
//! links), `ROADS.nde` (node positions) and `ROADS.wp` (link geometry), as
//! `aphavoc/source/ai/ai_misc/ai_route.c` reads them.
//!
//! Graph nodes are the OSM nodes where selected roads meet or end; chains of
//! degree-2 nodes are contracted into one link; only the largest connected
//! component is kept (the AI routes between any two nodes).

use crate::geo::Geo;
use crate::osm::{Road, RoadClass};
use crate::terrain::Terrain;
use anyhow::{bail, Result};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// ai_route.h: node:14 bits, number_of_links:7 bits
const MAX_NODES: usize = 16383;
const MAX_LINKS: usize = 127;
/// terrtype.h
const TERRAIN_TYPE_ROAD: i32 = 41;

pub struct Network {
    pub nodes: Vec<(f64, f64)>,
    /// (a, b, intermediate points), a < b
    pub links: Vec<(usize, usize, Vec<(f64, f64)>)>,
}

impl Network {
    pub fn build(roads: &[Road], max_class: RoadClass, geo: &Geo, extent: (f64, f64)) -> Result<Network> {
        let inside = |p: (f64, f64)| p.0 > 50.0 && p.1 > 50.0 && p.0 < extent.0 - 50.0 && p.1 < extent.1 - 50.0;
        let selected: Vec<Vec<(i64, (f64, f64))>> = roads
            .iter()
            .filter(|r| r.class <= max_class)
            .map(|r| r.nodes.iter().map(|(id, (lat, lon))| (*id, geo.to_map(*lat, *lon))).collect())
            .collect();
        // graph nodes: endpoints and nodes shared by ways
        let mut uses: HashMap<i64, usize> = HashMap::new();
        for w in &selected {
            for (i, (id, _)) in w.iter().enumerate() {
                *uses.entry(*id).or_default() += if i == 0 || i == w.len() - 1 { 2 } else { 1 };
            }
        }
        // segments between graph nodes, inside the map
        let mut edges: BTreeMap<(i64, i64), Vec<(f64, f64)>> = BTreeMap::new();
        let mut pos: HashMap<i64, (f64, f64)> = HashMap::new();
        for w in &selected {
            let mut start = 0;
            for i in 1..w.len() {
                if uses[&w[i].0] >= 2 {
                    let seg = &w[start..=i];
                    if seg.iter().all(|(_, p)| inside(*p)) && seg[0].0 != seg[seg.len() - 1].0 {
                        let (a, b) = (seg[0].0, seg[seg.len() - 1].0);
                        let mut mid: Vec<(f64, f64)> = seg[1..seg.len() - 1].iter().map(|(_, p)| *p).collect();
                        let key = if a < b {
                            (a, b)
                        } else {
                            mid.reverse();
                            (b, a)
                        };
                        let len = path_length(pos_of(&seg[0]), &mid, pos_of(&seg[seg.len() - 1]));
                        let keep = edges
                            .get(&key)
                            .map(|old| len < path_length(pos_of(&seg[0]), old, pos_of(&seg[seg.len() - 1])))
                            .unwrap_or(true);
                        if keep {
                            edges.insert(key, mid);
                        }
                        pos.insert(a, seg[0].1);
                        pos.insert(b, seg[seg.len() - 1].1);
                    }
                    start = i;
                }
            }
        }
        // contract degree-2 nodes (in node id order: deterministic)
        let mut adj: BTreeMap<i64, std::collections::BTreeSet<i64>> = BTreeMap::new();
        for &(a, b) in edges.keys() {
            adj.entry(a).or_default().insert(b);
            adj.entry(b).or_default().insert(a);
        }
        let candidates: Vec<i64> = adj.iter().filter(|(_, v)| v.len() == 2).map(|(k, _)| *k).collect();
        for n in candidates {
            let Some(nb) = adj.get(&n) else { continue };
            if nb.len() != 2 {
                continue;
            }
            let mut it = nb.iter();
            let (p, q) = match (it.next(), it.next()) {
                (Some(&p), Some(&q)) => (p, q),
                _ => continue,
            };
            if edges.contains_key(&(p.min(q), p.max(q))) {
                continue;
            }
            let oriented = |a: i64, b: i64, edges: &BTreeMap<(i64, i64), Vec<(f64, f64)>>| -> Vec<(f64, f64)> {
                let mid = edges[&(a.min(b), a.max(b))].clone();
                if a < b {
                    mid
                } else {
                    mid.into_iter().rev().collect()
                }
            };
            let mut path = oriented(p, n, &edges);
            path.push(pos[&n]);
            path.extend(oriented(n, q, &edges));
            edges.remove(&(p.min(n), p.max(n)));
            edges.remove(&(n.min(q), n.max(q)));
            if q < p {
                path.reverse();
            }
            edges.insert((p.min(q), p.max(q)), path);
            adj.remove(&n);
            if let Some(s) = adj.get_mut(&p) {
                s.remove(&n);
                s.insert(q);
            }
            if let Some(s) = adj.get_mut(&q) {
                s.remove(&n);
                s.insert(p);
            }
        }
        // largest connected component
        let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
        for &(a, b) in edges.keys() {
            adj.entry(a).or_default().push(b);
            adj.entry(b).or_default().push(a);
        }
        let mut seen: HashSet<i64> = HashSet::new();
        let mut best: Vec<i64> = Vec::new();
        let mut keys: Vec<i64> = adj.keys().copied().collect();
        keys.sort_unstable();
        for &s in &keys {
            if seen.contains(&s) {
                continue;
            }
            let mut comp = vec![s];
            seen.insert(s);
            let mut i = 0;
            while i < comp.len() {
                for &m in &adj[&comp[i]] {
                    if seen.insert(m) {
                        comp.push(m);
                    }
                }
                i += 1;
            }
            if comp.len() > best.len() {
                best = comp;
            }
        }
        best.sort_unstable();
        let index: HashMap<i64, usize> = best.iter().enumerate().map(|(i, id)| (*id, i)).collect();
        let nodes: Vec<(f64, f64)> = best.iter().map(|id| pos[id]).collect();
        let mut links = Vec::new();
        for ((a, b), mid) in &edges {
            if let (Some(&ia), Some(&ib)) = (index.get(a), index.get(b)) {
                let (ia, ib, mid) = if ia < ib {
                    (ia, ib, mid.clone())
                } else {
                    (ib, ia, mid.iter().rev().copied().collect())
                };
                links.push((ia, ib, simplify(nodes[ia], &mid, nodes[ib])));
            }
        }
        if nodes.len() > MAX_NODES {
            bail!("road network has {} nodes; EECH allows {MAX_NODES}", nodes.len());
        }
        let mut degree = vec![0usize; nodes.len()];
        for (a, b, _) in &links {
            degree[*a] += 1;
            degree[*b] += 1;
        }
        if degree.iter().any(|d| *d > MAX_LINKS) {
            bail!("a road node has more than {MAX_LINKS} links");
        }
        Ok(Network { nodes, links })
    }

    pub fn write(&self, dir: &Path, terrain: &Terrain) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let v3 = |out: &mut Vec<u8>, (x, z): (f64, f64)| {
            out.extend_from_slice(&(x as f32).to_le_bytes());
            out.extend_from_slice(&(terrain.elevation(x, z) as f32).to_le_bytes());
            out.extend_from_slice(&(z as f32).to_le_bytes());
        };
        let mut neighbours: Vec<Vec<(usize, i32)>> = vec![Vec::new(); self.nodes.len()];
        for (a, b, mid) in &self.links {
            let cost = path_length(self.nodes[*a], mid, self.nodes[*b]).round() as i32;
            neighbours[*a].push((*b, cost.max(1)));
            neighbours[*b].push((*a, cost.max(1)));
        }
        let mut dat = Vec::new();
        dat.extend_from_slice(&0f32.to_le_bytes());
        dat.extend_from_slice(&0f32.to_le_bytes());
        dat.extend_from_slice(&(self.nodes.len() as i32).to_le_bytes());
        for (i, n) in neighbours.iter().enumerate() {
            dat.extend_from_slice(&(i as i32).to_le_bytes());
            dat.extend_from_slice(&28i32.to_le_bytes()); // safe radius (ignored)
            dat.extend_from_slice(&(n.len() as i32).to_le_bytes());
            for (m, cost) in n {
                dat.extend_from_slice(&(*m as i32).to_le_bytes());
                dat.extend_from_slice(&cost.to_le_bytes());
            }
        }
        let mut nde = Vec::new();
        nde.extend_from_slice(&(self.nodes.len() as i32).to_le_bytes());
        for p in &self.nodes {
            v3(&mut nde, *p);
        }
        let mut wp = Vec::new();
        wp.extend_from_slice(&(self.links.len() as i32).to_le_bytes());
        for (a, b, mid) in &self.links {
            wp.extend_from_slice(&(*a as i32).to_le_bytes());
            wp.extend_from_slice(&(*b as i32).to_le_bytes());
            wp.extend_from_slice(&TERRAIN_TYPE_ROAD.to_le_bytes());
            wp.extend_from_slice(&(mid.len() as i32).to_le_bytes());
            for p in mid {
                v3(&mut wp, *p);
            }
        }
        std::fs::write(dir.join("ROADS.dat"), dat)?;
        std::fs::write(dir.join("ROADS.nde"), nde)?;
        std::fs::write(dir.join("ROADS.wp"), wp)?;
        Ok(())
    }
}

fn pos_of(n: &(i64, (f64, f64))) -> (f64, f64) {
    n.1
}

fn path_length(a: (f64, f64), mid: &[(f64, f64)], b: (f64, f64)) -> f64 {
    let mut len = 0.0;
    let mut last = a;
    for p in mid.iter().chain(std::iter::once(&b)) {
        len += ((p.0 - last.0).powi(2) + (p.1 - last.1).powi(2)).sqrt();
        last = *p;
    }
    len
}

/// keeps intermediate points at least 100 m apart; always at least one
fn simplify(a: (f64, f64), mid: &[(f64, f64)], b: (f64, f64)) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut last = a;
    for p in mid {
        let d = ((p.0 - last.0).powi(2) + (p.1 - last.1).powi(2)).sqrt();
        let to_end = ((p.0 - b.0).powi(2) + (p.1 - b.1).powi(2)).sqrt();
        if d >= 100.0 && to_end >= 50.0 {
            out.push(*p);
            last = *p;
        }
    }
    if out.is_empty() {
        out.push(((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0));
    }
    out
}
