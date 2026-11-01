/*
 * TEST-ONLY original route-geometry characterisation (EEDCS-50, D3). Compiled only with the `route-fixture` feature; never part of the shipping module.
 *
 * WHAT IT OBSERVES. The campaign's natural boot reads every airfield population (popread.c) and, for each airfield's landing, take-off and holding routes, runs the
 * ORIGINAL parser parse_waypoint_routes_from_object (routegen.c) over the raw line-face object the DLL's own model loader read from 3dobjs.*. This fixture records, in
 * order and without parsing anything itself, what that real parser reads and produces:
 *   - each route-object lookup popread makes (airfield scene, route sub-object kind, found or not): the semantic identity of the route (the compiled scene and
 *     sub-object names), as opposed to the legacy raw object number the lookup returns (kept only as a diagnostic word, `legacy_object`);
 *   - per parser call: its arguments (the matching slots as bit patterns), the raw object's counts and bounds, every node's raw stored int16 point and the float position
 *     the parser decoded from it, every line face the first pass reads (surface, face, face-local and resolved node indices, the surface colour and flags), and the tables
 *     the parser leaves: the totals, every final node (tree depth, tree number, node index, flags, reference count), every parent and child link by node index, the
 *     starting nodes, and the per-depth position and offsets (the product the airfield code consumes).
 * Everything is a value the original statements already read or computed (build/patches.rs, ROUTE_FIXTURE_PATCHES): the hooks add calls, they change no original
 * statement, argument, order or value, do no floating-point arithmetic (floats are copied as bit patterns), draw no random number and store no pointer. The parser's own
 * node numbers are indices into its node array; they are the only identities in the report besides the compiled scene and sub-object names.
 *
 * WHAT IT CANNOT SHOW. It does not reconstruct what the parser did not run: a route object that was never looked up or parsed in this boot is simply absent, and an empty
 * report says "no calls", which is NOT a measured original. A table that fills, a hook outside an open call, a call that never ends and a call that begins inside another
 * are each counted and make the summary say `complete=0`. A natural boot in which no call was recorded is `complete=0` as well. Reaching a route object through a boot is
 * not proof that every route object of every installation was parsed.
 *
 * It runs inside the engine entry (so it sees EECH's round-toward-zero mode) and changes nothing it observes.
 */

#ifdef EECH_ROUTE_FIXTURE

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_route_hooks.h"

#define RF_CALLS 1024
#define RF_LOOKUPS 4096
#define RF_SLOTS 8192
#define RF_NODES 32768
#define RF_FACES 32768
#define RF_FINALS 32768
#define RF_LINKS 131072
#define RF_STARTS 8192
#define RF_DEPTHS 8192
#define RF_OFFSETS 65536

struct rf_lookup
{
	int scene, kind, found, legacy;
};

struct rf_node
{
	int index;
	int raw[3];
	uint32_t decoded[3];
};

struct rf_face
{
	int surface, face, local[2], point[2], surface_points;
	unsigned int flags, surface_faces;
	int colour[3], polygons;
};

struct rf_final
{
	int index, depth, tree, node_index, primary, start, possible_start, references, in_route, processed, children, parents;
};

struct rf_link
{
	int node, parent_link, slot, target;
};

struct rf_start
{
	int slot, node;
};

struct rf_depth
{
	int depth, count, first_offset, stored_offsets;
	uint32_t position[3];
};

struct rf_call
{
	int object, slots, slot_first, slot_stored, lookup;
	int points, surfaces, faces;
	uint32_t bounds[6];
	int ended, interrupted;
	int node_first, node_count, face_first, face_count, final_first, final_count, link_first, link_count, start_first, start_count, depth_first, depth_count;
	int dropped_slots, dropped_nodes, dropped_faces, dropped_finals, dropped_links, dropped_starts, dropped_depths, dropped_offsets;
	int total_nodes, total_starts, total_depths, maximum_depth, have_totals;
};

static struct rf_call rf_calls[RF_CALLS];
static struct rf_lookup rf_lookups[RF_LOOKUPS];
static uint32_t rf_slot_bits[RF_SLOTS][3];
static struct rf_node rf_nodes[RF_NODES];
static struct rf_face rf_faces[RF_FACES];
static struct rf_final rf_finals[RF_FINALS];
static struct rf_link rf_links[RF_LINKS];
static struct rf_start rf_starts[RF_STARTS];
static struct rf_depth rf_depths[RF_DEPTHS];
static uint32_t rf_offset_bits[RF_OFFSETS][3];

static int rf_call_count, rf_lookup_count, rf_slot_count, rf_node_count, rf_face_count, rf_final_count, rf_link_count, rf_start_count, rf_depth_count, rf_offset_count;
static int rf_dropped_calls, rf_dropped_lookups, rf_orphans, rf_interrupted_calls;
/* the open call, or NULL; `rf_suppressed` is set while a begun call did not fit in the table (its hooks are then expected and counted nowhere) */
static struct rf_call *rf_open_call;
static int rf_suppressed;

static uint32_t bits_of (const float *value)
{
	uint32_t word;
	memcpy (&word, value, sizeof (word));
	return word;
}

void eech_rf_lookup (int scene, int kind, int found, int legacy_object)
{
	if (rf_lookup_count >= RF_LOOKUPS)
	{
		rf_dropped_lookups++;
		return;
	}
	rf_lookups[rf_lookup_count].scene = scene;
	rf_lookups[rf_lookup_count].kind = kind;
	rf_lookups[rf_lookup_count].found = found;
	rf_lookups[rf_lookup_count].legacy = legacy_object;
	rf_lookup_count++;
}

void eech_rf_begin (int object_index, int number_of_slots, const void *slots, const void *object)
{
	const struct OBJECT_3D *raw = (const struct OBJECT_3D *) object;
	struct rf_call *call;
	const float *bound;
	int i;

	if (rf_open_call)
	{
		rf_open_call->interrupted = 1;
		rf_interrupted_calls++;
		rf_open_call = NULL;
	}
	rf_suppressed = 0;
	if (rf_call_count >= RF_CALLS)
	{
		rf_dropped_calls++;
		rf_suppressed = 1;
		return;
	}
	call = &rf_calls[rf_call_count++];
	memset (call, 0, sizeof (*call));
	rf_open_call = call;
	call->object = object_index;
	call->slots = number_of_slots;
	call->slot_first = rf_slot_count;
	call->lookup = -1;
	for (i = rf_lookup_count - 1; i >= 0; i--)
	{
		if (rf_lookups[i].found && rf_lookups[i].legacy == object_index)
		{
			call->lookup = i;
			break;
		}
	}
	for (i = 0; slots && i < number_of_slots; i++)
	{
		const float *v = ((const float *) slots) + 3 * i;
		if (rf_slot_count >= RF_SLOTS)
		{
			call->dropped_slots++;
			continue;
		}
		rf_slot_bits[rf_slot_count][0] = bits_of (v);
		rf_slot_bits[rf_slot_count][1] = bits_of (v + 1);
		rf_slot_bits[rf_slot_count][2] = bits_of (v + 2);
		rf_slot_count++;
		call->slot_stored++;
	}
	call->points = raw->number_of_points;
	call->surfaces = raw->number_of_surfaces;
	call->faces = raw->number_of_faces;
	bound = (const float *) &raw->bounding_box;
	for (i = 0; i < 6; i++)
	{
		call->bounds[i] = bits_of (bound + i);
	}
	call->node_first = rf_node_count;
	call->face_first = rf_face_count;
	call->final_first = rf_final_count;
	call->link_first = rf_link_count;
	call->start_first = rf_start_count;
	call->depth_first = rf_depth_count;
}

#define OPEN_OR_RETURN(call) \
	struct rf_call *call = rf_open_call; \
	if (!call) \
	{ \
		if (!rf_suppressed) \
		{ \
			rf_orphans++; \
		} \
		return; \
	}

void eech_rf_node (int index, const void *point, const void *position)
{
	OPEN_OR_RETURN (call)
	const struct OBJECT_SHORT_3D_POINT *p = (const struct OBJECT_SHORT_3D_POINT *) point;
	const float *v = (const float *) position;
	if (rf_node_count >= RF_NODES)
	{
		call->dropped_nodes++;
		return;
	}
	rf_nodes[rf_node_count].index = index;
	rf_nodes[rf_node_count].raw[0] = p->x;
	rf_nodes[rf_node_count].raw[1] = p->y;
	rf_nodes[rf_node_count].raw[2] = p->z;
	rf_nodes[rf_node_count].decoded[0] = bits_of (v);
	rf_nodes[rf_node_count].decoded[1] = bits_of (v + 1);
	rf_nodes[rf_node_count].decoded[2] = bits_of (v + 2);
	rf_node_count++;
	call->node_count++;
}

void eech_rf_face (int surface, int face, int local0, int local1, int point0, int point1, const void *surface_description, int surface_points)
{
	OPEN_OR_RETURN (call)
	const struct FACE_SURFACE_DESCRIPTION *s = (const struct FACE_SURFACE_DESCRIPTION *) surface_description;
	if (rf_face_count >= RF_FACES)
	{
		call->dropped_faces++;
		return;
	}
	rf_faces[rf_face_count].surface = surface;
	rf_faces[rf_face_count].face = face;
	rf_faces[rf_face_count].local[0] = local0;
	rf_faces[rf_face_count].local[1] = local1;
	rf_faces[rf_face_count].point[0] = point0;
	rf_faces[rf_face_count].point[1] = point1;
	rf_faces[rf_face_count].surface_points = surface_points;
	rf_faces[rf_face_count].flags = s->surface_flags;
	rf_faces[rf_face_count].surface_faces = s->number_of_faces;
	rf_faces[rf_face_count].colour[0] = s->red;
	rf_faces[rf_face_count].colour[1] = s->green;
	rf_faces[rf_face_count].colour[2] = s->blue;
	rf_faces[rf_face_count].polygons = s->polygons;
	rf_face_count++;
	call->face_count++;
}

void eech_rf_final_totals (int node_total, int start_total, int depth_total, int maximum_depth)
{
	OPEN_OR_RETURN (call)
	call->total_nodes = node_total;
	call->total_starts = start_total;
	call->total_depths = depth_total;
	call->maximum_depth = maximum_depth;
	call->have_totals = 1;
}

void eech_rf_final_node (int index, int depth, int tree, int node_index, int primary, int start, int possible_start, int references, int in_route, int processed,
	int children, int parents)
{
	OPEN_OR_RETURN (call)
	struct rf_final *f;
	if (rf_final_count >= RF_FINALS)
	{
		call->dropped_finals++;
		return;
	}
	f = &rf_finals[rf_final_count++];
	f->index = index;
	f->depth = depth;
	f->tree = tree;
	f->node_index = node_index;
	f->primary = primary;
	f->start = start;
	f->possible_start = possible_start;
	f->references = references;
	f->in_route = in_route;
	f->processed = processed;
	f->children = children;
	f->parents = parents;
	call->final_count++;
}

void eech_rf_final_link (int node, int parent_link, int slot, int target)
{
	OPEN_OR_RETURN (call)
	if (rf_link_count >= RF_LINKS)
	{
		call->dropped_links++;
		return;
	}
	rf_links[rf_link_count].node = node;
	rf_links[rf_link_count].parent_link = parent_link;
	rf_links[rf_link_count].slot = slot;
	rf_links[rf_link_count].target = target;
	rf_link_count++;
	call->link_count++;
}

void eech_rf_final_start (int slot, int node)
{
	OPEN_OR_RETURN (call)
	if (rf_start_count >= RF_STARTS)
	{
		call->dropped_starts++;
		return;
	}
	rf_starts[rf_start_count].slot = slot;
	rf_starts[rf_start_count].node = node;
	rf_start_count++;
	call->start_count++;
}

void eech_rf_final_position (int depth, int count, const void *position, const void *offsets)
{
	OPEN_OR_RETURN (call)
	struct rf_depth *d;
	const float *v = (const float *) position;
	const float *o = (const float *) offsets;
	int i;
	if (rf_depth_count >= RF_DEPTHS)
	{
		call->dropped_depths++;
		return;
	}
	d = &rf_depths[rf_depth_count++];
	d->depth = depth;
	d->count = count;
	d->position[0] = bits_of (v);
	d->position[1] = bits_of (v + 1);
	d->position[2] = bits_of (v + 2);
	d->first_offset = rf_offset_count;
	d->stored_offsets = 0;
	for (i = 0; o && i < count; i++)
	{
		if (rf_offset_count >= RF_OFFSETS)
		{
			call->dropped_offsets++;
			continue;
		}
		rf_offset_bits[rf_offset_count][0] = bits_of (o + 3 * i);
		rf_offset_bits[rf_offset_count][1] = bits_of (o + 3 * i + 1);
		rf_offset_bits[rf_offset_count][2] = bits_of (o + 3 * i + 2);
		rf_offset_count++;
		d->stored_offsets++;
	}
	call->depth_count++;
}

void eech_rf_end (void)
{
	OPEN_OR_RETURN (call)
	call->ended = 1;
	rf_open_call = NULL;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* reports: `key=value` text lines; a report that does not fit is refused, never cut short */

struct out
{
	char *text;
	int capacity, used, overflow;
};

static void emit (struct out *out, const char *format, ...) __attribute__ ((format (printf, 2, 3)));

static void emit (struct out *out, const char *format, ...)
{
	va_list args;
	int n;
	if (out->overflow)
	{
		return;
	}
	va_start (args, format);
	n = vsnprintf (out->text + out->used, (size_t) (out->capacity - out->used), format, args);
	va_end (args);
	if (n < 0 || n >= out->capacity - out->used)
	{
		out->overflow = 1;
		return;
	}
	out->used += n;
}

static const char *scene_name (int scene)
{
	if (scene > OBJECT_3D_INVALID_OBJECT_INDEX && scene < OBJECT_3D_LAST && object_3d_scene_names[scene])
	{
		return object_3d_scene_names[scene];
	}
	return "-";
}

static const char *kind_name (int kind)
{
	if (kind >= 0 && kind < OBJECT_3D_SUB_OBJECT_LAST && object_3d_subobject_names[kind])
	{
		return object_3d_subobject_names[kind];
	}
	return "-";
}

static int call_dropped (const struct rf_call *c)
{
	return c->dropped_slots + c->dropped_nodes + c->dropped_faces + c->dropped_finals + c->dropped_links + c->dropped_starts + c->dropped_depths + c->dropped_offsets;
}

/* a call is a measured original only if it ended, was not cut by another call, lost no row, and recorded the totals and every row its totals promise */
static int call_complete (const struct rf_call *c)
{
	return c->ended && !c->interrupted && c->have_totals && call_dropped (c) == 0 && c->node_count == c->points && c->final_count == c->total_nodes &&
		c->start_count == c->total_starts && c->depth_count == c->total_depths;
}

int eech_route_summary_run (char *text, int capacity)
{
	struct out out;
	int i, incomplete = 0;
	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	for (i = 0; i < rf_call_count; i++)
	{
		if (!call_complete (&rf_calls[i]))
		{
			incomplete++;
		}
	}
	emit (&out, "route-summary calls=%d call_capacity=%d dropped_calls=%d lookups=%d lookup_capacity=%d dropped_lookups=%d orphans=%d interrupted=%d open=%d incomplete_calls=%d\n",
		rf_call_count, RF_CALLS, rf_dropped_calls, rf_lookup_count, RF_LOOKUPS, rf_dropped_lookups, rf_orphans, rf_interrupted_calls, rf_open_call ? 1 : 0, incomplete);
	emit (&out, "route-pools slots=%d/%d nodes=%d/%d faces=%d/%d finals=%d/%d links=%d/%d starts=%d/%d depths=%d/%d offsets=%d/%d\n", rf_slot_count, RF_SLOTS, rf_node_count, RF_NODES,
		rf_face_count, RF_FACES, rf_final_count, RF_FINALS, rf_link_count, RF_LINKS, rf_start_count, RF_STARTS, rf_depth_count, RF_DEPTHS, rf_offset_count, RF_OFFSETS);
	for (i = 0; i < rf_lookup_count; i++)
	{
		emit (&out, "route-lookup index=%d scene=%d scene_name=%s kind=%d kind_name=%s found=%d legacy_object=%d\n", i, rf_lookups[i].scene, scene_name (rf_lookups[i].scene),
			rf_lookups[i].kind, kind_name (rf_lookups[i].kind), rf_lookups[i].found, rf_lookups[i].legacy);
	}
	/* a boot that recorded no call has measured nothing */
	emit (&out, "route-complete value=%d\n", (rf_call_count > 0 && rf_dropped_calls == 0 && rf_dropped_lookups == 0 && rf_orphans == 0 && rf_interrupted_calls == 0 && !rf_open_call && incomplete == 0) ? 1 : 0);
	return out.overflow ? 1 : 0;
}

int eech_route_call_run (int index, char *text, int capacity)
{
	struct out out;
	const struct rf_call *c;
	int i, j;
	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	if (index < 0 || index >= rf_call_count)
	{
		return 1;
	}
	c = &rf_calls[index];
	emit (&out, "route-call index=%d legacy_object=%d lookup=%d slots=%d slots_stored=%d points=%d surfaces=%d faces=%d ended=%d interrupted=%d complete=%d\n", index, c->object,
		c->lookup, c->slots, c->slot_stored, c->points, c->surfaces, c->faces, c->ended, c->interrupted, call_complete (c));
	if (c->lookup >= 0)
	{
		emit (&out, "route-identity index=%d scene=%d scene_name=%s kind=%d kind_name=%s\n", index, rf_lookups[c->lookup].scene, scene_name (rf_lookups[c->lookup].scene),
			rf_lookups[c->lookup].kind, kind_name (rf_lookups[c->lookup].kind));
	}
	else
	{
		emit (&out, "route-identity index=%d unknown=1\n", index);
	}
	emit (&out, "route-bounds index=%d bits=%08x,%08x,%08x,%08x,%08x,%08x\n", index, c->bounds[0], c->bounds[1], c->bounds[2], c->bounds[3], c->bounds[4], c->bounds[5]);
	emit (&out, "route-dropped index=%d slots=%d nodes=%d faces=%d finals=%d links=%d starts=%d depths=%d offsets=%d\n", index, c->dropped_slots, c->dropped_nodes, c->dropped_faces,
		c->dropped_finals, c->dropped_links, c->dropped_starts, c->dropped_depths, c->dropped_offsets);
	emit (&out, "route-totals index=%d have=%d nodes=%d starts=%d depths=%d maximum_depth=%d rows_nodes=%d rows_faces=%d rows_finals=%d rows_links=%d rows_starts=%d rows_depths=%d\n", index,
		c->have_totals, c->total_nodes, c->total_starts, c->total_depths, c->maximum_depth, c->node_count, c->face_count, c->final_count, c->link_count, c->start_count, c->depth_count);
	for (i = 0; i < c->slot_stored; i++)
	{
		const uint32_t *b = rf_slot_bits[c->slot_first + i];
		emit (&out, "route-slot index=%d slot=%d bits=%08x,%08x,%08x\n", index, i, b[0], b[1], b[2]);
	}
	for (i = 0; i < c->node_count; i++)
	{
		const struct rf_node *n = &rf_nodes[c->node_first + i];
		emit (&out, "route-node index=%d node=%d raw=%d,%d,%d bits=%08x,%08x,%08x\n", index, n->index, n->raw[0], n->raw[1], n->raw[2], n->decoded[0], n->decoded[1], n->decoded[2]);
	}
	for (i = 0; i < c->face_count; i++)
	{
		const struct rf_face *f = &rf_faces[c->face_first + i];
		emit (&out, "route-face index=%d surface=%d face=%d local=%d,%d point=%d,%d colour=%d,%d,%d polygons=%d flags=%08x surface_points=%d surface_faces=%u\n", index, f->surface, f->face,
			f->local[0], f->local[1], f->point[0], f->point[1], f->colour[0], f->colour[1], f->colour[2], f->polygons, f->flags, f->surface_points, f->surface_faces);
	}
	for (i = 0; i < c->final_count; i++)
	{
		const struct rf_final *f = &rf_finals[c->final_first + i];
		emit (&out, "route-final index=%d node=%d depth=%d tree=%d node_index=%d primary=%d start=%d possible_start=%d references=%d in_route=%d processed=%d children=%d parents=%d\n", index,
			f->index, f->depth, f->tree, f->node_index, f->primary, f->start, f->possible_start, f->references, f->in_route, f->processed, f->children, f->parents);
	}
	for (i = 0; i < c->link_count; i++)
	{
		const struct rf_link *l = &rf_links[c->link_first + i];
		emit (&out, "route-link index=%d node=%d link=%s slot=%d target=%d\n", index, l->node, l->parent_link ? "parent" : "child", l->slot, l->target);
	}
	for (i = 0; i < c->start_count; i++)
	{
		const struct rf_start *s = &rf_starts[c->start_first + i];
		emit (&out, "route-start index=%d slot=%d node=%d\n", index, s->slot, s->node);
	}
	for (i = 0; i < c->depth_count; i++)
	{
		const struct rf_depth *d = &rf_depths[c->depth_first + i];
		emit (&out, "route-position index=%d depth=%d count=%d stored=%d bits=%08x,%08x,%08x\n", index, d->depth, d->count, d->stored_offsets, d->position[0], d->position[1], d->position[2]);
		for (j = 0; j < d->stored_offsets; j++)
		{
			const uint32_t *b = rf_offset_bits[d->first_offset + j];
			emit (&out, "route-offset index=%d depth=%d slot=%d bits=%08x,%08x,%08x\n", index, d->depth, j, b[0], b[1], b[2]);
		}
	}
	emit (&out, "route-end index=%d\n", index);
	return out.overflow ? 1 : 0;
}

#endif
