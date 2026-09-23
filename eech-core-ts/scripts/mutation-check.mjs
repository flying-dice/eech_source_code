#!/usr/bin/env node
//
// Negative controls: each mutant is a meaningful behavioural regression of the
// ported campaign code. The conformance suites must fail ("kill") every one.
// A surviving mutant means the tests cannot see that behaviour.
//
// Each mutant runs in a throwaway copy of the project; the working tree is
// never modified.
//

import { spawnSync } from "node:child_process";
import { cpSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

// suite "js": vitest (JavaScript semantics); suite "lua": TSTL + Lua 5.1 conformance runner;
// suite "cref": the C reference floating-point checks (rebuilds the harness)
const MUTANTS = [
	{
		name: "group requests supplies at exactly 100 (< becomes <=)",
		file: "src/entity/special/group/group.ts",
		from: "if (raw.supplies.ammo_supply_level < 100.0) {\n\t\t\tconst force",
		to: "if (raw.supplies.ammo_supply_level <= 100.0) {\n\t\t\tconst force",
		suite: "js",
	},
	{
		name: "group requests ammo and fuel in one assessment (else if becomes if)",
		file: "src/entity/special/group/group.ts",
		from: "} else if (raw.supplies.fuel_supply_level < 100.0) {",
		to: "}\n\t\tif (raw.supplies.fuel_supply_level < 100.0) {",
		suite: "js",
	},
	{
		name: "busy groups are resupplied (GROUP_MODE_IDLE test inverted)",
		file: "src/entity/special/group/group.ts",
		from: "=== GroupModeType.GROUP_MODE_IDLE) {",
		to: "!== GroupModeType.GROUP_MODE_IDLE) {",
		suite: "js",
	},
	{
		name: "rearming ignores keysite stock (bound upper limit 100)",
		file: "src/entity/special/group/group.ts",
		from: "required = toFloat32RTZ(bound(required, 0.0, level));\n\n\t\t\t\tlevel = f32Sub(level, required);\n\n\t\t\t\tsetClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL",
		to: "required = toFloat32RTZ(bound(required, 0.0, 100.0));\n\n\t\t\t\tlevel = f32Sub(level, required);\n\n\t\t\t\tsetClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL",
		suite: "js",
	},
	{
		name: "a keysite parent is ignored (always search the closest keysite)",
		file: "src/entity/special/group/group.ts",
		from: "if (!keysite || getLocalEntityType(keysite) !== EntityType.ENTITY_TYPE_KEYSITE) {\n\t\t\t\t\tkeysite = getClosestKeysite(\n\t\t\t\t\t\tEntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES,\n\t\t\t\t\t\traw.side,\n\t\t\t\t\t\tgetLocalEntityVec3dPtr(en, Vec3dType.VEC3D_TYPE_POSITION),\n\t\t\t\t\t\t1.0 * KILOMETRE,\n\t\t\t\t\t\tundefined,\n\t\t\t\t\t\ttrue,\n\t\t\t\t\t\tundefined,\n\t\t\t\t\t);\n\t\t\t\t}\n\n\t\t\t\tASSERT(keysite !== undefined, \"keysite\");\n\n\t\t\t\tlet level = getLocalEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL);",
		to: "{\n\t\t\t\t\tkeysite = getClosestKeysite(\n\t\t\t\t\t\tEntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES,\n\t\t\t\t\t\traw.side,\n\t\t\t\t\t\tgetLocalEntityVec3dPtr(en, Vec3dType.VEC3D_TYPE_POSITION),\n\t\t\t\t\t\t1.0 * KILOMETRE,\n\t\t\t\t\t\tundefined,\n\t\t\t\t\t\ttrue,\n\t\t\t\t\t\tundefined,\n\t\t\t\t\t);\n\t\t\t\t}\n\n\t\t\t\tASSERT(keysite !== undefined, \"keysite\");\n\n\t\t\t\tlet level = getLocalEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL);",
		suite: "js",
	},
	{
		name: "closest keysite has no early out (nearest wins instead of first within 1 km)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (range <= min_range) {",
		to: "if (range <= min_range && false) {",
		suite: "js",
	},
	{
		name: "closest keysite ties keep the last (< becomes <=)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (range < best_range && outside_of_range) {",
		to: "if (range <= best_range && outside_of_range) {",
		suite: "js",
	},
	{
		name: "closest keysite reports the approximate range for the closest keysite",
		file: "src/entity/special/keysite/keysite.ts",
		from: "best_range = get2dRange(keysite_pos, pos);",
		to: "best_range = getApprox2dRange(keysite_pos, pos);",
		suite: "js",
	},
	{
		name: "closest keysite ignores exclude_keysite",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (current_keysite !== exclude_keysite) {",
		to: "if (current_keysite !== exclude_keysite || true) {",
		suite: "js",
	},
	{
		name: "supply requests go to the first force regardless of side",
		file: "src/entity/special/force/force.ts",
		from: "if (getLocalEntityIntValue(force, IntType.INT_TYPE_SIDE) === side) {",
		to: "if (getLocalEntityIntValue(force, IntType.INT_TYPE_SIDE) === side || true) {",
		suite: "js",
	},
	{
		name: "C float narrowing of set_client_server_entity_float_value is lost",
		file: "src/entity/system/en_values.ts",
		from: "fnSetClientServerEntityFloatValue[getCommsModel()].lookup(en.type, type, FloatType[type])(en, type, toFloat32RTZ(value));",
		to: "fnSetClientServerEntityFloatValue[getCommsModel()].lookup(en.type, type, FloatType[type])(en, type, value);",
		suite: "js",
	},
	{
		name: "approximate range uses the smaller axis as the major term",
		file: "src/core/maths/range.ts",
		from: "if (dx > dz) {",
		to: "if (dx < dz) {",
		suite: "js",
	},
	{
		name: "in-use test relies on JavaScript truthiness (0 is true in Lua)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (getLocalEntityIntValue(current_keysite, IntType.INT_TYPE_IN_USE) !== 0) {",
		// hidden from TypeScript so only the Lua semantics can catch it
		to: "if ((getLocalEntityIntValue(current_keysite, IntType.INT_TYPE_IN_USE) as unknown as boolean)) {",
		suite: "lua",
	},
	{
		name: "update passes use the whole frame delta (sub-step never set)",
		file: "src/entity/special/update/update.ts",
		from: "\tsetManualDeltaTime(entity_update_delta_time);\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "frame delta is not restored after the update passes",
		file: "src/entity/special/update/update.ts",
		from: "\tif (!isFrameRateLocked()) {\n\t\tsetManualDeltaTime(delta_time);\n\t}",
		to: "",
		suite: "js",
	},
	{
		name: "sub-step count rounds instead of truncating",
		file: "src/entity/special/update/update.ts",
		from: "const entity_update_iterations = toCInt(f32Mul(getDeltaTime(), frame_rate) + 1.0);",
		to: "const entity_update_iterations = Math.round(f32Mul(getDeltaTime(), frame_rate) + 1.0);",
		suite: "js",
	},
	{
		name: "a locked frame rate is still subdivided",
		file: "src/entity/special/update/update.ts",
		from: "\tlet delta_time = 0;\n\n\tif (!isFrameRateLocked()) {",
		to: "\tlet delta_time = 0;\n\n\tif (true) {",
		suite: "js",
	},
	{
		name: "the update walk reads the successor after the update (self-removal breaks the walk)",
		file: "src/entity/special/update/update.ts",
		from: "\t\t\tsetUpdateSucc(getLocalEntityChildSucc(en, ListType.LIST_TYPE_UPDATE));\n\n\t\t\tupdateClientServerEntity(en);",
		to: "\t\t\tupdateClientServerEntity(en);\n\n\t\t\tsetUpdateSucc(getLocalEntityChildSucc(en, ListType.LIST_TYPE_UPDATE));",
		suite: "js",
	},
	{
		name: "a group leaves the update list when either timer expires",
		file: "src/entity/special/group/group.ts",
		from: "if (raw.sleep === 0.0 && raw.assist_timer === 0.0) {",
		to: "if (raw.sleep === 0.0 || raw.assist_timer === 0.0) {",
		suite: "js",
	},
	{
		name: "an overshooting sleep timer is not clamped to zero",
		file: "src/entity/special/group/group.ts",
		from: "\t\traw.sleep = max(raw.sleep, 0.0);\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "timer setters only insert positive values into the update list",
		file: "src/entity/special/group/group.ts",
		from: "if (value !== 0.0 && !getLocalEntityParent(en, ListType.LIST_TYPE_UPDATE)) {",
		to: "if (value > 0.0 && !getLocalEntityParent(en, ListType.LIST_TYPE_UPDATE)) {",
		suite: "js",
	},
	{
		name: "the sleep and assist timer accessors are swapped",
		file: "src/entity/special/group/group.ts",
		from: "fnGetLocalEntityFloatValue.overload(GROUP, FloatType.FLOAT_TYPE_SLEEP, (en) => getLocalEntityData<GroupRaw>(en).sleep);",
		to: "fnGetLocalEntityFloatValue.overload(GROUP, FloatType.FLOAT_TYPE_SLEEP, (en) => getLocalEntityData<GroupRaw>(en).assist_timer);",
		suite: "js",
	},
	{
		name: "timer setters test the value by JavaScript truthiness (0 is true in Lua)",
		file: "src/entity/special/group/group.ts",
		from: "if (value !== 0.0 && !getLocalEntityParent(en, ListType.LIST_TYPE_UPDATE)) {",
		// hidden from TypeScript so only the Lua semantics can catch it
		to: "if ((value as unknown as boolean) && !getLocalEntityParent(en, ListType.LIST_TYPE_UPDATE)) {",
		suite: "lua",
	},
	// ---- issue #7: round-toward-zero float arithmetic (docs/fidelity/fpu-semantics.md)
	{
		name: "float narrowing rounds to nearest instead of toward zero",
		file: "src/core/float32.ts",
		from: "return sign * Math.floor(magnitude / quantum) * quantum;\n}\n\n// the float next to f",
		to: "return sign * Math.round(magnitude / quantum) * quantum;\n}\n\n// the float next to f",
		suite: "js",
	},
	{
		name: "float narrowing rounds to nearest instead of toward zero (Lua)",
		file: "src/core/float32.ts",
		from: "return sign * Math.floor(magnitude / quantum) * quantum;\n}\n\n// the float next to f",
		to: "return sign * Math.round(magnitude / quantum) * quantum;\n}\n\n// the float next to f",
		suite: "lua",
	},
	{
		name: "a sum that rounds onto a float in double is not stepped toward zero",
		file: "src/core/float32.ts",
		from: "if (f === r && f !== 0 && ((error < 0 && f > 0) || (error > 0 && f < 0))) {",
		to: "if (false) {",
		suite: "js",
	},
	{
		name: "a sum that rounds onto a float in double is not stepped toward zero (Lua)",
		file: "src/core/float32.ts",
		from: "if (f === r && f !== 0 && ((error < 0 && f > 0) || (error > 0 && f < 0))) {",
		to: "if (false) {",
		suite: "lua",
	},
	{
		name: "the step below a power of two uses the full spacing",
		file: "src/core/float32.ts",
		from: "\t\tquantum = quantum / 2;\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "the sum error term has the wrong sign",
		file: "src/core/float32.ts",
		from: "return a - (s - bVirtual) + (b - bVirtual);",
		to: "return -(a - (s - bVirtual) + (b - bVirtual));",
		suite: "js",
	},
	{
		name: "float multiplication rounds to nearest",
		file: "src/core/float32.ts",
		from: "return toFloat32RTZ(a * b);",
		to: "return toFloat32(a * b);",
		suite: "js",
	},
	{
		name: "float division rounds to nearest",
		file: "src/core/float32.ts",
		from: "return toFloat32RTZ(a / b);",
		to: "return toFloat32(a / b);",
		suite: "js",
	},
	{
		name: "square root rounds to nearest",
		file: "src/core/float32.ts",
		from: "return toFloat32RTZ(Math.sqrt(x));",
		to: "return toFloat32(Math.sqrt(x));",
		suite: "js",
	},
	{
		name: "the compile-time delta initialiser is rounded toward zero",
		file: "src/core/time.ts",
		from: "export function resetDeltaTime(): void {\n\tsystem_delta_time = toFloat32(0.1);",
		to: "export function resetDeltaTime(): void {\n\tsystem_delta_time = toFloat32RTZ(0.1);",
		suite: "js",
	},
	{
		name: "timer subtraction is not narrowed to float",
		file: "src/entity/special/group/group.ts",
		from: "raw.sleep = f32Sub(raw.sleep, getDeltaTime());",
		to: "raw.sleep = raw.sleep - getDeltaTime();",
		suite: "js",
	},
	{
		name: "a C-reference environment that still rounds to nearest",
		file: "c-reference/harness.c",
		from: "#define HARNESS_MXCSR_RC 0x6000		/* round toward zero */",
		to: "#define HARNESS_MXCSR_RC 0x0000		/* round toward zero */",
		suite: "cref",
	},
	// ---- slice 3: entity lifecycle, cargo, sectors, heap
	{
		name: "crates are not linked into their keysite's cargo list",
		file: "src/entity/mobile/cargo/cargo.ts",
		from: "if (parent) {\n\t\t\tinsertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_CARGO, parent, undefined);",
		to: "if (parent && false) {\n\t\t\tinsertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_CARGO, parent, undefined);",
		suite: "js",
	},
	{
		name: "crates never join the sector list under their position",
		file: "src/entity/mobile/cargo/cargo.ts",
		from: "insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_SECTOR, getLocalSectorEntity(raw.mob.position), undefined);",
		to: "getLocalSectorEntity(raw.mob.position);",
		suite: "js",
	},
	{
		name: "destroyed crates stay in their sector list (orphaned children)",
		file: "src/entity/mobile/cargo/cargo.ts",
		from: "\tdeleteLocalEntityFromParentsChildList(en, ListType.LIST_TYPE_SECTOR);\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "destroyed crates stay in their keysite's cargo list",
		file: "src/entity/mobile/cargo/cargo.ts",
		from: "\tdeleteLocalEntityFromParentsChildList(en, ListType.LIST_TYPE_CARGO);\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "the server does not replicate a creation (create_remote skipped)",
		file: "src/entity/mobile/cargo/cargo.ts",
		from: "\t\tcreateRemote(type, en.index, attributes);\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "a cargo's side defaults to neutral instead of ENTITY_SIDE_UNINITIALISED",
		file: "src/entity/mobile/cargo/cargo.ts",
		from: "side: ENTITY_SIDE_UNINITIALISED,",
		to: "side: 0,",
		suite: "js",
	},
	{
		name: "a mobile's side is stored as a full int (no 2-bit field)",
		file: "src/entity/mobile/mobile.ts",
		from: "mob(en).side = storeUnsignedBitfield(value, NUM_SIDE_BITS);",
		to: "mob(en).side = value;",
		suite: "js",
	},
	{
		name: "a sector's x_sector is stored as a full int (no 8-bit field)",
		file: "src/entity/special/sector/sector.ts",
		from: "getLocalEntityData<SectorRaw>(en).x_sector = storeUnsignedBitfield(value, NUM_SECTOR_BITS);",
		to: "getLocalEntityData<SectorRaw>(en).x_sector = value;",
		suite: "js",
	},
	{
		name: "freed entities are never reused (not pushed on the free list)",
		file: "src/entity/system/en_heap.ts",
		from: "\ten.pred = -1;\n\n\theap.firstFreeEntity = en.index;",
		to: "\ten.pred = -1;\n",
		suite: "js",
	},
	{
		name: "sector cells round positions to nearest instead of truncating",
		file: "src/entity/system/en_world.ts",
		from: "return cIntDivide(toCInt(x), world_map.sector_side_length);",
		to: "return cIntDivide(Math.round(x), world_map.sector_side_length);",
		suite: "js",
	},
	{
		name: "the map's far edge is outside the map (<= becomes <)",
		file: "src/entity/system/en_world.ts",
		from: "pos.x <= world_map.max_map_x",
		to: "pos.x < world_map.max_map_x",
		suite: "js",
	},
	{
		name: "MAX_MAP_X lacks EECH's - 1.0",
		file: "src/entity/system/en_world.ts",
		from: "world_map.max_map_x = f32Add(toFloat32RTZ(num_map_x_sectors * sector_side_length), -1.0);",
		to: "world_map.max_map_x = toFloat32RTZ(num_map_x_sectors * sector_side_length);",
		suite: "js",
	},
	{
		name: "list setters accept self-links (the set_pred.h ASSERT is dropped)",
		file: "src/entity/system/en_list.ts",
		from: "\tASSERT(en !== child_pred, \"en != child_pred\");\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "sectors are recreated over sectors still in use (SC_CREAT check dropped)",
		file: "src/entity/special/sector/sector.ts",
		from: "throw new EechFatalError(\"SC_CREAT: uninitialised sector entity\");",
		to: "break;",
		suite: "js",
	},
	{
		name: "allocating an index in use is not refused (get_free_entity check dropped)",
		file: "src/entity/system/en_heap.ts",
		from: "if (en.type !== EntityType.ENTITY_TYPE_UNKNOWN) {",
		to: "if (false) {",
		suite: "js",
	},
	{
		name: "allocating a middle free entry does not relink its predecessor",
		file: "src/entity/system/en_heap.ts",
		from: "\tif (pred) {\n\t\tpred.succ = en.succ;\n\t} else {\n\t\theap.firstFreeEntity = en.succ;\n\t}",
		to: "\tif (!pred) {\n\t\theap.firstFreeEntity = en.succ;\n\t}",
		suite: "js",
	},
	{
		name: "allocating by index leaves the entry off the used list",
		file: "src/entity/system/en_heap.ts",
		from: "\ten.pred = -1;\n\n\theap.firstUsedEntity = en.index;\n\n\treturn en;",
		to: "\ten.pred = -1;\n\n\treturn en;",
		suite: "js",
	},
	// ---- slice 4: keysite.c :: update_keysite_cargo
	{
		name: "update_keysite_cargo runs while the game is initialising",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (getGameStatus() === GameStatusType.GAME_STATUS_INITIALISING) {",
		to: "if (getGameStatus() === GameStatusType.GAME_STATUS_INITIALISING && false) {",
		suite: "js",
	},
	{
		name: "a dead keysite still keeps crates",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (raw.alive === 0 || raw.in_use === 0) {",
		to: "if (raw.in_use === 0) {",
		suite: "js",
	},
	{
		name: "a keysite not in use still keeps crates",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (raw.alive === 0 || raw.in_use === 0) {",
		to: "if (raw.alive === 0) {",
		suite: "js",
	},
	{
		name: "the alive test relies on JavaScript truthiness (0 is true in Lua)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (raw.alive === 0 || raw.in_use === 0) {",
		// hidden from TypeScript so only the Lua semantics can catch it
		to: "if (!(raw.alive as unknown as boolean) || raw.in_use === 0) {",
		suite: "lua",
	},
	{
		name: "an exact remainder of 0 destroys the crate (< becomes <=)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (temp_cargo_level < 0.0) {",
		to: "if (temp_cargo_level <= 0.0) {",
		suite: "js",
	},
	{
		name: "creation also fills an exact multiple (> becomes >=)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "\t\twhile (temp_cargo_level > cargo_size) {",
		to: "\t\twhile (temp_cargo_level >= cargo_size) {",
		suite: "js",
	},
	{
		name: "a destroyed crate still advances the row",
		file: "src/entity/special/keysite/keysite.ts",
		from: "\t\t\t\tdestroyClientServerEntityFamily(destroy_cargo);\n\n\t\t\t\tcontinue;",
		to: "\t\t\t\tdestroyClientServerEntityFamily(destroy_cargo);\n\n\t\t\t\tposition.x = advanceCrateRow(position.x, xmin, xmax);\n\n\t\t\t\tcontinue;",
		suite: "js",
	},
	{
		name: "crates of the other supply are counted",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (getLocalEntityIntValue(cargo, IntType.INT_TYPE_ENTITY_SUB_TYPE) === sub_type) {",
		to: "if (getLocalEntityIntValue(cargo, IntType.INT_TYPE_ENTITY_SUB_TYPE) >= 0) {",
		suite: "js",
	},
	{
		name: "a call that creates crates also notifies (else-if becomes if)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "\t} else if (cargo_level <= KEYSITE_SUPPLY_REQUEST_THRESHOLD) {",
		to: "\t}\n\tif (cargo_level <= KEYSITE_SUPPLY_REQUEST_THRESHOLD) {",
		suite: "js",
	},
	{
		name: "the threshold is tested on the remainder, not the level",
		file: "src/entity/special/keysite/keysite.ts",
		from: "} else if (cargo_level <= KEYSITE_SUPPLY_REQUEST_THRESHOLD) {",
		to: "} else if (temp_cargo_level <= KEYSITE_SUPPLY_REQUEST_THRESHOLD) {",
		suite: "js",
	},
	{
		name: "the threshold is exclusive (<= becomes <)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "} else if (cargo_level <= KEYSITE_SUPPLY_REQUEST_THRESHOLD) {",
		to: "} else if (cargo_level < KEYSITE_SUPPLY_REQUEST_THRESHOLD) {",
		suite: "js",
	},
	{
		name: "ammo notification reads the fuel usage column",
		file: "src/entity/special/keysite/keysite.ts",
		from: "if (KEYSITE_DATABASE_AMMO_SUPPLY_USAGE[raw.sub_type] < 0.0) {",
		to: "if (KEYSITE_DATABASE_FUEL_SUPPLY_USAGE[raw.sub_type] < 0.0) {",
		suite: "js",
	},
	{
		name: "fuel crates stand in the ammo row",
		file: "src/entity/special/keysite/keysite.ts",
		from: "position.z = f32Add(position.z, f32Mul(sub_type, f32Add(f32Sub(zmax, zmin), 1)));",
		to: "position.z = f32Add(position.z, f32Mul(0, f32Add(f32Sub(zmax, zmin), 1)));",
		suite: "js",
	},
	{
		name: "crates are not lifted by the bounding box floor",
		file: "src/entity/special/keysite/keysite.ts",
		from: "\tposition.y = f32Sub(position.y, ymin);\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "crates are spaced without the 1 m gap",
		file: "src/entity/special/keysite/keysite.ts",
		from: "return f32Add(x, f64AddRTZ(f32Sub(xmax, xmin), 1.0));",
		to: "return f32Add(x, f64AddRTZ(f32Sub(xmax, xmin), 0.0));",
		suite: "js",
	},
	{
		name: "the crate-row + 1.0 rounds to nearest in double",
		file: "src/entity/special/keysite/keysite.ts",
		from: "return f32Add(x, f64AddRTZ(f32Sub(xmax, xmin), 1.0));",
		to: "return f32Add(x, f32Sub(xmax, xmin) + 1.0);",
		suite: "js",
	},
	{
		name: "the crate-row + 1.0 is a float sum instead of a double one",
		file: "src/entity/special/keysite/keysite.ts",
		from: "return f32Add(x, f64AddRTZ(f32Sub(xmax, xmin), 1.0));",
		to: "return f32Add(x, f32Add(f32Sub(xmax, xmin), 1.0));",
		suite: "js",
	},
	{
		name: "the crate-row width is not rounded to float (evaluated in double)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "return f32Add(x, f64AddRTZ(f32Sub(xmax, xmin), 1.0));",
		to: "return f32Add(x, f64AddRTZ(xmax - xmin, 1.0));",
		suite: "js",
	},
	{
		name: "the crate-row + 1.0 rounds to nearest in double (Lua)",
		file: "src/entity/special/keysite/keysite.ts",
		from: "return f32Add(x, f64AddRTZ(f32Sub(xmax, xmin), 1.0));",
		to: "return f32Add(x, f32Sub(xmax, xmin) + 1.0);",
		suite: "lua",
	},
	{
		name: "the keysite side is not replicated with the crate",
		file: "src/entity/special/keysite/keysite.ts",
		from: "value: getLocalEntityIntValue(en, IntType.INT_TYPE_SIDE) },",
		to: "value: 0 },",
		suite: "js",
	},
	{
		name: "the game status survives a new campaign core",
		file: "src/index.ts",
		from: "\tresetGameStatus();\n",
		to: "\n",
		suite: "js",
	},
	{
		name: "a double sum that rounds up onto a double is not stepped toward zero",
		file: "src/core/float32.ts",
		from: "if (s !== 0 && ((error < 0 && s > 0) || (error > 0 && s < 0))) {\n\t\treturn nextDoubleTowardZero(s);",
		to: "if (false) {\n\t\treturn nextDoubleTowardZero(s);",
		suite: "js",
	},
	{
		name: "a double sum that rounds up onto a double is not stepped toward zero (Lua)",
		file: "src/core/float32.ts",
		from: "if (s !== 0 && ((error < 0 && s > 0) || (error > 0 && s < 0))) {\n\t\treturn nextDoubleTowardZero(s);",
		to: "if (false) {\n\t\treturn nextDoubleTowardZero(s);",
		suite: "lua",
	},
	{
		name: "Slice 5a: the response ignores the game status",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (getGameStatus() !== GameStatusType.GAME_STATUS_INITIALISED || getCommsModel() === CommsModelType.COMMS_MODEL_CLIENT) {",
		to: "if (getCommsModel() === CommsModelType.COMMS_MODEL_CLIENT) {",
		suite: "js",
	},
	{
		name: "Slice 5a: the response runs on a comms client",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (getGameStatus() !== GameStatusType.GAME_STATUS_INITIALISED || getCommsModel() === CommsModelType.COMMS_MODEL_CLIENT) {",
		to: "if (getGameStatus() !== GameStatusType.GAME_STATUS_INITIALISED) {",
		suite: "js",
	},
	{
		name: "Slice 5a: entity_is_object_of_task counts completed tasks",
		file: "src/entity/special/task/task.ts",
		from: "if (getLocalEntityIntValue(this_task, IntType.INT_TYPE_TASK_STATE) !== TaskStateType.TASK_STATE_COMPLETED) {",
		to: "if (getLocalEntityIntValue(this_task, IntType.INT_TYPE_TASK_STATE) !== -1) {",
		suite: "js",
	},
	{
		name: "Slice 5a: entity_is_object_of_task counts tasks of any side",
		file: "src/entity/special/task/task.ts",
		from: "if (getLocalEntityIntValue(this_task, IntType.INT_TYPE_SIDE) === side) {",
		to: "if (side === side) {",
		suite: "js",
	},
	{
		name: "Slice 5a: entity_is_object_of_task does not check the entity type",
		file: "src/entity/special/task/task.ts",
		from: "if (getLocalEntityType(this_task) === EntityType.ENTITY_TYPE_TASK) {",
		to: "if (this_task !== undefined) {",
		suite: "js",
	},
	{
		name: "Slice 5a: the duplicate walk skips completed tasks (asymmetry 'fixed')",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_TASK_USER_DATA) === sub_type) {",
		to: "if (getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_TASK_USER_DATA) === sub_type && getLocalEntityIntValue(task, IntType.INT_TYPE_TASK_STATE) !== 2) {",
		suite: "js",
	},
	{
		name: "Slice 5a: the duplicate walk checks the side (asymmetry 'fixed')",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_TASK_USER_DATA) === sub_type) {",
		to: "if (getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_TASK_USER_DATA) === sub_type && getLocalEntityIntValue(task, IntType.INT_TYPE_SIDE) === getLocalEntityIntValue(en, IntType.INT_TYPE_SIDE)) {",
		suite: "js",
	},
	{
		name: "Slice 5a: the duplicate walk skips waypoints (RECON quirk 'fixed')",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (getLocalEntityIntValue(task, IntType.INT_TYPE_ENTITY_SUB_TYPE) === EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY) {",
		to: "if (task.type === EntityType.ENTITY_TYPE_TASK && getLocalEntityIntValue(task, IntType.INT_TYPE_ENTITY_SUB_TYPE) === EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY) {",
		suite: "js",
	},
	{
		name: "Slice 5a: the duplicate walk does not check the task type",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (getLocalEntityIntValue(task, IntType.INT_TYPE_ENTITY_SUB_TYPE) === EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY) {",
		to: "if (task !== undefined) {",
		suite: "js",
	},
	{
		name: "Slice 5a: the duplicate decision is skipped",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "\t\t\t\t\treturn 0;\n\t\t\t\t}\n\t\t\t}\n\n\t\t\ttask = getLocalEntityChildSucc(task, ListType.LIST_TYPE_TASK_DEPENDENT);",
		to: "\t\t\t\t\tbreak;\n\t\t\t\t}\n\t\t\t}\n\n\t\t\ttask = getLocalEntityChildSucc(task, ListType.LIST_TYPE_TASK_DEPENDENT);",
		suite: "js",
	},
	{
		name: "Slice 5a: an airbase does not supply itself (F2 'fixed': the requester is excluded)",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "const airbase = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, SUPPLIER_EARLY_OUT_RANGE, airbase_actual_range, true, undefined);",
		to: "const airbase = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, SUPPLIER_EARLY_OUT_RANGE, airbase_actual_range, true, en);",
		suite: "js",
	},
	{
		name: "Slice 5a: an equidistant airbase replaces the factory (< becomes <=)",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (airbase_actual_range.value < factory_actual_range.value) {",
		to: "if (airbase_actual_range.value <= factory_actual_range.value) {",
		suite: "js",
	},
	{
		name: "Slice 5a: ammo prefers an oil refinery",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "factory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY, side, pos, SUPPLIER_EARLY_OUT_RANGE, factory_actual_range, true, undefined);\n\n\t\tif (!factory) {\n\t\t\tfactory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY,",
		to: "factory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY, side, pos, SUPPLIER_EARLY_OUT_RANGE, factory_actual_range, true, undefined);\n\n\t\tif (!factory) {\n\t\t\tfactory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY,",
		suite: "js",
	},
	{
		name: "Slice 5a: fuel prefers a factory",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "factory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY, side, pos, SUPPLIER_EARLY_OUT_RANGE, factory_actual_range, true, undefined);\n\n\t\tif (!factory) {\n\t\t\tfactory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY,",
		to: "factory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY, side, pos, SUPPLIER_EARLY_OUT_RANGE, factory_actual_range, true, undefined);\n\n\t\tif (!factory) {\n\t\t\tfactory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY,",
		suite: "js",
	},
	{
		name: "Slice 5a: no factory fallback for ammo",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "\t\tif (!factory) {\n\t\t\tfactory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY,",
		to: "\t\tif (false) {\n\t\t\tfactory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY,",
		suite: "js",
	},
	{
		name: "Slice 5a: the supplier search early-exits at 1 km instead of 10 km",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "const SUPPLIER_EARLY_OUT_RANGE = toFloat32(10 * KILOMETRE);",
		to: "const SUPPLIER_EARLY_OUT_RANGE = toFloat32(1 * KILOMETRE);",
		suite: "js",
	},
	{
		name: "Slice 5a: the cargo walk takes the head crate whatever its sub type",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (getLocalEntityIntValue(cargo, IntType.INT_TYPE_ENTITY_SUB_TYPE) === sub_type) {\n\t\t\t\tbreak;",
		to: "if (cargo !== undefined) {\n\t\t\t\tbreak;",
		suite: "js",
	},
	{
		name: "Slice 5a: a supply task is asked for without cargo",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "\t\tif (cargo) {\n\t\t\t//\n\t\t\t// create task",
		to: "\t\tif (true) {\n\t\t\t//\n\t\t\t// create task",
		suite: "js",
	},
	{
		name: "Slice 5a: the supply task's priority comes from another task_database row",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "TASK_DATABASE_TASK_PRIORITY[EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY]",
		to: "TASK_DATABASE_TASK_PRIORITY[EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_REPAIR]",
		suite: "js",
	},
	{
		name: "Slice 5a: the supply task moves by any means (MOVEMENT_TYPE_AIR replaced)",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "const movement_type = MovementType.MOVEMENT_TYPE_AIR;",
		to: "const movement_type = MovementType.MOVEMENT_TYPE_ALL;",
		suite: "js",
	},
	{
		name: "Slice 5a: a waypoint's task user data is 1 (the en_float.c default is 0.0)",
		file: "src/entity/special/waypoint/waypoint.ts",
		from: "fnGetLocalEntityFloatValue.overload(WAYPOINT, FloatType.FLOAT_TYPE_TASK_USER_DATA, defaultGetEntityFloatValue);",
		to: "fnGetLocalEntityFloatValue.overload(WAYPOINT, FloatType.FLOAT_TYPE_TASK_USER_DATA, () => 1);",
		suite: "js",
	},
	{
		name: "Slice 5a: the F2 self-supply is 'fixed' (Lua)",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "const airbase = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, SUPPLIER_EARLY_OUT_RANGE, airbase_actual_range, true, undefined);",
		to: "const airbase = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, SUPPLIER_EARLY_OUT_RANGE, airbase_actual_range, true, en);",
		suite: "lua",
	},
	{
		name: "Slice 5a: the duplicate walk skips completed tasks (Lua)",
		file: "src/entity/special/force/fc_msgs.ts",
		from: "if (getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_TASK_USER_DATA) === sub_type) {",
		to: "if (getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_TASK_USER_DATA) === sub_type && getLocalEntityIntValue(task, IntType.INT_TYPE_TASK_STATE) !== 2) {",
		suite: "lua",
	},
	{
		name: "Slice 5b: the create_supply_task observer skips construction",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\t\tobserver(requester, supplier, cargo, movement_type, priority, start_keysite, end_keysite);\n",
		to: "\t\tobserver(requester, supplier, cargo, movement_type, priority, start_keysite, end_keysite);\n\t\treturn undefined;\n",
		suite: "js",
	},
	{
		name: "Slice 5b: initialisation keeps a test's create_supply_task observer",
		file: "src/index.ts",
		from: "\tresetCreateSupplyTaskObserver();\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: the task id wraps by 4096",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\t\tid -= id_max;",
		to: "\t\tid -= id_max + 1;",
		suite: "js",
	},
	{
		name: "Slice 5b: the task id wraps at 4095 (> becomes >=)",
		file: "src/ai/taskgen/taskgen.ts",
		from: "while (id > id_max) {",
		to: "while (id >= id_max) {",
		suite: "js",
	},
	{
		name: "Slice 5b: the force's task counter is not advanced",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\tforce_raw.task_generation[sub_type].created++;\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: F1: the compatibility height is 1.0, not 0.0",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\t\ty: 0.0,\n\t\tz: f32Add(stop.z, -(direction.z * 4.0 * KILOMETRE)),",
		to: "\t\ty: 1.0,\n\t\tz: f32Add(stop.z, -(direction.z * 4.0 * KILOMETRE)),",
		suite: "js",
	},
	{
		name: "Slice 5b: route nodes are floored, not ceiled",
		file: "src/ai/taskgen/taskgen.ts",
		from: "{ x: Math.ceil(position.x), y: Math.ceil(position.y), z: Math.ceil(position.z) }",
		to: "{ x: Math.floor(position.x), y: Math.floor(position.y), z: Math.floor(position.z) }",
		suite: "js",
	},
	{
		name: "Slice 5b: the route length counts the terminator",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\t// decreased by 1 so as not to include the terminator.\n\troute_length--;\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: a stop timer of 0 is still set (> becomes >=)",
		file: "src/ai/taskgen/taskgen.ts",
		from: "if (stop_timer > 0.0) {",
		to: "if (stop_timer >= 0.0) {",
		suite: "js",
	},
	{
		name: "Slice 5b: a primary task is not put on its start keysite's unassigned list",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\tif (TASK_DATABASE_PRIMARY_TASK[sub_type] !== 0) {\n\t\tASSERT(start_keysite !== undefined, \"start_keysite\");\n\n\t\tsetClientServerEntityParent(new_task, ListType.LIST_TYPE_UNASSIGNED_TASK, start_keysite);\n\t}\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: the enemy force is told about tasks against its own side",
		file: "src/ai/taskgen/taskgen.ts",
		from: "if (getLocalEntityIntValue(task_objective, IntType.INT_TYPE_SIDE) !== side) {",
		to: "if (getLocalEntityIntValue(task_objective, IntType.INT_TYPE_SIDE) === side) {",
		suite: "js",
	},
	{
		name: "Slice 5b: the sector task list uses the last route node, not the objective",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\tif (task_objective) {\n\t\tposition = getLocalEntityVec3dPtr(task_objective, Vec3dType.VEC3D_TYPE_POSITION);\n\t}\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: F2: a task that would start at its requester is created",
		file: "src/ai/taskgen/taskgen.ts",
		from: " || start_ks.value === requester) {",
		to: ") {",
		suite: "js",
	},
	{
		name: "Slice 5b: the direction ignores the height difference",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\t\ty: f32Sub(stop.y, start.y),",
		to: "\t\ty: 0,",
		suite: "js",
	},
	{
		name: "Slice 5b: finish is not kept off the map perimeter",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\tboundPositionToAdjustedMapArea(finish);\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: the task's user data is not the cargo sub type",
		file: "src/ai/taskgen/taskgen.ts",
		from: "setLocalEntityFloatValue(new_task, FloatType.FLOAT_TYPE_TASK_USER_DATA, getLocalEntityIntValue(cargo, IntType.INT_TYPE_ENTITY_SUB_TYPE));",
		to: "setLocalEntityFloatValue(new_task, FloatType.FLOAT_TYPE_TASK_USER_DATA, 0);",
		suite: "js",
	},
	{
		name: "Slice 5b: ground tasks check air force capacity",
		file: "src/ai/taskgen/taskgen.ts",
		from: "start_keysite.value = findMostSuitableKeysiteForTask(sub_type, side, start_pos, false);",
		to: "start_keysite.value = findMostSuitableKeysiteForTask(sub_type, side, start_pos, true);",
		suite: "js",
	},
	{
		name: "Slice 5b: busy groups score as idle groups",
		file: "src/entity/special/task/task.ts",
		from: "score = f32Add(score, KEYSITE_TASK_BUSY_GROUP_COUNT_BIAS);",
		to: "score = f32Add(score, KEYSITE_TASK_IDLE_GROUP_COUNT_BIAS);",
		suite: "js",
	},
	{
		name: "Slice 5b: group scores are not capped at 12",
		file: "src/entity/special/task/task.ts",
		from: "\t\t\t\t\tscore = min(score, KEYSITE_TASK_MAX_GROUP_COUNT_BIAS);\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: start keysite ties keep the last (> becomes >=)",
		file: "src/entity/special/task/task.ts",
		from: "if (score > best_score) {",
		to: "if (score >= best_score) {",
		suite: "js",
	},
	{
		name: "Slice 5b: an unusable keysite is not halved",
		file: "src/entity/special/task/task.ts",
		from: "\t\t\t\t\t\t\tscore = f32Mul(score, 0.5);\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: unassigned tasks of the type do not count against a keysite",
		file: "src/entity/special/task/task.ts",
		from: "\t\t\t\t\t\t\t\ttask_count = f32Add(task_count, 1.0);\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: keysites 100 km away are in range",
		file: "src/entity/special/task/task.ts",
		from: "if (range < max_range) {",
		to: "if (range < max_range * 2) {",
		suite: "js",
	},
	{
		name: "Slice 5b: the range bias is squared, not to the fourth power",
		file: "src/entity/special/task/task.ts",
		from: "score = f32Mul(score, 2.0 * f32Mul(f32Mul(f32Mul(x, x), x), x));",
		to: "score = f32Mul(score, 2.0 * f32Mul(x, x));",
		suite: "js",
	},
	{
		name: "Slice 5b: any keysite's landing types suit",
		file: "src/entity/special/task/task.ts",
		from: "cBitAnd(getLocalEntityIntValue(keysite, IntType.INT_TYPE_LANDING_TYPES), task_landing_types) !== 0)",
		to: "true)",
		suite: "js",
	},
	{
		name: "Slice 5b: air force capacity is not checked",
		file: "src/entity/special/task/task.ts",
		from: "if (!check_capacity || KEYSITE_DATABASE_AIR_FORCE_CAPACITY[keysite_type] >= TASK_DATABASE_KEYSITE_AIR_FORCE_CAPACITY[task_type]) {",
		to: "if (true) {",
		suite: "js",
	},
	{
		name: "Slice 5b: dead groups score",
		file: "src/entity/special/task/task.ts",
		from: "if (getLocalEntityIntValue(group, IntType.INT_TYPE_ALIVE) !== 0) {",
		to: "if (true) {",
		suite: "js",
	},
	{
		name: "Slice 5b: unsuitable groups score",
		file: "src/entity/special/task/task.ts",
		from: "if (getGroupToTaskSuitability(group_type, task_type) > 0.0) {",
		to: "if (true) {",
		suite: "js",
	},
	{
		name: "Slice 5b: a keysite not in use is a candidate",
		file: "src/entity/special/task/task.ts",
		from: "getLocalEntityIntValue(keysite, IntType.INT_TYPE_IN_USE) !== 0 &&",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: difficulty starts from the first node even with a start keysite",
		file: "src/entity/special/task/task.ts",
		from: "\tif (start_keysite) {\n\t\t//\n\t\t// start at the assigned keysite",
		to: "\tif (start_keysite && false) {\n\t\t//\n\t\t// start at the assigned keysite",
		suite: "js",
	},
	{
		name: "Slice 5b: the last route node is not assessed",
		file: "src/entity/special/task/task.ts",
		from: "\tassessTaskSectorDifficulty(task_en, getXSector(last_pos.x), getZSector(last_pos.z), counts);\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: difficulty terms are not capped at 5",
		file: "src/entity/special/task/task.ts",
		from: "min(cIntDivide(air_threats, 2), 5) + min(cIntDivide(enemy_sectors, 2), 5)",
		to: "cIntDivide(air_threats, 2) + cIntDivide(enemy_sectors, 2)",
		suite: "js",
	},
	{
		name: "Slice 5b: enemy defence levels include neutral",
		file: "src/entity/special/sector/sector.ts",
		from: "for (let loop = EntitySide.ENTITY_SIDE_NEUTRAL + 1; loop < EntitySide.NUM_ENTITY_SIDES; loop++) {",
		to: "for (let loop = EntitySide.ENTITY_SIDE_NEUTRAL; loop < EntitySide.NUM_ENTITY_SIDES; loop++) {",
		suite: "js",
	},
	{
		name: "Slice 5b: a sector side tie is BLUE (> becomes >=)",
		file: "src/entity/special/sector/sector.ts",
		from: "raw.sector_side[EntitySide.ENTITY_SIDE_BLUE_FORCE] > raw.sector_side[EntitySide.ENTITY_SIDE_RED_FORCE]",
		to: "raw.sector_side[EntitySide.ENTITY_SIDE_BLUE_FORCE] >= raw.sector_side[EntitySide.ENTITY_SIDE_RED_FORCE]",
		suite: "js",
	},
	{
		name: "Slice 5b: single player transmits the task pointers",
		file: "src/entity/system/en_comms.ts",
		from: "export function transmitTaskPointers(task: Entity, route: TaskRoutePointers): void {\n\tif (!transmission) {\n\t\treturn;\n\t}\n",
		to: "export function transmitTaskPointers(task: Entity, route: TaskRoutePointers): void {\n",
		suite: "js",
	},
	{
		name: "Slice 5b: multiplayer packing does not check route nodes",
		file: "src/entity/system/en_comms.ts",
		from: "\tASSERT(pointInsideMapVolume(v) || v.y === -10000, \"point_inside_map_volume (v) || v->y == -10000\");\n",
		to: "",
		suite: "js",
	},
	{
		name: "Slice 5b: the campaign screen hears about free flight",
		file: "src/ui_menu/campaign/ca_msgs.ts",
		from: "if (getGameType() !== GameType.GAME_TYPE_CAMPAIGN && getGameType() !== GameType.GAME_TYPE_SKIRMISH) {",
		to: "if (getGameType() === GameType.GAME_TYPE_DEMO) {",
		suite: "js",
	},
	{
		name: "Slice 5b: non-primary tasks notify the campaign screen",
		file: "src/entity/special/task/task.ts",
		from: "\t\tif (TASK_DATABASE_PRIMARY_TASK[sub_type] !== 0) {\n\t\t\tnotifyCampaignScreenMissionCreated(receiver);",
		to: "\t\t{\n\t\t\tnotifyCampaignScreenMissionCreated(receiver);",
		suite: "js",
	},
	{
		name: "Slice 5b: review rule: an unported LINK_PARENT arm returns instead of failing loudly",
		file: "src/entity/special/task/task.ts",
		from: "\t\tthrow new UnportedBehaviourError(`ts_msgs.c :: response_to_link_parent (${ListType[list_type]})`);",
		to: "\t\treturn 1;",
		suite: "js",
	},
	{
		name: "Slice 5b: review rule: a group's unported member LINK_CHILD returns instead of failing loudly",
		file: "src/entity/special/group/group.ts",
		from: "\t\tthrow new UnportedBehaviourError(\"gp_msgs.c :: response_to_link_child (LIST_TYPE_MEMBER)\");",
		to: "\t\treturn 1;",
		suite: "js",
	},
	{
		name: "Slice 5b: the task id wraps by 4096 (Lua)",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\t\tid -= id_max;",
		to: "\t\tid -= id_max + 1;",
		suite: "lua",
	},
	{
		name: "Slice 5b: the range bias is cubed (Lua)",
		file: "src/entity/special/task/task.ts",
		from: "score = f32Mul(score, 2.0 * f32Mul(f32Mul(f32Mul(x, x), x), x));",
		to: "score = f32Mul(score, 2.0 * f32Mul(f32Mul(x, x), x));",
		suite: "lua",
	},
	{
		name: "Slice 5b: F1: the compatibility height is 1.0 (Lua)",
		file: "src/ai/taskgen/taskgen.ts",
		from: "\t\ty: 0.0,\n\t\tz: f32Add(stop.z, direction.z * 2.0 * KILOMETRE),",
		to: "\t\ty: 1.0,\n\t\tz: f32Add(stop.z, direction.z * 2.0 * KILOMETRE),",
		suite: "lua",
	},
];

function run(cwd, command, args) {
	return spawnSync(command, args, { cwd, encoding: "utf8" });
}

function runSuite(cwd, suite) {
	if (suite === "js") {
		return run(cwd, join(cwd, "node_modules", ".bin", "vitest"), ["run", "--reporter=dot"]);
	}
	if (suite === "cref") {
		// the canonical oracle's environment guard and the C float arithmetic checks
		return run(cwd, join(cwd, "node_modules", ".bin", "vitest"), [
			"run",
			"--reporter=dot",
			"--config",
			"vitest.cref.config.ts",
			"test/c-reference/fpu-environment.cref.test.ts",
			"test/c-reference/float32-rtz.cref.test.ts",
		]);
	}
	const build = run(cwd, join(cwd, "node_modules", ".bin", "tstl"), ["-p", "tsconfig.lua-test.json"]);
	if (build.status !== 0) {
		return build;
	}
	return run(cwd, process.execPath, ["scripts/lua.mjs", "run", "build/lua-test/conformance.lua"]);
}

function copyProject() {
	const dir = mkdtempSync(join(tmpdir(), "eech-core-ts-mutant-"));
	for (const entry of ["src", "test", "scripts", "c-reference", "package.json", "tsconfig.json", "tsconfig.lua-test.json", "vitest.config.ts", "vitest.cref.config.ts"]) {
		cpSync(join(projectRoot, entry), join(dir, entry), { recursive: true });
	}
	symlinkSync(join(projectRoot, "node_modules"), join(dir, "node_modules"), "dir");
	return dir;
}

let survivors = 0;

const baseline = copyProject();
try {
	for (const suite of ["js", "lua"]) {
		const result = runSuite(baseline, suite);
		if (result.status !== 0) {
			console.error(`baseline ${suite} suite fails without mutation:\n${result.stdout}${result.stderr}`);
			process.exit(1);
		}
	}
} finally {
	rmSync(baseline, { recursive: true, force: true });
}

// MUTANT_FILTER=<text> runs only the mutants whose name contains it (while developing a slice)
const selected = MUTANTS.filter((mutant) => !process.env.MUTANT_FILTER || mutant.name.includes(process.env.MUTANT_FILTER));

for (const mutant of selected) {
	const dir = copyProject();
	try {
		const path = join(dir, mutant.file);
		const source = readFileSync(path, "utf8");
		if (!source.includes(mutant.from)) {
			console.error(`mutant "${mutant.name}": pattern not found in ${mutant.file}`);
			process.exit(1);
		}
		writeFileSync(path, source.replace(mutant.from, mutant.to));
		const result = runSuite(dir, mutant.suite);
		if (result.status === 0) {
			survivors += 1;
			console.log(`SURVIVED  [${mutant.suite}] ${mutant.name}`);
		} else {
			console.log(`killed    [${mutant.suite}] ${mutant.name}`);
		}
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
}

if (survivors > 0) {
	console.error(`${survivors} of ${selected.length} mutants survived`);
	process.exit(1);
}

console.log(`all ${selected.length} mutants killed`);
