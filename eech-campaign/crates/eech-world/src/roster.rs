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
//! entity (a structure, a city, a keysite, a weapon, an effect) has no pose record in the host; its sector is computed from the position the campaign states at the insertion, and ONLY the sector and the
//! sequence are kept: no coordinate is stored. The physical static pose of those entities, their shape and their live flags remain the campaign's (declared, required to move).
//!
//! A key is issued at an insertion and never again: a relink is a removal and a new insertion with a new sequence number, exactly the list operation it mirrors, and a removed key can never name
//! another entity.

use std::collections::{BTreeMap, HashMap};

/// The host's whole answer to a refused roster question: a named reason, never a default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal(pub String);

impl Refusal {
    fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

/// The sector index pair of a position: the original's `get_x_sector` and `get_z_sector` (a truncation toward zero, then the integer division by the side length).
pub fn sector_of(x_word: u32, z_word: u32, side: i32) -> Result<(i32, i32), Refusal> {
    if side < 1 {
        return Err(Refusal::new(format!("the sector side length {side} is not positive")));
    }
    let axis = |word: u32| -> Result<i32, Refusal> {
        let value = f32::from_bits(word);
        if !value.is_finite() || value.abs() > 1.0e9 {
            return Err(Refusal::new("a position outside the world has no sector"));
        }
        // `as` truncates toward zero, which is what `convert_float_to_int` does under the original's round-toward-zero
        Ok((value as i32) / side)
    };
    Ok((axis(x_word)?, axis(z_word)?))
}

struct Entry {
    sector: (i32, i32),
    sequence: u64,
    entity_type: i32,
    /// the semantic identity of a hosted mobile; none for every other entity (the explicit mapping from the scan key to the movement record)
    host_id: Option<String>,
}

/// The host's roster of the entities in the sector lists.
#[derive(Default)]
pub struct SectorRoster {
    /// the next key and the next insertion sequence; both only grow, so neither is ever reused
    next_key: u64,
    next_sequence: u64,
    entries: HashMap<u64, Entry>,
    sectors: HashMap<(i32, i32), BTreeMap<u64, u64>>,
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
            },
        );
        key
    }

    /// Removes an entity from its sector's list. The others keep their order.
    pub fn remove(&mut self, key: u64) -> Result<(), Refusal> {
        let entry = self.entries.remove(&key).ok_or_else(|| Refusal::new(self.unknown(key)))?;
        if let Some(sector) = self.sectors.get_mut(&entry.sector) {
            sector.remove(&entry.sequence);
            if sector.is_empty() {
                self.sectors.remove(&entry.sector);
            }
        }
        Ok(())
    }

    /// The first entity a scan of the sector visits: its oldest insertion.
    pub fn first(&self, sector: (i32, i32)) -> Option<u64> {
        self.sectors.get(&sector).and_then(|order| order.values().next().copied())
    }

    /// The entity a scan visits after `key` in the sector it is standing in: the next insertion. `sector` is the sector the scan is in, and an entity that is not in it is refused by name.
    pub fn next(&self, sector: (i32, i32), key: u64) -> Result<Option<u64>, Refusal> {
        let entry = self.entries.get(&key).ok_or_else(|| Refusal::new(self.unknown(key)))?;
        if entry.sector != sector {
            return Err(Refusal::new(format!(
                "the roster entry {key} stands in sector {:?}, not in the sector {sector:?} the scan is in",
                entry.sector
            )));
        }
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
    }

    fn unknown(&self, key: u64) -> String {
        if (1..=self.next_key).contains(&key) {
            format!("the roster entry {key} has ended: it was removed or cleared")
        } else {
            format!("the roster entry {key} was never issued")
        }
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
}
