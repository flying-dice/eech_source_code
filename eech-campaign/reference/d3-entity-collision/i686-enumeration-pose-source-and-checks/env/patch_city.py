from pathlib import Path
p = Path("R:/d3-evidence/collision-characterization-20261029/eech-campaign/crates/eech-world/csrc/entity_candidate_step.c")
d = p.read_bytes()
for lhs in (b"target_min_position.y", b"target_max_position.y", b"target_min_position.x", b"target_min_position.z", b"target_max_position.x", b"target_max_position.z"):
    axis = lhs[-1:]
    old = lhs + b" += target_position." + axis + b";\n"
    new = lhs + b" = NARROW(" + lhs + b" + target_position." + axis + b");\n"
    assert d.count(old) == 1, old
    d = d.replace(old, new)
p.write_bytes(d)
print("ok")
