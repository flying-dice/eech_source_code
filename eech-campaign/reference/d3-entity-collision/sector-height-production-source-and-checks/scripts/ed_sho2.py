p = "eech_sho_funnel_check.c"
d = open(p, "rb").read().decode("utf-8")
old = "(OUT)[9] = (E)->sub_type; } while (0)"
assert d.count(old) == 1
d = d.replace(old, "(OUT)[9] = (E)->sub_type; (OUT)[10] = 0.0f; } while (0)")
open(p, "wb").write(d.encode("utf-8"))
print("ok")
