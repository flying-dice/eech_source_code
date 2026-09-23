//
// GENERATED FILE - DO NOT EDIT.
// Produced by scripts/gen-c-sources.mjs from the original EECH C sources.
// Regenerate with `npm run gen:c`; `npm run check:c` detects drift.
//

// C provenance: aphavoc/source/ai/taskgen/croute.c :: route_biasing_database [] (static; indexed by MovementType)
// C provenance: aphavoc/source/ai/taskgen/croute.c :: route_biasing_database [].elevation_bias (float)
export const ROUTE_BIASING_ELEVATION_BIAS: readonly number[] = [
	1, // 0 MOVEMENT_TYPE_NONE
	5, // 1 MOVEMENT_TYPE_AIR
	1, // 2 MOVEMENT_TYPE_GROUND
	9999, // 3 MOVEMENT_TYPE_SEA
	1, // 4 MOVEMENT_TYPE_ALL
];

// C provenance: aphavoc/source/ai/taskgen/croute.c :: route_biasing_database [].range_bias (float)
export const ROUTE_BIASING_RANGE_BIAS: readonly number[] = [
	1, // 0 MOVEMENT_TYPE_NONE
	0.5, // 1 MOVEMENT_TYPE_AIR
	1, // 2 MOVEMENT_TYPE_GROUND
	0.10000000149011612, // 3 MOVEMENT_TYPE_SEA
	1, // 4 MOVEMENT_TYPE_ALL
];

// C provenance: aphavoc/source/ai/taskgen/croute.c :: route_biasing_database [].side_bias (float)
export const ROUTE_BIASING_SIDE_BIAS: readonly number[] = [
	1, // 0 MOVEMENT_TYPE_NONE
	1, // 1 MOVEMENT_TYPE_AIR
	1, // 2 MOVEMENT_TYPE_GROUND
	1, // 3 MOVEMENT_TYPE_SEA
	1, // 4 MOVEMENT_TYPE_ALL
];

// C provenance: aphavoc/source/ai/taskgen/croute.c :: route_biasing_database [].min_route_range (float)
export const ROUTE_BIASING_MIN_ROUTE_RANGE: readonly number[] = [
	5000, // 0 MOVEMENT_TYPE_NONE
	5000, // 1 MOVEMENT_TYPE_AIR
	5000, // 2 MOVEMENT_TYPE_GROUND
	5000, // 3 MOVEMENT_TYPE_SEA
	5000, // 4 MOVEMENT_TYPE_ALL
];

// C provenance: aphavoc/source/ai/taskgen/croute.c :: route_biasing_database [].route_deviation_size (float)
export const ROUTE_BIASING_ROUTE_DEVIATION_SIZE: readonly number[] = [
	3, // 0 MOVEMENT_TYPE_NONE
	3, // 1 MOVEMENT_TYPE_AIR
	3, // 2 MOVEMENT_TYPE_GROUND
	2, // 3 MOVEMENT_TYPE_SEA
	3, // 4 MOVEMENT_TYPE_ALL
];

// C provenance: aphavoc/source/ai/taskgen/croute.c :: route_biasing_database [].num_route_samples (float)
export const ROUTE_BIASING_NUM_ROUTE_SAMPLES: readonly number[] = [
	8, // 0 MOVEMENT_TYPE_NONE
	8, // 1 MOVEMENT_TYPE_AIR
	8, // 2 MOVEMENT_TYPE_GROUND
	6, // 3 MOVEMENT_TYPE_SEA
	8, // 4 MOVEMENT_TYPE_ALL
];

// C provenance: aphavoc/source/ai/taskgen/croute.c :: route_biasing_database [].optimise_tolerance (float)
export const ROUTE_BIASING_OPTIMISE_TOLERANCE: readonly number[] = [
	0.9399999976158142, // 0 MOVEMENT_TYPE_NONE
	0.9399999976158142, // 1 MOVEMENT_TYPE_AIR
	0.9399999976158142, // 2 MOVEMENT_TYPE_GROUND
	0.9399999976158142, // 3 MOVEMENT_TYPE_SEA
	0.9399999976158142, // 4 MOVEMENT_TYPE_ALL
];
