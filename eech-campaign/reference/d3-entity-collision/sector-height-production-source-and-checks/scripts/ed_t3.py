p = "model_target_point.rs"
d = open(p, "rb").read().decode("utf-8")
pairs = [
("""    let key = roster.borrow_mut().insert((0, 0), FIXED_TYPE as i32, None).to_string();
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    (lua, roster, key)""", """    let key = roster.borrow_mut().insert((0, 0), FIXED_TYPE as i32, None).to_string();
    // the sector's tallest structure is the host's word, and every fixed candidate question below carries 1000.0 as it (the campaign only passes it on): the host holds exactly that word
    roster.borrow_mut().restore_sector_height((0, 0), 1000.0f32.to_bits());
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    (lua, roster, key)"""),
("""    let key = roster.borrow_mut().insert((0, 0), entity_type, None).to_string();
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    (lua, roster, key)""", """    let key = roster.borrow_mut().insert((0, 0), entity_type, None).to_string();
    // the sector's tallest structure is the host's word, and every candidate question below carries 1000.0 as it unless it says otherwise: the host holds exactly that word
    roster.borrow_mut().restore_sector_height((0, 0), 1000.0f32.to_bits());
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    (lua, roster, key)"""),
]
for old, new in pairs:
    assert d.count(old) == 1, old[:60]
    d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
