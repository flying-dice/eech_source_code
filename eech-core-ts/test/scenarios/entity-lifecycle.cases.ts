//
// Slice 3 behaviour matrix: the campaign entity lifecycle, exercised through
// the original cargo create / destroy path.
//
// Every case runs three ways: JavaScript (test/unit/entity-lifecycle.test.ts),
// Lua 5.1 (test/lua/conformance.ts) and the executed original C
// (test/c-reference/entity-lifecycle.cref.test.ts), where the whole output of
// TypeScript must also equal the C's.
//
// `expected` lists lines derived by hand from the C source. They must appear
// in the output in this order (other lines may come between them). An
// expected line also matches an output line that continues it after a space.
// Entities that never received a label (creation ended early) print as an
// empty label.
//
// Common layout, unless a case says otherwise: heap 16; session 0, update 1,
// force0 (blue) 2, keysite0 3 (blue); map 2 x 2 sectors of 1024, so
// MAX_MAP_X/Z = 2047, MID_MAP_X/Z = 1023.5, MID_MAP_Y = 28767.5, and the
// sectors are sector0_0 4, sector1_0 5, sector0_1 6, sector1_1 7 (z-major).
// The first free index after the map is 8.
//
// Float bit patterns: 0 00000000, 5 40a00000, 512 44000000, 1023.5 447fe000,
// 1024 44800000, 1500 44bb8000, 1536 44c00000, 2047 44ffe000,
// 28767.5 46e0bf00.
//

import { EntitySide, EntitySubTypeKeysite, EntityType, IntType, ListType, Vec3dType } from "../../src/generated/c-enums";
import type { KeysiteSpec } from "./campaign-scenario";
import { crateAttributes, type LifecycleAttribute, type LifecycleOp, type LifecycleSpec } from "./lifecycle-scenario";

export interface LifecycleCase {
	id: string;
	// C provenance of the behaviour under test
	c: string;
	spec: LifecycleSpec;
	expected: string[];
}

const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
const RED = EntitySide.ENTITY_SIDE_RED_FORCE;

const CARGO = EntityType.ENTITY_TYPE_CARGO;

function keysite(side: EntitySide, x: number, z: number): KeysiteSpec {
	return { side, subType: EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, inUse: true, x, z, ammo: 100, fuel: 100 };
}

const MAP: LifecycleOp = { kind: "map", xSectors: 2, zSectors: 2, sideLength: 1024 };

function crate(label: string, x: number, z: number): LifecycleOp {
	return { kind: "create", label, type: CARGO, index: -1, attributes: crateAttributes("keysite0", BLUE, 2, x, 0, z) };
}

function create(label: string, attributes: LifecycleAttribute[]): LifecycleOp {
	return { kind: "create", label, type: CARGO, index: -1, attributes };
}

function destroy(label: string): LifecycleOp {
	return { kind: "destroy", label };
}

function base(ops: LifecycleOp[]): LifecycleSpec {
	return { heap: 16, forces: [BLUE], keysites: [keysite(BLUE, 1500, 1500)], ops: [MAP, ...ops] };
}

const SIDE = IntType.INT_TYPE_SIDE;
const SUB_TYPE = IntType.INT_TYPE_ENTITY_SUB_TYPE;
const ALIVE = IntType.INT_TYPE_ALIVE;
const POSITION = Vec3dType.VEC3D_TYPE_POSITION;

const SECTORS_EMPTY = ["sector sector0_0 4 0 0 -", "sector sector1_0 5 1 0 -", "sector sector0_1 6 0 1 -"];

function numbers(from: number, to: number): string {
	const parts: string[] = [];
	for (let i = from; i <= to; i++) {
		parts.push(`${i}`);
	}
	return parts.join(" ");
}

function manySectorLines(count: number): string[] {
	const lines: string[] = [];
	for (let x = 0; x < count; x++) {
		// struct SECTOR :: x_sector is an 8-bit field
		lines.push(`sector sector${x}_0 ${4 + x} ${x % 256} 0 -`);
	}
	return lines;
}

export const ENTITY_LIFECYCLE_CASES: LifecycleCase[] = [
	// ---------------------------------------------------------------- creation
	{
		id: "crate-created-through-the-real-construction-path",
		c: "cg_creat.c :: create_server -> create_local (heap, attributes, LIST_TYPE_CARGO, LIST_TYPE_SECTOR) -> create_remote (ENTITY_COMMS_CREATE)",
		spec: base([crate("c0", 1500, 1500)]),
		expected: [
			"transmit-create 4 8 parent 6 keysite0 int 191 1 int 53 2 vec3d 8 44bb8000 00000000 44bb8000 end",
			"created c0 8",
			"result ok",
			`heap free ${numbers(9, 15)}`,
			"heap used c0 sector1_1 sector0_1 sector1_0 sector0_0 keysite0 force0 update session",
			"cargo c0 8 1 2 1 44bb8000 00000000 44bb8000 keysite0 sector1_1",
			"keysite keysite0 c0",
			...SECTORS_EMPTY,
			"sector sector1_1 7 1 1 c0",
		],
	},
	{
		id: "crates-are-inserted-at-the-head-of-the-keysite-and-sector-lists",
		c: "cg_creat.c :: insert_local_entity_into_parents_child_list (..., NULL): pred NULL inserts at the head",
		spec: base([crate("c0", 1500, 1500), crate("c1", 1536, 1536)]),
		expected: ["created c0 8", "created c1 9", "keysite keysite0 c1 c0", "sector sector1_1 7 1 1 c1 c0"],
	},
	{
		id: "no-parent-attribute-means-no-cargo-list",
		c: "cg_creat.c :: if (raw->cargo_link.parent) insert (LIST_TYPE_CARGO); the sector insert is unconditional",
		spec: base([create("c0", [{ kind: "int", type: SIDE, value: BLUE }, { kind: "vec3d", type: POSITION, x: 512, y: 0, z: 1536 }])]),
		expected: ["transmit-create 4 8 int 191 1 vec3d 8 44000000 00000000 44c00000 end", "cargo c0 8 1 -1 1 44000000 00000000 44c00000 NULL sector0_1", "keysite keysite0 -", "sector sector0_1 6 0 1 c0"],
	},
	{
		id: "a-null-parent-attribute-means-no-cargo-list",
		c: "en_attrs.c :: ENTITY_ATTR_PARENT stores NULL; cg_creat.c skips the LIST_TYPE_CARGO insert",
		spec: base([create("c0", [{ kind: "parent", type: ListType.LIST_TYPE_CARGO, target: "NULL" }])]),
		expected: ["transmit-create 4 8 parent 6 NULL end", "cargo c0 8 3 -1 1 447fe000 46e0bf00 447fe000 NULL sector0_0", "keysite keysite0 -"],
	},
	{
		id: "absent-attributes-keep-the-working-defaults",
		c: "cg_creat.c :: sub_type = ENTITY_SUB_TYPE_UNINITIALISED, position = MID_MAP_X/Y/Z, alive = TRUE, side = ENTITY_SIDE_UNINITIALISED",
		spec: base([create("c0", [])]),
		expected: ["transmit-create 4 8 end", "created c0 8", "cargo c0 8 3 -1 1 447fe000 46e0bf00 447fe000 NULL sector0_0", "sector sector0_0 4 0 0 c0"],
	},
	{
		id: "side-is-a-two-bit-field",
		c: "mb_int.c :: raw->side = value, with side : NUM_SIDE_BITS (2); the transmitted attribute keeps the int",
		spec: base([create("c0", [{ kind: "int", type: SIDE, value: 5 }]), create("c1", [{ kind: "int", type: SIDE, value: -1 }])]),
		expected: ["transmit-create 4 8 int 191 5 end", "transmit-create 4 9 int 191 -1 end", "cargo c1 9 3 -1 1", "cargo c0 8 1 -1 1"],
	},
	{
		id: "alive-is-a-one-bit-field",
		c: "mb_int.c :: raw->alive = value, with alive : NUM_ALIVE_BITS (1); a dead cargo skips the sector vehicle check",
		spec: base([create("c0", [{ kind: "int", type: ALIVE, value: 2 }]), create("c1", [{ kind: "int", type: ALIVE, value: 3 }])]),
		expected: ["cargo c1 9 3 -1 1", "cargo c0 8 3 -1 0", "sector sector0_0 4 0 0 c1 c0"],
	},
	{
		id: "sub-type-is-a-full-int",
		c: "mb_int.c :: raw->sub_type = value (entity_sub_types, not a bit-field)",
		spec: base([create("c0", [{ kind: "int", type: SUB_TYPE, value: 100000 }])]),
		expected: ["cargo c0 8 3 100000 1"],
	},
	{
		id: "later-attributes-overwrite-earlier-ones",
		c: "en_attrs.c :: set_local_entity_attributes applies the list in order",
		spec: base([
			create("c0", [
				{ kind: "int", type: SIDE, value: 1 },
				{ kind: "vec3d", type: POSITION, x: 1500, y: 5, z: 1500 },
				{ kind: "int", type: SIDE, value: 2 },
				{ kind: "vec3d", type: POSITION, x: 512, y: 0, z: 512 },
			]),
		]),
		expected: ["cargo c0 8 2 -1 1 44000000 00000000 44000000 NULL sector0_0"],
	},
	{
		id: "a-parent-attribute-for-the-sector-list-is-overwritten",
		c: "en_attrs.c :: ENTITY_ATTR_PARENT only sets the link's parent; cg_creat.c then inserts into the sector under the position",
		spec: base([create("c0", [{ kind: "parent", type: ListType.LIST_TYPE_SECTOR, target: "sector0_0" }, { kind: "vec3d", type: POSITION, x: 1500, y: 0, z: 512 }])]),
		expected: ["transmit-create 4 8 parent 33 sector0_0 vec3d 8 44bb8000 00000000 44000000 end", "cargo c0 8 3 -1 1 44bb8000 00000000 44000000 NULL sector1_0", "sector sector0_0 4 0 0 -", "sector sector1_0 5 1 0 c0"],
	},
	{
		id: "a-child-pred-attribute-is-overwritten-by-the-head-insertion",
		c: "en_attrs.c :: ENTITY_ATTR_CHILD_PRED sets child_pred; insert_local_entity_into_parents_child_list (..., NULL) resets it",
		spec: base([crate("c0", 1500, 1500), create("c1", [...crateAttributes("keysite0", BLUE, 2, 1500, 0, 1500), { kind: "pred", type: ListType.LIST_TYPE_CARGO, target: "c0" }])]),
		expected: ["transmit-create 4 9 parent 6 keysite0 int 191 1 int 53 2 vec3d 8 44bb8000 00000000 44bb8000 pred 6 c0 end", "keysite keysite0 c1 c0", "sector sector1_1 7 1 1 c1 c0"],
	},

	// ------------------------------------------------------------ sector lookup
	{
		id: "sector-cells-truncate-positions",
		c: "en_world.h :: get_x_sector: convert_float_to_int (fistp, round toward zero) then / SECTOR_SIDE_LENGTH",
		spec: base([
			create("a", [{ kind: "vec3d", type: POSITION, x: 1023.75, y: 0, z: 0 }]),
			create("b", [{ kind: "vec3d", type: POSITION, x: 1024, y: 0, z: 1023.75 }]),
			create("c", [{ kind: "vec3d", type: POSITION, x: 0, y: 0, z: 1024 }]),
		]),
		expected: ["sector sector0_0 4 0 0 a", "sector sector1_0 5 1 0 b", "sector sector0_1 6 0 1 c"],
	},
	{
		id: "the-map-edges-are-inside",
		c: "en_world.h :: point_inside_map_area: MIN <= x <= MAX (MAX_MAP_X = x_sectors * side - 1.0)",
		spec: base([create("a", [{ kind: "vec3d", type: POSITION, x: 2047, y: 0, z: 2047 }]), create("b", [{ kind: "vec3d", type: POSITION, x: 0, y: -100, z: 0 }])]),
		expected: ["result ok", "sector sector0_0 4 0 0 b", "sector sector1_1 7 1 1 a"],
	},
	{
		id: "a-position-past-the-map-edge-is-fatal",
		c: "sector.c :: get_local_sector_entity: debug_fatal (\"Position off map...\"), after the cargo list insert",
		spec: base([crate("c0", 2047.5, 1500)]),
		expected: ["result fatal Position off map: (x = %f, z = %f)", "heap used  sector1_1", "cargo  8 1 2 1 44fff000 00000000 44bb8000 keysite0 NULL", "keysite keysite0 "],
	},
	{
		id: "a-negative-position-is-off-the-map",
		c: "en_world.h :: point_inside_map_area (MIN_MAP_X = 0.0)",
		spec: base([create("c0", [{ kind: "vec3d", type: POSITION, x: 5, y: 0, z: -0.5 }])]),
		expected: ["result fatal Position off map: (x = %f, z = %f)"],
	},

	// ----------------------------------------------------------------- map setup
	{
		id: "sector-entities-are-created-z-major",
		c: "sc_seccreat.c :: create_local_sector_entities: for z, for x: create_local_entity (ENTITY_TYPE_SECTOR, INT X_SECTOR, INT Z_SECTOR)",
		spec: { heap: 16, forces: [BLUE], keysites: [keysite(BLUE, 1500, 1500)], ops: [{ kind: "map", xSectors: 3, zSectors: 2, sideLength: 512 }] },
		expected: [
			"result ok",
			`heap free ${numbers(10, 15)}`,
			"heap used sector2_1 sector1_1 sector0_1 sector2_0 sector1_0 sector0_0 keysite0 force0 update session",
			"sector sector0_0 4 0 0 -",
			"sector sector1_0 5 1 0 -",
			"sector sector2_0 6 2 0 -",
			"sector sector0_1 7 0 1 -",
			"sector sector2_1 9 2 1 -",
		],
	},
	{
		id: "the-map-extents-are-floats",
		c: "en_world.c :: set_entity_world_map_size: MAX = (float) (n * side) - 1.0, MID = MIN + (MAX - MIN) * 0.5",
		spec: { heap: 16, forces: [], keysites: [], ops: [{ kind: "map", xSectors: 3, zSectors: 1, sideLength: 512 }, create("c0", [])] },
		// MID_MAP_X = 1535 * 0.5 = 767.5 (0x443fe000), MID_MAP_Z = 511 * 0.5 = 255.5 (0x437f8000)
		// indices: session 0, update 1, sectors 2..4
		expected: ["cargo c0 5 3 -1 1 443fe000 46e0bf00 437f8000 NULL sector1_0"],
	},
	{
		id: "sector-coordinates-are-eight-bit-fields",
		c: "sector.h :: struct SECTOR :: x_sector : NUM_SECTOR_BITS (8)",
		spec: { heap: 270, forces: [BLUE], keysites: [keysite(BLUE, 0, 0)], ops: [{ kind: "map", xSectors: 257, zSectors: 1, sideLength: 1 }] },
		expected: ["result ok", ...manySectorLines(257)],
	},
	{
		id: "a-side-length-that-is-not-a-power-of-two-asserts",
		c: "en_world.c :: ASSERT (int_bit_count (sector_side_length) == 1)",
		spec: { heap: 16, forces: [BLUE], keysites: [], ops: [{ kind: "map", xSectors: 2, zSectors: 2, sideLength: 1000 }] },
		expected: ["result assert int_bit_count (sector_side_length) == 1", `heap free ${numbers(3, 15)}`],
	},
	{
		id: "a-map-without-sectors-asserts",
		c: "en_world.c :: ASSERT (num_map_x_sectors > 0), ASSERT (num_map_z_sectors > 0)",
		spec: { heap: 16, forces: [], keysites: [], ops: [{ kind: "map", xSectors: 2, zSectors: 0, sideLength: 1024 }] },
		expected: ["result assert num_map_z_sectors > 0"],
	},
	{
		id: "creating-the-map-twice-is-fatal",
		c: "sc_seccreat.c :: sectors of the old map still in use: debug_fatal (\"SC_CREAT: uninitialised sector entity\")",
		spec: base([MAP]),
		expected: ["result fatal SC_CREAT: uninitialised sector entity"],
	},
	{
		id: "running-out-of-heap-while-creating-sectors-is-fatal",
		c: "en_heap.c :: get_free_entity returns NULL; en_creat.c :: create_local_entity: debug_fatal (\"... Limit of %d reached\")",
		spec: { heap: 6, forces: [BLUE], keysites: [keysite(BLUE, 1500, 1500)], ops: [MAP] },
		// the two sectors created before the failure are never labelled
		expected: ["result fatal EN_CREATE: CREATE_LOCAL_ENTITY : unable to create entity %s. Limit of %d reached", "heap free", "heap used   keysite0 force0 update session"],
	},

	// --------------------------------------------------------- index validation
	{
		id: "a-server-create-with-an-index-asserts",
		c: "en_valid.c :: assert_local_create_entity_index: on the server the index must be ENTITY_INDEX_DONT_CARE",
		spec: base([{ kind: "create", label: "c0", type: CARGO, index: 9, attributes: [] }]),
		expected: ["result assert assert_local_create_entity_index ((index))", `heap free ${numbers(8, 15)}`],
	},
	{
		id: "an-entity-type-out-of-range-asserts",
		c: "en_creat.c :: ASSERT ((type > ENTITY_TYPE_UNKNOWN) && (type < NUM_ENTITY_TYPES))",
		spec: base([{ kind: "create", label: "c0", type: EntityType.ENTITY_TYPE_UNKNOWN, index: -1, attributes: [] }]),
		expected: ["result assert (type > ENTITY_TYPE_UNKNOWN) && (type < NUM_ENTITY_TYPES)"],
	},
	{
		id: "num-entity-types-is-out-of-range",
		c: "en_creat.c :: ASSERT ((type > ENTITY_TYPE_UNKNOWN) && (type < NUM_ENTITY_TYPES))",
		spec: base([{ kind: "create", label: "c0", type: EntityType.NUM_ENTITY_TYPES, index: -1, attributes: [] }]),
		expected: ["result assert (type > ENTITY_TYPE_UNKNOWN) && (type < NUM_ENTITY_TYPES)"],
	},

	// -------------------------------------------------------------- heap limits
	{
		id: "running-out-of-heap-while-creating-cargo-is-fatal",
		c: "cg_creat.c :: create_local returns NULL, create_server skips create_remote; en_creat.c: debug_fatal on the server",
		spec: { heap: 10, forces: [BLUE], keysites: [keysite(BLUE, 1500, 1500)], ops: [MAP, crate("c0", 1500, 1500), crate("c1", 1500, 1500), crate("c2", 1500, 1500)] },
		expected: ["created c0 8", "created c1 9", "result fatal EN_CREATE: CREATE_CLIENT_SERVER_ENTITY : unable to create entity %s. Limit of %d reached", "heap free", "keysite keysite0 c1 c0"],
	},

	// -------------------------------------------------------------- destruction
	{
		id: "destroying-the-only-crate-restores-the-graph",
		c: "cg_dstry.c :: destroy_server_family -> destroy_server: transmit ENTITY_COMMS_DESTROY first, then destroy_local unlinks and frees",
		spec: base([crate("c0", 1500, 1500), destroy("c0")]),
		expected: [
			"created c0 8",
			"transmit-destroy c0",
			"result ok",
			`heap free ${numbers(8, 15)}`,
			"heap used sector1_1 sector0_1 sector1_0 sector0_0 keysite0 force0 update session",
			"keysite keysite0 -",
			...SECTORS_EMPTY,
			"sector sector1_1 7 1 1 -",
		],
	},
	{
		id: "destroying-the-head-crate",
		c: "en_list.c :: delete_local_entity_from_parents_child_list (no pred: the parent's first child becomes the successor)",
		spec: base([crate("c0", 1500, 1500), crate("c1", 1500, 1500), crate("c2", 1500, 1500), destroy("c2")]),
		expected: ["transmit-destroy c2", "keysite keysite0 c1 c0", "sector sector1_1 7 1 1 c1 c0"],
	},
	{
		id: "destroying-a-middle-crate",
		c: "en_list.c :: delete_local_entity_from_parents_child_list (pred and succ relinked)",
		spec: base([crate("c0", 1500, 1500), crate("c1", 1500, 1500), crate("c2", 1500, 1500), destroy("c1")]),
		expected: ["transmit-destroy c1", "heap used c2 c0", "keysite keysite0 c2 c0", "sector sector1_1 7 1 1 c2 c0"],
	},
	{
		id: "destroying-the-tail-crate",
		c: "en_list.c :: delete_local_entity_from_parents_child_list (no succ)",
		spec: base([crate("c0", 1500, 1500), crate("c1", 1500, 1500), crate("c2", 1500, 1500), destroy("c0")]),
		expected: ["transmit-destroy c0", "keysite keysite0 c2 c1", "sector sector1_1 7 1 1 c2 c1"],
	},
	{
		id: "the-most-recently-freed-index-is-reused-first",
		c: "en_heap.c :: set_free_entity pushes at the head of the free list; get_free_entity takes the head",
		spec: base([crate("c0", 1500, 1500), crate("c1", 1500, 1500), crate("c2", 1500, 1500), destroy("c0"), destroy("c2"), crate("c3", 512, 512)]),
		expected: ["created c3 10", "result ok", `heap free 8 ${numbers(11, 15)}`, "heap used c3 c1 sector1_1"],
	},
	{
		id: "destroying-a-freed-entity-does-nothing",
		c: "en_dstry.c :: default_destroy_entity_family (a freed entity has ENTITY_TYPE_UNKNOWN; nothing overloads it)",
		spec: base([crate("c0", 1500, 1500), destroy("c0"), destroy("c0")]),
		expected: ["transmit-destroy c0", "result ok", `heap free ${numbers(8, 15)}`],
	},
	{
		id: "crates-of-two-keysites-in-one-sector",
		c: "cg_dstry.c :: destroy_local deletes from LIST_TYPE_CARGO and LIST_TYPE_SECTOR independently",
		spec: {
			heap: 16,
			forces: [BLUE, RED],
			keysites: [keysite(BLUE, 1500, 1500), keysite(RED, 1600, 1600)],
			ops: [
				{ kind: "map", xSectors: 2, zSectors: 2, sideLength: 1024 },
				crate("b0", 1500, 1500),
				{ kind: "create", label: "r0", type: CARGO, index: -1, attributes: crateAttributes("keysite1", RED, 2, 1600, 0, 1600) },
				crate("b1", 1500, 1500),
				destroy("b0"),
			],
		},
		// indices: session 0, update 1, force0 2, force1 3, keysite0 4, keysite1 5, sectors 6..9
		expected: ["created b0 10", "created r0 11", "created b1 12", "keysite keysite0 b1", "keysite keysite1 r0", "sector sector1_1 9 1 1 b1 r0"],
	},
];
