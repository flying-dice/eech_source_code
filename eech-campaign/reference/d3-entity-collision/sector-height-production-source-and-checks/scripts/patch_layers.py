import sys
ROOT = sys.argv[1]

def patch(rel, pairs, enc="utf-8"):
    p = ROOT + "/" + rel
    d = open(p, "rb").read().decode(enc)
    for old, new in pairs:
        assert d.count(old) == 1, (rel, old[:80], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode(enc))

patch("eech-campaign/crates/eech-engine-sys/src/lib.rs", [(
"""pub const EECH_MODEL_SCAN_EXPANDED: c_int = 9;
""", """pub const EECH_MODEL_SCAN_EXPANDED: c_int = 9;
/// the answers to the sector height read and restore (the scan question's phases 17 and 18): `candidate_count` (3) words of `candidate` are the sector's x and z as asked, then the word the host holds (the restored word as stored)
pub const EECH_MODEL_SCAN_HEIGHT: c_int = 10;
pub const EECH_MODEL_SCAN_RESTORED: c_int = 11;
""")])

patch("eech-campaign/crates/eech-engine/src/lib.rs", [
("""/// `Placed` (the key it issued is the answer's `key`), `Removed` and `Cleared`; a city's box expansion with `Expanded` (the six words of the box the host built, minimum then maximum, and the city's key as `key`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScanDecision {
    First([f32; 2]),
    Expanded([f32; 6]),
""", """/// `Placed` (the key it issued is the answer's `key`), `Removed` and `Cleared`; a city's box expansion with `Expanded` (the six words of the box the host built, minimum then maximum, and the city's key as `key`).
/// A sector's high-water height is answered with `Height` and its restore with `Restored`: three words, the sector's x and z as asked and then the word the host holds, each kept as the 32 bits it came in (the
/// height word is never classified, so a negative, infinite or NaN word crosses exactly).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScanDecision {
    First([f32; 2]),
    Expanded([f32; 6]),
    Height([f32; 3]),
    Restored([f32; 3]),
"""),
("""                    ScanDecision::Expanded(words) => {
                        reply.candidate[..6].copy_from_slice(&words);
                        reply.candidate_count = 6;
                        sys::EECH_MODEL_SCAN_EXPANDED
                    }
""", """                    ScanDecision::Expanded(words) => {
                        reply.candidate[..6].copy_from_slice(&words);
                        reply.candidate_count = 6;
                        sys::EECH_MODEL_SCAN_EXPANDED
                    }
                    ScanDecision::Height(words) => {
                        reply.candidate[..3].copy_from_slice(&words);
                        reply.candidate_count = 3;
                        sys::EECH_MODEL_SCAN_HEIGHT
                    }
                    ScanDecision::Restored(words) => {
                        reply.candidate[..3].copy_from_slice(&words);
                        reply.candidate_count = 3;
                        sys::EECH_MODEL_SCAN_RESTORED
                    }
"""),
])

patch("eech-campaign/crates/eech-dc/src/module.rs", [
("""		if result == 'expanded' then
			local p, why = list(a.p, 6)
			if not p then return nil, 'p ' .. why end
			return 'expanded', p, nil, nil, nil, nil, nil, key
		end
""", """		if result == 'expanded' then
			local p, why = list(a.p, 6)
			if not p then return nil, 'p ' .. why end
			return 'expanded', p, nil, nil, nil, nil, nil, key
		end
		-- a sector's high-water height (phases 17 and 18): the sector's x, z and the word, as three BIT PATTERNS only (a number would be a float, and the word is raw: never a number here), and no placement name
		if result == 'height' or result == 'restored' then
			if type(a.p) ~= 'string' then return nil, 'p must be three bit patterns, the sector x, z and the word' end
			local p, why = list(a.p, 3)
			if not p then return nil, 'p ' .. why end
			if key ~= '' then return nil, 'name is only for first, next and placed' end
			return result, p, nil, nil, nil, nil, nil, key
		end
"""),
("""		return nil, 'result must be first, expanded, next, offer, done_none, done_hit, placed, removed, cleared or refused'""",
 """		return nil, 'result must be first, expanded, height, restored, next, offer, done_none, done_hit, placed, removed, cleared or refused'"""),
("""        (ModelQuery::EntityScan { .. }, "first") => {""", """        (ModelQuery::EntityScan { .. }, "height" | "restored") => {
            let words = bridged_words::<3>(&a.ok_or_else(missing)?, "p", operation)?;
            Ok(ModelAnswer::EntityScan {
                decision: if tag == "height" { ScanDecision::Height(words) } else { ScanDecision::Restored(words) },
                key: g.unwrap_or_default(),
            })
        }
        (ModelQuery::EntityScan { .. }, "first") => {"""),
], enc="latin-1")
print("ok")
