p = "src/entity_host.rs"
d = open(p, "rb").read().decode("utf-8")
old = """    if let Some(at) = words.iter().position(|word| !finite(*word)) {
        return Err(format!("fact {at} of an entity candidate question is not a finite number"));
    }"""
new = """    // Fact 12 is the sector's tallest structure, the HOST's own raw word (`crate::roster::SectorRoster::sector_height`): any bit pattern is valid history (a negative, infinite or NaN word restored from a session is
    // neither clamped nor refused), so it is the one fact that is not required to be finite. The join to the host's word (`model_host::fixed_join`) compares its bits, and the decision compares it the way the original does
    // (a strict greater-than, false for a NaN).
    if let Some(at) = words.iter().enumerate().position(|(at, word)| at != CANDIDATE_TALLEST && !finite(*word)) {
        return Err(format!("fact {at} of an entity candidate question is not a finite number"));
    }"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
