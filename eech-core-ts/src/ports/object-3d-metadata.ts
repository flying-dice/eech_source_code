//
// Port: the 3D object database.
//
// C provenance: modules/3d/3dobjvis.c :: get_object_3d_bounding_box
//               modules/3d/objects.h :: struct OBJECT_3D_BOUNDS
//
// EECH loads every 3D object's geometry from the game's object files at run
// time and answers get_object_3d_bounding_box (object_3d_index_numbers) from
// that database. The campaign reads object dimensions through it:
// keysite.c :: update_keysite_cargo lays crates out by the size of
// OBJECT_3D_SINGLE_CRATE (and sc_msgs.c reads fixed entities' bounds). The
// object data is the environment's (a game installation, or whatever model a
// host maps an EECH object index to), so the campaign core asks this port.
//
// The query stays at EECH's level: an object index in, the six bounds out.
// Values are narrowed to C `float` by the core, not by the adapter.
//

export interface Object3DBounds {
	xmin: number;
	xmax: number;
	ymin: number;
	ymax: number;
	zmin: number;
	zmax: number;
}

export interface Object3DMetadata {
	// C: get_object_3d_bounding_box (object) -> struct OBJECT_3D_BOUNDS *.
	// objectIndex is an EECH object_3d_index_numbers value (e.g.
	// OBJECT_3D_SINGLE_CRATE, exported by the core).
	getBoundingBox(objectIndex: number): Object3DBounds;
}
