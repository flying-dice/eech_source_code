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
// by a `// <ENUM_MEMBER>` banner comment and whose fields carry trailing
// `// <field>` comments (the layout of every EECH *_dbase.c file).
//
export function parseDatabaseColumn(source, enumPrefix, fieldComment) {
	const lines = source.split(/\r?\n/);
	const rows = [];
	let current;
	for (const line of lines) {
		const banner = new RegExp(`^\\s*//\\s*(${enumPrefix}[A-Z0-9_]+)\\s*$`).exec(line);
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
		"c-enums.ts": generateEnums(),
		"c-group-database.ts": generateGroupDatabase(),
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
