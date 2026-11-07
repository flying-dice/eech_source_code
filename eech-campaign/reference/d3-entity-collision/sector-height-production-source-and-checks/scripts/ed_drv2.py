p = "entity-authority.lua"
d = open(p, "rb").read().decode("utf-8")
old = "city_state = 0, city_expand = 0, pose = 0 }"
assert d.count(old) == 2
d = d.replace(old, "city_state = 0, city_expand = 0, sector_height = 0, pose = 0 }")
open(p, "wb").write(d.encode("utf-8"))
print("ok")
