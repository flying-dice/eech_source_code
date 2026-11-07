//! The host's PHYSICAL per-entity UPDATED / MOVED / ROTATED store (EEDCS-49, D3, Task49): the three words, their set, their destructive read and their reset, as ONE responsibility the campaign no longer holds.
//!
//! What the original is, and what this reproduces (characterised by the 199-expectation and 56-expectation real-C controls of `eech-engine-sys`, `physical_flags` and `physical_flags_consumer`, original-solver
//! profile): three global bit arrays, one bit per entity heap slot (word = slot >> 5, bit = slot & 31); a set only ORs the bit (the original's setter is TRUE-only); a read is `word &= mask` and returns that: the
//! entity's own mask (0x80000000 for bit 31, a negative `int` in C), 0 when the bit was clear, and the bit of EVERY OTHER entity of the word is destroyed, the entity's own kept (a repeated read answers the mask
//! again); the reset is a quarter-memset (a count of BYTES), so it reaches the words of slots 0 to 131071 and never those above, whose bits survive a reset until a word-mate's read or another overwrite clears them.
//!
//! What the host is told, and what it is not. A raw heap slot, pointer or layout never crosses: the campaign states, for each word it uses, ONE opaque GROUP (the entities that share the word) and the host issues
//! its key (`open`); it then names the group by that key and the entity by its LANE, the bit it has in the word (0 to 31), and the array (0 UPDATED, 1 MOVED, 2 ROTATED). Whether a group lies inside the extent the
//! original's reset reaches is one fact the campaign states when it opens the group (`eligible`), and `reset` clears the array's word of every group that has it. The grouping is the campaign's (it is a property of
//! its heap), the bits and everything done to them are the host's. A key that was never issued, a malformed key, an array or a lane out of its domain are refused by name and change nothing.
//!
//! LIFETIME. The store is the host's: it lives with the host binding that holds it and is NOT cleared by a campaign boot or a new roster identity, because the original's static arrays are not (a slot that is
//! reused inherits the history of the slot). Nothing here normalises a raw word to a boolean, and no state is mirrored anywhere else.

use crate::roster::Refusal;

/// the three words: UPDATED, MOVED and ROTATED
pub const ARRAYS: usize = 3;

struct Group {
    /// whether the group lies inside the extent the original's quarter-memset reset reaches (the campaign states it once, when it opens the group)
    eligible: bool,
    /// the group's 32-bit word of each array: one bit per lane
    words: [u32; ARRAYS],
}

/// The host's flag store: the groups the campaign has opened, in the order it opened them.
#[derive(Default)]
pub struct FlagStore {
    groups: Vec<Group>,
}

impl FlagStore {
    /// Opens a group (the entities that share one word) and returns the key the host issues for it: `f` and the decimal number of the group, which names nothing else and is never a roster key. The group's three
    /// words start clear, as the original's static arrays do.
    pub fn open(&mut self, eligible: bool) -> String {
        self.groups.push(Group { eligible, words: [0; ARRAYS] });
        format!("f{}", self.groups.len())
    }

    /// The number of groups opened.
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    fn index(&self, key: &str) -> Result<usize, Refusal> {
        let digits = key
            .strip_prefix('f')
            .filter(|digits| !digits.is_empty() && !digits.starts_with('0') && digits.bytes().all(|byte| byte.is_ascii_digit()))
            .ok_or_else(|| {
                Refusal(format!(
                    "[flag.key.malformed] \"{key}\" is not a flag group key: `f` and a decimal number without a leading zero"
                ))
            })?;
        match digits.parse::<usize>() {
            Ok(number) if (1..=self.groups.len()).contains(&number) => Ok(number - 1),
            _ => Err(Refusal(format!("[flag.key.unknown] the host issued no flag group \"{key}\""))),
        }
    }

    fn check(array: usize, lane: u32) -> Result<(), Refusal> {
        if array >= ARRAYS {
            return Err(Refusal(format!("[flag.array] the array {array} is not 0 (UPDATED), 1 (MOVED) or 2 (ROTATED)")));
        }
        if lane > 31 {
            return Err(Refusal(format!("[flag.lane] the lane {lane} is not a bit of a 32-bit word")));
        }
        Ok(())
    }

    /// The SET: the lane's bit of the group's word is set (the original's setter only ever ORs a bit). Answers the lane's mask.
    pub fn set(&mut self, key: &str, array: usize, lane: u32) -> Result<u32, Refusal> {
        Self::check(array, lane)?;
        let at = self.index(key)?;
        let mask = 1u32 << lane;
        self.groups[at].words[array] |= mask;
        Ok(mask)
    }

    /// The destructive READ, the original's `word &= mask` and its value: the raw word, which is the lane's own mask when its bit was set and 0 when it was not. Every OTHER lane of the group's word is cleared and
    /// the lane's own bit is kept, so a repeated read answers the same mask.
    pub fn read(&mut self, key: &str, array: usize, lane: u32) -> Result<u32, Refusal> {
        Self::check(array, lane)?;
        let at = self.index(key)?;
        let mask = 1u32 << lane;
        let word = &mut self.groups[at].words[array];
        *word &= mask;
        Ok(*word)
    }

    /// The RESET of one array: its word of every group inside the original's reset extent is cleared (a group outside it keeps its bits). Answers the number of groups cleared.
    pub fn reset(&mut self, array: usize) -> Result<u32, Refusal> {
        Self::check(array, 0)?;
        let mut cleared = 0;
        for group in self.groups.iter_mut().filter(|group| group.eligible) {
            group.words[array] = 0;
            cleared += 1;
        }
        Ok(cleared)
    }

    /// OBSERVATION for the host's own tests and diagnostics: the group's whole word of one array, as the host holds it now. Not a question the campaign can ask.
    pub fn word(&self, key: &str, array: usize) -> Result<u32, Refusal> {
        Self::check(array, 0)?;
        Ok(self.groups[self.index(key)?].words[array])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_is_opened_with_clear_words_and_a_flag_key_that_is_never_a_roster_key() {
        let mut store = FlagStore::default();
        assert!(store.is_empty());
        let first = store.open(true);
        let second = store.open(false);
        assert_eq!((first.as_str(), second.as_str(), store.len()), ("f1", "f2", 2));
        for array in 0..ARRAYS {
            assert_eq!(store.word(&first, array).unwrap(), 0);
            assert_eq!(store.word(&second, array).unwrap(), 0);
        }
        assert!(crate::roster::parse_key(&first).is_none(), "a flag key is not a roster key");
    }

    #[test]
    fn a_set_ors_the_lane_and_a_read_returns_the_raw_mask_destroying_every_other_lane_but_not_its_own() {
        let mut store = FlagStore::default();
        let key = store.open(true);
        assert_eq!(store.set(&key, 0, 0).unwrap(), 1);
        assert_eq!(store.set(&key, 0, 5).unwrap(), 0x20);
        assert_eq!(store.set(&key, 0, 31).unwrap(), 0x8000_0000);
        assert_eq!(store.word(&key, 0).unwrap(), 0x8000_0021);
        // the other two words are untouched
        assert_eq!((store.word(&key, 1).unwrap(), store.word(&key, 2).unwrap()), (0, 0));
        // the read of lane 5 answers its own mask, destroys lanes 0 and 31 and keeps lane 5; a repeated read answers the mask again
        assert_eq!(store.read(&key, 0, 5).unwrap(), 0x20);
        assert_eq!(store.word(&key, 0).unwrap(), 0x20);
        assert_eq!(store.read(&key, 0, 5).unwrap(), 0x20);
        // a read of a lane whose bit is clear answers 0 and destroys the set neighbour
        assert_eq!(store.read(&key, 0, 6).unwrap(), 0);
        assert_eq!(store.word(&key, 0).unwrap(), 0);
        // bit 31: the raw word is 0x80000000, which is a negative int in C and never normalised
        store.set(&key, 0, 31).unwrap();
        assert_eq!(store.read(&key, 0, 31).unwrap(), 0x8000_0000);
        assert_eq!(store.read(&key, 0, 31).unwrap(), 0x8000_0000);
    }

    #[test]
    fn two_groups_and_three_arrays_never_touch_each_other() {
        let mut store = FlagStore::default();
        let a = store.open(true);
        let b = store.open(true);
        store.set(&a, 0, 3).unwrap();
        store.set(&a, 1, 3).unwrap();
        store.set(&b, 2, 3).unwrap();
        assert_eq!(store.read(&b, 0, 3).unwrap(), 0, "another group's UPDATED lane 3 is clear");
        assert_eq!(store.read(&a, 2, 3).unwrap(), 0, "another array's lane 3 of the same group is clear");
        assert_eq!(store.read(&a, 0, 3).unwrap(), 8);
        assert_eq!(store.read(&a, 1, 3).unwrap(), 8, "the read of one array destroyed nothing of another");
        assert_eq!(store.read(&b, 2, 3).unwrap(), 8);
    }

    #[test]
    fn a_reset_clears_one_array_of_every_group_inside_the_extent_and_leaves_the_others_and_the_groups_outside() {
        let mut store = FlagStore::default();
        let inside = store.open(true);
        let outside = store.open(false);
        for key in [&inside, &outside] {
            for array in 0..ARRAYS {
                store.set(key, array, 7).unwrap();
            }
        }
        assert_eq!(store.reset(0).unwrap(), 1, "one group lies inside the extent");
        assert_eq!(store.word(&inside, 0).unwrap(), 0);
        assert_eq!(
            store.word(&outside, 0).unwrap(),
            0x80,
            "a group outside the extent keeps its bit across the reset"
        );
        assert_eq!(
            (store.word(&inside, 1).unwrap(), store.word(&inside, 2).unwrap()),
            (0x80, 0x80),
            "the other arrays are untouched"
        );
        // the bit outside the extent survives until a word-mate's read clears it (a read of another lane destroys it)
        assert_eq!(store.reset(0).unwrap(), 1);
        assert_eq!(store.word(&outside, 0).unwrap(), 0x80);
        assert_eq!(store.read(&outside, 0, 8).unwrap(), 0);
        assert_eq!(store.word(&outside, 0).unwrap(), 0, "a word-mate's read cleared it: not permanent");
        assert_eq!(store.reset(1).unwrap(), 1);
        assert_eq!(store.reset(2).unwrap(), 1);
        assert_eq!(store.word(&inside, 1).unwrap(), 0);
        assert_eq!(store.word(&outside, 2).unwrap(), 0x80);
    }

    #[test]
    fn a_malformed_unknown_or_out_of_domain_question_is_refused_by_name_and_changes_nothing() {
        let mut store = FlagStore::default();
        let key = store.open(true);
        store.set(&key, 0, 2).unwrap();
        let refused = |result: Result<u32, Refusal>, tag: &str| {
            let Err(Refusal(text)) = result else { panic!("accepted: wanted {tag}") };
            assert!(text.starts_with(&format!("[{tag}]")), "{text}");
        };
        refused(store.set("f2", 0, 2), "flag.key.unknown");
        refused(store.read("f0", 0, 2), "flag.key.malformed");
        refused(store.read("f01", 0, 2), "flag.key.malformed");
        refused(store.read("7", 0, 2), "flag.key.malformed");
        refused(store.read("", 0, 2), "flag.key.malformed");
        refused(store.read("f", 0, 2), "flag.key.malformed");
        refused(store.read("f99999999999999999999999", 0, 2), "flag.key.unknown");
        refused(store.set(&key, 3, 2), "flag.array");
        refused(store.read(&key, 0, 32), "flag.lane");
        refused(store.reset(3), "flag.array");
        assert_eq!(store.word(&key, 0).unwrap(), 4, "a refused question changed nothing");
        assert_eq!(store.read(&key, 0, 2).unwrap(), 4);
    }
}
