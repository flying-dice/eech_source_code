#!/usr/bin/env node
//
// Generates src/generated/*.ts from the original EECH C sources.
//
// Enum ordinals and database values are behaviour: EECH indexes function
// tables, databases and save data by them. They are therefore never typed in
// by hand. This script parses them out of the original C and emits TypeScript
// with identical names, ordinals and values.
//
//   node scripts/gen-c-sources.mjs          write the generated files
//   node scripts/gen-c-sources.mjs --check  fail if a file is out of date
//

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(here, "..");
const repoRoot = join(projectRoot, "..");
const generatedDir = join(projectRoot, "src", "generated");

// [C enum tag, header path relative to repo root, TS enum name]
const ENUMS = [
	["ENTITY_TYPES", "aphavoc/source/entity/system/en_types/en_types.h", "EntityType"],
	["ENTITY_SIDES", "aphavoc/source/entity/system/en_types/en_side.h", "EntitySide"],
	["LIST_TYPES", "aphavoc/source/entity/system/en_funcs/en_list.h", "ListType"],
	["INT_TYPES", "aphavoc/source/entity/system/en_funcs/en_int.h", "IntType"],
	["FLOAT_TYPES", "aphavoc/source/entity/system/en_funcs/en_float.h", "FloatType"],
	["VEC3D_TYPES", "aphavoc/source/entity/system/en_funcs/en_vec3d.h", "Vec3dType"],
	["PTR_TYPES", "aphavoc/source/entity/system/en_funcs/en_ptr.h", "PtrType"],
	["ENTITY_MESSAGES", "aphavoc/source/entity/system/en_msgs/en_msgs.h", "EntityMessage"],
	["ENTITY_SUB_TYPE_CARGO", "aphavoc/source/entity/system/en_types/en_sbtyp.h", "EntitySubTypeCargo"],
	["ENTITY_SUB_TYPE_GROUPS", "aphavoc/source/entity/system/en_types/en_sbtyp.h", "EntitySubTypeGroup"],
	["ENTITY_SUB_TYPE_KEYSITES", "aphavoc/source/entity/system/en_types/en_sbtyp.h", "EntitySubTypeKeysite"],
	["GROUP_MODE_TYPES", "aphavoc/source/ai_extrn.h", "GroupModeType"],
	["COMMS_MODEL_TYPES", "aphavoc/source/comms/comms.h", "CommsModelType"],
	["RESUPPLY_SOURCE_TYPE", "aphavoc/source/entity/system/en_types/en_suply.h", "ResupplySourceType"],
	["GAME_STATUS_TYPES", "aphavoc/source/global.h", "GameStatusType"],
	["ENTITY_SUB_TYPE_TASKS", "aphavoc/source/entity/system/en_types/en_sbtyp.h", "EntitySubTypeTask"],
	["ENTITY_SUB_TYPE_WAYPOINTS", "aphavoc/source/entity/system/en_types/en_sbtyp.h", "EntitySubTypeWaypoint"],
	["ENTITY_SUB_TYPE_LANDING", "aphavoc/source/entity/system/en_types/en_sbtyp.h", "EntitySubTypeLanding"],
	["KEYSITE_AIR_FORCE_CAPACITY_TYPES", "aphavoc/source/entity/special/keysite/keysite.h", "KeysiteAirForceCapacityType"],
	["KEYSITE_USABLE_STATES", "aphavoc/source/entity/system/en_types/en_state.h", "KeysiteUsableState"],
	["GAME_TYPES", "aphavoc/source/global.h", "GameType"],
	["FORMATION_TYPES", "aphavoc/source/entity/system/en_types/en_forms.h", "FormationType"],
	["TASK_STATE_TYPES", "aphavoc/source/entity/system/en_types/en_task.h", "TaskStateType"],
	["MOVEMENT_TYPES", "aphavoc/source/ai_extrn.h", "MovementType"],
	["TASK_CATEGORY_TYPES", "aphavoc/source/entity/system/en_types/en_task.h", "TaskCategoryType"],
	["ENTITY_SUB_TYPE_AIRCRAFT", "aphavoc/source/entity/system/en_types/en_sbtyp.h", "EntitySubTypeAircraft"],
	// slice 6b
	["ENTITY_SUB_TYPE_GUIDES", "aphavoc/source/entity/system/en_types/en_sbtyp.h", "EntitySubTypeGuide"],
	["POSITION_TYPES", "aphavoc/source/entity/system/en_types/en_wp.h", "PositionType"],
	["GUIDE_CRITERIA_TYPES", "aphavoc/source/entity/special/guide/guide.h", "GuideCriteriaType"],
];

function stripComments(text) {
	return text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, "");
}

export function parseEnum(source, tag) {
	const text = stripComments(source);
	const match = new RegExp(`enum\\s+${tag}\\s*\\{([\\s\\S]*?)\\}`).exec(text);
	if (!match) {
		throw new Error(`enum ${tag} not found`);
	}
	if (/^\s*#/m.test(match[1])) {
		throw new Error(`enum ${tag} contains preprocessor directives; extend the generator`);
	}
	const members = [];
	let next = 0;
	for (const raw of match[1].split(",")) {
		const item = raw.trim();
		if (item === "") {
			continue;
		}
		const assign = /^([A-Za-z_][A-Za-z0-9_]*)\s*(?:=\s*(-?\d+))?$/.exec(item);
		if (!assign) {
			throw new Error(`enum ${tag}: cannot parse member "${item}"`);
		}
		const value = assign[2] !== undefined ? Number(assign[2]) : next;
		members.push([assign[1], value]);
		next = value + 1;
	}
	return members;
}

const HEADER = [
	"//",
	"// GENERATED FILE - DO NOT EDIT.",
	"// Produced by scripts/gen-c-sources.mjs from the original EECH C sources.",
	"// Regenerate with `npm run gen:c`; `npm run check:c` detects drift.",
	"//",
	"",
];

//
// Extracts one column of a C database initialiser whose entries are introduced
// by a `// <ENUM_MEMBER>` banner comment (optionally followed by a comma) and whose fields carry trailing
// `// <field>` comments (the layout of every EECH *_dbase.c file).
//
export function parseDatabaseColumn(source, enumPrefix, fieldComment) {
	const lines = source.split(/\r?\n/);
	const rows = [];
	let current;
	for (const line of lines) {
		const banner = new RegExp(`^\\s*//\\s*(${enumPrefix}[A-Z0-9_]+)\\s*,?\\s*$`).exec(line);
		if (banner) {
			current = banner[1];
			continue;
		}
		const field = new RegExp(`^\\s*([^,/]+?)\\s*,?\\s*//\\s*${fieldComment}\\s*$`).exec(line);
		if (field) {
			if (current === undefined) {
				throw new Error(`${fieldComment} value before any ${enumPrefix} banner`);
			}
			rows.push([current, field[1]]);
			current = undefined;
		}
	}
	return rows;
}

function generateGroupDatabase() {
	const header = "aphavoc/source/entity/special/group/gp_dbase.c";
	const source = readFileSync(join(repoRoot, header), "latin1");
	const groups = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_GROUPS")
		.filter(([name]) => !name.startsWith("NUM_"));
	const rows = parseDatabaseColumn(source, "ENTITY_SUB_TYPE_GROUP_", "resupply_source");
	if (rows.length !== groups.length) {
		throw new Error(`group_database: ${rows.length} resupply_source rows for ${groups.length} group sub types`);
	}
	const lines = [...HEADER];
	lines.push('import { ResupplySourceType } from "./c-enums";', "");
	lines.push(`// C provenance: ${header} :: group_database [NUM_ENTITY_SUB_TYPE_GROUPS] .resupply_source`);
	lines.push("// (compiled defaults; wutcfg.c / gwutcfg.c may override them at load time)");
	lines.push("// Indexed by EntitySubTypeGroup, as group_database [raw->sub_type] is in C.");
	lines.push("export const GROUP_DATABASE_RESUPPLY_SOURCE: readonly ResupplySourceType[] = [");
	rows.forEach(([name, value], i) => {
		if (name !== groups[i][0]) {
			throw new Error(`group_database entry ${i} is ${name}, expected ${groups[i][0]}`);
		}
		lines.push(`\tResupplySourceType.${value}, // ${i} ${name}`);
	});
	lines.push("];", "");

	const landings = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_LANDING");
	const movements = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/ai_extrn.h"), "latin1"), "MOVEMENT_TYPES");
	const column = (comment) => databaseColumn(source, groups, "ENTITY_SUB_TYPE_GROUP_", comment, "group_database");
	const emit = (tsName, field, values) => {
		lines.push(`// C provenance: ${header} :: group_database [].${field}`);
		lines.push(`export const ${tsName}: readonly number[] = [`);
		values.forEach((value, i) => lines.push(`\t${value}, // ${i} ${groups[i][0]}`));
		lines.push("];", "");
	};
	emit("GROUP_DATABASE_MOVEMENT_TYPE", "movement_type (MovementType)", column("\\.movement type").map((v) => enumMemberValue(movements, v, ".movement type")));
	emit("GROUP_DATABASE_DEFAULT_LANDING_TYPE", "default_landing_type (EntitySubTypeLanding)", column("landing type").map((v) => enumMemberValue(landings, v, "landing type")));
	emit("GROUP_DATABASE_DEFAULT_ENGAGE_ENEMY", "default_engage_enemy (TRUE 1, FALSE 0)", column("default_engage_enemy").map((v) => booleanLiteral(v, "default_engage_enemy")));
	emit("GROUP_DATABASE_MINIMUM_IDLE_COUNT", "minimum_idle_count", column("minimum_idle_count").map((v) => integerLiteral(v, "minimum_idle_count")));
	// slice 6a: which force registry a group type joins, and the aircraft its members default to
	// (the cruise-velocity invariant of test/unit/supply-task-assignment.test.ts)
	const listTypes = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_funcs/en_list.h"), "latin1"), "LIST_TYPES");
	const entityTypes = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_types.h"), "latin1"), "ENTITY_TYPES");
	const aircraft = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_AIRCRAFT");
	emit("GROUP_DATABASE_REGISTRY_LIST_TYPE", "registry_list_type (ListType)", column("registery list").map((v) => enumMemberValue(listTypes, v, "registery list")));
	const entityTypeNames = column("default_entity_type(?: ;.*)?");
	emit("GROUP_DATABASE_DEFAULT_ENTITY_TYPE", "default_entity_type (EntityType)", entityTypeNames.map((v) => enumMemberValue(entityTypes, v, "default_entity_type")));
	for (const [comment, tsName] of [
		["default_blue_force_sub_type", "GROUP_DATABASE_DEFAULT_BLUE_FORCE_AIRCRAFT_SUB_TYPE"],
		["default_red_force_sub_type", "GROUP_DATABASE_DEFAULT_RED_FORCE_AIRCRAFT_SUB_TYPE"],
	]) {
		const values = column(comment).map((v, i) => {
			const isAircraftGroup = entityTypeNames[i] === "ENTITY_TYPE_HELICOPTER" || entityTypeNames[i] === "ENTITY_TYPE_FIXED_WING";
			if (isAircraftGroup !== v.startsWith("ENTITY_SUB_TYPE_AIRCRAFT_")) {
				throw new Error(`group_database entry ${i}: ${comment} ${v} does not match default_entity_type ${entityTypeNames[i]}`);
			}
			return isAircraftGroup ? enumMemberValue(aircraft, v, comment) : "-1";
		});
		emit(tsName, `${comment} (EntitySubTypeAircraft of an aircraft group; -1: not an aircraft group)`, values);
	}
	for (const [field, , groupComment] of AI_STAT_FIELDS) {
		emit(`GROUP_DATABASE_AI_STATS_${field.toUpperCase()}`, `ai_stats.${field}`, column(escapeRegExp(groupComment)).map((v) => integerLiteral(v, groupComment)));
	}
	return lines.join("\n");
}

// [macro, header path relative to repo root]: object-like macros whose value is a plain numeric literal
const NUMERIC_DEFINES = [
	["FUEL_USAGE_ACCELERATOR", "aphavoc/source/entity/system/en_types/en_suply.h"],
	["AMMO_USAGE_ACCELERATOR", "aphavoc/source/entity/system/en_types/en_suply.h"],
	["KEYSITE_SUPPLY_REQUEST_THRESHOLD", "aphavoc/source/entity/system/en_types/en_suply.h"],
	["CARGO_AMMO_SIZE", "aphavoc/source/entity/mobile/cargo/cargo.h"],
	["CARGO_FUEL_SIZE", "aphavoc/source/entity/mobile/cargo/cargo.h"],
	// slice 5b: task bit-field widths (en_int.h)
	["NUM_TASK_ID_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["NUM_ROUTE_LENGTH_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["NUM_TASK_DIFFICULTY_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["NUM_CRITICAL_TASK_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["NUM_MOVEMENT_TYPE_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["NUM_SIDE_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["NUM_ALIVE_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["NUM_LANDING_TYPE_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["NUM_KEYSITE_USABLE_STATE_BITS", "aphavoc/source/entity/system/en_funcs/en_int.h"],
	["SECONDS_IN_A_MINUTE", "modules/maths/constant.h"],
	["MAX_ROUTE_NODES", "aphavoc/source/ai/taskgen/taskgen.h"],
];

// OBJECT_3D_INDEX_NUMBERS members the port names: [member, header]
const OBJECT_3D_INDICES = [["OBJECT_3D_SINGLE_CRATE", "modules/3d/3dmodels.h"]];

//
// The object index enum of modules/3d/3dmodels.h is an X-macro list:
// OBJECT_3D_INDEX (x) declares OBJECT_3D_x and OBJECT_3D_INDEX_ (x) declares x,
// in order from 0, between OBJECT_3D_DECLARATION ({) and OBJECT_3D_DECLARATION (};).
//
export function parseObject3dIndex(source, member) {
	const start = source.indexOf("OBJECT_3D_DECLARATION({)");
	const end = source.indexOf("OBJECT_3D_DECLARATION(};)", start);
	if (start < 0 || end < 0) {
		throw new Error("object 3d index list not found");
	}
	const body = stripComments(source.slice(start, end));
	if (/^\s*#/m.test(body)) {
		throw new Error("object 3d index list contains preprocessor directives; extend the generator");
	}
	let index = 0;
	for (const m of body.matchAll(/OBJECT_3D_INDEX(_?)\s*\(\s*([A-Za-z0-9_]+)\s*\)/g)) {
		const name = m[1] === "_" ? m[2] : `OBJECT_3D_${m[2]}`;
		if (name === member) {
			return index;
		}
		index += 1;
	}
	throw new Error(`${member} not found in the object 3d index list`);
}

export function parseNumericDefine(source, name) {
	const match = new RegExp(`^\\s*#define\\s+${name}\\s+([^\\s/]+)\\s*(?://.*|/\\*.*)?$`, "m").exec(source);
	if (!match) {
		throw new Error(`#define ${name} not found`);
	}
	// one level of parentheses around the literal, e.g. (12)
	const literal = /^\((.*)\)$/.exec(match[1]) ? match[1].slice(1, -1) : match[1];
	if (!/^-?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?[fF]?$/.test(literal)) {
		throw new Error(`#define ${name} is not a numeric literal: ${match[1]}; extend the generator`);
	}
	return literal.replace(/[fF]$/, "");
}

function generateConstants() {
	const lines = [...HEADER];
	for (const [name, header] of NUMERIC_DEFINES) {
		const value = parseNumericDefine(readFileSync(join(repoRoot, header), "latin1"), name);
		lines.push(`// C provenance: #define ${name} (${header})`);
		lines.push(`export const ${name} = ${value};`, "");
	}
	for (const [member, header] of OBJECT_3D_INDICES) {
		const index = parseObject3dIndex(readFileSync(join(repoRoot, header), "latin1"), member);
		lines.push(`// C provenance: enum OBJECT_3D_INDEXS :: ${member} (${header}, OBJECT_3D_INDEX list position)`);
		lines.push(`export const ${member} = ${index};`, "");
	}
	return lines.join("\n");
}

function generateKeysiteDatabase() {
	const header = "aphavoc/source/entity/special/keysite/ks_dbase.c";
	const source = readFileSync(join(repoRoot, header), "latin1");
	const keysites = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_KEYSITES")
		.filter(([name]) => !name.startsWith("NUM_"));
	const lines = [...HEADER];
	for (const [field, tsName] of [
		["ammo", "KEYSITE_DATABASE_AMMO_SUPPLY_USAGE"],
		["fuel", "KEYSITE_DATABASE_FUEL_SUPPLY_USAGE"],
	]) {
		const rows = parseDatabaseColumn(source, "ENTITY_SUB_TYPE_KEYSITE_", field);
		if (rows.length !== keysites.length) {
			throw new Error(`keysite_database: ${rows.length} ${field} rows for ${keysites.length} keysite sub types`);
		}
		lines.push(`// C provenance: ${header} :: keysite_database [NUM_ENTITY_SUB_TYPE_KEYSITES] .default_supply_usage.${field}_supply_level`);
		lines.push("// (compiled defaults; wutcfg.c may override them at load time from the WUT file)");
		lines.push("// Indexed by EntitySubTypeKeysite, as keysite_database [raw->sub_type] is in C.");
		lines.push(`export const ${tsName}: readonly number[] = [`);
		rows.forEach(([name, value], i) => {
			if (name !== keysites[i][0]) {
				throw new Error(`keysite_database entry ${i} is ${name}, expected ${keysites[i][0]}`);
			}
			if (!/^[-+]?(\d+\.?\d*|\.\d+)$/.test(value)) {
				throw new Error(`keysite_database ${name} ${field} is not a numeric literal: ${value}`);
			}
			lines.push(`\t${value.replace(/^\+/, "")}, // ${i} ${name}`);
		});
		lines.push("];", "");
	}
	const capacities = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/special/keysite/keysite.h"), "latin1"), "KEYSITE_AIR_FORCE_CAPACITY_TYPES");
	const capacity = databaseColumn(source, keysites, "ENTITY_SUB_TYPE_KEYSITE_", "air force capacity", "keysite_database");
	for (const [comment, tsName, field] of [
		["task assign count", "KEYSITE_DATABASE_ASSIGN_TASK_COUNT", "assign_task_count (unsigned int : 2)"],
		["task reserve count", "KEYSITE_DATABASE_RESERVE_TASK_COUNT", "reserve_task_count (unsigned int : 2)"],
	]) {
		const values = databaseColumn(source, keysites, "ENTITY_SUB_TYPE_KEYSITE_", comment, "keysite_database").map((v) => integerLiteral(v, comment));
		lines.push(`// C provenance: ${header} :: keysite_database [].${field}`);
		lines.push(`export const ${tsName}: readonly number[] = [`);
		values.forEach((value, i) => lines.push(`\t${value}, // ${i} ${keysites[i][0]}`));
		lines.push("];", "");
	}
	lines.push(`// C provenance: ${header} :: keysite_database [].air_force_capacity (KeysiteAirForceCapacityType)`);
	lines.push("export const KEYSITE_DATABASE_AIR_FORCE_CAPACITY: readonly number[] = [");
	capacity.forEach((value, i) => lines.push(`\t${enumMemberValue(capacities, value, "air force capacity")}, // ${i} ${keysites[i][0]}`));
	lines.push("];", "");
	return lines.join("\n");
}

// One column of an EECH database, checked against the enum it is indexed by.
// `banner` maps an entry's banner to the enum member it stands for.
function databaseColumn(source, members, enumPrefix, fieldComment, what, banner = (name, member) => name === member) {
	const rows = parseDatabaseColumn(source, enumPrefix, fieldComment);
	if (rows.length !== members.length) {
		throw new Error(`${what}: ${rows.length} "${fieldComment}" rows for ${members.length} members`);
	}
	return rows.map(([name, value], i) => {
		if (!banner(name, members[i][0])) {
			throw new Error(`${what} entry ${i} is ${name}, expected ${members[i][0]}`);
		}
		return value.trim();
	});
}

function integerLiteral(value, what) {
	if (!/^-?\d+$/.test(value)) {
		throw new Error(`${what}: not an integer literal: ${value}`);
	}
	return value;
}

function booleanLiteral(value, what) {
	if (value !== "TRUE" && value !== "FALSE") {
		throw new Error(`${what}: not TRUE or FALSE: ${value}`);
	}
	return value === "TRUE" ? "1" : "0";
}

function enumMemberValue(members, value, what) {
	for (const [name, ordinal] of members) {
		if (name === value) {
			return String(ordinal);
		}
	}
	throw new Error(`${what}: ${value} is not a member of the expected enum`);
}

const AI_STAT_FIELDS = [
	["air_attack_strength", "Air Attack strength", ".air attack strength"],
	["ground_attack_strength", "Ground Attack strength", ".ground attack strength"],
	["movement_speed", "Movement Speed", ".movement speed"],
	["movement_stealth", "Movement Stealth", ".movement stealth"],
	["cargo_space", "Cargo Space", ".cargo space"],
	["troop_space", "Troop Space", ".troop space"],
];

function escapeRegExp(text) {
	return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

//
// ts_dbase.c :: task_database [].landing_types: the expression after the
// `// landing types` comment, a sum of (1 << ENTITY_SUB_TYPE_LANDING_*) terms
// (or 0) up to its trailing comma.
//
function parseTaskLandingTypes(source, landings) {
	const lines = source.split(/\r?\n/);
	const values = [];
	for (let i = 0; i < lines.length; i++) {
		if (!/^\s*\/\/\s*landing types\s*$/.test(lines[i])) {
			continue;
		}
		let expression = "";
		for (i++; i < lines.length; i++) {
			expression += ` ${lines[i].trim()}`;
			if (/,\s*$/.test(lines[i])) {
				break;
			}
		}
		expression = expression.trim().replace(/,$/, "").trim();
		if (expression === "0") {
			values.push(0);
			continue;
		}
		let value = 0;
		for (const term of expression.split("+")) {
			const m = /^\(1 << (ENTITY_SUB_TYPE_LANDING_[A-Z_]+)\)$/.exec(term.trim());
			if (!m) {
				throw new Error(`task_database landing types: cannot parse "${expression}"`);
			}
			value += Math.pow(2, Number(enumMemberValue(landings, m[1], "landing types")));
		}
		values.push(value);
	}
	return values;
}

//
// ts_dbase.c :: task_database. Two banners name their entry without the
// TASK_ infix (ENTITY_SUB_TYPE_TRANSFER_FIXED_WING, _HELICOPTER); entries are
// matched to the enum by position, and a banner must equal its enum member
// with or without that infix.
//
function generateTaskDatabase() {
	const header = "aphavoc/source/entity/special/task/ts_dbase.c";
	const source = readFileSync(join(repoRoot, header), "latin1");
	const tasks = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_TASKS")
		.filter(([name]) => !name.startsWith("NUM_"));
	const rows = parseDatabaseColumn(source, "ENTITY_SUB_TYPE_", "task priority");
	if (rows.length !== tasks.length) {
		throw new Error(`task_database: ${rows.length} task priority rows for ${tasks.length} task sub types`);
	}
	const lines = [...HEADER];
	lines.push(`// C provenance: ${header} :: task_database [NUM_ENTITY_SUB_TYPE_TASKS] .task_priority`);
	lines.push("// (compiled defaults; the float field is initialised from an integer literal)");
	lines.push("// Indexed by EntitySubTypeTask, as task_database [sub_type] is in C.");
	lines.push("export const TASK_DATABASE_TASK_PRIORITY: readonly number[] = [");
	rows.forEach(([name, value], i) => {
		const member = tasks[i][0];
		if (name !== member && name !== member.replace(/^ENTITY_SUB_TYPE_TASK_/, "ENTITY_SUB_TYPE_")) {
			throw new Error(`task_database entry ${i} is ${name}, expected ${member}`);
		}
		if (!/^\d+$/.test(value)) {
			throw new Error(`task_database ${member} task priority is not an integer literal: ${value}`);
		}
		lines.push(`\t${value}, // ${i} ${member}`);
	});
	lines.push("];", "");

	const sbtyp = readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1");
	const landings = parseEnum(sbtyp, "ENTITY_SUB_TYPE_LANDING");
	const movements = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/ai_extrn.h"), "latin1"), "MOVEMENT_TYPES");
	const capacities = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/special/keysite/keysite.h"), "latin1"), "KEYSITE_AIR_FORCE_CAPACITY_TYPES");
	const banner = (name, member) => name === member || name === member.replace(/^ENTITY_SUB_TYPE_TASK_/, "ENTITY_SUB_TYPE_");
	const column = (comment) => databaseColumn(source, tasks, "ENTITY_SUB_TYPE_", comment, "task_database", banner);
	const emit = (tsName, field, values) => {
		lines.push(`// C provenance: ${header} :: task_database [].${field}`);
		lines.push(`export const ${tsName}: readonly number[] = [`);
		values.forEach((value, i) => lines.push(`\t${value}, // ${i} ${tasks[i][0]}`));
		lines.push("];", "");
	};
	const categories = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_task.h"), "latin1"), "TASK_CATEGORY_TYPES");
	emit("TASK_DATABASE_TASK_CATEGORY", "task_category (TaskCategoryType)", column("task category").map((v) => enumMemberValue(categories, v, "task category")));
	emit("TASK_DATABASE_MINIMUM_MEMBER_COUNT", "minimum_member_count", column("Minimum Member Count").map((v) => integerLiteral(v, "Minimum Member Count")));
	emit("TASK_DATABASE_PRIMARY_TASK", "primary_task (TRUE 1, FALSE 0)", column("primary task").map((v) => booleanLiteral(v, "primary task")));
	// slice 6b: assign_task_to_group and create_generic_waypoint_route
	emit("TASK_DATABASE_ADD_START_WAYPOINT", "add_start_waypoint (TRUE 1, FALSE 0)", column("add start waypoint").map((v) => booleanLiteral(v, "add start waypoint")));
	emit("TASK_DATABASE_ASSESS_LANDING", "assess_landing (TRUE 1, FALSE 0)", column("Assess landing on completion").map((v) => booleanLiteral(v, "Assess landing on completion")));
	emit("TASK_DATABASE_TASK_ROUTE_SEARCH", "task_route_search (TRUE 1, FALSE 0)", column("Task route search").map((v) => booleanLiteral(v, "Task route search")));
	emit("TASK_DATABASE_ENGAGE_ENEMY", "engage_enemy (TRUE 1, FALSE 0)", column("Engage Enemy").map((v) => booleanLiteral(v, "Engage Enemy")));
	emit("TASK_DATABASE_MOVEMENT_TYPE", "movement_type (MovementType)", column("Movement Type").map((v) => enumMemberValue(movements, v, "Movement Type")));
	emit("TASK_DATABASE_KEYSITE_AIR_FORCE_CAPACITY", "keysite_air_force_capacity (KeysiteAirForceCapacityType)", column("keysite air force capacity").map((v) => enumMemberValue(capacities, v, "keysite air force capacity")));
	const landingTypes = parseTaskLandingTypes(source, landings);
	if (landingTypes.length !== tasks.length) {
		throw new Error(`task_database: ${landingTypes.length} landing types for ${tasks.length} tasks`);
	}
	emit("TASK_DATABASE_LANDING_TYPES", "landing_types (bits of EntitySubTypeLanding)", landingTypes.map(String));
	for (const [field, taskComment] of AI_STAT_FIELDS) {
		emit(`TASK_DATABASE_AI_STATS_${field.toUpperCase()}`, `ai_stats.${field}`, column(escapeRegExp(taskComment)).map((v) => integerLiteral(v, taskComment)));
	}
	return lines.join("\n");
}

//
// ac_dbase.c :: aircraft_database [].cruise_velocity. Every entry is
// knots_to_metres_per_second (<double literal>) (convert.h), a compile-time
// constant: KNOTS * (ONE_NAUTICAL_MILE / (60.0f * 60.0f)) with
// ONE_NAUTICAL_MILE = EQUATORIAL_EARTH_CIRCUM / (360.0f * 60.0f) and
// EQUATORIAL_EARTH_CIRCUM = 2.0f * PI * EQUATORIAL_EARTH_RADIUS (constant.h).
// The float sub-expressions round to nearest float; KNOTS is a double literal,
// so the product is a double rounded to nearest float by the initialiser. The
// generated values are checked bit for bit against the compiled C database
// (test/c-reference/aircraft-database.cref.test.ts).
//
function knotsToMetresPerSecond(knots) {
	const f = Math.fround;
	const pi = f(Number(parseNumericDefineExpression("PI")));
	const radius = f(Number(parseNumericDefineExpression("EQUATORIAL_EARTH_RADIUS")));
	const circumference = f(f(2 * pi) * radius);
	const nauticalMile = f(circumference / f(360 * 60));
	return f(knots * f(nauticalMile / f(60 * 60)));
}

function parseNumericDefineExpression(name) {
	return parseNumericDefine(readFileSync(join(repoRoot, "modules/maths/constant.h"), "latin1"), name);
}

function checkConversionMacros() {
	const expected = [
		["modules/maths/constant.h", "PI", "(3.14159265359f)"],
		["modules/maths/constant.h", "EQUATORIAL_EARTH_RADIUS", "(6378160.0f)"],
		["modules/maths/constant.h", "EQUATORIAL_EARTH_CIRCUM", "(2.0f * PI * EQUATORIAL_EARTH_RADIUS)"],
		["modules/maths/constant.h", "ONE_NAUTICAL_MILE", "(EQUATORIAL_EARTH_CIRCUM / (360.0f * 60.0f))"],
		["modules/maths/convert.h", "knots_to_metres_per_second(KNOTS)", "((KNOTS) * (ONE_NAUTICAL_MILE / (60.0f * 60.0f)))"],
	];
	for (const [file, name, body] of expected) {
		const source = readFileSync(join(repoRoot, file), "latin1");
		const match = new RegExp(`^\\s*#define\\s+${escapeRegExp(name)}\\s+(.*?)\\s*$`, "m").exec(source);
		if (!match || match[1] !== body) {
			throw new Error(`#define ${name} (${file}) changed; re-derive knotsToMetresPerSecond`);
		}
	}
}

function generateAircraftDatabase() {
	const header = "aphavoc/source/entity/mobile/aircraft/ac_dbase.c";
	const source = readFileSync(join(repoRoot, header), "latin1");
	const aircraft = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_AIRCRAFT")
		.filter(([name]) => !name.startsWith("NUM_"));
	checkConversionMacros();
	const values = databaseColumn(source, aircraft, "ENTITY_SUB_TYPE_AIRCRAFT_", "cruise_velocity", "aircraft_database").map((v) => {
		const m = /^knots_to_metres_per_second \((\d+\.\d*)\)$/.exec(v);
		if (!m) {
			throw new Error(`aircraft_database cruise_velocity: not knots_to_metres_per_second (<double literal>): ${v}`);
		}
		return [m[1], knotsToMetresPerSecond(Number(m[1]))];
	});
	const lines = [...HEADER];
	lines.push(`// C provenance: ${header} :: aircraft_database [NUM_ENTITY_SUB_TYPE_AIRCRAFT] .cruise_velocity`);
	lines.push("// (compiled defaults: knots_to_metres_per_second (KNOTS), convert.h, evaluated at float");
	lines.push("// precision with round to nearest, as the compiler folds it; values are float32)");
	lines.push("// Indexed by EntitySubTypeAircraft, as aircraft_database [raw->mob.sub_type] is in C.");
	lines.push("export const AIRCRAFT_DATABASE_CRUISE_VELOCITY: readonly number[] = [");
	values.forEach(([knots, value], i) => lines.push(`\t${value}, // ${i} ${aircraft[i][0]} (${knots} knots)`));
	lines.push("];", "");
	// slice 6b: the route's waypoint altitude
	const altitudes = databaseColumn(source, aircraft, "ENTITY_SUB_TYPE_AIRCRAFT_", "cruise_altitude", "aircraft_database").map((v) => floatLiteral(v, "cruise_altitude"));
	lines.push(`// C provenance: ${header} :: aircraft_database [].cruise_altitude (a double literal stored as float: round to nearest)`);
	lines.push("export const AIRCRAFT_DATABASE_CRUISE_ALTITUDE: readonly number[] = [");
	altitudes.forEach((value, i) => lines.push(`\t${value}, // ${i} ${aircraft[i][0]}`));
	lines.push("];", "");
	return lines.join("\n");
}

// A floating literal of a float field: the compiler rounds it to nearest.
function floatLiteral(value, what) {
	if (!/^-?(\d+\.?\d*|\.\d+)f?$/.test(value)) {
		throw new Error(`${what}: not a floating literal: ${value}`);
	}
	return String(Math.fround(Number(value.replace(/f$/, ""))));
}

//
// A float field's initialiser in the waypoint database: a literal, or
// `N * KILOMETRE` (constant.h: KILOMETRE (1000 * METRE), METRE (1.0f)): an int
// or double literal times a float, folded to nearest (exact for the values used).
//
function waypointDistance(value, what) {
	const m = /^(\d+\.?\d*) \* KILOMETRE$/.exec(value);
	if (m) {
		return String(Math.fround(Number(m[1]) * 1000));
	}
	if (/^\d+$/.test(value)) {
		return String(Math.fround(Number(value)));
	}
	return floatLiteral(value, what);
}

//
// wp_dbase.c :: waypoint_database [NUM_ENTITY_SUB_TYPE_WAYPOINTS]. Entries have
// no banners: each opens with its name (`"..."`, // Name) and is matched to the
// enum by position. Each holds the entity-wide fields, then four blocks of the
// same fields for FIXED WING, HELICOPTER, GROUND and SEA, in that order.
//
function generateWaypointDatabase() {
	const header = "aphavoc/source/entity/special/waypoint/wp_dbase.c";
	const source = readFileSync(join(repoRoot, header), "latin1");
	const waypoints = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_WAYPOINTS").filter(([name]) => !name.startsWith("NUM_"));
	const guides = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_GUIDES");
	const positions = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_wp.h"), "latin1"), "POSITION_TYPES");
	const lines = source.split(/\r?\n/);
	const entries = [];
	let entry;
	let section;
	const field = (line, comment) => new RegExp(`^\\s*([^/]+?)\\s*,\\s*//\\s*${comment}\\s*$`).exec(line);
	for (const line of lines) {
		const name = /^\s*"([^"]*)",\s*\/\/\s*Name\s*$/.exec(line);
		if (name) {
			entry = { name: name[1], sections: [] };
			entries.push(entry);
			section = undefined;
			continue;
		}
		if (!entry) continue;
		const marker = /^\s*\/\/\s*(FIXED WING|HELICOPTER|GROUND|SEA)\s*$/.exec(line);
		if (marker) {
			section = {};
			entry.sections.push(section);
			continue;
		}
		const guide = field(line, "guide sub type");
		if (guide && !section) {
			entry.guide = guide[1];
			continue;
		}
		if (!section) continue;
		for (const [key, comment] of [
			["minimum_previous_waypoint_distance", "Minimum previous waypoint distance"],
			["reached_radius", "Reached Radius"],
			["velocity", "Velocity"],
			["criteria_last_to_reach", "Criteria Last To Reach"],
			["criteria_transmit_recon", "Criteria Transmit Recon"],
			["position_type", "Position type"],
		]) {
			const m = field(line, comment);
			if (m) {
				if (section[key] !== undefined) throw new Error(`waypoint_database ${entry.name}: ${comment} twice in one block`);
				section[key] = m[1];
			}
		}
	}
	if (entries.length !== waypoints.length) {
		throw new Error(`waypoint_database: ${entries.length} entries for ${waypoints.length} waypoint sub types`);
	}
	const out = [...HEADER];
	out.push(`// C provenance: ${header} :: waypoint_database [NUM_ENTITY_SUB_TYPE_WAYPOINTS] (compiled defaults; no WUT override of it is ported)`);
	out.push("// Indexed by EntitySubTypeWaypoint. The per-mobile columns are indexed [waypoint][block]:");
	out.push("// block 0 FIXED WING (fw_), 1 HELICOPTER (hc_), 2 GROUND (rv_), 3 SEA (sh_); wp_dbase.c's get_waypoint_database_*");
	out.push("// accessors choose the block by the mobile's entity type.");
	out.push("");
	const perEntry = (tsName, field, values) => {
		out.push(`// C provenance: ${header} :: waypoint_database [].${field}`);
		out.push(`export const ${tsName}: readonly number[] = [`);
		values.forEach((value, i) => out.push(`\t${value}, // ${i} ${waypoints[i][0]}`));
		out.push("];", "");
	};
	const perBlock = (tsName, field, convert) => {
		out.push(`// C provenance: ${header} :: waypoint_database [].{fw,hc,rv,sh}_${field}`);
		out.push(`export const ${tsName}: readonly (readonly number[])[] = [`);
		entries.forEach((e, i) => {
			if (e.sections.length !== 4) throw new Error(`waypoint_database ${e.name}: ${e.sections.length} mobile blocks`);
			out.push(`\t[${e.sections.map((sec) => { if (sec[field] === undefined) throw new Error(`waypoint_database ${e.name}: no ${field}`); return convert(sec[field]); }).join(", ")}], // ${i} ${waypoints[i][0]}`);
		});
		out.push("];", "");
	};
	perEntry("WAYPOINT_DATABASE_GUIDE_SUB_TYPE", "guide_sub_type (EntitySubTypeGuide)", entries.map((e) => enumMemberValue(guides, e.guide, "guide sub type")));
	perBlock("WAYPOINT_DATABASE_MINIMUM_PREVIOUS_WAYPOINT_DISTANCE", "minimum_previous_waypoint_distance", (v) => waypointDistance(v, "Minimum previous waypoint distance"));
	perBlock("WAYPOINT_DATABASE_REACHED_RADIUS", "reached_radius", (v) => waypointDistance(v, "Reached Radius"));
	perBlock("WAYPOINT_DATABASE_VELOCITY", "velocity", (v) => waypointDistance(v, "Velocity"));
	perBlock("WAYPOINT_DATABASE_CRITERIA_LAST_TO_REACH", "criteria_last_to_reach", (v) => booleanLiteral(v, "Criteria Last To Reach"));
	perBlock("WAYPOINT_DATABASE_CRITERIA_TRANSMIT_RECON", "criteria_transmit_recon", (v) => booleanLiteral(v, "Criteria Transmit Recon"));
	perBlock("WAYPOINT_DATABASE_POSITION_TYPE", "position_type", (v) => enumMemberValue(positions, v, "Position type"));
	return out.join("\n");
}

//
// A guide criterion value: a literal or rad (DEG) (convert.h: ((DEG) * (PI /
// 180.0f)), PI (3.14159265359f)): a double literal times a float, stored as
// float, folded to nearest by the compiler.
//
function guideCriterionValue(value, what) {
	const m = /^rad \((\d+\.?\d*)\)$/.exec(value);
	if (m) {
		checkDefine("modules/maths/convert.h", "rad(DEG)", "((DEG) * (PI / 180.0f))");
		const f = Math.fround;
		return String(f(Number(m[1]) * f(f(Number(parseNumericDefineExpression("PI"))) / 180)));
	}
	return floatLiteral(value, what);
}

function checkDefine(file, name, body) {
	const source = readFileSync(join(repoRoot, file), "latin1");
	const match = new RegExp(`^\\s*#define\\s+${escapeRegExp(name)}\\s+(.*?)\\s*$`, "m").exec(source);
	if (!match || match[1] !== body) {
		throw new Error(`#define ${name} (${file}) changed; re-derive its evaluation`);
	}
}

//
// gd_dbase.c :: guide_database [NUM_ENTITY_SUB_TYPE_GUIDES]: each entry's
// criteria [NUM_GUIDE_CRITERIA_TYPES], `{ VALID, VALUE },  // GUIDE_CRITERIA_*`.
//
function generateGuideDatabase() {
	const header = "aphavoc/source/entity/special/guide/gd_dbase.c";
	const source = readFileSync(join(repoRoot, header), "latin1");
	const guides = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/system/en_types/en_sbtyp.h"), "latin1"), "ENTITY_SUB_TYPE_GUIDES").filter(([name]) => !name.startsWith("NUM_"));
	const criteria = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/entity/special/guide/guide.h"), "latin1"), "GUIDE_CRITERIA_TYPES").filter(([name]) => !name.startsWith("NUM_"));
	const rows = [];
	let current;
	for (const line of source.split(/\r?\n/)) {
		const banner = /^\s*\/\/\s*(ENTITY_SUB_TYPE_GUIDE_[A-Z0-9_]+)\s*,?\s*$/.exec(line);
		if (banner) {
			current = { name: banner[1], criteria: [] };
			rows.push(current);
			continue;
		}
		const c = /^\s*\{\s*(TRUE|FALSE)\s*,\s*([^}]+?)\s*\}\s*,\s*\/\/\s*(GUIDE_CRITERIA_[A-Z_]+)\s*$/.exec(line);
		if (c) {
			if (!current) throw new Error("guide_database: criterion before any banner");
			current.criteria.push([c[3], booleanLiteral(c[1], c[3]), guideCriterionValue(c[2], c[3])]);
		}
	}
	if (rows.length !== guides.length) {
		throw new Error(`guide_database: ${rows.length} entries for ${guides.length} guide sub types`);
	}
	const out = [...HEADER];
	out.push(`// C provenance: ${header} :: guide_database [NUM_ENTITY_SUB_TYPE_GUIDES] .criteria [NUM_GUIDE_CRITERIA_TYPES]`);
	out.push("// (compiled defaults; no GWUT override is ported). [guide sub type][criterion] = [valid (TRUE 1), value (float)]");
	out.push("export const GUIDE_DATABASE_CRITERIA: readonly (readonly (readonly [number, number])[])[] = [");
	rows.forEach((r, i) => {
		if (r.name !== guides[i][0]) throw new Error(`guide_database entry ${i} is ${r.name}, expected ${guides[i][0]}`);
		if (r.criteria.length !== criteria.length) throw new Error(`guide_database ${r.name}: ${r.criteria.length} criteria`);
		r.criteria.forEach(([name], k) => { if (name !== criteria[k][0]) throw new Error(`guide_database ${r.name}: criterion ${k} is ${name}`); });
		out.push(`\t[${r.criteria.map(([, valid, value]) => `[${valid}, ${value}]`).join(", ")}], // ${i} ${r.name}`);
	});
	out.push("];", "");
	return out.join("\n");
}

//
// croute.c :: route_biasing_database [NUM_MOVEMENT_TYPES]: banners
// `//MOVEMENT_TYPE_*,`; float fields from double literals (round to nearest).
//
function generateRouteBiasingDatabase() {
	const header = "aphavoc/source/ai/taskgen/croute.c";
	const source = readFileSync(join(repoRoot, header), "latin1");
	const movements = parseEnum(readFileSync(join(repoRoot, "aphavoc/source/ai_extrn.h"), "latin1"), "MOVEMENT_TYPES").filter(([name]) => !name.startsWith("NUM_"));
	const fields = [
		["elevation_bias", "elevation bias"],
		["range_bias", "range bias"],
		["side_bias", "side bias"],
		["min_route_range", "min route range"],
		["route_deviation_size", "route deviation size"],
		["num_route_samples", "num route samples"],
		["optimise_tolerance", "optimise tolerance"],
	];
	const out = [...HEADER];
	out.push(`// C provenance: ${header} :: route_biasing_database [] (static; indexed by MovementType)`);
	for (const [fieldName, comment] of fields) {
		const values = databaseColumn(source, movements, "MOVEMENT_TYPE_", comment, "route_biasing_database").map((v) => floatLiteral(v, comment));
		out.push(`// C provenance: ${header} :: route_biasing_database [].${fieldName} (float)`);
		out.push(`export const ROUTE_BIASING_${fieldName.toUpperCase()}: readonly number[] = [`);
		values.forEach((value, i) => out.push(`\t${value}, // ${i} ${movements[i][0]}`));
		out.push("];", "");
	}
	return out.join("\n");
}

function generateEnums() {
	const lines = [...HEADER];
	for (const [tag, header, tsName] of ENUMS) {
		const members = parseEnum(readFileSync(join(repoRoot, header), "latin1"), tag);
		lines.push(`// C provenance: enum ${tag} (${header})`);
		lines.push(`export enum ${tsName} {`);
		for (const [name, value] of members) {
			lines.push(`\t${name} = ${value},`);
		}
		lines.push("}", "");
	}
	return lines.join("\n");
}

export function generate() {
	return {
		"c-aircraft-database.ts": generateAircraftDatabase(),
		"c-constants.ts": generateConstants(),
		"c-guide-database.ts": generateGuideDatabase(),
		"c-route-biasing-database.ts": generateRouteBiasingDatabase(),
		"c-waypoint-database.ts": generateWaypointDatabase(),
		"c-enums.ts": generateEnums(),
		"c-group-database.ts": generateGroupDatabase(),
		"c-keysite-database.ts": generateKeysiteDatabase(),
		"c-task-database.ts": generateTaskDatabase(),
	};
}

const isMain = process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1];

if (isMain) {
	const check = process.argv.includes("--check");
	let drift = false;
	for (const [name, text] of Object.entries(generate())) {
		const file = join(generatedDir, name);
		if (check) {
			let current = "";
			try {
				current = readFileSync(file, "utf8");
			} catch {
				// a missing file is drift
			}
			if (current !== text) {
				console.error(`${relative(projectRoot, file)} is out of date with the EECH C sources; run npm run gen:c`);
				drift = true;
			}
		} else {
			writeFileSync(file, text);
			console.log(`wrote ${relative(projectRoot, file)}`);
		}
	}
	if (drift) {
		process.exit(1);
	}
	if (check) {
		console.log("generated sources match the EECH C sources");
	}
}
