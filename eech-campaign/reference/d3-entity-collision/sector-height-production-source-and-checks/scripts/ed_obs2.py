p = "sector_height.rs"
d = open(p, "rb").read().decode("utf-8")
old = "use eech_engine_sys as _;\n"
new = """//!
//! TWO PROFILES (Task49, the host-owned sector height). In the hosted profile (the default build) a sector's height is the HOST's word and this control pins that: the campaign's link response raises and writes
//! nothing, the getter and the pack read the host's word, the unpack states each saved word to it. The ORIGINAL body (the raise, the stored member, the pack lines) is compiled only where the original entity solver is
//! kept, so the accepted characterisation above runs there, unchanged, with `--features sector-height-check,collision-fixture` (plus a differential of the host's raise expression beside the original response).
use eech_engine_sys as _;
"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
