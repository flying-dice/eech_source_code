import pathlib
p=pathlib.Path('R:/d3-evidence/collision-characterization-20261029/eech-campaign/crates/eech-dc/src/module.rs')
raw=p.read_bytes();tag=b'\n#[cfg(test)]\n#[path = "native_flag_read_refusal_check.rs"]\nmod native_flag_read_refusal_check;\n'
assert b'native_flag_read_refusal_check' not in raw
p.write_bytes(raw+tag)
assert p.read_bytes().count(b'\0')==raw.count(b'\0')
