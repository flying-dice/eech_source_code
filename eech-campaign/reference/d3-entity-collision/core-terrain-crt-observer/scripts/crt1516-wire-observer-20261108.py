import pathlib,json
W=pathlib.Path('R:/d3-evidence/collision-characterization-20261029');S=W/'eech-campaign/crates/eech-engine-sys'
def new(p,s):
    with p.open('x',encoding='utf-8',newline='\n') as f:f.write(s)
def edit(p,old,newtext,count=1):
    raw=p.read_bytes();a=old.encode();b=newtext.encode();assert raw.count(a)==count,(p,raw.count(a),old);p.write_bytes(raw.replace(a,b))
rows=[]
def patch(file,old,newtext,count=1):
    assert (W/file).read_bytes().count(old.encode())==count,(file,old)
    rows.append((file,old,newtext,count))
terrain='modules/3d/terrain/terrdata.c';skin='modules/3d/3dobjid.c'
patch(terrain,'\tASSERT ( !terrain_initialised );','\teech_core_crt_load_begin (session_cloud_puffs, get_global_season ());\n\tASSERT ( !terrain_initialised );')
for kind,name in [(1,'FOREST_TREE_OBJECT'),(2,'CLOUD')]:
    variable='terrain_3d_tree_object' if kind==1 else 'terrain_3d_cloud_puff_object'
    original=variable+' = construct_3d_object_by_name ( "'+name+'" );'
    replacement='eech_core_crt_ctor ('+str(kind)+', 0);\n\t'+original+'\n\teech_core_crt_ctor (0, '+variable+' ? '+variable+'->object_number : -1);'
    patch(terrain,original,replacement,2 if kind==1 else 1)
t=(W/terrain).read_text();end=t.index('void unload_3d_terrain');at=t.rindex('\treturn ( TRUE );',0,end)
old=t[at:end];patch(terrain,old,'\teech_core_crt_load_end (current_terrain_version.map_number, terrain_3d_tree_map_width, terrain_3d_tree_map_height);\n'+old)
old='\tskin = &objects_3d_scene_database[scene->object_number].skin;\n\tif ( skin->animation < 0 )'
patch(skin,old,'\tskin = &objects_3d_scene_database[scene->object_number].skin;\n\teech_core_crt_skin_begin (scene->object_number, skin->animation, get_global_season (), skin->number_of_default, skin->number_of_desert, skin->number_of_winter);\n\tif ( skin->animation < 0 )')
old='\tif ( skin->animation < 0 )\n\t{\n\t\treturn;\n\t}'
patch(skin,old,'\tif ( skin->animation < 0 )\n\t{\n\t\teech_core_crt_skin_end ();\n\t\treturn;\n\t}')
old='\tnumber_of_textures = texture_animations[skin->animation].number_of_frames;\n\ttexture = 0;'
patch(skin,old,'\tnumber_of_textures = texture_animations[skin->animation].number_of_frames;\n\teech_core_crt_skin_frames (number_of_textures);\n\ttexture = 0;')
old='\n}\n\nvoid skin_next ( object_3d_instance* scene )'
patch(skin,old,'\n\teech_core_crt_skin_end ();\n}\n\nvoid skin_next ( object_3d_instance* scene )')
code='// TEST-ONLY, strict additive staged observer hooks; no original RNG expression replaced.\nuse super::patches::Patch;\npub const PATCHES: &[Patch] = &[\n'
for i,(file,old,replacement,count) in enumerate(rows):
    code+='    Patch { file: '+json.dumps(file)+', id: "CRT'+str(i+1)+'", why: "observe existing Core loader/constructor/skin boundary", original: '+json.dumps(old)+', replacement: '+json.dumps(replacement)+', count: '+str(count)+' },\n'
code+=''' ];
pub fn files() -> Vec<&'static str> {
    let mut v: Vec<_> = PATCHES.iter().map(|p|p.file).collect();v.sort_unstable();v.dedup();v
}
pub fn apply(file: &str, source: &str) -> String {
    let mut out=source.to_owned();
    for p in PATCHES.iter().filter(|p|p.file==file) {assert_eq!(out.matches(p.original).count(),p.count,"{} in {}",p.id,file);out=out.replace(p.original,p.replacement);}
    out
}
'''
new(S/'build/core_crt_observer.rs',code)
edit(S/'build/main.rs','mod patches;','mod patches;\nmod core_crt_observer;')
old='    // 1a3b. TEST-ONLY (feature sector-height-observer'
newtext='''    if env::var_os("CARGO_FEATURE_CORE_TERRAIN_CRT_OBSERVER").is_some() {
        assert!(env::var_os("CARGO_FEATURE_TARGET_POINT_FIXTURE").is_none(), "Core CRT observer cannot share the target-point RNG wrapper");
        for file in core_crt_observer::files() {
            let staged=tree.join(file);let bytes=fs::read(&staged).unwrap();
            let source:String=bytes.iter().map(|&b|b as char).collect();
            let patched:Vec<u8>=core_crt_observer::apply(file,&source).chars().map(|c|c as u32 as u8).collect();
            write_if_changed(&staged,&patched);
        }
    }

'''+old
edit(S/'build/main.rs',old,newtext)
old='    if windows {\n        build.flag("-include").flag(manifest.join("csrc/eech_win32_names.h").to_str().unwrap());'
newtext='''    if env::var_os("CARGO_FEATURE_CORE_TERRAIN_CRT_OBSERVER").is_some() {
        build.define("EECH_CORE_TERRAIN_CRT_OBSERVER",None);
        build.flag("-include").flag(manifest.join("csrc/eech_core_crt_observer.h").to_str().unwrap());
        build.file(manifest.join("csrc/eech_core_crt_observer.c"));
    }
'''+old
edit(S/'build/main.rs',old,newtext)
for crate,feature in [('eech-engine-sys','[]'),('eech-engine','["eech-engine-sys/core-terrain-crt-observer"]'),('eech-dc','["eech-engine/core-terrain-crt-observer"]')]:
    p=W/'eech-campaign/crates'/crate/'Cargo.toml';edit(p,'[features]\n','[features]\n# TEST-ONLY Core loader CRT observer; never a shipping profile.\ncore-terrain-crt-observer = '+feature+'\n')
p=S/'csrc/eech_engine.c';raw=p.read_bytes();assert b'eech_engine_core_terrain_crt_summary' not in raw
raw+=b'''\n#ifdef EECH_CORE_TERRAIN_CRT_OBSERVER
int eech_engine_core_terrain_crt_summary (char *text, int capacity)
{
    ENTER (OPERATION_OBSERVE, NULL);
    if (state != ENGINE_RUNNING) { LEAVE (EECH_ENGINE_STATE); }
    if (!text || capacity <= 0 || eech_core_crt_report (text, capacity)) { LEAVE (EECH_ENGINE_BAD_ARGUMENT); }
    LEAVE (EECH_ENGINE_OK);
}
#endif
''';p.write_bytes(raw)
with (S/'src/lib.rs').open('a',encoding='utf-8') as f:f.write('\n#[cfg(feature = "core-terrain-crt-observer")]\nextern "C" { pub fn eech_engine_core_terrain_crt_summary(text: *mut c_char, capacity: c_int) -> c_int; }\n')
p=W/'eech-campaign/crates/eech-engine/src/lib.rs'
old='    pub fn target_point_summary(&self) -> Result<String, EngineError> {'
pos=p.read_bytes().index(old.encode());start=p.read_bytes().rindex(b'    /// TEST-ONLY',0,pos)
method='''    /// TEST-ONLY Core CRT observation; original module CRT only, never Host CRT.
    #[cfg(feature = "core-terrain-crt-observer")]
    pub fn core_terrain_crt_summary(&self) -> Result<String, EngineError> {
        let mut text=vec![0 as std::os::raw::c_char;65536];
        // SAFETY: the engine receives the live buffer and exact capacity, and terminates successful output.
        check(unsafe {sys::eech_engine_core_terrain_crt_summary(text.as_mut_ptr(),text.len() as i32)})?;
        // SAFETY: successful output is NUL terminated within the supplied buffer.
        Ok(unsafe {std::ffi::CStr::from_ptr(text.as_ptr())}.to_string_lossy().into_owned())
    }

'''
raw=p.read_bytes();p.write_bytes(raw[:start]+method.encode()+raw[start:])
p=W/'eech-campaign/crates/eech-dc/src/module.rs';old='        methods.add_method("target_point_summary", |_, this, ()| this.engine.target_point_summary().map_err(lua_error));'
raw=p.read_bytes();start=raw.rindex(b'        // TEST-ONLY',0,raw.index(old.encode()))
insert='''        // TEST-ONLY Core CRT observer, not a shipping capability.
        #[cfg(feature = "core-terrain-crt-observer")]
        methods.add_method("core_terrain_crt_summary", |_, this, ()| this.engine.core_terrain_crt_summary().map_err(lua_error));
''';p.write_bytes(raw[:start]+insert.encode()+raw[start:])
print('Wired dedicated observer feature and eight additive staged hooks; original source files untouched')
