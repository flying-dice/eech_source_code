p = "src/entity_host.rs"
d = open(p, "rb").read().decode("utf-8")
old = """    if let Some(at) = facts.iter().position(|value| !value.is_finite()) {
        return Err(format!("fact {at} of an entity candidate question is not a finite number"));
    }
    for at in [6, 7, 8, 9, 10, 11, 22, 23, 42, 43] {
        if facts[at] != 0.0 && facts[at] != 1.0 {"""
new = """    // fact 12 is the host's own raw sector word, any bit pattern (see `classify_candidate_words`)
    if let Some(at) = facts.iter().enumerate().position(|(at, value)| at != CANDIDATE_TALLEST && !value.is_finite()) {
        return Err(format!("fact {at} of an entity candidate question is not a finite number"));
    }
    for at in [6, 7, 8, 9, 10, 11, 22, 23, 42, 43] {
        if facts[at] != 0.0 && facts[at] != 1.0 {"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
