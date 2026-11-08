import pathlib,subprocess
W=pathlib.Path('R:/d3-evidence/collision-characterization-20261029')
p=W/'eech-campaign/crates/eech-engine-sys/src/lib.rs';raw=p.read_bytes()
old=b'extern "C" { pub fn eech_engine_core_terrain_crt_summary(text: *mut c_char, capacity: c_int) -> c_int; }'
assert raw.count(old)==1
new=b'''extern "C" {
    pub fn eech_engine_core_terrain_crt_summary(text: *mut c_char, capacity: c_int) -> c_int;
    // Feature-only observer controls; same native implementation, no copied CRT.
    pub fn eech_core_crt_rand(file: *const c_char, line: c_int) -> c_int;
    pub fn eech_core_crt_srand(seed: u32, file: *const c_char, line: c_int);
    pub fn eech_core_crt_load_begin(clouds: c_int, season: c_int);
    pub fn eech_core_crt_load_end(map: c_int, width: c_int, height: c_int);
    pub fn eech_core_crt_boundary(name: *const c_char);
    pub fn eech_core_crt_ctor(kind: c_int, scene: c_int);
    pub fn eech_core_crt_skin_begin(scene: c_int, animation: c_int, season: c_int, normal: c_int, desert: c_int, winter: c_int);
    pub fn eech_core_crt_skin_frames(frames: c_int);
    pub fn eech_core_crt_skin_end();
    pub fn eech_core_crt_report(text: *mut c_char, capacity: c_int) -> c_int;
}'''
p.write_bytes(raw.replace(old,new))
p=W/'eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c';raw=p.read_bytes()
for original,name in [(b'\treinitialise_entity_system ();',b'before_reinitialise'),(b'\tcampaign_roster_reset ();',b'after_reinitialise'),(b'\tcreate_campaign (get_current_game_session ());',b'before_create')]:
    assert raw.count(original)==1
    hook=b'#ifdef EECH_CORE_TERRAIN_CRT_OBSERVER\n\teech_core_crt_boundary ("'+name+b'");\n#endif\n'
    raw=raw.replace(original,hook+original)
p.write_bytes(raw)
print('Feature-only ABI binds actual observer library; boot phase markers added without replacing original calls')
