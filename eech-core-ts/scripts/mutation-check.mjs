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

// suite "js": vitest (JavaScript semantics); suite "lua": TSTL + Lua 5.1 conformance runner
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
		from: "required = toFloat32(bound(required, 0.0, level));\n\n\t\t\t\tlevel = toFloat32(level - required);\n\n\t\t\t\tsetClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL",
		to: "required = toFloat32(bound(required, 0.0, 100.0));\n\n\t\t\t\tlevel = toFloat32(level - required);\n\n\t\t\t\tsetClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL",
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
		from: "fnSetClientServerEntityFloatValue[getCommsModel()].lookup(en.type, type, FloatType[type])(en, type, toFloat32(value));",
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
		from: "const entity_update_iterations = toCInt(toFloat32(getDeltaTime() * frame_rate) + 1.0);",
		to: "const entity_update_iterations = Math.round(toFloat32(getDeltaTime() * frame_rate) + 1.0);",
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
		from: "world_map.max_map_x = toFloat32(toFloat32(num_map_x_sectors * sector_side_length) - 1.0);",
		to: "world_map.max_map_x = toFloat32(num_map_x_sectors * sector_side_length);",
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
];

function run(cwd, command, args) {
	return spawnSync(command, args, { cwd, encoding: "utf8" });
}

function runSuite(cwd, suite) {
	if (suite === "js") {
		return run(cwd, join(cwd, "node_modules", ".bin", "vitest"), ["run", "--reporter=dot"]);
	}
	const build = run(cwd, join(cwd, "node_modules", ".bin", "tstl"), ["-p", "tsconfig.lua-test.json"]);
	if (build.status !== 0) {
		return build;
	}
	return run(cwd, process.execPath, ["scripts/lua.mjs", "run", "build/lua-test/conformance.lua"]);
}

function copyProject() {
	const dir = mkdtempSync(join(tmpdir(), "eech-core-ts-mutant-"));
	for (const entry of ["src", "test", "scripts", "c-reference", "package.json", "tsconfig.json", "tsconfig.lua-test.json", "vitest.config.ts"]) {
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

for (const mutant of MUTANTS) {
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
	console.error(`${survivors} of ${MUTANTS.length} mutants survived`);
	process.exit(1);
}

console.log(`all ${MUTANTS.length} mutants killed`);
