#!/usr/bin/env node
//
// Generates the C reference harness sources from the original EECH C:
//
//   build/c-reference/project.h        the reduced project.h that the original
//                                       translation units include: the harness
//                                       environment, verbatim fragments and
//                                       WHOLE original headers
//   build/c-reference/eech_extracted.c verbatim copies of original functions and
//                                       definitions that live in files too large
//                                       to compile whole
//
// Original translation units that are compiled unchanged are listed in
// REAL_TRANSLATION_UNITS; build.mjs compiles them against the generated
// project.h.
//
// Each fragment is preceded by a #line directive to its origin. Extraction
// fails loudly if an original definition cannot be found.
//

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(here, "..");
const repoRoot = join(projectRoot, "..");

export const AP_SOURCE = "aphavoc/source";

// Original EECH translation units compiled unchanged into the harness.
export const REAL_TRANSLATION_UNITS = [
	"aphavoc/source/entity/special/group/gp_dbase.c",
	"aphavoc/source/entity/special/group/gp_float.c",
	"aphavoc/source/entity/special/group/gp_int.c",
	"aphavoc/source/entity/special/group/gp_list.c",
	"aphavoc/source/entity/special/group/gp_ptr.c",
	"aphavoc/source/entity/special/group/gp_updt.c",
	"aphavoc/source/entity/special/group/gp_vec3d.c",
	"aphavoc/source/entity/special/update/up_list.c",
	"aphavoc/source/entity/special/update/up_msgs.c",
];

// Contents of the generated project.h, in order.
// kind "include": a whole original header (path relative to aphavoc/source)
export const PROJECT_H = [
	{ kind: "local", name: "eech_harness_env.h" },
	{ kind: "struct", name: "VEC3D", file: "modules/maths/vector.h" },
	{ kind: "regex", pattern: "\\ntypedef struct VEC3D vec3d;", file: "modules/maths/vector.h" },
	{ kind: "define", name: "bound", file: "modules/maths/miscmath.h" },
	{ kind: "define", name: "METRE", file: "modules/maths/constant.h" },
	{ kind: "define", name: "KILOMETRE", file: "modules/maths/constant.h" },
	{ kind: "regex", pattern: "\\nextern float\\n\\tsystem_delta_time,\\n\\tsystem_one_over_delta_time;", file: "modules/system/time.h" },
	{ kind: "regex", pattern: "\\nextern int\\n\\tlocked_frame_rate;", file: "modules/system/time.h" },
	{ kind: "regex", pattern: "\\nextern void set_manual_delta_time \\(float delta_time\\);", file: "modules/system/time.h" },
	{ kind: "define", name: "get_delta_time", file: "modules/system/time.h" },
	{ kind: "enum", name: "COMMS_MODEL_TYPES", file: "aphavoc/source/comms/comms.h" },
	{ kind: "regex", pattern: "\\ntypedef enum COMMS_MODEL_TYPES comms_model_types;", file: "aphavoc/source/comms/comms.h" },
	{ kind: "enum", name: "COMMS_DATA_FLOW_TYPES", file: "aphavoc/source/comms/comms.h" },
	{ kind: "regex", pattern: "\\ntypedef enum COMMS_DATA_FLOW_TYPES comms_data_flow_types;", file: "aphavoc/source/comms/comms.h" },
	{ kind: "regex", pattern: "\\nextern comms_model_types\\n\\s*system_comms_model;", file: "aphavoc/source/comms/comms.h" },
	{ kind: "regex", pattern: "\\nextern comms_data_flow_types\\n\\s*system_comms_data_flow;", file: "aphavoc/source/comms/comms.h" },
	{ kind: "define", name: "get_comms_model", file: "aphavoc/source/comms/comms.h" },
	{ kind: "define", name: "get_comms_data_flow", file: "aphavoc/source/comms/comms.h" },
	{ kind: "enum", name: "MAP_LAYER_CONTROL_TYPES", file: "aphavoc/source/ui_menu/ingame/common/map.h" },
	{ kind: "define", name: "MAP_LAYER_CONTROL_NONE", file: "aphavoc/source/ui_menu/ingame/common/map.h" },
	{ kind: "enum", name: "MAP_ICON_TYPE", file: "aphavoc/source/ui_menu/ingame/common/map.h" },
	{ kind: "include", name: "entity/system/en_types/en_types.h" },
	{ kind: "include", name: "entity/system/en_main/en_heap.h" },
	{ kind: "include", name: "entity/system/en_msgs/en_msgs.h" },
	{ kind: "include", name: "entity/system/en_funcs/en_funcs.h" },
	{ kind: "include", name: "entity/system/en_comms/en_comms.h" },
	{ kind: "include", name: "entity/system/en_debug/en_dbmsg.h" },
	{ kind: "include", name: "ai_extrn.h" },
	{ kind: "include", name: "entity/special/update/up_update.h" },
	{ kind: "include", name: "entity/special/division/division.h" },
	{ kind: "include", name: "entity/special/group/group.h" },
	{ kind: "include", name: "entity/tacview/tacview.h" },
	{ kind: "regex", pattern: "\\nextern entity\\n\\t\\*session_entity;", file: "aphavoc/source/entity/special/session/session.h" },
	{ kind: "define", name: "get_session_entity", file: "aphavoc/source/entity/special/session/session.h" },
	{ kind: "prototype", name: "get_local_force_entity", file: "aphavoc/source/entity/special/force/force.h" },
	{ kind: "prototype", name: "add_group_type_to_force_info", file: "aphavoc/source/entity/special/force/force.h" },
	{ kind: "prototype", name: "remove_group_type_from_force_info", file: "aphavoc/source/entity/special/force/force.h" },
	{ kind: "prototype", name: "get_closest_keysite", file: "aphavoc/source/entity/special/keysite/keysite.h" },
	{ kind: "local", name: "eech_harness_decls.h" },
];

// Contents of the generated eech_extracted.c, in order.
export const EXTRACTED_C = [
	{ kind: "raw", text: '#include "project.h"\n\n#define DEBUG_MODULE 0\n\n#define LANDING_DEBUG 0\n' },

	// modules/system/time.c: the delta time the entity update loop reads and overrides
	{ kind: "define", name: "DELTA_TIME_HISTORY_SIZE", file: "modules/system/time.c" },
	{ kind: "regex", pattern: "\\nfloat\\n\\tsystem_delta_time_average = 0\\.1,\\n\\tsystem_delta_time = 0\\.1,\\n\\tsystem_one_over_delta_time = 10\\.0;", file: "modules/system/time.c" },
	{ kind: "regex", pattern: "\\nint\\n\\tlocked_frame_rate = FALSE;", file: "modules/system/time.c" },
	{ kind: "regex", pattern: "\\nstatic float\\n\\tsystem_delta_time_history\\[DELTA_TIME_HISTORY_SIZE\\];", file: "modules/system/time.c" },
	{ kind: "regex", pattern: "\\nstatic int\\n\\tsystem_delta_time_history_position;", file: "modules/system/time.c" },
	{ kind: "function", name: "set_manual_delta_time", signature: "void set_manual_delta_time (float delta_time)", file: "modules/system/time.c" },

	// modules/maths/range.c
	{ kind: "function", name: "get_2d_range", signature: "float get_2d_range (const vec3d *v1, const vec3d *v2)", file: "modules/maths/range.c" },
	{ kind: "function", name: "get_approx_2d_range", signature: "float get_approx_2d_range (const vec3d *v1, const vec3d *v2)", file: "modules/maths/range.c" },

	// entity system defaults and runtime
	{ kind: "function", name: "default_set_entity_int_value", signature: "static void default_set_entity_int_value (entity *en, int_types type, int value)", file: "aphavoc/source/entity/system/en_funcs/en_int.c" },
	{ kind: "function", name: "default_set_entity_float_value", signature: "static void default_set_entity_float_value (entity *en, float_types type, float value)", file: "aphavoc/source/entity/system/en_funcs/en_float.c" },
	{ kind: "function", name: "notify_local_entity", signature: "int notify_local_entity (entity_messages message, entity *receiver, entity *sender, ...)", file: "aphavoc/source/entity/system/en_msgs/en_msgs.c" },
	{ kind: "function", name: "default_message_response", signature: "static int default_message_response (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/system/en_msgs/en_msgs.c" },
	{ kind: "function", name: "insert_local_entity_into_parents_child_list", signature: "void insert_local_entity_into_parents_child_list (entity *en, list_types type, entity *parent, entity *pred)", file: "aphavoc/source/entity/system/en_funcs/en_list.c" },
	{ kind: "function", name: "delete_local_entity_from_parents_child_list", signature: "void delete_local_entity_from_parents_child_list (entity *en, list_types type)", file: "aphavoc/source/entity/system/en_funcs/en_list.c" },

	// entity/special/update/up_update.c: the entity update loop
	{ kind: "regex", pattern: "\\nentity\\n\\t\\*update_entity = NULL,\\n\\t\\*update_succ = NULL;", file: "aphavoc/source/entity/special/update/up_update.c" },
	{ kind: "regex", pattern: "\\nunsigned int\\n\\tmoved_entities[^;]*;", file: "aphavoc/source/entity/special/update/up_update.c" },
	{ kind: "function", name: "update_client_server_entities", signature: "void update_client_server_entities (void)", file: "aphavoc/source/entity/special/update/up_update.c" },
	{ kind: "function", name: "set_entity_update_frame_rate", signature: "int set_entity_update_frame_rate (int frame_rate)", file: "aphavoc/source/entity/special/update/up_update.c" },

	// campaign functions (slice 1)
	{ kind: "function", name: "get_local_force_entity", signature: "entity *get_local_force_entity (entity_sides side)", file: "aphavoc/source/entity/special/force/force.c" },
	{
		kind: "function",
		name: "get_closest_keysite",
		signature: "entity *get_closest_keysite (entity_sub_types type, entity_sides side, vec3d *pos, float min_range, float *actual_range, int outside_of_range, entity *exclude_keysite)",
		file: "aphavoc/source/entity/special/keysite/keysite.c",
	},
	{ kind: "function", name: "assess_group_supplies", signature: "void assess_group_supplies (entity *en)", file: "aphavoc/source/entity/special/group/group.c" },

	// group link/unlink parent responses (gp_msgs.c is otherwise not compiled)
	{ kind: "function", name: "response_to_link_parent", signature: "static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
	{ kind: "function", name: "response_to_unlink_parent", signature: "static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
	{
		kind: "wrap",
		prologue: "void harness_overload_group_link_parent_responses (void)\n{",
		epilogue: "}",
		parts: [
			{ kind: "regex", pattern: "\\n\\tmessage_responses\\[ENTITY_TYPE_GROUP\\]\\[ENTITY_MESSAGE_LINK_PARENT\\][^;]*;", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
			{ kind: "regex", pattern: "\\n\\tmessage_responses\\[ENTITY_TYPE_GROUP\\]\\[ENTITY_MESSAGE_UNLINK_PARENT\\][^;]*;", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
		],
	},
	{
		kind: "raw",
		text: "/* C defaults for the update entity: its link responses are only overloaded under DEBUG_MODULE (up_msgs.c) */\nvoid harness_default_update_link_responses (void)\n{\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_CHILD] = default_message_response;\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_PARENT] = default_message_response;\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_UNLINK_PARENT] = default_message_response;\n}\n\nvoid (*harness_default_set_entity_int_value) (entity *en, int_types type, int value) = default_set_entity_int_value;\n\nvoid (*harness_default_set_entity_float_value) (entity *en, float_types type, float value) = default_set_entity_float_value;\n",
	},
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

function readSource(file) {
	return readFileSync(join(repoRoot, file), "latin1").replace(/\r\n/g, "\n");
}

export function extractOne(spec) {
	if (spec.kind === "local") {
		return `#include "${spec.name}"\n`;
	}
	if (spec.kind === "include") {
		return `#include "${spec.name}"\n`;
	}
	if (spec.kind === "raw") {
		return `${spec.text}\n`;
	}
	if (spec.kind === "wrap") {
		return `${spec.prologue}\n${spec.parts.map(extractOne).join("")}${spec.epilogue}\n`;
	}
	const text = readSource(spec.file);
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
	} else if (spec.kind === "define") {
		const match = new RegExp(`\\n#define ${spec.name}\\b[^\\n]*`).exec(text);
		if (!match) {
			throw new Error(`#define ${spec.name} not found in ${spec.file}`);
		}
		start = match.index + 1;
		end = match.index + match[0].length;
	} else if (spec.kind === "prototype") {
		const match = new RegExp(`\\nextern [^\\n(]*\\b${spec.name} \\([^\\n]*\\);`).exec(text);
		if (!match) {
			throw new Error(`prototype ${spec.name} not found in ${spec.file}`);
		}
		start = match.index + 1;
		end = match.index + match[0].length;
	} else if (spec.kind === "regex") {
		const match = new RegExp(spec.pattern).exec(text);
		if (!match) {
			throw new Error(`/${spec.pattern}/ not found in ${spec.file}`);
		}
		start = match.index + (match[0].startsWith("\n") ? 1 : 0);
		end = match.index + match[0].length;
	} else {
		throw new Error(`unknown extract kind ${spec.kind}`);
	}
	return `#line ${lineOf(text, start)} "${spec.file}"\n${text.slice(start, end)}\n`;
}

const BANNER = "/* GENERATED by c-reference/extract.mjs - verbatim original EECH C. Do not edit. */\n";

export function generateProjectH() {
	return [BANNER, "#ifndef EECH_HARNESS_PROJECT_H", "#define EECH_HARNESS_PROJECT_H", "", ...PROJECT_H.map(extractOne), "#endif", ""].join("\n");
}

export function generateExtracted() {
	return [BANNER, ...EXTRACTED_C.map(extractOne)].join("\n");
}

export function writeGenerated(outDir) {
	mkdirSync(outDir, { recursive: true });
	writeFileSync(join(outDir, "project.h"), generateProjectH());
	writeFileSync(join(outDir, "eech_extracted.c"), generateExtracted());
}

const isMain = process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1];

if (isMain) {
	writeGenerated(join(projectRoot, "build", "c-reference"));
	console.log("wrote build/c-reference/project.h and eech_extracted.c");
}
