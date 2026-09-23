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
	emit("TASK_DATABASE_PRIMARY_TASK", "primary_task (TRUE 1, FALSE 0)", column("primary task").map((v) => booleanLiteral(v, "primary task")));
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
		"c-constants.ts": generateConstants(),
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
