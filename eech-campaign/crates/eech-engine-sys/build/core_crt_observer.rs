// TEST-ONLY, strict additive staged observer hooks; no original RNG expression replaced.
use super::patches::Patch;
pub const PATCHES: &[Patch] = &[
    Patch { file: "modules/3d/terrain/terrdata.c", id: "CRT1", why: "observe existing Core loader/constructor/skin boundary", original: "\tASSERT ( !terrain_initialised );", replacement: "\teech_core_crt_load_begin (session_cloud_puffs, get_global_season ());\n\tASSERT ( !terrain_initialised );", count: 1 },
    Patch { file: "modules/3d/terrain/terrdata.c", id: "CRT2", why: "observe existing Core loader/constructor/skin boundary", original: "terrain_3d_tree_object = construct_3d_object_by_name ( \"FOREST_TREE_OBJECT\" );", replacement: "eech_core_crt_ctor (1, 0);\n\tterrain_3d_tree_object = construct_3d_object_by_name ( \"FOREST_TREE_OBJECT\" );\n\teech_core_crt_ctor (0, terrain_3d_tree_object ? terrain_3d_tree_object->object_number : -1);", count: 2 },
    Patch { file: "modules/3d/terrain/terrdata.c", id: "CRT3", why: "observe existing Core loader/constructor/skin boundary", original: "terrain_3d_cloud_puff_object = construct_3d_object_by_name ( \"CLOUD\" );", replacement: "eech_core_crt_ctor (2, 0);\n\tterrain_3d_cloud_puff_object = construct_3d_object_by_name ( \"CLOUD\" );\n\teech_core_crt_ctor (0, terrain_3d_cloud_puff_object ? terrain_3d_cloud_puff_object->object_number : -1);", count: 1 },
    Patch { file: "modules/3d/terrain/terrdata.c", id: "CRT4", why: "observe existing Core loader/constructor/skin boundary", original: "\treturn ( TRUE );\n}\n\n\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n\n", replacement: "\teech_core_crt_load_end (current_terrain_version.map_number, terrain_3d_tree_map_width, terrain_3d_tree_map_height);\n\treturn ( TRUE );\n}\n\n\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n\n", count: 1 },
    Patch { file: "modules/3d/3dobjid.c", id: "CRT5", why: "observe existing Core loader/constructor/skin boundary", original: "\tskin = &objects_3d_scene_database[scene->object_number].skin;\n\tif ( skin->animation < 0 )", replacement: "\tskin = &objects_3d_scene_database[scene->object_number].skin;\n\teech_core_crt_skin_begin (scene->object_number, skin->animation, get_global_season (), skin->number_of_default, skin->number_of_desert, skin->number_of_winter);\n\tif ( skin->animation < 0 )", count: 1 },
    Patch { file: "modules/3d/3dobjid.c", id: "CRT6", why: "observe existing Core loader/constructor/skin boundary", original: "\tif ( skin->animation < 0 )\n\t{\n\t\treturn;\n\t}", replacement: "\tif ( skin->animation < 0 )\n\t{\n\t\teech_core_crt_skin_end ();\n\t\treturn;\n\t}", count: 1 },
    Patch { file: "modules/3d/3dobjid.c", id: "CRT7", why: "observe existing Core loader/constructor/skin boundary", original: "\tnumber_of_textures = texture_animations[skin->animation].number_of_frames;\n\ttexture = 0;", replacement: "\tnumber_of_textures = texture_animations[skin->animation].number_of_frames;\n\teech_core_crt_skin_frames (number_of_textures);\n\ttexture = 0;", count: 1 },
    Patch { file: "modules/3d/3dobjid.c", id: "CRT8", why: "observe existing Core loader/constructor/skin boundary", original: "\n}\n\nvoid skin_next ( object_3d_instance* scene )", replacement: "\n\teech_core_crt_skin_end ();\n}\n\nvoid skin_next ( object_3d_instance* scene )", count: 1 },
 ];
pub fn files() -> Vec<&'static str> {
    let mut v: Vec<_> = PATCHES.iter().map(|p| p.file).collect();
    v.sort_unstable();
    v.dedup();
    v
}
pub fn apply(file: &str, source: &str) -> String {
    let mut out = source.to_owned();
    for p in PATCHES.iter().filter(|p| p.file == file) {
        assert_eq!(out.matches(p.original).count(), p.count, "{} in {}", p.id, file);
        out = out.replace(p.original, p.replacement);
    }
    out
}
