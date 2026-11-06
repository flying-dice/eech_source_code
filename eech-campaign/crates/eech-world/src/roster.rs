//! The host's physical sector roster (EEDCS-49, D3, sixth slice): which entities stand in which sector of the world, and in what order a scan visits them.
//!
//! # What this is
//!
//! `get_line_of_sight_collision_entity` visits the entities of each sector from the TAIL of the sector's list backwards: the sector's oldest insertion first, because every insertion is at the head
//! (`insert_local_entity_into_parents_child_list` always passes a NULL predecessor for a sector). A farther entity in an earlier list position wins over a nearer one, so the order is the result.
//! This is that membership and order, held by the HOST: the campaign reports each insertion into and removal from a sector list (`en_list.c` is the one funnel every creator, relink, unpack and
//! destroyer goes through) and the host answers with a key it issued, decides which sector the entity stands in from its OWN state, and then serves the scan from its own order. The campaign's lists stay
//! behind as a conformance mirror that stops the scan by name when it disagrees; they are never the answer and never a fallback.
//!
//! # One record, no second pose
//!
//! A hosted mobile's position is the one record the movement registry (`Movers`) holds: its sector is computed from that record, and the position the campaign states is only compared with it. Every other
//! entity (a structure, a city, a keysite, a weapon, an effect) has no entry in `Movers`; its sector is computed from the position the campaign states at the insertion, and the placement keeps only the
//! sector and the sequence.
//!
//! A FIXED-family placement (an object, a city building, a crater, a pylon, a piece of scenery, a bridge segment, a site and its updatable; a city is not one) also has its COLLISION state held by its
//! RECORD, and nowhere else in the host: the position, heading, shape and the two flags its type derives (`FixedState`), as the campaign states them at every write it makes to one (the insertion, a
//! setter, a kill's destroyed shape, a restore, a session unpack, a bridge's damage or fall). The record is the one the placements of the entity share, so the state lives with the entity's placements,
//! follows the link of a building's two placements, and ends with the record's last placement; a relink is a removal and a new insertion, so the new record holds none until the campaign states it again.
//! A candidate question names the placement the host served and is decided from that record's words; the campaign's own words for the same facts are only compared with them (`model_host`).
//!
//! A CITY's record holds the city's OWN collision state instead (`CityState`): its position and bounding box, as the campaign states them at the insertion (which a relink repeats), at a position written in the same
//! sector and at each expansion of the box that a building's link makes. The record is the one the city's list hangs from, so the state outlives the city's transient sector placement while buildings are listed and ends
//! with the record. The terrain the structures stand on is NOT held here (declared follow-on).
//!
//! A key is issued at an insertion and never again: a relink is a removal and a new insertion with a new sequence number, exactly the list operation it mirrors, and a removed key can never name
//! another entity.
//!
//! # City lists, records and object identities
//!
//! A city's buildings (`LIST_TYPE_CITY_BUILDING`) are listed under the city's sector placement, head first: the campaign inserts a building at the head or after a predecessor, and a scan walks the list from
//! the head. The host holds that order (`child_insert`, `child_first`, `child_next`) as it holds a sector's, and the campaign's own list is only the mirror the scan's answer is compared with. A building has two
//! placements, one in its city's list and one in its sector's: the campaign declares them the SAME entity (`link`) as soon as the second exists, so the host holds one record for the entity; and when the campaign
//! asks the host for the entity's legacy object-report identity (lazily, once) the record is named by it (`name`). An identity names one record, a record has one identity, and both end with the record's last
//! placement. Every key is validated against the live roster and its parent before anything is served or changed.
//!
//! A city list hangs from the city's RECORD, not from the city's current sector placement. The city's sector placement is transient: the fixed-position setter every city shares removes it from its sector list and
//! inserts it again whenever the position crosses a sector, and a session restore does the same, while the city's buildings stay listed. So a city's sector placement may leave while buildings are listed: the record
//! (and the object identity that names it) stays while the list is non-empty, the list is not served or extended until a placement joins the record again (`link` with the record the list hangs from), and the record
//! ends with its last placement once the list is empty. The list is named on the wire by the key the campaign first named it by (the city's sector placement key at the first building), which stays the list's name
//! whatever placements the city has had since.

use std::collections::{BTreeMap, HashMap};

/// The host's whole answer to a refused roster question: a named reason, never a default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal(pub String);

impl Refusal {
    fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

/// The bits of the largest magnitude a coordinate, cursor or whole-number word may state (1.0e9): the bits of a non-negative single order like its value.
const WORD_LIMIT: u32 = 0x4e6e_6b28;
/// The bits of the smallest magnitude that is not a finite number (infinity; every NaN, quiet or signaling, is above it).
const WORD_NONFINITE: u32 = 0x7f80_0000;

/// A single-precision word read as a number by INTEGER operations only: its value truncated toward zero, and whether it had no fractional part. No floating-point instruction touches the word, so the caller's rounding
/// mode, exception masks and raised flags are never changed and a signaling NaN is classified without being compared or converted. A word that is not finite, or whose magnitude exceeds 1.0e9, is refused by name and is
/// never coerced. The truncation is exactly the original's `convert_float_to_int` (under round-toward-zero) for every value of the domain.
pub fn word_truncated(word: u32) -> Result<(i32, bool), Refusal> {
    let magnitude = word & 0x7fff_ffff;
    if magnitude >= WORD_NONFINITE {
        return Err(Refusal::new("the word is not a finite number"));
    }
    if magnitude > WORD_LIMIT {
        return Err(Refusal::new("the word is outside the world"));
    }
    let exponent = (magnitude >> 23) as i32 - 127;
    // below one (zero and the subnormals included): truncates to zero, and is whole only when it is zero
    if exponent < 0 {
        return Ok((0, magnitude == 0));
    }
    let significand = (magnitude & 0x007f_ffff) | 0x0080_0000;
    // the limit is below 2^30, so the exponent is at most 29 and the value fits
    let (value, whole) = if exponent >= 23 {
        (significand << (exponent - 23), true)
    } else {
        let dropped = 23 - exponent;
        (significand >> dropped, significand & ((1 << dropped) - 1) == 0)
    };
    let value = value as i32;
    Ok((if word >> 31 == 1 { -value } else { value }, whole))
}

/// A word that states a whole number within `low..=high`, by integer operations only (see [`word_truncated`]); none for a fractional, non-finite or out-of-range word.
pub fn word_whole(word: u32, low: i32, high: i32) -> Option<i32> {
    match word_truncated(word) {
        Ok((value, true)) if (low..=high).contains(&value) => Some(value),
        _ => None,
    }
}

/// The key a roster event or a standing scan names: one to twenty decimal digits without a leading zero (the campaign's `roster_key_is_valid`), and then a number the host could have issued.
pub fn parse_key(text: &str) -> Option<u64> {
    if text.is_empty() || text.len() > 20 || text.starts_with('0') || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// The sector index pair of a position: the original's `get_x_sector` and `get_z_sector` (a truncation toward zero, then the integer division by the side length), by integer operations only.
pub fn sector_of(x_word: u32, z_word: u32, side: i32) -> Result<(i32, i32), Refusal> {
    if side < 1 {
        return Err(Refusal::new(format!("the sector side length {side} is not positive")));
    }
    let axis = |word: u32| -> Result<i32, Refusal> {
        word_truncated(word)
            .map(|(value, _)| value / side)
            .map_err(|_| Refusal::new("a position outside the world has no sector"))
    };
    Ok((axis(x_word)?, axis(z_word)?))
}

/// The campaign's entity types (`en_types.h`) the host classifies a placement by when it decides what the placement's record must hold: a city (it holds its own position and box, `CityState`),
/// a building listed in a city, and the fixed family whose collision state the record holds (`place_fixed_family`, mb_hostplace.c: a building listed in a city, a crater, an object, a pylon, scenery, a bridge
/// segment, a site and an updatable site). The values are compared with the campaign's own source by this module's tests.
pub const ENTITY_CITY: i32 = 5;
pub const ENTITY_CITY_BUILDING: i32 = 6;
pub const FIXED_FAMILY: [i32; 8] = [6, 7, 17, 21, 24, 26, 29, 30];

/// Whether an entity type is of the fixed family: the placements whose record holds a fixed collision state, and whose decision needs it once the campaign has read their fixed facts.
pub fn is_fixed_family(entity_type: i32) -> bool {
    FIXED_FAMILY.contains(&entity_type)
}

/// The longest legacy object-report identity, in bytes (`CAMPAIGN_OBJECT_ID_MAX`, mb_hostobj.h): its own domain, independent of the placement key a NAME carries before it.
pub const OBJECT_ID_MAX: usize = 128;

struct Entry {
    sector: (i32, i32),
    sequence: u64,
    entity_type: i32,
    /// the semantic identity of a hosted mobile; none for every other entity (the explicit mapping from the scan key to the movement record)
    host_id: Option<String>,
    /// the record this placement belongs to: the entity's one physical record, which its other placements (a city building's place in its city's list) and its legacy object-report identity share
    record: u64,
}

/// A building's place in its city's list (`LIST_TYPE_CITY_BUILDING`): the record of the city the list hangs from (stable across the city's sector relinks), and the record the building shares with its own sector placement.
struct Child {
    parent: u64,
    entity_type: i32,
    record: u64,
}

/// The collision state of a fixed-family placement, as the campaign stated it (mb_hostmodel.h, FIXED_STATE): the words a narrow-phase decision about the placement is made from. Every float is the bit
/// pattern the campaign held and is never loaded as a float here; the shape is the scene index the placement has NOW (a kill, a restore and a bridge's damage each change it, and a destroyed structure
/// still collides with its destroyed shape), and the two flags are the ones the entity's type derives (collision-test-fixed, align-with-terrain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedState {
    pub position: [u32; 3],
    pub heading: u32,
    pub shape: i32,
    pub collision_fixed: bool,
    pub align: bool,
}

/// The collision state of a city placement, as the campaign stated it (mb_hostmodel.h, CITY_STATE): the words a candidate question about the city is decided from once the campaign has read them. Every float is the
/// bit pattern the entity held and is never loaded as a float here. The box of a city that has no building yet is the original's INVERTED initial box (minimum 1.0e6, maximum -1.0e6), a valid statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CityState {
    pub position: [u32; 3],
    pub box_min: [u32; 3],
    pub box_max: [u32; 3],
}

/// What the host holds of a record's collision state: nothing was ever stated (a record of a type that has none, or a campaign that did not state it), the fixed family's state, a city's own state, or the host's own
/// withdrawal of the state it held (the placement stays standing; the next statement of the campaign restores the state).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedHold {
    Never,
    Held(FixedState),
    City(CityState),
    Withdrawn,
}

/// The entity's one record: the placements that share it (at most one in a sector list and one in a city's list), the legacy object-report identity the campaign has asked the host to issue for it
/// (lazily, once), if any, and the fixed collision state it holds. The state lives and ends with the record: a relink is a removal and a new insertion, so the new placement's record holds none until the
/// campaign states it again, and the two placements of a building share the one.
struct Record {
    sector: Option<u64>,
    child: Option<u64>,
    object: Option<String>,
    fixed: FixedHold,
}

/// The host's roster of the entities in the sector lists.
#[derive(Default)]
pub struct SectorRoster {
    /// the next key and the next insertion sequence; both only grow, so neither is ever reused
    next_key: u64,
    next_sequence: u64,
    entries: HashMap<u64, Entry>,
    sectors: HashMap<(i32, i32), BTreeMap<u64, u64>>,
    /// the city lists: for the record of the city, its children's keys in the list's order (the head first: a child is inserted at the head or after a predecessor, and a scan walks head to tail)
    lists: HashMap<u64, Vec<u64>>,
    children: HashMap<u64, Child>,
    records: HashMap<u64, Record>,
    /// the object-report identities by record: an identity names one record
    objects: HashMap<String, u64>,
}

impl SectorRoster {
    /// Inserts an entity at the head of a sector's list: it is visited AFTER every entity that was there before. Returns the key the host issued.
    pub fn insert(&mut self, sector: (i32, i32), entity_type: i32, host_id: Option<&str>) -> u64 {
        self.next_key += 1;
        self.next_sequence += 1;
        let (key, sequence) = (self.next_key, self.next_sequence);
        self.sectors.entry(sector).or_default().insert(sequence, key);
        self.entries.insert(
            key,
            Entry {
                sector,
                sequence,
                entity_type,
                host_id: host_id.map(str::to_string),
                record: key,
            },
        );
        self.records.insert(
            key,
            Record {
                sector: Some(key),
                child: None,
                object: None,
                fixed: FixedHold::Never,
            },
        );
        key
    }

    /// Removes an entity from its sector's list. The others keep their order.
    pub fn remove(&mut self, key: u64) -> Result<(), Refusal> {
        if !self.entries.contains_key(&key) {
            return Err(Refusal::new(self.unknown(key)));
        }
        let Some(entry) = self.entries.remove(&key) else {
            return Err(Refusal::new(self.unknown(key)));
        };
        if let Some(sector) = self.sectors.get_mut(&entry.sector) {
            sector.remove(&entry.sequence);
            if sector.is_empty() {
                self.sectors.remove(&entry.sector);
            }
        }
        self.leave(entry.record, key);
        Ok(())
    }

    /// One placement leaves its record.
    fn leave(&mut self, record: u64, key: u64) {
        if let Some(entry) = self.records.get_mut(&record) {
            if entry.sector == Some(key) {
                entry.sector = None;
            }
            if entry.child == Some(key) {
                entry.child = None;
            }
        }
        self.retire(record);
    }

    /// A record, and the object identity it was named by, end with its last placement, unless buildings are still listed under it: a city whose sector placement has left (a relink) keeps its record and its
    /// list until a placement joins it again, and ends when its list is empty.
    fn retire(&mut self, record: u64) {
        let placed = self.records.get(&record).is_some_and(|entry| entry.sector.is_some() || entry.child.is_some());
        if placed || self.lists.get(&record).is_some_and(|list| !list.is_empty()) {
            return;
        }
        self.lists.remove(&record);
        if let Some(object) = self.records.remove(&record).and_then(|ended| ended.object) {
            self.objects.remove(&object);
        }
    }

    /// The record whose city list a key of the identity place names: a live sector entry names the record it belongs to, and a key that is no live entry names the record it was issued for while that record is
    /// still held by a placement or a list (the key a city's list was first named by, after the city's placement was relinked).
    fn list_record(&self, key: u64) -> Result<u64, Refusal> {
        if let Some(entry) = self.entries.get(&key) {
            return Ok(entry.record);
        }
        let held = self
            .records
            .get(&key)
            .is_some_and(|record| record.sector.is_some() || self.lists.get(&key).is_some_and(|list| !list.is_empty()));
        if held {
            Ok(key)
        } else {
            Err(Refusal::new(self.unknown(key)))
        }
    }

    /// Inserts a building into its city's list: at the head, or after `pred` (a live child of the same list). `named` is the key the list is named by (see `list_record`); the city must hold a sector placement.
    pub fn child_insert(&mut self, named: u64, pred: Option<u64>, entity_type: i32) -> Result<u64, Refusal> {
        let parent = self.list_record(named).map_err(|why| Refusal::new(format!("[parent] {}", why.0)))?;
        if self.records.get(&parent).is_some_and(|record| record.sector.is_none()) {
            return Err(Refusal::new(format!("[parent] the city of the list {named} holds no sector placement")));
        }
        let at = match pred {
            None => 0,
            Some(pred) => {
                let child = self
                    .children
                    .get(&pred)
                    .ok_or_else(|| Refusal::new(format!("[pred] {}", self.not_a_child(pred))))?;
                if child.parent != parent {
                    return Err(Refusal::new(format!(
                        "[pred] the city child {pred} is listed under {}, not under the parent {named}",
                        child.parent
                    )));
                }
                self.lists
                    .get(&parent)
                    .and_then(|list| list.iter().position(|key| *key == pred))
                    .map_or(0, |index| index + 1)
            }
        };
        self.next_key += 1;
        let key = self.next_key;
        self.lists.entry(parent).or_default().insert(at, key);
        self.children.insert(
            key,
            Child {
                parent,
                entity_type,
                record: key,
            },
        );
        self.records.insert(
            key,
            Record {
                sector: None,
                child: Some(key),
                object: None,
                fixed: FixedHold::Never,
            },
        );
        Ok(key)
    }

    /// Removes a building from its city's list; the others keep their order. `named` is the list the campaign names: a child listed under another is refused by name. The list may be one whose city holds no
    /// sector placement at the moment (a building is unlisted whenever the campaign releases it).
    pub fn child_remove(&mut self, named: u64, key: u64) -> Result<(), Refusal> {
        let parent = self.child_of(named, key)?;
        let record = self.children.get(&key).map_or(key, |child| child.record);
        if let Some(list) = self.lists.get_mut(&parent) {
            list.retain(|child| *child != key);
        }
        self.children.remove(&key);
        self.leave(record, key);
        self.retire(parent);
        Ok(())
    }

    /// The record of the list `named` names, when `key` is listed under it: a child that is not, or is listed under another, is refused by name.
    fn child_of(&self, named: u64, key: u64) -> Result<u64, Refusal> {
        let child = self.children.get(&key).ok_or_else(|| Refusal::new(self.not_a_child(key)))?;
        let parent = self.list_record(named)?;
        if child.parent != parent {
            return Err(Refusal::new(format!(
                "the city child {key} is listed under {}, not under the parent {named} the campaign names",
                child.parent
            )));
        }
        Ok(parent)
    }

    /// The record of the list `named` names, when its city holds a sector placement: a list is served only while it does.
    fn served_list(&self, named: u64) -> Result<u64, Refusal> {
        let parent = self.list_record(named)?;
        if self.records.get(&parent).is_some_and(|record| record.sector.is_none()) {
            return Err(Refusal::new(format!("the city of the list {named} holds no sector placement")));
        }
        Ok(parent)
    }

    /// The first building a scan of the city's list visits (the head), when the city holds a sector placement.
    pub fn child_first(&self, named: u64) -> Result<Option<u64>, Refusal> {
        let parent = self.served_list(named)?;
        Ok(self.lists.get(&parent).and_then(|list| list.first().copied()))
    }

    /// The building a scan visits after `key` in the list `named` names: refused by name when `key` is not a live child of it.
    pub fn child_next(&self, named: u64, key: u64) -> Result<Option<u64>, Refusal> {
        let parent = self.served_list(named)?;
        self.child_of(named, key)?;
        Ok(self
            .lists
            .get(&parent)
            .and_then(|list| list.iter().position(|child| *child == key).and_then(|index| list.get(index + 1)))
            .copied())
    }

    /// The record a placement (a sector entry or a city child) belongs to.
    pub fn record(&self, key: u64) -> Option<u64> {
        self.entries
            .get(&key)
            .map(|entry| entry.record)
            .or_else(|| self.children.get(&key).map(|child| child.record))
    }

    fn set_record(&mut self, key: u64, record: u64) {
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.record = record;
        } else if let Some(child) = self.children.get_mut(&key) {
            child.record = record;
        }
    }

    /// Declares a sector placement and a city child to be the SAME entity: they share one record, which keeps the object identity it has. A record holds at most one placement of each kind, so a sector
    /// placement joins a record only if that record has no sector placement yet, and a child only if it has no child yet: a relink is a new sector placement joining the child's record, and a creation a
    /// new child joining the new sector placement's.
    ///
    /// The second key may instead name the record a city's list hangs from, when that record's sector placement has left (the city was relinked): the new sector placement then continues the record, which keeps
    /// its list, its object identity and its key, and the placement's own fresh record ends (`rejoin`).
    pub fn link(&mut self, sector: u64, child: u64) -> Result<(), Refusal> {
        if self.entries.contains_key(&sector) && !self.children.contains_key(&child) && self.records.contains_key(&child) {
            return self.rejoin(sector, child);
        }
        let (Some(a), Some(b)) = (
            self.entries.get(&sector).map(|entry| entry.record),
            self.children.get(&child).map(|child| child.record),
        ) else {
            let missing = if self.entries.contains_key(&sector) { child } else { sector };
            return Err(Refusal::new(format!("[link] {}", self.unknown(missing))));
        };
        if a == b {
            return Ok(());
        }
        let (Some(held), Some(joining)) = (self.records.get(&a), self.records.get(&b)) else {
            return Err(Refusal::new(format!("[link] the placements {sector} and {child} have no record")));
        };
        if held.child.is_some() || joining.sector.is_some() {
            return Err(Refusal::new(format!(
                "[link] the placements {sector} and {child} belong to records that already hold a {}",
                if held.child.is_some() { "city child" } else { "sector placement" }
            )));
        }
        let (kept, moved) = (held.object.clone(), joining.object.clone());
        if kept.is_some() && moved.is_some() && kept != moved {
            return Err(Refusal::new("[link] the two placements are named by different object identities".to_string()));
        }
        let fixed = merge_fixed(held.fixed, joining.fixed)?;
        self.records.remove(&b);
        if let Some(object) = &moved {
            self.objects.insert(object.clone(), a);
        }
        if let Some(record) = self.records.get_mut(&a) {
            record.child = Some(child);
            record.fixed = fixed;
            if record.object.is_none() {
                record.object = moved;
            }
        }
        self.set_record(child, a);
        Ok(())
    }

    /// A city's new sector placement continues the record its list hangs from. The record must have lost its sector placement while it kept a list, and the new placement must stand alone in its fresh record.
    fn rejoin(&mut self, sector: u64, record: u64) -> Result<(), Refusal> {
        let fresh = self.record(sector).ok_or_else(|| Refusal::new(format!("[link] {}", self.unknown(sector))))?;
        let (Some(held), Some(joining)) = (self.records.get(&record), self.records.get(&fresh)) else {
            return Err(Refusal::new(format!("[link] the placement {sector} and the record {record} have no record")));
        };
        if held.sector.is_some() || !self.lists.get(&record).is_some_and(|list| !list.is_empty()) {
            return Err(Refusal::new(format!(
                "[link] the record {record} is not a city list whose sector placement has left"
            )));
        }
        if joining.child.is_some() || self.lists.get(&fresh).is_some_and(|list| !list.is_empty()) {
            return Err(Refusal::new(format!("[link] the placement {sector} already holds a city child or a list")));
        }
        let (kept, moved) = (held.object.clone(), joining.object.clone());
        if kept.is_some() && moved.is_some() && kept != moved {
            return Err(Refusal::new("[link] the two placements are named by different object identities".to_string()));
        }
        let fixed = merge_fixed(held.fixed, joining.fixed)?;
        self.records.remove(&fresh);
        if let Some(object) = &moved {
            self.objects.insert(object.clone(), record);
        }
        if let Some(held) = self.records.get_mut(&record) {
            held.sector = Some(sector);
            held.fixed = fixed;
            if held.object.is_none() {
                held.object = moved;
            }
        }
        self.set_record(sector, record);
        Ok(())
    }

    /// The placement a key names, as a sector entry (`child` false) or a city child (`child` true): its record's id, entity type and fixed collision state. A key of the other kind, an ended one and one never
    /// issued are refused by name: the host serves a candidate by a placement of the kind its branch has, never by another.
    pub fn placement_fixed(&self, key: u64, child: bool) -> Result<(u64, i32, FixedHold), Refusal> {
        let (record, entity_type) = if child {
            let entry = self.children.get(&key).ok_or_else(|| Refusal::new(self.not_a_child(key)))?;
            (entry.record, entry.entity_type)
        } else {
            let entry = self.entries.get(&key).ok_or_else(|| Refusal::new(self.unknown(key)))?;
            (entry.record, entry.entity_type)
        };
        let held = self
            .records
            .get(&record)
            .map(|record| record.fixed)
            .ok_or_else(|| Refusal::new(format!("the placement {key} has no record")))?;
        Ok((record, entity_type, held))
    }

    /// States the fixed collision state of the record a placement belongs to (a sector entry or a city child; the campaign states it under whichever placement exists). The type the campaign states must be the
    /// placement's own and of the fixed family (a city, a mobile and a weapon have none: refused before the record changes), and a statement replaces what the record held (it also restores a state the host withdrew).
    pub fn set_fixed(&mut self, key: u64, entity_type: i32, state: FixedState) -> Result<(), Refusal> {
        let record = self.record(key).ok_or_else(|| Refusal::new(self.unknown(key)))?;
        let placed = self
            .entity_type(key)
            .or_else(|| self.child_type(key))
            .ok_or_else(|| Refusal::new(self.unknown(key)))?;
        if placed != entity_type {
            return Err(Refusal::new(format!(
                "the report states the entity type {entity_type} but the placement {key} is of type {placed}"
            )));
        }
        if !is_fixed_family(placed) {
            return Err(Refusal::new(format!(
                "the placement {key} is of entity type {placed}, which is not of the fixed family: the host holds no fixed collision state for it"
            )));
        }
        let Some(entry) = self.records.get_mut(&record) else {
            return Err(Refusal::new(format!("the placement {key} has no record")));
        };
        entry.fixed = FixedHold::Held(state);
        Ok(())
    }

    /// States the city's own collision state of the record a placement belongs to (the campaign states it under the city's sector placement). The placement must be a city (a fixed-family placement, a mobile and a
    /// building listed under a city have none: refused before the record changes), and a statement replaces what the record held (it also restores a state the host withdrew).
    pub fn set_city(&mut self, key: u64, state: CityState) -> Result<(), Refusal> {
        let record = self.record(key).ok_or_else(|| Refusal::new(self.unknown(key)))?;
        let placed = self
            .entity_type(key)
            .or_else(|| self.child_type(key))
            .ok_or_else(|| Refusal::new(self.unknown(key)))?;
        if placed != ENTITY_CITY {
            return Err(Refusal::new(format!(
                "the placement {key} is of entity type {placed}, which is not a city: the host holds no city collision state for it"
            )));
        }
        let Some(entry) = self.records.get_mut(&record) else {
            return Err(Refusal::new(format!("the placement {key} has no record")));
        };
        entry.fixed = FixedHold::City(state);
        Ok(())
    }

    /// The HOST's own retirement of the fixed collision state of the record a placement belongs to: the placement stays standing and the record keeps its placements, its object identity and its list; only the
    /// state goes. The campaign never asks for it. A record that holds no state has nothing to withdraw.
    pub fn withdraw_fixed(&mut self, key: u64) -> Result<(), Refusal> {
        let record = self.record(key).ok_or_else(|| Refusal::new(self.unknown(key)))?;
        let Some(entry) = self.records.get_mut(&record) else {
            return Err(Refusal::new(format!("the placement {key} has no record")));
        };
        if !matches!(entry.fixed, FixedHold::Held(_) | FixedHold::City(_)) {
            return Err(Refusal::new(format!(
                "the record of the placement {key} holds no fixed physical state to withdraw"
            )));
        }
        entry.fixed = FixedHold::Withdrawn;
        Ok(())
    }

    /// Names the record of a placement by the legacy object-report identity the host issued for it (the campaign asks for one lazily, once per entity, and reports it here): an identity names ONE record,
    /// and a record has one identity.
    pub fn name(&mut self, key: u64, object: &str) -> Result<(), Refusal> {
        let record = self.record(key).ok_or_else(|| Refusal::new(format!("[name] {}", self.unknown(key))))?;
        if let Some(existing) = self.objects.get(object) {
            return Err(Refusal::new(format!("[name] the object identity names the record {existing} already")));
        }
        let Some(entry) = self.records.get_mut(&record) else {
            return Err(Refusal::new(format!("[name] the placement {key} has no record")));
        };
        if entry.object.is_some() {
            return Err(Refusal::new(format!("[name] the record {record} of the placement {key} is named already")));
        }
        entry.object = Some(object.to_string());
        self.objects.insert(object.to_string(), record);
        Ok(())
    }

    /// The legacy object-report identity of the record a placement belongs to.
    pub fn object_of(&self, key: u64) -> Option<&str> {
        self.record(key)
            .and_then(|record| self.records.get(&record))
            .and_then(|record| record.object.as_deref())
    }

    /// The record an object-report identity names.
    pub fn object_record(&self, object: &str) -> Option<u64> {
        self.objects.get(object).copied()
    }

    /// How many placements share a record.
    pub fn record_members(&self, record: u64) -> u32 {
        self.records
            .get(&record)
            .map_or(0, |record| u32::from(record.sector.is_some()) + u32::from(record.child.is_some()))
    }

    /// The type of the building a city child key names.
    pub fn child_type(&self, key: u64) -> Option<i32> {
        self.children.get(&key).map(|child| child.entity_type)
    }

    /// The first entity a scan of the sector visits: its oldest insertion.
    pub fn first(&self, sector: (i32, i32)) -> Option<u64> {
        self.sectors.get(&sector).and_then(|order| order.values().next().copied())
    }

    /// Whether `key` names a live entry, refused by name when it was never issued or has ended.
    pub fn live(&self, key: u64) -> Result<(), Refusal> {
        self.entries.get(&key).map(|_| ()).ok_or_else(|| Refusal::new(self.unknown(key)))
    }

    /// The live entry `key` names, when it stands in `sector`, the sector a scan is in: an entry that is not in it is refused by name.
    fn entry_in(&self, sector: (i32, i32), key: u64) -> Result<&Entry, Refusal> {
        let entry = self.entries.get(&key).ok_or_else(|| Refusal::new(self.unknown(key)))?;
        if entry.sector != sector {
            return Err(Refusal::new(format!(
                "the roster entry {key} stands in sector {:?}, not in the sector {sector:?} the scan is in",
                entry.sector
            )));
        }
        Ok(entry)
    }

    /// Whether `key` names a live entry that stands in `sector`, the sector a scan is in: any other is refused by name.
    pub fn standing(&self, sector: (i32, i32), key: u64) -> Result<(), Refusal> {
        self.entry_in(sector, key).map(|_| ())
    }

    /// The entity a scan visits after `key` in the sector it is standing in: the next insertion. `sector` is the sector the scan is in, and an entity that is not in it is refused by name.
    pub fn next(&self, sector: (i32, i32), key: u64) -> Result<Option<u64>, Refusal> {
        let entry = self.entry_in(sector, key)?;
        Ok(self
            .sectors
            .get(&sector)
            .and_then(|order| order.range(entry.sequence + 1..).next().map(|(_, key)| *key)))
    }

    pub fn host_id(&self, key: u64) -> Option<&str> {
        self.entries.get(&key).and_then(|entry| entry.host_id.as_deref())
    }

    pub fn entity_type(&self, key: u64) -> Option<i32> {
        self.entries.get(&key).map(|entry| entry.entity_type)
    }

    /// The sector of an entry, if it is in the roster.
    pub fn sector(&self, key: u64) -> Option<(i32, i32)> {
        self.entries.get(&key).map(|entry| entry.sector)
    }

    /// How many entities the roster holds.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Forgets every entry (a new campaign boot). Keys and sequence numbers keep growing: a key of an earlier boot can never name an entity of this one.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.sectors.clear();
        self.lists.clear();
        self.children.clear();
        self.records.clear();
        self.objects.clear();
    }

    /// The reason `key` is not a city child: it names a sector entry, or it is unknown or ended.
    fn not_a_child(&self, key: u64) -> String {
        if self.entries.contains_key(&key) {
            format!("the roster entry {key} is a sector entry, not a city child")
        } else {
            self.unknown(key)
        }
    }

    fn unknown(&self, key: u64) -> String {
        if self.children.contains_key(&key) {
            return format!("the roster entry {key} is a city child, not a sector entry");
        }
        if (1..=self.next_key).contains(&key) {
            format!("the roster entry {key} has ended: it was removed or cleared")
        } else {
            format!("the roster entry {key} was never issued")
        }
    }
}

/// What the record two placements of one entity end up holding: a state beats nothing, a withdrawal beats nothing, and two statements that differ are refused (they come from one entity, so one state).
fn merge_fixed(kept: FixedHold, moved: FixedHold) -> Result<FixedHold, Refusal> {
    match (kept, moved) {
        (held, FixedHold::Never) | (FixedHold::Never, held) => Ok(held),
        (a, b) if a == b => Ok(a),
        _ => Err(Refusal::new("[link] the two placements hold different fixed physical state".to_string())),
    }
}

/// The shared handle the model provider reads and writes: the one roster of the host, like [`crate::movement::SharedMovers`].
#[derive(Clone)]
pub struct SharedRoster(pub std::rc::Rc<std::cell::RefCell<SectorRoster>>);

#[cfg(test)]
mod tests {
    use super::*;

    fn walk(roster: &SectorRoster, sector: (i32, i32)) -> Vec<u64> {
        let mut seen = Vec::new();
        let mut at = roster.first(sector);
        while let Some(key) = at {
            seen.push(key);
            at = roster.next(sector, key).expect("a live entry in its own sector");
        }
        seen
    }

    #[test]
    fn a_sector_is_visited_oldest_insertion_first_and_a_relink_goes_to_the_end() {
        let mut roster = SectorRoster::default();
        let (a, b, c) = (roster.insert((1, 1), 7, None), roster.insert((1, 1), 9, None), roster.insert((1, 1), 7, None));
        assert_eq!(walk(&roster, (1, 1)), vec![a, b, c]);
        // a relink is a removal and a new insertion: the entity is now the newest
        roster.remove(a).expect("live");
        let a2 = roster.insert((1, 1), 7, None);
        assert_eq!(walk(&roster, (1, 1)), vec![b, c, a2]);
        assert_ne!(a, a2, "a key is never issued twice");
        // removing the middle one leaves the others in the same order
        roster.remove(c).expect("live");
        assert_eq!(walk(&roster, (1, 1)), vec![b, a2]);
        assert_eq!(roster.first((0, 0)), None);
    }

    #[test]
    fn an_ended_unknown_or_misplaced_entry_is_refused_by_name_never_served() {
        let mut roster = SectorRoster::default();
        let a = roster.insert((2, 3), 1, Some("eech-world:1"));
        roster.remove(a).expect("live");
        assert!(roster.remove(a).unwrap_err().0.contains("has ended"));
        assert!(roster.next((2, 3), a).unwrap_err().0.contains("has ended"));
        assert!(roster.remove(99).unwrap_err().0.contains("never issued"));
        let b = roster.insert((2, 3), 1, None);
        assert!(roster.next((0, 0), b).unwrap_err().0.contains("not in the sector"));
        roster.clear();
        assert!(roster.next((2, 3), b).unwrap_err().0.contains("has ended"));
        let c = roster.insert((2, 3), 1, None);
        assert!(c > b, "a cleared roster never reissues an earlier key");
    }

    #[test]
    fn the_sector_of_a_position_truncates_toward_zero_like_the_original() {
        let bits = |value: f32| value.to_bits();
        assert_eq!(sector_of(bits(0.0), bits(999.9), 1000), Ok((0, 0)));
        assert_eq!(sector_of(bits(1000.0), bits(2999.99), 1000), Ok((1, 2)));
        // the original truncates a negative coordinate toward zero too, so a position just below zero is still sector 0
        assert_eq!(sector_of(bits(-0.5), bits(-999.0), 1000), Ok((0, 0)));
        assert!(sector_of(f32::NAN.to_bits(), bits(0.0), 1000).is_err());
        assert!(sector_of(bits(f32::INFINITY), bits(0.0), 1000).is_err());
        assert!(sector_of(bits(0.0), bits(0.0), 0).is_err());
    }

    #[test]
    fn the_host_maps_a_hosted_mobile_by_its_semantic_identity_and_keeps_no_coordinate() {
        let mut roster = SectorRoster::default();
        let mobile = roster.insert((4, 4), 2, Some("eech-world:12"));
        let structure = roster.insert((4, 4), 20, None);
        assert_eq!(roster.host_id(mobile), Some("eech-world:12"));
        assert_eq!(roster.host_id(structure), None);
        assert_eq!(roster.entity_type(structure), Some(20));
        assert_eq!(roster.sector(mobile), Some((4, 4)));
        assert_eq!(roster.len(), 2);
    }

    #[test]
    fn the_integer_classification_is_the_original_truncation_for_every_word_of_the_domain_and_refuses_the_rest_by_name() {
        assert_eq!(WORD_LIMIT, 1.0e9f32.to_bits(), "the limit is the bits of 1.0e9");
        assert_eq!(WORD_NONFINITE, f32::INFINITY.to_bits());
        // the truncation equals the compiler's float-to-integer truncation (toward zero) over a sweep of every magnitude and both signs, with the whole-number test
        let mut state = 0x2545_f491u32;
        let mut words: Vec<u32> = vec![
            0,
            0x8000_0000,
            1,
            0x8000_0001,
            0x007f_ffff,
            0x0080_0000,
            0x3f7f_ffff,
            0x3f80_0000,
            0xbf80_0000,
            0x3fff_ffff,
            WORD_LIMIT,
            WORD_LIMIT | 0x8000_0000,
        ];
        for _ in 0..200_000 {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            words.push(state);
        }
        let mut accepted = 0;
        for word in words {
            let value = f32::from_bits(word);
            match word_truncated(word) {
                Ok((truncated, whole)) => {
                    accepted += 1;
                    assert!(value.is_finite() && value.abs() <= 1.0e9, "{word:#010x}");
                    assert_eq!(truncated, value as i32, "{word:#010x}");
                    assert_eq!(whole, value == value.trunc(), "{word:#010x}");
                }
                Err(_) => assert!(
                    !value.is_finite() || value.abs() > 1.0e9,
                    "{word:#010x} was refused although it is in the domain"
                ),
            }
        }
        assert!(accepted > 1000, "the sweep reached the domain");
        for refused in [
            0x7f80_0000,
            0xff80_0000,
            0x7fc0_0000,
            0x7fa0_0001,
            0xffa0_0001,
            0x7fff_ffff,
            WORD_LIMIT + 1,
            (WORD_LIMIT + 1) | 0x8000_0000,
        ] {
            assert!(word_truncated(refused).is_err(), "{refused:#010x}");
        }
        // whole numbers within a range: a signed zero is zero, a fraction is not whole, a limit is inclusive
        assert_eq!(word_whole(0x8000_0000, 0, 3), Some(0));
        assert_eq!(word_whole(2.0f32.to_bits(), 0, 3), Some(2));
        assert_eq!(word_whole(2.5f32.to_bits(), 0, 3), None);
        assert_eq!(word_whole(4.0f32.to_bits(), 0, 3), None);
        assert_eq!(word_whole((-1.0f32).to_bits(), 0, 3), None);
        assert_eq!(word_whole(0x7fa0_0001, 0, 3), None);
        assert_eq!(word_whole(16_777_216.0f32.to_bits(), 0, 16_777_216), Some(16_777_216));
    }

    #[test]
    fn a_roster_key_is_one_to_twenty_digits_without_a_leading_zero_and_nothing_else() {
        assert_eq!(parse_key("1"), Some(1));
        assert_eq!(parse_key("18446744073709551615"), Some(u64::MAX));
        for bad in [
            "",
            "0",
            "012",
            "+1",
            "-1",
            "1 ",
            " 1",
            "1.5",
            "eech-world:1",
            "18446744073709551616",
            "100000000000000000000",
            "٣",
        ] {
            assert_eq!(parse_key(bad), None, "{bad:?}");
        }
    }

    fn state(shape: i32) -> FixedState {
        FixedState {
            position: [1.0f32.to_bits(), 2.0f32.to_bits(), 3.0f32.to_bits()],
            heading: 0.5f32.to_bits(),
            shape,
            collision_fixed: true,
            align: false,
        }
    }

    fn held(roster: &SectorRoster, key: u64, child: bool) -> FixedHold {
        roster.placement_fixed(key, child).expect("a live placement of its kind").2
    }

    #[test]
    fn a_placements_fixed_state_is_its_records_stated_by_type_replaced_by_a_statement_and_withdrawn_only_by_the_host() {
        let mut roster = SectorRoster::default();
        let key = roster.insert((0, 0), 17, None);
        assert_eq!(held(&roster, key, false), FixedHold::Never, "a record holds none until the campaign states it");
        roster.set_fixed(key, 17, state(7)).expect("the placement's own type");
        assert_eq!(held(&roster, key, false), FixedHold::Held(state(7)));
        // a statement replaces what the record held; another type's statement is refused and changes nothing
        roster.set_fixed(key, 17, state(8)).unwrap();
        assert_eq!(held(&roster, key, false), FixedHold::Held(state(8)));
        assert!(roster.set_fixed(key, 21, state(9)).unwrap_err().0.contains("entity type 21"));
        assert_eq!(held(&roster, key, false), FixedHold::Held(state(8)));
        assert!(roster.set_fixed(99, 17, state(1)).unwrap_err().0.contains("never issued"));
        // a sector placement is not a city child and the other way round: the kind a branch has is the kind it names
        assert!(roster.placement_fixed(key, true).unwrap_err().0.contains("sector entry"));
        // the host's own withdrawal removes only the state: the placement stands, and the next statement restores it
        roster.withdraw_fixed(key).unwrap();
        assert_eq!(held(&roster, key, false), FixedHold::Withdrawn);
        assert_eq!(roster.sector(key), Some((0, 0)));
        assert!(roster.withdraw_fixed(key).unwrap_err().0.contains("holds no fixed physical state to withdraw"));
        roster.set_fixed(key, 17, state(7)).unwrap();
        assert_eq!(held(&roster, key, false), FixedHold::Held(state(7)));
        // a record that never held a state has nothing to withdraw
        let other = roster.insert((0, 0), 17, None);
        assert!(roster.withdraw_fixed(other).is_err());
        assert_eq!(held(&roster, other, false), FixedHold::Never);
    }

    #[test]
    fn the_state_follows_the_record_through_the_link_of_a_buildings_two_placements_and_a_relink_and_ends_with_the_last_placement() {
        let mut roster = SectorRoster::default();
        let city = roster.insert((0, 0), 5, None);
        let child = roster.child_insert(city, None, 17).unwrap();
        roster.set_fixed(child, 17, state(7)).unwrap();
        assert_eq!(held(&roster, child, true), FixedHold::Held(state(7)));
        // the sector placement is a new record that joins the child's: it holds the state from then on
        let sector = roster.insert((0, 0), 17, None);
        assert_eq!(held(&roster, sector, false), FixedHold::Never);
        roster.link(sector, child).unwrap();
        assert_eq!(held(&roster, sector, false), FixedHold::Held(state(7)));
        assert_eq!(held(&roster, child, true), FixedHold::Held(state(7)), "one record, one state");
        // a statement under either placement is the one record's
        roster.set_fixed(sector, 17, state(9)).unwrap();
        assert_eq!(held(&roster, child, true), FixedHold::Held(state(9)));
        // a relink: the sector placement leaves (the child keeps the record and its state) and a new one joins it
        roster.remove(sector).unwrap();
        assert_eq!(held(&roster, child, true), FixedHold::Held(state(9)));
        let relinked = roster.insert((1, 1), 17, None);
        assert_eq!(held(&roster, relinked, false), FixedHold::Never);
        roster.link(relinked, child).unwrap();
        assert_eq!(held(&roster, relinked, false), FixedHold::Held(state(9)));
        // the child leaves the city's list while the sector placement stays: the record stays; the last placement ends it and the state with it
        roster.child_remove(city, child).unwrap();
        assert_eq!(held(&roster, relinked, false), FixedHold::Held(state(9)));
        roster.remove(relinked).unwrap();
        assert!(roster.placement_fixed(relinked, false).unwrap_err().0.contains("has ended"));
        // a relink of an entity with no child is a removal and a new insertion: a new record that holds none until it is stated
        let alone = roster.insert((0, 0), 17, None);
        roster.set_fixed(alone, 17, state(7)).unwrap();
        roster.remove(alone).unwrap();
        let again = roster.insert((0, 0), 17, None);
        assert_eq!(held(&roster, again, false), FixedHold::Never);
        // a boot ends every record and its state
        roster.set_fixed(again, 17, state(7)).unwrap();
        roster.clear();
        assert!(roster.placement_fixed(again, false).is_err());
    }

    #[test]
    fn two_statements_that_differ_are_never_merged_by_a_link_and_a_withdrawal_is_kept() {
        let mut roster = SectorRoster::default();
        let city = roster.insert((0, 0), 5, None);
        let child = roster.child_insert(city, None, 17).unwrap();
        let sector = roster.insert((0, 0), 17, None);
        roster.set_fixed(child, 17, state(7)).unwrap();
        roster.set_fixed(sector, 17, state(8)).unwrap();
        let refusal = roster.link(sector, child).unwrap_err();
        assert!(refusal.0.contains("different fixed physical state"), "{}", refusal.0);
        // nothing was merged by the refusal: each placement still holds its own
        assert_eq!(held(&roster, child, true), FixedHold::Held(state(7)));
        assert_eq!(held(&roster, sector, false), FixedHold::Held(state(8)));
        // the same statement under both is one state; a withdrawal is kept against a record that never held one
        roster.set_fixed(sector, 17, state(7)).unwrap();
        roster.link(sector, child).unwrap();
        assert_eq!(held(&roster, sector, false), FixedHold::Held(state(7)));
        let city2 = roster.insert((2, 2), 5, None);
        let child2 = roster.child_insert(city2, None, 17).unwrap();
        roster.set_fixed(child2, 17, state(7)).unwrap();
        roster.withdraw_fixed(child2).unwrap();
        let sector2 = roster.insert((2, 2), 17, None);
        roster.link(sector2, child2).unwrap();
        assert_eq!(held(&roster, sector2, false), FixedHold::Withdrawn);
    }

    #[test]
    fn a_placement_of_no_fixed_family_can_hold_no_fixed_state_and_a_refused_statement_changes_nothing() {
        let mut roster = SectorRoster::default();
        // a city (it holds its own state, not the family's), a fixed wing, a helicopter, a person, a sound effect and a weapon
        for entity_type in [ENTITY_CITY, 10, 14, 19, 32, 37] {
            let key = roster.insert((0, 0), entity_type, None);
            let refusal = roster.set_fixed(key, entity_type, state(7)).unwrap_err();
            assert!(refusal.0.contains("not of the fixed family"), "{entity_type}: {}", refusal.0);
            assert_eq!(held(&roster, key, false), FixedHold::Never, "{entity_type}: nothing was recorded");
            assert!(roster.withdraw_fixed(key).is_err(), "{entity_type}: nothing to withdraw");
        }
        for entity_type in FIXED_FAMILY {
            let key = roster.insert((0, 0), entity_type, None);
            roster.set_fixed(key, entity_type, state(7)).unwrap();
            assert_eq!(held(&roster, key, false), FixedHold::Held(state(7)), "{entity_type}");
        }
    }

    fn city_state(seed: u32) -> CityState {
        CityState {
            position: [seed, seed + 1, seed + 2],
            box_min: [seed + 3, seed + 4, seed + 5],
            box_max: [seed + 6, seed + 7, seed + 8],
        }
    }

    #[test]
    fn a_citys_own_state_is_its_records_stated_by_type_replaced_by_a_statement_and_withdrawn_only_by_the_host() {
        let mut roster = SectorRoster::default();
        let city = roster.insert((0, 0), ENTITY_CITY, None);
        assert_eq!(held(&roster, city, false), FixedHold::Never, "a record holds none until the campaign states it");
        roster.set_city(city, city_state(1)).unwrap();
        assert_eq!(held(&roster, city, false), FixedHold::City(city_state(1)));
        // a statement replaces the last (the insertion's inverted initial box, then each building's expansion)
        let sentinel = CityState {
            position: [10, 11, 12],
            box_min: [1.0e6f32.to_bits(); 3],
            box_max: [(-1.0e6f32).to_bits(); 3],
        };
        roster.set_city(city, sentinel).unwrap();
        assert_eq!(
            held(&roster, city, false),
            FixedHold::City(sentinel),
            "the inverted box is a valid statement as stated"
        );
        roster.set_city(city, city_state(5)).unwrap();
        // only the host retires it; the placement stays standing, the campaign's next statement restores it
        roster.withdraw_fixed(city).unwrap();
        assert_eq!(held(&roster, city, false), FixedHold::Withdrawn);
        assert!(roster.withdraw_fixed(city).unwrap_err().0.contains("holds no fixed physical state to withdraw"));
        roster.set_city(city, city_state(5)).unwrap();
        assert_eq!(held(&roster, city, false), FixedHold::City(city_state(5)));
        // the family's statement is not a city's and a city's is not the family's
        assert!(roster.set_fixed(city, ENTITY_CITY, state(7)).unwrap_err().0.contains("not of the fixed family"));
        let object = roster.insert((0, 0), 17, None);
        assert!(roster.set_city(object, city_state(1)).unwrap_err().0.contains("not a city"));
        assert_eq!(held(&roster, object, false), FixedHold::Never, "a refused statement changes nothing");
        assert!(roster.set_city(99, city_state(1)).unwrap_err().0.contains("never issued"));
    }

    #[test]
    fn a_placement_that_is_not_a_city_can_hold_no_city_state_and_a_building_listed_under_a_city_is_not_one() {
        let mut roster = SectorRoster::default();
        for entity_type in FIXED_FAMILY.into_iter().chain([10, 14, 19, 32, 37]) {
            let key = roster.insert((0, 0), entity_type, None);
            let refusal = roster.set_city(key, city_state(1)).unwrap_err();
            assert!(refusal.0.contains("not a city"), "{entity_type}: {}", refusal.0);
            assert_eq!(held(&roster, key, false), FixedHold::Never, "{entity_type}");
        }
        let city = roster.insert((0, 0), ENTITY_CITY, None);
        let child = roster.child_insert(city, None, ENTITY_CITY_BUILDING).unwrap();
        assert!(roster.set_city(child, city_state(1)).unwrap_err().0.contains("not a city"));
        assert_eq!(held(&roster, child, true), FixedHold::Never);
    }

    #[test]
    fn a_citys_state_follows_its_record_across_a_sector_relink_while_buildings_are_listed_and_ends_with_the_record() {
        let mut roster = SectorRoster::default();
        let city = roster.insert((0, 0), ENTITY_CITY, None);
        roster.set_city(city, city_state(1)).unwrap();
        let building = roster.child_insert(city, None, ENTITY_CITY_BUILDING).unwrap();
        // the first building's link expands the box: the city states it again (under the placement it has)
        roster.set_city(city, city_state(2)).unwrap();
        // the shared fixed setter removes the sector placement and inserts it again: the record, which the list hangs from, keeps the last state until the new placement is declared to continue it
        roster.remove(city).unwrap();
        let relinked = roster.insert((1, 1), ENTITY_CITY, None);
        assert_eq!(
            held(&roster, relinked, false),
            FixedHold::Never,
            "the new placement is a fresh record until it is linked"
        );
        roster.link(relinked, city).unwrap();
        assert_eq!(
            held(&roster, relinked, false),
            FixedHold::City(city_state(2)),
            "the record continues with its state"
        );
        // the insertion then states the new position under the new key: the one record's state is replaced
        roster.set_city(relinked, city_state(3)).unwrap();
        assert_eq!(held(&roster, relinked, false), FixedHold::City(city_state(3)));
        // a withdrawal is kept by the link, and the next statement restores it
        roster.withdraw_fixed(relinked).unwrap();
        roster.remove(relinked).unwrap();
        let again = roster.insert((2, 2), ENTITY_CITY, None);
        roster.link(again, city).unwrap();
        assert_eq!(held(&roster, again, false), FixedHold::Withdrawn);
        roster.set_city(again, city_state(4)).unwrap();
        assert_eq!(held(&roster, again, false), FixedHold::City(city_state(4)));
        // the last building leaves while the city stays placed: the record stays; the placement then leaves with nothing listed and the record and its state end with it
        roster.child_remove(city, building).unwrap();
        assert_eq!(held(&roster, again, false), FixedHold::City(city_state(4)));
        roster.remove(again).unwrap();
        assert!(roster.placement_fixed(again, false).unwrap_err().0.contains("has ended"));
        // an empty city's relink is a removal and a new insertion: a new record that holds none until it is stated
        let empty = roster.insert((0, 0), ENTITY_CITY, None);
        roster.set_city(empty, city_state(1)).unwrap();
        roster.remove(empty).unwrap();
        let fresh = roster.insert((0, 0), ENTITY_CITY, None);
        assert_eq!(held(&roster, fresh, false), FixedHold::Never);
        // a statement made BEFORE the link would carry a state of its own that differs from the record's: the link refuses it, never merges two cities' words (the campaign links first, then states)
        let listed = roster.insert((0, 0), ENTITY_CITY, None);
        roster.set_city(listed, city_state(1)).unwrap();
        roster.child_insert(listed, None, ENTITY_CITY_BUILDING).unwrap();
        roster.remove(listed).unwrap();
        let early = roster.insert((0, 0), ENTITY_CITY, None);
        roster.set_city(early, city_state(9)).unwrap();
        assert!(roster.link(early, listed).unwrap_err().0.contains("different fixed physical state"));
        // a boot ends every record and its state
        roster.clear();
        assert!(roster.placement_fixed(fresh, false).is_err());
    }

    const ENTITY_TYPES_H: &str = include_str!("../../../../aphavoc/source/entity/system/en_types/en_types.h");
    const PLACE_FAMILY_C: &str = include_str!("../../../../aphavoc/source/entity/mobile/mb_hostplace.c");

    /// the value of a campaign entity type: its place in the campaign's own `enum ENTITY_TYPES`, which gives no member a value of its own
    fn campaign_entity_type(name: &str) -> i32 {
        let body = ENTITY_TYPES_H
            .split("enum ENTITY_TYPES")
            .nth(1)
            .and_then(|rest| rest.split("};").next())
            .expect("the enum");
        assert!(!body.contains('='), "an explicit value would make the order an unsafe count");
        let members: Vec<&str> = body.lines().filter_map(|line| line.trim().strip_suffix(',')).collect();
        i32::try_from(
            members
                .iter()
                .position(|member| *member == name)
                .unwrap_or_else(|| panic!("{name} is not an entity type")),
        )
        .unwrap()
    }

    #[test]
    fn the_entity_types_the_host_classifies_a_placement_by_are_the_campaigns_own_and_the_fixed_family_is_its_reporter_s() {
        assert_eq!(ENTITY_CITY, campaign_entity_type("ENTITY_TYPE_CITY"));
        assert_eq!(ENTITY_CITY_BUILDING, campaign_entity_type("ENTITY_TYPE_CITY_BUILDING"));
        // the family is the cases of the campaign's own reporter (`place_fixed_family`), which the engine's source guard holds equal to the types that overload the fixed functions, a city excepted
        let start = PLACE_FAMILY_C.find("static int place_fixed_family").expect("the family");
        let end = PLACE_FAMILY_C[start..].find("\n}\n").expect("its end") + start;
        let mut family: Vec<i32> = PLACE_FAMILY_C[start..end]
            .lines()
            .filter_map(|line| line.trim().strip_prefix("case ").map(|rest| rest.trim_end_matches(':').trim()))
            .map(campaign_entity_type)
            .collect();
        family.sort_unstable();
        assert_eq!(
            family, FIXED_FAMILY,
            "the host holds a state for the types the campaign reports, no more and no fewer"
        );
        assert!(!is_fixed_family(ENTITY_CITY));
        assert!(is_fixed_family(ENTITY_CITY_BUILDING));
    }
}
