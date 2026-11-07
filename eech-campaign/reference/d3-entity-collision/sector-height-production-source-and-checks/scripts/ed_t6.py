p = "entity_authority.rs"
d = open(p, "rb").read().decode("utf-8")
old = 'states nothing but the position, the box minimum, the box maximum, the sub type and its phase'
new = 'states nothing but the position, the box minimum, the box maximum, the shape, the sub type and its phase'
assert d.count(old) == 2, d.count(old)
d = d.replace(old, new)
old2 = 'if fault == "stray_fact" then words [10] = "0x3f800000"'
assert d.count(old2) == 1
d = d.replace(old2, 'if fault == "stray_fact" then words [15] = "0x3f800000"')
open(p, "wb").write(d.encode("utf-8"))
print("ok")
