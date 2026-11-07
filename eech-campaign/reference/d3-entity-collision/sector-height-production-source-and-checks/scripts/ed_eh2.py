p = "src/entity_host.rs"
d = open(p, "rb").read().decode("utf-8")
old = "pub const CANDIDATE_MOBILE: usize = 6;\n"
new = "pub const CANDIDATE_MOBILE: usize = 6;\n/// the sector's tallest structure: the host's own raw word, any bit pattern (see `classify_candidate_words`)\npub const CANDIDATE_TALLEST: usize = 12;\n"
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
p = "src/model_host.rs"
d = open(p, "rb").read().decode("utf-8")
old = """        const TALLEST: usize = 12;
        let sector = roster"""
new = """        const TALLEST: usize = entity_host::CANDIDATE_TALLEST;
        let sector = roster"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
