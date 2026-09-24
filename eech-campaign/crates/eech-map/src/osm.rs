//! What the map needs from an OpenStreetMap extract (.osm.pbf): land cover
//! polygons, the main road network, settlements, aerodromes and the
//! infrastructure that becomes campaign keysites.

use anyhow::Result;
use osmpbf::{Element, ElementReader, RelMemberType};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Cover {
    Field,
    Forest,
    Urban,
    Industrial,
    Military,
    Water,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoadClass {
    Motorway,
    Trunk,
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceKind {
    City,
    Town,
    Village,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteKind {
    Aerodrome,
    PowerPlant,
    Substation,
    Mast,
    Industrial,
    Military,
    Fuel,
}

pub type LatLon = (f64, f64);

#[derive(Debug)]
pub struct Area {
    pub cover: Cover,
    pub ring: Vec<LatLon>,
}

#[derive(Debug)]
pub struct Road {
    pub class: RoadClass,
    pub nodes: Vec<(i64, LatLon)>,
    pub bridge: bool,
}

#[derive(Debug)]
pub struct Place {
    pub name: String,
    pub kind: PlaceKind,
    pub position: LatLon,
}

#[derive(Debug)]
pub struct Site {
    pub name: String,
    pub kind: SiteKind,
    pub position: LatLon,
    /// area in square degrees (0 for nodes), to rank sites by size
    pub size: f64,
}

#[derive(Default, Debug)]
pub struct Extract {
    pub areas: Vec<Area>,
    pub roads: Vec<Road>,
    pub places: Vec<Place>,
    pub sites: Vec<Site>,
    pub rivers: Vec<Vec<LatLon>>,
}

fn cover_of(tags: &HashMap<&str, &str>) -> Option<Cover> {
    let get = |k: &str| tags.get(k).copied();
    match (get("landuse"), get("natural"), get("waterway")) {
        (_, Some("water"), _) | (Some("reservoir" | "basin"), _, _) | (_, _, Some("riverbank")) => Some(Cover::Water),
        (Some("forest"), _, _) | (_, Some("wood"), _) => Some(Cover::Forest),
        (Some("residential" | "commercial" | "retail" | "construction"), _, _) => Some(Cover::Urban),
        (Some("industrial" | "quarry" | "railway"), _, _) => Some(Cover::Industrial),
        (Some("military"), _, _) => Some(Cover::Military),
        (Some("farmland" | "meadow" | "grass" | "orchard" | "vineyard" | "farmyard" | "allotments"), _, _) | (_, Some("grassland" | "heath" | "scrub"), _) => {
            Some(Cover::Field)
        }
        _ => None,
    }
}

fn road_of(tags: &HashMap<&str, &str>) -> Option<RoadClass> {
    match tags.get("highway").copied()? {
        "motorway" | "motorway_link" => Some(RoadClass::Motorway),
        "trunk" | "trunk_link" => Some(RoadClass::Trunk),
        "primary" | "primary_link" => Some(RoadClass::Primary),
        "secondary" | "secondary_link" => Some(RoadClass::Secondary),
        "tertiary" => Some(RoadClass::Tertiary),
        _ => None,
    }
}

fn site_of(tags: &HashMap<&str, &str>) -> Option<SiteKind> {
    let get = |k: &str| tags.get(k).copied();
    if get("aeroway") == Some("aerodrome") {
        return Some(SiteKind::Aerodrome);
    }
    match get("power") {
        Some("plant") => return Some(SiteKind::PowerPlant),
        Some("substation") => return Some(SiteKind::Substation),
        _ => {}
    }
    if matches!(get("man_made"), Some("mast" | "tower")) && matches!(get("tower:type"), Some("communication")) {
        return Some(SiteKind::Mast);
    }
    if get("amenity") == Some("fuel") || get("industrial") == Some("oil") || get("man_made") == Some("storage_tank") {
        return Some(SiteKind::Fuel);
    }
    match get("landuse") {
        Some("industrial") => Some(SiteKind::Industrial),
        Some("military") => Some(SiteKind::Military),
        _ => None,
    }
}

enum WayUse {
    Area(Cover),
    Road(RoadClass, bool),
    River,
    Site(SiteKind, String),
    RelationMember,
}

pub fn read(path: &Path) -> Result<Extract> {
    // pass 1: multipolygon relations of interest -> their outer ways
    let mut relation_ways: HashMap<i64, Cover> = HashMap::new();
    ElementReader::from_path(path)?.for_each(|e| {
        if let Element::Relation(r) = e {
            let tags: HashMap<&str, &str> = r.tags().collect();
            if tags.get("type").copied() != Some("multipolygon") {
                return;
            }
            if let Some(cover) = cover_of(&tags) {
                for m in r.members() {
                    if m.member_type == RelMemberType::Way && m.role().map(|role| role != "inner").unwrap_or(true) {
                        relation_ways.insert(m.member_id, cover);
                    }
                }
            }
        }
    })?;

    // pass 2: ways
    let mut ways: Vec<(WayUse, Vec<i64>)> = Vec::new();
    ElementReader::from_path(path)?.for_each(|e| {
        let (id, tags, refs): (i64, HashMap<&str, &str>, Vec<i64>) = match &e {
            Element::Way(w) => (w.id(), w.tags().collect(), w.refs().collect()),
            _ => return,
        };
        let closed = refs.len() > 3 && refs.first() == refs.last();
        if let Some(class) = road_of(&tags) {
            ways.push((WayUse::Road(class, tags.contains_key("bridge")), refs));
            return;
        }
        if tags.get("waterway").copied() == Some("river") {
            ways.push((WayUse::River, refs));
            return;
        }
        if closed {
            if let Some(kind) = site_of(&tags) {
                let name = display_name(&tags);
                ways.push((WayUse::Site(kind, name), refs.clone()));
            }
            if let Some(cover) = cover_of(&tags) {
                ways.push((WayUse::Area(cover), refs));
                return;
            }
        }
        if relation_ways.contains_key(&id) {
            ways.push((WayUse::RelationMember, refs));
            // remember which cover it carries
            ways.last_mut().map(|w| w.0 = WayUse::Area(relation_ways[&id]));
        }
    })?;

    // pass 3: node coordinates, and tagged nodes
    let needed: HashSet<i64> = ways.iter().flat_map(|(_, refs)| refs.iter().copied()).collect();
    let mut coords: HashMap<i64, LatLon> = HashMap::with_capacity(needed.len());
    let mut out = Extract::default();
    ElementReader::from_path(path)?.for_each(|e| {
        let (id, lat, lon, tags): (i64, f64, f64, HashMap<&str, &str>) = match &e {
            Element::Node(n) => (n.id(), n.lat(), n.lon(), n.tags().collect()),
            Element::DenseNode(n) => (n.id(), n.lat(), n.lon(), n.tags().collect()),
            _ => return,
        };
        if needed.contains(&id) {
            coords.insert(id, (lat, lon));
        }
        if tags.is_empty() {
            return;
        }
        let name = display_name(&tags);
        if let Some(kind) = match tags.get("place").copied() {
            Some("city") => Some(PlaceKind::City),
            Some("town") => Some(PlaceKind::Town),
            Some("village") => Some(PlaceKind::Village),
            _ => None,
        } {
            if !name.is_empty() {
                out.places.push(Place {
                    name: name.clone(),
                    kind,
                    position: (lat, lon),
                });
            }
        }
        if let Some(kind) = site_of(&tags) {
            out.sites.push(Site {
                name,
                kind,
                position: (lat, lon),
                size: 0.0,
            });
        }
    })?;

    let resolve = |refs: &[i64]| -> Vec<(i64, LatLon)> { refs.iter().filter_map(|r| coords.get(r).map(|c| (*r, *c))).collect() };
    for (usage, refs) in ways {
        let nodes = resolve(&refs);
        if nodes.len() < 2 {
            continue;
        }
        match usage {
            WayUse::Area(cover) => out.areas.push(Area {
                cover,
                ring: nodes.into_iter().map(|n| n.1).collect(),
            }),
            WayUse::Road(class, bridge) => out.roads.push(Road { class, nodes, bridge }),
            WayUse::River => out.rivers.push(nodes.into_iter().map(|n| n.1).collect()),
            WayUse::Site(kind, name) => {
                let ring: Vec<LatLon> = nodes.iter().map(|n| n.1).collect();
                let (lat, lon) = centroid(&ring);
                out.sites.push(Site {
                    name,
                    kind,
                    position: (lat, lon),
                    size: ring_area(&ring).abs(),
                });
            }
            WayUse::RelationMember => {}
        }
    }
    Ok(out)
}

pub fn ring_area(ring: &[LatLon]) -> f64 {
    let mut a = 0.0;
    for w in ring.windows(2) {
        a += w[0].1 * w[1].0 - w[1].1 * w[0].0;
    }
    a / 2.0
}

pub fn centroid(ring: &[LatLon]) -> LatLon {
    let n = ring.len().max(1) as f64;
    let (lat, lon) = ring.iter().fold((0.0, 0.0), |(a, b), p| (a + p.0, b + p.1));
    (lat / n, lon / n)
}

/// the name EECH shows: the local name when it is in Latin script, else the
/// English or international one (EECH's names are ASCII; other scripts would
/// come out as underscores)
fn display_name(tags: &HashMap<&str, &str>) -> String {
    let local = tags.get("name").copied().unwrap_or_default();
    if local.chars().all(|c| (c as u32) < 0x250) {
        return local.to_string();
    }
    tags.get("name:en").or_else(|| tags.get("int_name")).copied().unwrap_or(local).to_string()
}
