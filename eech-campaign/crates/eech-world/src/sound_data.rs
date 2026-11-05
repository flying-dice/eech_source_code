//! The reference host's physical sample metadata (EEDCS-51, D3): where the sound header and data files are, whether they are present, what the header holds and what it says about a named
//! sample, and the rate of each sample asset. The campaign names its descriptors and asks once, at boot (`sound_catalogue` of `host.env_provider ()`, docs/d3-sound-lifetime.md); this module
//! is the host's answer. The campaign opens and parses no sound file and holds no rate for this.
//!
//! What the original did, and what is kept (aphavoc/source/appsound/snd_data.c `open_application_sound_file_system`, `get_sound_block_header_index`; modules/system/files.c `file_exist`, `safe_fopen`):
//!
//! * The pair is `audio\sound.hdr` and `audio\sound.dat`. `file_exist` is a READ OPEN, tried in the original's order for each file: relative to the working directory (`cohokum`), under the
//!   installation root's `cohokum\` AGAIN (the same directory in an installation, and still its own attempt), then under `common\`. The header is looked for first and the data file ONLY if the header was found (the
//!   original's `&&`). When both were found the header is opened again (the search is repeated), read, parsed and closed, and only then is the data file opened again (also searched again) and kept open while the
//!   descriptors are looked up. Either file missing at the existence check is an absent pair, a legitimate answer (the retail installation holds neither); a file found and then gone at its second open is a refusal
//!   (`safe_fopen` is fatal in the original). Names are matched ignoring case, as the retail installation mixes it. The host's own attempts are recorded in order (`Attempt`); they are the HOST's file operations
//!   and no operation of the campaign is recorded for them.
//! * The header is little-endian: an `i32` count, then for each entry `i32` sample index, `i32` data offset, `i32` data length, `i32` name length and the name bytes (the name length counts the
//!   terminating NUL; the name is `audio\<asset>`). Bytes after the last entry are ignored.
//! * A descriptor looks up `audio\<asset>`: the FIRST entry with an equal rolling unsigned 32-bit hash (`hash += char; hash <<= 1`, the char signed, case sensitive), a case-insensitive equal name
//!   and the exact sample index. Its length is the size, and a length of 0 is a size, not a miss. A descriptor with no such entry keeps size 0 (the table's default).
//!
//! What is NOT kept: the original parses with unchecked reads, so a header it could only misread (a negative count or length, a truncated entry, a name with no terminator, a count the file cannot hold)
//! is a REFUSAL here with a named error, never a guess and never a fallback to the campaign. An entry's offset and length are not checked against the data file: the original only copies the length at
//! initialisation, and the data is read by the sample loader, which is not part of this answer.

use crate::sound_assets::SAMPLE_RATES;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The sound directory and the two files of the pair, as the host knows them (the campaign's table names assets only).
pub const DIRECTORY: &str = "audio";
pub const HEADER_FILE: &str = "sound.hdr";
pub const DATA_FILE: &str = "sound.dat";

/// The rate in hertz of a sample asset, by the exact name the campaign's table gives it.
pub fn rate_of(asset: &str) -> Option<i32> {
    SAMPLE_RATES
        .binary_search_by(|(name, _)| name.as_bytes().cmp(asset.as_bytes()))
        .ok()
        .map(|position| SAMPLE_RATES[position].1)
}

/// The original's rolling hash of a name (modules `get_sound_block_header_index`): each char is added as a SIGNED char to an unsigned 32-bit word, which is then shifted left by one.
pub fn rolling_hash(name: &[u8]) -> u32 {
    let mut hash = 0u32;
    for byte in name {
        hash = hash.wrapping_add(i32::from(*byte as i8) as u32);
        hash <<= 1;
    }
    hash
}

/// One entry of the header: the name is the bytes up to its first NUL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub index: i32,
    pub offset: i32,
    pub length: i32,
    pub name: Vec<u8>,
    hash: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub records: Vec<Record>,
}

fn word(bytes: &[u8], at: usize) -> Option<i32> {
    bytes
        .get(at..at.checked_add(4)?)
        .map(|slice| i32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

/// Parses a header, or names the first thing the original could only have misread.
pub fn parse_header(bytes: &[u8]) -> Result<Header, String> {
    let count = word(bytes, 0).ok_or_else(|| format!("[sound-data-provider:header] the header has {} bytes where its count needs 4", bytes.len()))?;
    if count < 0 {
        return Err(format!("[sound-data-provider:header] the header's entry count {count} is negative"));
    }
    // every entry is at least 16 bytes of fields and a name of at least its NUL
    let most = (bytes.len() - 4) / 17;
    if usize::try_from(count).map_or(true, |count| count > most) {
        return Err(format!(
            "[sound-data-provider:header] the header's entry count {count} cannot fit in its {} bytes",
            bytes.len()
        ));
    }
    let mut records = Vec::with_capacity(count as usize);
    let mut at = 4usize;
    for position in 0..count {
        let fields = |field: usize| word(bytes, at + 4 * field);
        let (index, offset, length, name_length) = match (fields(0), fields(1), fields(2), fields(3)) {
            (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
            _ => return Err(format!("[sound-data-provider:header] entry {position} is truncated in its fields")),
        };
        if offset < 0 || length < 0 {
            return Err(format!(
                "[sound-data-provider:header] entry {position} has a negative offset {offset} or length {length}"
            ));
        }
        let name_length = usize::try_from(name_length)
            .ok()
            .filter(|length| *length >= 1)
            .ok_or_else(|| format!("[sound-data-provider:header] entry {position} has the name length {name_length}"))?;
        let start = at + 16;
        let raw = bytes
            .get(start..start.saturating_add(name_length))
            .ok_or_else(|| format!("[sound-data-provider:header] entry {position}'s name is truncated"))?;
        let end = raw
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(|| format!("[sound-data-provider:header] entry {position}'s name has no terminating NUL"))?;
        let name = raw[..end].to_vec();
        records.push(Record {
            index,
            offset,
            length,
            hash: rolling_hash(&name),
            name,
        });
        at = start + name_length;
    }
    Ok(Header { records })
}

impl Header {
    /// The FIRST entry whose hash, case-insensitive name and sample index all match (`get_sound_block_header_index`).
    pub fn find(&self, filename: &[u8], index: i32) -> Option<&Record> {
        let hash = rolling_hash(filename);
        self.records
            .iter()
            .find(|record| record.hash == hash && record.name.eq_ignore_ascii_case(filename) && record.index == index)
    }
}

/// `name` in `dir`, exactly, else ignoring case.
fn find_ignoring_case(dir: &Path, name: &str) -> Option<PathBuf> {
    let exact = dir.join(name);
    if exact.exists() {
        return Some(exact);
    }
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.file_name().to_string_lossy().eq_ignore_ascii_case(name))
        .map(|entry| entry.path())
}

fn directory(root: &Path, parts: &[&str]) -> Option<PathBuf> {
    let mut path = root.to_path_buf();
    for part in parts {
        path = find_ignoring_case(&path, part)?;
    }
    path.is_dir().then_some(path)
}

/// The places a read-open of the original tries, in its order (modules/system/files.c `file_exist`, `safe_fopen`): relative to the working directory (`cohokum`), under the installation root's `cohokum\` AGAIN
/// (in an installation the same directory, and still a separate attempt), then under `common\`. They are never coalesced: each attempt is made, and counted, on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    WorkingDirectory,
    InstallationCohokum,
    InstallationCommon,
}

const PLACES: [Place; 3] = [Place::WorkingDirectory, Place::InstallationCohokum, Place::InstallationCommon];

/// Why a file is being opened: the existence check (`file_exist` opens and closes at once) or the open that is kept and read (`safe_fopen`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Exists,
    Open,
}

/// One read-open attempt of the HOST, in the order it was made. These are the host's file operations; the campaign makes none here and none are recorded for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub file: &'static str,
    pub purpose: Purpose,
    pub place: Place,
    pub ok: bool,
}

/// The points of the original's sequence at which a test may change the filesystem (the original had no such point; they are where a file could disappear between its operations).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// both existence checks passed, before the header is opened
    Checked,
    /// the header has been read, parsed and closed, before the data file is opened
    HeaderClosed,
}

/// One read-open attempt at one place: the file, found ignoring case under `<top>/audio`, a regular file that opens. Every attempt is recorded.
fn try_open(root: &Path, place: Place, file: &'static str, purpose: Purpose, trace: &mut Vec<Attempt>) -> Option<File> {
    let top = match place {
        Place::WorkingDirectory | Place::InstallationCohokum => "cohokum",
        Place::InstallationCommon => "common",
    };
    let opened = directory(root, &[top, DIRECTORY])
        .and_then(|dir| find_ignoring_case(&dir, file))
        .filter(|path| path.is_file())
        .and_then(|path| File::open(path).ok());
    trace.push(Attempt {
        file,
        purpose,
        place,
        ok: opened.is_some(),
    });
    opened
}

/// `file_exist`: tries the places in order and stops at the first that opens; the file is closed at once.
fn exists(root: &Path, file: &'static str, trace: &mut Vec<Attempt>) -> bool {
    PLACES.iter().any(|place| try_open(root, *place, file, Purpose::Exists, trace).is_some())
}

/// `safe_fopen`: the same search, and the file stays open.
fn open(root: &Path, file: &'static str, trace: &mut Vec<Attempt>) -> Option<File> {
    PLACES.iter().find_map(|place| try_open(root, *place, file, Purpose::Open, trace))
}

/// What the host answers for one descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub rate: i32,
    pub original_rate: i32,
    pub matched: bool,
    pub size: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub header_present: bool,
    pub header_count: i32,
    pub entries: Vec<Entry>,
}

/// The answer for the descriptors `(slot, asset name)` of a catalogue of `slots` slots, from the installation at `root`. The whole answer is built before it is returned: any refusal returns nothing.
pub fn answer(root: &Path, slots: i32, descriptors: &[(i32, String)]) -> Result<Reply, String> {
    answer_in_order(root, slots, descriptors, &mut |_| {}, &mut Vec::new())
}

/// `answer`, with the host's read-open attempts recorded in the order made and `between` called at the points of `Stage`. The sequence is the original's (`initialise_application_sound_effects`):
/// `file_exist (header)` and, only if that passed, `file_exist (data)`; if both passed, `safe_fopen (header)`, read and parsed, closed; then `safe_fopen (data)`, kept open while the descriptors are looked
/// up, and closed. A header or data file that was found and then cannot be opened, or a malformed header, is a refusal by name: the original would have stopped (`safe_fopen` is fatal) or misread.
fn answer_in_order(root: &Path, slots: i32, descriptors: &[(i32, String)], between: &mut dyn FnMut(Stage), trace: &mut Vec<Attempt>) -> Result<Reply, String> {
    if slots < 1 {
        return Err(format!("[sound-data-provider:request] the catalogue domain {slots} is not a size"));
    }
    let mut rates = Vec::with_capacity(descriptors.len());
    for (position, (slot, asset)) in descriptors.iter().enumerate() {
        if *slot < 0 || *slot >= slots {
            return Err(format!(
                "[sound-data-provider:request] descriptor {position} has slot {slot} where the domain is 0 to {}",
                slots - 1
            ));
        }
        if asset.is_empty() || asset.contains('\0') {
            return Err(format!(
                "[sound-data-provider:request] descriptor {position} (slot {slot}) has no usable asset name"
            ));
        }
        let rate = rate_of(asset)
            .ok_or_else(|| format!("[sound-data-provider:asset] the host has no sample asset named {asset:?} (descriptor {position}, slot {slot})"))?;
        rates.push(rate);
    }
    // file_exist (header) && file_exist (data): the data file is not looked for when the header is not there
    let present = exists(root, HEADER_FILE, trace) && exists(root, DATA_FILE, trace);
    let mut header = None;
    let mut _data_open = None;
    if present {
        between(Stage::Checked);
        let mut file = open(root, HEADER_FILE, trace)
            .ok_or_else(|| "[sound-data-provider:header] the header was found by the existence check and could not be opened".to_string())?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| format!("[sound-data-provider:header] the header could not be read: {error}"))?;
        header = Some(parse_header(&bytes)?);
        drop(file);
        between(Stage::HeaderClosed);
        _data_open = Some(
            open(root, DATA_FILE, trace)
                .ok_or_else(|| "[sound-data-provider:data] the data file was found by the existence check and could not be opened".to_string())?,
        );
    }
    let entries = descriptors
        .iter()
        .zip(rates)
        .map(|((slot, asset), rate)| {
            let found = header.as_ref().and_then(|header| {
                let filename = format!("{DIRECTORY}\\{asset}");
                header.find(filename.as_bytes(), *slot)
            });
            Entry {
                rate,
                original_rate: rate,
                matched: found.is_some(),
                size: found.map_or(0, |record| record.length),
            }
        })
        .collect();
    Ok(Reply {
        header_present: header.is_some(),
        header_count: header.as_ref().map_or(0, |header| header.records.len() as i32),
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A private installation root on a real filesystem, removed when the test ends.
    struct Root(PathBuf);

    impl Root {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("eech-sound-data-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
            std::fs::create_dir_all(&dir).expect("a scratch root");
            Root(dir)
        }

        fn put(&self, directory: &str, file: &str, bytes: &[u8]) {
            let dir = self.0.join(directory);
            std::fs::create_dir_all(&dir).expect("a directory");
            std::fs::write(dir.join(file), bytes).expect("a file");
        }
    }

    impl Drop for Root {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A header of `(index, offset, length, name)` entries in the original's layout.
    fn header(entries: &[(i32, i32, i32, &str)]) -> Vec<u8> {
        let mut bytes = (entries.len() as i32).to_le_bytes().to_vec();
        for (index, offset, length, name) in entries {
            let name = [name.as_bytes(), &[0]].concat();
            for number in [*index, *offset, *length, name.len() as i32] {
                bytes.extend_from_slice(&number.to_le_bytes());
            }
            bytes.extend_from_slice(&name);
        }
        bytes
    }

    /// The seven entries of the measured ORIGINAL control (`reference/d3-environment-characterization/sound-header-present-native-7574`): the key and name of thunder (93, matched, length 0),
    /// lightning (94, length 96 and a later duplicate of length 48), the keys 2087 (40) and 2051 (24), an asset no descriptor names, and a real name under another key.
    fn control_header() -> Vec<u8> {
        header(&[
            (93, 0, 0, "audio\\thunder.pcm"),
            (94, 0, 96, "audio\\lstrike.pcm"),
            (2087, 96, 40, "audio\\ch46e_windup_ext.pcm"),
            (2051, 136, 24, "audio\\kiowa_windup_ext.pcm"),
            (94, 160, 48, "audio\\lstrike.pcm"),
            (2051, 0, 16, "audio\\fixture-unmatched.pcm"),
            (2087, 16, 16, "audio\\comanche_windup_ext.pcm"),
        ])
    }

    fn descriptors() -> Vec<(i32, String)> {
        [
            (93, "thunder.pcm"),
            (94, "lstrike.pcm"),
            (2087, "ch46e_windup_ext.pcm"),
            (2051, "kiowa_windup_ext.pcm"),
            (1954, "comanche_windup_ext.pcm"),
            (2, "aprotore.pcm"),
        ]
        .iter()
        .map(|(slot, name)| (*slot, name.to_string()))
        .collect()
    }

    fn install(root: &Root, directory: &str, header_bytes: &[u8]) {
        root.put(directory, HEADER_FILE, header_bytes);
        root.put(directory, DATA_FILE, &[7u8; 208]);
    }

    #[test]
    fn the_asset_table_is_sorted_unique_and_every_rate_is_positive() {
        assert!(SAMPLE_RATES.windows(2).all(|pair| pair[0].0.as_bytes() < pair[1].0.as_bytes()));
        assert!(SAMPLE_RATES.iter().all(|(name, rate)| !name.is_empty() && *rate > 0));
        assert_eq!(rate_of("thunder.pcm"), Some(22050));
        assert_eq!(rate_of("THUNDER.PCM"), None, "the campaign's names are exact");
        assert_eq!(rate_of("no-such-asset.pcm"), None);
    }

    /// The descriptor table in snd_data.c is the original; this table is the host's own copy of its names and rates, so the two must not drift (tools/generate-sound-assets.py regenerates it).
    #[test]
    fn the_hosts_asset_rates_are_the_originals_descriptor_table_for_every_name() {
        // the original source is embedded by the compiler from the path relative to THIS file, so the test binary carries the exact table it was built against and needs no source tree at run time
        const SOURCE: &[u8] = include_bytes!("../../../../aphavoc/source/appsound/snd_data.c");
        let text: String = SOURCE.iter().map(|byte| *byte as char).collect();
        // strip comments
        let mut clean = String::new();
        let mut rest = text.as_str();
        while !rest.is_empty() {
            if let Some(tail) = rest.strip_prefix("/*") {
                rest = &tail[tail.find("*/").expect("a closed comment") + 2..];
            } else if let Some(tail) = rest.strip_prefix("//") {
                rest = tail.find('\n').map_or("", |at| &tail[at..]);
            } else {
                let step = rest.chars().next().unwrap().len_utf8();
                clean.push_str(&rest[..step]);
                rest = &rest[step..];
            }
        }
        let start = clean.find("application_sound_samples[]").expect("the table");
        let table = &clean[start..clean[start..].find("};").map(|end| start + end).expect("its end")];
        let mut found = std::collections::BTreeMap::new();
        let mut entries = 0usize;
        for entry in table.split('{').skip(2) {
            // { SOUND_SAMPLE_INDEX_X, ENTITY_SIDE_Y, "name", rate, ...
            let mut parts = entry.splitn(5, ',');
            let (symbol, side, name, rate) = (
                parts.next().unwrap().trim(),
                parts.next().unwrap().trim(),
                parts.next().unwrap().trim(),
                parts.next().unwrap().trim(),
            );
            assert!(symbol.starts_with("SOUND_SAMPLE_INDEX_") && side.starts_with("ENTITY_SIDE_"), "{symbol} {side}");
            if name == "NULL" {
                continue;
            }
            let name = name
                .strip_prefix('"')
                .and_then(|name| name.strip_suffix('"'))
                .expect("a string")
                .replace("\\\\", "\\");
            let rate: i32 = rate.parse().expect("a rate");
            assert_eq!(found.insert(name.clone(), rate).unwrap_or(rate), rate, "{name} has two rates");
            entries += 1;
        }
        assert!(entries >= found.len() && !found.is_empty());
        let host: std::collections::BTreeMap<String, i32> = SAMPLE_RATES.iter().map(|(name, rate)| (name.to_string(), *rate)).collect();
        assert_eq!(host, found, "tools/generate-sound-assets.py must be run again");
    }

    #[test]
    fn the_hash_is_the_originals_rolling_unsigned_sum_of_signed_chars() {
        assert_eq!(rolling_hash(b""), 0);
        assert_eq!(rolling_hash(b"a"), 194);
        assert_eq!(rolling_hash(b"ab"), (194 + 98) << 1);
        // a char above 0x7f is negative (the original's char is signed): -23 added to 0 wraps, and the shift drops the top bit
        assert_eq!(rolling_hash(&[0xe9]), 0xffff_ffe9u32 << 1);
        // the case of a name is part of its hash
        assert_ne!(rolling_hash(b"audio\\thunder.pcm"), rolling_hash(b"AUDIO\\THUNDER.PCM"));
    }

    #[test]
    fn an_installation_with_no_pair_is_a_legitimate_answer_with_the_hosts_own_rates_and_no_sizes() {
        let root = Root::new();
        root.put("cohokum", "game.cfg", b"x");
        let reply = answer(&root.0, 2122, &descriptors()).expect("an absent pair is not a refusal");
        assert!(!reply.header_present);
        assert_eq!(reply.header_count, 0);
        assert_eq!(reply.entries.len(), 6);
        assert!(reply
            .entries
            .iter()
            .all(|entry| !entry.matched && entry.size == 0 && entry.rate == entry.original_rate));
        assert_eq!(reply.entries[0].rate, 22050, "thunder");
        // a rate the host owns is not the campaign's to supply: the 11025 and 44100 Hz assets answer as they are
        let speech = answer(
            &root.0,
            2122,
            &[(5, "SPEECH\\CIS\\ATC\\0.PCM".to_string()), (6, "MUSIC\\MUSIC_ACTION1.wav".to_string())],
        )
        .unwrap();
        assert_eq!((speech.entries[0].rate, speech.entries[1].rate), (11025, 44100));
    }

    #[test]
    fn a_header_with_no_data_file_or_a_data_file_with_no_header_is_an_absent_pair() {
        for (file, other) in [(HEADER_FILE, DATA_FILE), (DATA_FILE, HEADER_FILE)] {
            let root = Root::new();
            root.put("cohokum/audio", file, &control_header());
            let reply = answer(&root.0, 2122, &descriptors()).unwrap();
            assert!(
                !reply.header_present && reply.entries.iter().all(|entry| !entry.matched),
                "{file} alone, without {other}"
            );
        }
        // a directory in the place of a file is not a file that opens
        let root = Root::new();
        std::fs::create_dir_all(root.0.join("cohokum/audio/sound.hdr")).unwrap();
        root.put("cohokum/audio", DATA_FILE, &[0u8; 8]);
        assert!(!answer(&root.0, 2122, &descriptors()).unwrap().header_present);
    }

    #[test]
    fn the_measured_original_control_header_is_answered_as_the_original_answered_it() {
        let root = Root::new();
        install(&root, "cohokum/audio", &control_header());
        let reply = answer(&root.0, 2122, &descriptors()).expect("a valid header");
        assert!(reply.header_present);
        assert_eq!(reply.header_count, 7);
        let sizes: Vec<(bool, i32)> = reply.entries.iter().map(|entry| (entry.matched, entry.size)).collect();
        assert_eq!(
            sizes,
            [
                (true, 0),  // thunder: matched with a length of 0, which is a size and not a miss
                (true, 96), // lightning: the FIRST of two entries for the key and name wins over the later 48
                (true, 40), // key 2087, at or above 2069
                (true, 24), // key 2051
                (false, 0), // comanche_windup_ext.pcm is in the header under another key: the index does not match
                (false, 0), // an asset the header does not hold
            ]
        );
        assert!(reply.entries.iter().all(|entry| entry.rate == entry.original_rate && entry.rate > 0));
    }

    #[test]
    fn the_first_entry_wins_whatever_follows_it_and_a_zero_length_match_is_not_a_miss() {
        let root = Root::new();
        install(
            &root,
            "cohokum/audio",
            &header(&[
                (93, 0, 0, "audio\\thunder.pcm"),
                (93, 0, 500, "audio\\thunder.pcm"),
                (94, 0, 7, "audio\\lstrike.pcm"),
            ]),
        );
        let reply = answer(&root.0, 2122, &[(93, "thunder.pcm".into()), (94, "lstrike.pcm".into())]).unwrap();
        assert_eq!((reply.entries[0].matched, reply.entries[0].size), (true, 0));
        assert_eq!((reply.entries[1].matched, reply.entries[1].size), (true, 7));
    }

    #[test]
    fn the_name_is_compared_after_a_case_sensitive_hash_so_a_name_in_another_case_does_not_match() {
        let root = Root::new();
        install(
            &root,
            "cohokum/audio",
            &header(&[(93, 0, 9, "AUDIO\\THUNDER.PCM"), (94, 0, 3, "audio\\lstrike.pcm")]),
        );
        let reply = answer(&root.0, 2122, &[(93, "thunder.pcm".into()), (94, "lstrike.pcm".into())]).unwrap();
        assert!(!reply.entries[0].matched, "the hash of a name in another case differs, as in the original");
        assert!(reply.entries[1].matched);
    }

    #[test]
    fn the_installation_search_order_is_cohokum_then_common_and_names_are_found_ignoring_case() {
        let root = Root::new();
        install(&root, "common/audio", &header(&[(94, 0, 11, "audio\\lstrike.pcm")]));
        let from_common = answer(&root.0, 2122, &[(94, "lstrike.pcm".into())]).unwrap();
        assert_eq!(
            (from_common.entries[0].matched, from_common.entries[0].size),
            (true, 11),
            "common is searched when cohokum has no pair"
        );
        // a pair under cohokum is found first, whatever the case of its directories and files
        let root = Root::new();
        install(&root, "common/audio", &header(&[(94, 0, 11, "audio\\lstrike.pcm")]));
        root.put("COHOKUM/AUDIO", "SOUND.HDR", &header(&[(94, 0, 22, "audio\\lstrike.pcm")]));
        root.put("COHOKUM/AUDIO", "SOUND.DAT", &[1u8; 64]);
        let from_cohokum = answer(&root.0, 2122, &[(94, "lstrike.pcm".into())]).unwrap();
        assert_eq!(from_cohokum.entries[0].size, 22);
        // each file is searched on its own, as file_exist is: a header in cohokum with the data only in common is still a pair
        let root = Root::new();
        root.put("cohokum/audio", HEADER_FILE, &header(&[(94, 0, 33, "audio\\lstrike.pcm")]));
        root.put("common/audio", DATA_FILE, &[1u8; 64]);
        let split = answer(&root.0, 2122, &[(94, "lstrike.pcm".into())]).unwrap();
        assert_eq!((split.header_present, split.entries[0].size), (true, 33));
    }

    #[test]
    fn a_header_the_original_could_only_misread_is_refused_by_name_and_answers_nothing() {
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("empty", vec![]),
            ("three bytes", vec![1, 0, 0]),
            ("negative count", (-1i32).to_le_bytes().to_vec()),
            ("a count the file cannot hold", 5i32.to_le_bytes().to_vec()),
            ("truncated fields", {
                let mut bytes = header(&[(1, 0, 4, "audio\\thunder.pcm")]);
                bytes.truncate(4 + 10);
                bytes[0] = 1;
                bytes
            }),
            ("truncated name", {
                let mut bytes = header(&[(1, 0, 4, "audio\\thunder.pcm")]);
                bytes.truncate(bytes.len() - 3);
                bytes
            }),
            ("a name length of zero", {
                let mut bytes = header(&[(1, 0, 4, "audio\\thunder.pcm")]);
                bytes[16..20].copy_from_slice(&0i32.to_le_bytes());
                bytes
            }),
            ("an unterminated name", {
                let mut bytes = header(&[(1, 0, 4, "audio\\thunder.pcm")]);
                let last = bytes.len() - 1;
                bytes[last] = b'x';
                bytes
            }),
            ("a negative length", header(&[(1, 0, -4, "audio\\thunder.pcm")])),
            ("a negative offset", header(&[(1, -8, 4, "audio\\thunder.pcm")])),
        ];
        for (label, bytes) in cases {
            let root = Root::new();
            install(&root, "cohokum/audio", &bytes);
            let error = answer(&root.0, 2122, &descriptors()).expect_err(label);
            assert!(error.starts_with("[sound-data-provider:header]"), "{label}: {error}");
        }
        // a valid header followed by bytes the original never reads is still valid
        let root = Root::new();
        let mut bytes = control_header();
        bytes.extend_from_slice(&[0xff; 9]);
        install(&root, "cohokum/audio", &bytes);
        assert_eq!(answer(&root.0, 2122, &descriptors()).unwrap().header_count, 7);
    }

    #[test]
    fn a_request_the_host_cannot_answer_is_refused_whole_before_a_file_is_read() {
        let root = Root::new();
        install(&root, "cohokum/audio", b"not a header at all");
        for (label, slots, request) in [
            ("a slot outside the domain", 100, vec![(100, "thunder.pcm".to_string())]),
            ("a negative slot", 100, vec![(-1, "thunder.pcm".to_string())]),
            ("an empty name", 100, vec![(1, String::new())]),
            (
                "an asset the host has no rate for",
                100,
                vec![(1, "thunder.pcm".to_string()), (2, "nothing-like-it.pcm".to_string())],
            ),
            ("no domain", 0, vec![(0, "thunder.pcm".to_string())]),
        ] {
            let error = answer(&root.0, slots, &request).expect_err(label);
            assert!(
                error.starts_with("[sound-data-provider:request]") || error.starts_with("[sound-data-provider:asset]"),
                "{label}: {error}"
            );
        }
        // an empty request is a valid question about no descriptor, and the pair is still reported
        let root = Root::new();
        install(&root, "cohokum/audio", &control_header());
        let none = answer(&root.0, 2122, &[]).unwrap();
        assert_eq!((none.header_present, none.header_count, none.entries.len()), (true, 7, 0));
    }

    #[test]
    fn the_catalogue_domain_is_not_the_named_count() {
        // slot 2105 is valid in a domain of 2122 whatever number of descriptors is named; the host neither assumes nor bounds that number
        let root = Root::new();
        let many: Vec<(i32, String)> = SAMPLE_RATES
            .iter()
            .enumerate()
            .map(|(position, (name, _))| ((position % 2122) as i32, name.to_string()))
            .collect();
        let reply = answer(&root.0, 2122, &many).unwrap();
        assert_eq!(reply.entries.len(), SAMPLE_RATES.len());
        assert!(answer(&root.0, 2122, &[(2105, "thunder.pcm".to_string())]).is_ok());
        assert!(answer(&root.0, 2122, &[(2122, "thunder.pcm".to_string())]).is_err());
    }

    // ---- review ENV-SOUND-METADATA-2: the original's short-circuit, repeated search and repeated read-open order (the host's own attempts, recorded in order) ----

    use Place::{InstallationCohokum as Ic, InstallationCommon as Cm, WorkingDirectory as Wd};
    use Purpose::{Exists, Open};

    fn at(file: &'static str, purpose: Purpose, place: Place, ok: bool) -> Attempt {
        Attempt { file, purpose, place, ok }
    }

    /// The answer and the host's attempts for the control request, with `between` called at the stages.
    fn traced(root: &Root, between: &mut dyn FnMut(Stage)) -> (Result<Reply, String>, Vec<Attempt>) {
        let mut trace = Vec::new();
        let result = answer_in_order(&root.0, 2122, &descriptors(), between, &mut trace);
        (result, trace)
    }

    #[test]
    fn a_present_pair_is_four_read_opens_in_the_originals_order() {
        let root = Root::new();
        install(&root, "cohokum/audio", &control_header());
        let (result, trace) = traced(&root, &mut |_| {});
        assert!(result.is_ok());
        // file_exist (header), file_exist (data), safe_fopen (header), safe_fopen (data): each found at its first place
        assert_eq!(
            trace,
            [
                at(HEADER_FILE, Exists, Wd, true),
                at(DATA_FILE, Exists, Wd, true),
                at(HEADER_FILE, Open, Wd, true),
                at(DATA_FILE, Open, Wd, true)
            ]
        );
    }

    #[test]
    fn an_absent_header_is_three_attempts_and_the_data_file_is_never_looked_for() {
        let root = Root::new();
        root.put("cohokum/audio", DATA_FILE, &[0u8; 8]);
        let (result, trace) = traced(&root, &mut |_| {});
        assert!(!result.expect("an absent pair is an answer").header_present);
        // the working directory, the installation's cohokum again (a separate attempt), then common; the short-circuit means no attempt at the data file
        assert_eq!(
            trace,
            [
                at(HEADER_FILE, Exists, Wd, false),
                at(HEADER_FILE, Exists, Ic, false),
                at(HEADER_FILE, Exists, Cm, false)
            ]
        );
    }

    #[test]
    fn a_header_with_no_data_file_looks_for_the_data_file_in_all_three_places_and_opens_nothing() {
        let root = Root::new();
        root.put("cohokum/audio", HEADER_FILE, &control_header());
        let (result, trace) = traced(&root, &mut |_| {});
        assert!(!result.expect("an absent pair is an answer").header_present);
        assert_eq!(
            trace,
            [
                at(HEADER_FILE, Exists, Wd, true),
                at(DATA_FILE, Exists, Wd, false),
                at(DATA_FILE, Exists, Ic, false),
                at(DATA_FILE, Exists, Cm, false)
            ]
        );
    }

    #[test]
    fn every_search_is_repeated_from_the_first_place_for_a_header_in_common_and_data_in_cohokum() {
        let root = Root::new();
        root.put("common/audio", HEADER_FILE, &control_header());
        root.put("cohokum/audio", DATA_FILE, &[0u8; 8]);
        let (result, trace) = traced(&root, &mut |_| {});
        assert_eq!(result.expect("a pair split over the two places").header_count, 7);
        assert_eq!(
            trace,
            [
                // file_exist (header): the first two places fail, common has it
                at(HEADER_FILE, Exists, Wd, false),
                at(HEADER_FILE, Exists, Ic, false),
                at(HEADER_FILE, Exists, Cm, true),
                // file_exist (data): the first place has it
                at(DATA_FILE, Exists, Wd, true),
                // safe_fopen (header) searches again from the first place; safe_fopen (data) too
                at(HEADER_FILE, Open, Wd, false),
                at(HEADER_FILE, Open, Ic, false),
                at(HEADER_FILE, Open, Cm, true),
                at(DATA_FILE, Open, Wd, true),
            ]
        );
    }

    #[test]
    fn a_malformed_header_is_refused_before_the_data_file_is_opened() {
        let root = Root::new();
        install(&root, "cohokum/audio", &(-1i32).to_le_bytes());
        let (result, trace) = traced(&root, &mut |_| {});
        assert!(result.expect_err("a negative count").starts_with("[sound-data-provider:header]"));
        assert!(!trace.iter().any(|attempt| attempt.file == DATA_FILE && attempt.purpose == Open), "{trace:?}");
    }

    #[test]
    fn a_data_file_that_is_gone_when_it_is_opened_again_after_the_header_is_refused_by_name() {
        let root = Root::new();
        install(&root, "cohokum/audio", &control_header());
        let data = root.0.join("cohokum/audio/sound.dat");
        let (result, trace) = traced(&root, &mut |stage| {
            if stage == Stage::HeaderClosed {
                std::fs::remove_file(&data).expect("the data file goes");
            }
        });
        assert!(result.expect_err("the data file disappeared").starts_with("[sound-data-provider:data]"));
        // the second open searched all three places and found nothing
        assert_eq!(
            &trace[trace.len() - 3..],
            [at(DATA_FILE, Open, Wd, false), at(DATA_FILE, Open, Ic, false), at(DATA_FILE, Open, Cm, false)]
        );
    }

    #[test]
    fn a_header_that_is_gone_between_the_existence_check_and_its_open_is_refused_by_name() {
        let root = Root::new();
        install(&root, "cohokum/audio", &control_header());
        let header = root.0.join("cohokum/audio/sound.hdr");
        let (result, _) = traced(&root, &mut |stage| {
            if stage == Stage::Checked {
                std::fs::remove_file(&header).expect("the header goes");
            }
        });
        assert!(result.expect_err("the header disappeared").starts_with("[sound-data-provider:header]"));
    }
}
