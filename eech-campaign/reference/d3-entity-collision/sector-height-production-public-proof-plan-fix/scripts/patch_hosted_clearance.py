import sys

ROOT = sys.argv[1] + "/eech-campaign/crates/eech-engine-sys/"


def patch(rel, pairs):
    p = ROOT + rel
    d = open(p, "rb").read().decode("utf-8")
    for old, new in pairs:
        assert d.count(old) == 1, (rel, old[:80], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode("utf-8"))


patch("csrc/eech_sector_height_check.c", [("""/* every dispatch table the initialisers write, saved whole and copied back */""", """#if defined(EECH_SECTOR_HEIGHT_OBSERVER) && defined(EECH_CAMPAIGN_HOST_ENTITY)

int eech_sho_clearance_run (int index, char *text, int capacity);

/* ---- the observer's HOSTED clearance helper (Task49): the original statement it follows consumed the HOST's word through the getter, so the helper must not read the retired member (a canary here) nor ask the host again,
   and its record must say `tallest=uncaptured`; the sector, the terrain elevation and the min_height after the statement are kept. Returns the record's own return code, or -2 if the host was asked ---- */
int eech_sho_hosted_clearance_check (char *text, int capacity)
{
	entity
		en;

	float
		terrain,
		minimum;

	sector
		*raw;

	int
		asks,
		result;

	terrain = 12.5f;

	minimum = 99.0f;

	asks = host_height_asks;

	make_sector (&en);

	raw = (sector *) get_local_entity_data (&en);

	raw->x_sector = 3;

	raw->z_sector = 4;

	store_word (&en, 0xc0ffee11u);

	eech_sho_clearance_leader ();

	eech_sho_clearance_follow ();

	eech_sho_clearance_tallest (&en, &terrain, &minimum);

	eech_sho_clearance_final (&minimum);

	free_sector (&en);

	result = (host_height_asks == asks) ? eech_sho_clearance_run (0, text, capacity) : -2;

	return result;
}

#endif

/* every dispatch table the initialisers write, saved whole and copied back */""")])

d = open(ROOT + "Cargo.toml", "rb").read().decode("utf-8")
old = """[[test]]
name = "sector_height_observer"
path = "tests/sector_height_observer.rs"
required-features = ["sector-height-check", "sector-height-observer"]
"""
new = old + """
[[test]]
name = "sector_height_hosted_clearance"
path = "tests/sector_height_hosted_clearance.rs"
required-features = ["sector-height-check", "sector-height-observer"]
"""
assert d.count(old) == 1
open(ROOT + "Cargo.toml", "wb").write(d.replace(old, new).encode("utf-8"))

open(ROOT + "tests/sector_height_hosted_clearance.rs", "wb").write(b'''//! TEST-ONLY (EEDCS-49, D3, Task49; features `sector-height-check` + `sector-height-observer`, HOSTED profile only): the observer's clearance helper in a hosted build.
//!
//! In a hosted build the fixed-wing clearance consumes the HOST's word through the sector getter, inside the original statement the observer's hook follows. The retired member is not that value and a second
//! query of the host would not be the value the original consumed, so the helper (csrc/eech_sho.c, `eech_sho_clearance_tallest`) must capture NO word: the record says `tallest=uncaptured`, keeps the sector, the
//! terrain elevation and the min_height after the statement, never reads the member (a canary here) and never asks the host. This is a boundary check of the helper; it does not reach the guarded clearance in the
//! campaign (no real-C or native run does: ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1). The observer's tables are private statics of one process: ONE test. It is meaningful only where the original body is
//! not compiled, so it holds nothing in the original-solver profile.
#![cfg(not(feature = "collision-fixture"))]
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_sho_hosted_clearance_check(text: *mut c_char, capacity: c_int) -> c_int;
}

#[test]
fn the_hosted_clearance_record_captures_no_consumed_word_and_never_reads_the_retired_member_or_asks_the_host() {
    let mut text = vec![0 as c_char; 4096];
    let code = unsafe { eech_sho_hosted_clearance_check(text.as_mut_ptr(), text.len() as c_int) };
    let line: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    assert_eq!(code, 0, "the helper asked the host, or its record could not be written: {line}");
    assert!(line.starts_with("sho-clearance i=0 "), "{line}");
    assert!(line.contains(" sx=3 sz=4 tallest=uncaptured "), "{line}");
    assert!(!line.contains("c0ffee11"), "the retired member was read: {line}");
    // 12.5 and 99.0 as the words the hook was handed, and the record is complete (the final min_height was seen)
    assert!(line.contains("terrain=0x41480000") && line.contains("after_tallest=0x42c60000") && line.contains("final=0x42c60000 final_seen=1"), "{line}");
}
''')
print("ok")
