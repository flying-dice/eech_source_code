#!/usr/bin/env node
//
// Copies original EECH C definitions verbatim into build/c-reference/eech_extracted.c
// so the harness executes the original code, not a re-implementation.
//
// Each extracted block is preceded by a #line directive, so compiler
// diagnostics and debuggers point at the original file and line.
//

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(here, "..");
const repoRoot = join(projectRoot, "..");

// kind: "function" (signature line through matching brace), "enum" (enum TAG { ... };),
// "struct" (struct TAG { ... };), "define" (single #define line)
export const EXTRACTS = [
	{ kind: "enum", name: "ENTITY_TYPES", file: "aphavoc/source/entity/system/en_types/en_types.h" },
	{ kind: "enum", name: "ENTITY_SIDES", file: "aphavoc/source/entity/system/en_types/en_side.h" },
	{ kind: "enum", name: "LIST_TYPES", file: "aphavoc/source/entity/system/en_funcs/en_list.h" },
	{ kind: "enum", name: "INT_TYPES", file: "aphavoc/source/entity/system/en_funcs/en_int.h" },
	{ kind: "enum", name: "FLOAT_TYPES", file: "aphavoc/source/entity/system/en_funcs/en_float.h" },
	{ kind: "enum", name: "VEC3D_TYPES", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.h" },
	{ kind: "enum", name: "PTR_TYPES", file: "aphavoc/source/entity/system/en_funcs/en_ptr.h" },
	{ kind: "enum", name: "ENTITY_MESSAGES", file: "aphavoc/source/entity/system/en_msgs/en_msgs.h" },
	{ kind: "enum", name: "ENTITY_SUB_TYPE_CARGO", file: "aphavoc/source/entity/system/en_types/en_sbtyp.h" },
	{ kind: "enum", name: "ENTITY_SUB_TYPE_GROUPS", file: "aphavoc/source/entity/system/en_types/en_sbtyp.h" },
	{ kind: "enum", name: "ENTITY_SUB_TYPE_KEYSITES", file: "aphavoc/source/entity/system/en_types/en_sbtyp.h" },
	{ kind: "enum", name: "GROUP_MODE_TYPES", file: "aphavoc/source/ai_extrn.h" },
	{ kind: "enum", name: "COMMS_MODEL_TYPES", file: "aphavoc/source/comms/comms.h" },
	{ kind: "enum", name: "RESUPPLY_SOURCE_TYPE", file: "aphavoc/source/entity/system/en_types/en_suply.h" },
	{ kind: "struct", name: "SUPPLY_TYPE", file: "aphavoc/source/entity/system/en_types/en_suply.h" },
	{ kind: "struct", name: "VEC3D", file: "modules/maths/vector.h" },
	{ kind: "define", name: "bound", file: "modules/maths/miscmath.h" },
	{ kind: "define", name: "METRE", file: "modules/maths/constant.h" },
	{ kind: "define", name: "KILOMETRE", file: "modules/maths/constant.h" },
	{ kind: "define", name: "FUEL_USAGE_ACCELERATOR", file: "aphavoc/source/entity/system/en_types/en_suply.h" },
	{ kind: "define", name: "AMMO_USAGE_ACCELERATOR", file: "aphavoc/source/entity/system/en_types/en_suply.h" },
	{ kind: "define", name: "DEBUG_SUPPLY", file: "aphavoc/source/entity/system/en_types/en_suply.h" },
	{ kind: "shim", name: "eech_shim_types" },
	{ kind: "function", name: "get_2d_range", signature: "float get_2d_range (const vec3d *v1, const vec3d *v2)", file: "modules/maths/range.c" },
	{ kind: "function", name: "get_approx_2d_range", signature: "float get_approx_2d_range (const vec3d *v1, const vec3d *v2)", file: "modules/maths/range.c" },
	{ kind: "function", name: "notify_local_entity", signature: "int notify_local_entity (entity_messages message, entity *receiver, entity *sender, ...)", file: "aphavoc/source/entity/system/en_msgs/en_msgs.c" },
	{ kind: "function", name: "get_local_force_entity", signature: "entity *get_local_force_entity (entity_sides side)", file: "aphavoc/source/entity/special/force/force.c" },
	{
		kind: "function",
		name: "get_closest_keysite",
		signature: "entity *get_closest_keysite (entity_sub_types type, entity_sides side, vec3d *pos, float min_range, float *actual_range, int outside_of_range, entity *exclude_keysite)",
		file: "aphavoc/source/entity/special/keysite/keysite.c",
	},
	{ kind: "function", name: "assess_group_supplies", signature: "void assess_group_supplies (entity *en)", file: "aphavoc/source/entity/special/group/group.c" },
];

function lineOf(text, offset) {
	return text.slice(0, offset).split("\n").length;
}

function extractBraced(text, start, what) {
	const open = text.indexOf("{", start);
	let depth = 0;
	for (let i = open; i < text.length; i++) {
		if (text[i] === "{") {
			depth++;
		} else if (text[i] === "}") {
			depth--;
			if (depth === 0) {
				return i + 1;
			}
		}
	}
	throw new Error(`unbalanced braces extracting ${what}`);
}

export function extractOne(spec) {
	const path = join(repoRoot, spec.file);
	const text = readFileSync(path, "latin1").replace(/\r\n/g, "\n");
	let start;
	let end;
	if (spec.kind === "function") {
		start = text.indexOf(`\n${spec.signature}\n`);
		if (start < 0) {
			throw new Error(`${spec.signature} not found in ${spec.file}`);
		}
		start += 1;
		end = extractBraced(text, start, spec.name);
	} else if (spec.kind === "enum" || spec.kind === "struct") {
		const match = new RegExp(`\\n${spec.kind} ${spec.name}\\s*\\n`).exec(text);
		if (!match) {
			throw new Error(`${spec.kind} ${spec.name} not found in ${spec.file}`);
		}
		start = match.index + 1;
		end = extractBraced(text, start, spec.name);
		if (text[end] !== ";") {
			throw new Error(`${spec.kind} ${spec.name} not terminated by ;`);
		}
		end += 1;
	} else {
		const match = new RegExp(`\\n#define ${spec.name}\\b[^\\n]*`).exec(text);
		if (!match) {
			throw new Error(`#define ${spec.name} not found in ${spec.file}`);
		}
		start = match.index + 1;
		end = match.index + match[0].length;
	}
	return `#line ${lineOf(text, start)} "${spec.file}"\n${text.slice(start, end)}\n`;
}

export function generateExtracted() {
	const parts = [
		"/* GENERATED by c-reference/extract.mjs - verbatim copies of original EECH C. Do not edit. */",
		"",
	];
	for (const spec of EXTRACTS) {
		if (spec.kind === "shim") {
			parts.push(`#include "${spec.name}.h"`, "");
		} else {
			parts.push(extractOne(spec));
		}
	}
	return parts.join("\n");
}

const isMain = process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1];

if (isMain) {
	const outDir = join(projectRoot, "build", "c-reference");
	mkdirSync(outDir, { recursive: true });
	writeFileSync(join(outDir, "eech_extracted.c"), generateExtracted());
	console.log("wrote build/c-reference/eech_extracted.c");
}
