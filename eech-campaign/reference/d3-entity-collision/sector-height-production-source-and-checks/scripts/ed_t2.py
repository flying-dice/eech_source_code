p = "entity_authority.rs"
d = open(p, "rb").read().decode("utf-8")
old = """    let real_host = lua.create_table().expect("a table");
    real_host.set("args", lua.create_table().expect("a table")).expect("args");
    eech_world::model_host::register(&lua, &real_host).expect("the real provider registers");"""
new = """    // the provider holds the host's roster, whose arrival of a fixed structure raises the sector's height from the host's own scene table: it needs an installation, here the minimal one (every scene slot the zero box)
    let model_root = model_root::Root::minimal();
    let real_host = lua.create_table().expect("a table");
    let host_args = lua.create_table().expect("a table");
    host_args.set("root", model_root.dir.to_string_lossy().to_string()).expect("root");
    real_host.set("args", host_args).expect("args");
    eech_world::model_host::register(&lua, &real_host).expect("the real provider registers");"""
assert d.count(old) == 1
d = d.replace(old, new)
# declare the support module after the file's leading doc/inner attributes: before the first `use`
import re
m = re.search(r"^use ", d, flags=re.M)
assert m
d = d[:m.start()] + '#[path = "support/model_root.rs"]\nmod model_root;\n\n' + d[m.start():]
open(p, "wb").write(d.encode("utf-8"))
print("ok")
