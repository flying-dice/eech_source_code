/*
 * TEST-ONLY original collision-model characterisation (EEDCS-50). Compiled only with the `model-fixture` feature; never part of the
 * shipping module.
 *
 * It asks the ORIGINAL compiled accessors of modules/3d about the models the retail data loaded, and reports what they answer:
 *
 *   get_object_3d_collision_object_geometry_triangle   the collision triangles, one call per depth, ascending, one buffer each
 *   get_object_3d_bounding_box_without_lines           the bounds the collision tests use
 *   get_object_3d_bounding_box, get_object_3d_radius   the other scene accessors the entity queries read
 *
 * The answers are the original's. Nothing here computes a triangle, a bound or a count the original did not return. Floats are
 * reported as their bit patterns (0x%08x), so no decimal round trip can lose anything.
 *
 * What this file DOES compute is a read-only SAFETY verdict: whether the original decoder can be called at all, and to which depth,
 * without reading memory the loader did not back. The decoder is not defensive. It reads obj->surfaces[0] before it looks at the
 * face count (a NULL surface list is a crash, the cause of the C1 patch for object index 0); at the end of the walk it reads
 * obj->surfaces[number_of_surfaces], ONE SURFACE PAST the object's own; and it trusts every face/point index. The verdict is made from
 * the loaded tables only, before any call, and is reported with its reason. It is not an expected answer: a model the verdict
 * refuses is reported refused (with its metadata), never passed and never "fixed". The verdict walks the faces with the surface
 * association the decoder assumes when no surface is empty; an object with an empty surface is refused because there the decoder's
 * association would differ.
 *
 * Backing the loader does not expose (the file-mapped and block sizes are static in 3dobjdb.c) is observed from the table itself:
 * the loader lays every non-custom object's faces / point list / surface points out back to back, so the distance from an object's
 * array to the next-higher array of the same kind is the object's declared extent. The last object of a block has no such
 * neighbour: its extent is unobservable and the object is refused (`capacity_unobservable`), not guessed. The surfaces are one
 * reallocated block of exactly the sum of every object's surfaces, whose end is known, so the one-past read is checked exactly.
 *
 * Pure observer: the entries below write no table, scene, object, pointer or model byte, draw no random number, make no
 * physical query beyond the accessors named, and touch no entity (the use count only reads each entity's shape through the
 * ordinary getter, whose default for a type without a shape is the invalid index). `eech_model_state_run` hashes every table
 * these entries read, so a driver can show before and after that nothing moved.
 */

#ifdef EECH_MODEL_FIXTURE

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

#define PAGE_SCENES 96
#define PAGE_TRIANGLES 256
#define USE_TYPE_SLOTS 4

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

typedef char vec3d_is_three_floats[(sizeof (vec3d) == 3 * sizeof (float)) ? 1 : -1];

/* ---------------------------------------------------------------------------------------------------------------------------- */

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

static void out_begin (struct out *out, char *text, int capacity)
{
	out->text = text;
	out->capacity = capacity;
	out->used = 0;
	out->overflow = 0;
	text[0] = 0;
}

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static void bounds_text (char *buffer, size_t capacity, const struct OBJECT_3D_BOUNDS *b)
{
	snprintf (buffer, capacity, "0x%08x,0x%08x,0x%08x,0x%08x,0x%08x,0x%08x", float_bits (b->xmin), float_bits (b->xmax), float_bits (b->ymin),
		float_bits (b->ymax), float_bits (b->zmin), float_bits (b->zmax));
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the loaded tables, and the extent the loader's layout makes observable */

static const char *scene_name (int scene)
{
	const char *name = object_3d_scene_names[scene];
	return name ? name : "-";
}

static int scene_in_range (int scene)
{
	return scene > OBJECT_3D_INVALID_OBJECT_INDEX && scene < OBJECT_3D_LAST;
}

/* the one reallocated surface block: its length is the sum of every object's surfaces, as the loader sized it */
static long surface_block_length (void)
{
	long sum = 0;
	int i;
	for (i = 1; i <= total_number_of_raw_3d_objects; i++)
	{
		sum += objects_3d_data[i].number_of_surfaces;
	}
	return sum;
}

enum field { F_FACES, F_POINTLIST, F_SURFPOINTS };

static const void *field_pointer (const object_3d *o, int field)
{
	switch (field)
	{
		case F_FACES: return o->faces;
		case F_POINTLIST: return o->object_faces_point_plain_list;
		default: return o->surface_points;
	}
}

static size_t field_element (int field)
{
	switch (field)
	{
		case F_FACES: return sizeof (object_3d_face);
		case F_POINTLIST: return sizeof (point_3d_short_reference);
		default: return sizeof (point_3d_plain_reference);
	}
}

static const char *field_name (int field)
{
	switch (field)
	{
		case F_FACES: return "faces";
		case F_POINTLIST: return "point_list";
		default: return "surface_points";
	}
}

/* elements from object `c`'s array to the next-higher array of the same kind among the other non-custom objects; -1 when there is no
   neighbour (unobservable), -2 when the distance is not a whole number of elements */
static long observed_capacity (int c, int field)
{
	uintptr_t base = (uintptr_t) field_pointer (&objects_3d_data[c], field);
	uintptr_t best = 0;
	int i;
	for (i = 1; i <= total_number_of_raw_3d_objects; i++)
	{
		const object_3d *o = &objects_3d_data[i];
		uintptr_t other;
		if (i == c || o->custom)
		{
			continue;
		}
		other = (uintptr_t) field_pointer (o, field);
		if (other > base && (best == 0 || other < best))
		{
			best = other;
		}
	}
	if (best == 0)
	{
		return -1;
	}
	if ((best - base) % field_element (field))
	{
		return -2;
	}
	return (long) ((best - base) / field_element (field));
}

struct analysis
{
	const char *status; /* safe, empty, or the refusal / absence class */
	const char *detail; /* which table the refusal concerns, or "-" */
	int collision_index;
	int number_of_points, number_of_faces, number_of_surfaces, custom;
	long surface_faces, polygon_surface_faces, nonpolygon_surfaces, zero_face_surfaces, surface_points_sum;
	long face_points_sum, fan_triangles;
	int face_points_min, face_points_max;
	long over_surface, over_points;
	long cap_faces, cap_point_list, cap_surface_points, max_surface_address;
	int end_read_backed;
};

static int analysis_probeable (const struct analysis *a)
{
	return !strcmp (a->status, "safe") || !strcmp (a->status, "empty");
}

/* the deepest depth the original may be asked: the end probe (depth == fan_triangles) only when the one-past surface read is backed */
static long analysis_max_depth (const struct analysis *a)
{
	return a->end_read_backed ? a->fan_triangles : a->fan_triangles - 1;
}

static void analyse (struct analysis *a, int scene)
{
	const object_3d *obj;
	long block_length;
	const face_surface_description *block, *block_end;
	int s;
	int c = objects_3d_scene_database[scene].collision_object_index;

	memset (a, 0, sizeof (*a));
	a->status = "safe";
	a->detail = "-";
	a->collision_index = c;
	a->face_points_min = -1;
	a->cap_faces = a->cap_point_list = a->cap_surface_points = a->max_surface_address = -3;

	if (c == -1)
	{
		a->status = "absent";
		return;
	}
	if (c == 0 && objects_3d_scene_database[scene].index == 0)
	{
		a->status = "no_scene";
		return;
	}
	if (c < 1 || c > total_number_of_raw_3d_objects)
	{
		a->status = "invalid_index";
		return;
	}

	obj = &objects_3d_data[c];
	a->number_of_points = obj->number_of_points;
	a->number_of_faces = obj->number_of_faces;
	a->number_of_surfaces = obj->number_of_surfaces;
	a->custom = obj->custom;

	/* the surfaces: the decoder reads surfaces[0] unconditionally */
	block = object_database_surfaces;
	block_length = surface_block_length ();
	if (!block || obj->number_of_surfaces < 1 || !obj->surfaces)
	{
		a->status = "null_surfaces";
		return;
	}
	block_end = block + block_length;
	if (obj->surfaces < block || obj->surfaces + obj->number_of_surfaces > block_end)
	{
		a->status = "surfaces_outside_block";
		return;
	}
	/* the one-past read of the end of the walk */
	a->end_read_backed = (obj->surfaces + obj->number_of_surfaces) < block_end;

	for (s = 0; s < obj->number_of_surfaces; s++)
	{
		const face_surface_description *surface = &obj->surfaces[s];
		a->surface_faces += surface->number_of_faces;
		if (surface->polygons)
		{
			a->polygon_surface_faces += surface->number_of_faces;
		}
		else
		{
			a->nonpolygon_surfaces++;
		}
		if (surface->number_of_faces == 0)
		{
			a->zero_face_surfaces++;
		}
		a->surface_points_sum += surface->number_of_points ? surface->number_of_points : 256;
	}

	if (obj->custom)
	{
		a->status = "custom_object";
		return;
	}
	if (obj->number_of_faces == 0 && a->surface_faces == 0)
	{
		a->status = "empty";
		a->end_read_backed = 1; /* the loop body never runs: only surfaces[0] is read */
		return;
	}
	if (obj->number_of_faces != a->surface_faces || obj->number_of_faces != a->polygon_surface_faces)
	{
		a->status = "face_mismatch";
		return;
	}
	if (a->zero_face_surfaces)
	{
		a->status = "zero_face_surface";
		return;
	}
	if (!obj->faces || !obj->object_faces_point_plain_list || !obj->surface_points || !obj->points)
	{
		a->status = "null_arrays";
		return;
	}

	/* the faces array: its extent must cover number_of_faces */
	a->cap_faces = observed_capacity (c, F_FACES);
	if (a->cap_faces < 0 || a->cap_faces < obj->number_of_faces)
	{
		a->status = a->cap_faces < 0 ? "capacity_unobservable" : "capacity_short";
		a->detail = field_name (F_FACES);
		return;
	}
	{
		int f;
		for (f = 0; f < obj->number_of_faces; f++)
		{
			int n = obj->faces[f].number_of_points;
			if (n < 3)
			{
				a->status = "degenerate_face";
				return;
			}
			a->face_points_sum += n;
			a->fan_triangles += n - 2;
			if (a->face_points_min < 0 || n < a->face_points_min)
			{
				a->face_points_min = n;
			}
			if (n > a->face_points_max)
			{
				a->face_points_max = n;
			}
		}
	}

	/* the point list: its extent must cover every face's points */
	a->cap_point_list = observed_capacity (c, F_POINTLIST);
	if (a->cap_point_list < 0 || a->cap_point_list < a->face_points_sum)
	{
		a->status = a->cap_point_list < 0 ? "capacity_unobservable" : "capacity_short";
		a->detail = field_name (F_POINTLIST);
		return;
	}

	/* pass 1: every face point's reference must lie inside its surface's point range; the highest address must lie inside the surface points */
	{
		int f, j, surface = 0;
		long remaining = obj->surfaces[0].number_of_faces, base = 0, position = 0;
		a->max_surface_address = -1;
		for (f = 0; f < obj->number_of_faces; f++)
		{
			long np = obj->surfaces[surface].number_of_points ? obj->surfaces[surface].number_of_points : 256;
			for (j = 0; j < obj->faces[f].number_of_points; j++)
			{
				int ref = obj->object_faces_point_plain_list[position + j].point;
				if (ref >= np)
				{
					a->over_surface++;
				}
				else if (base + ref > a->max_surface_address)
				{
					a->max_surface_address = base + ref;
				}
			}
			position += obj->faces[f].number_of_points;
			if (--remaining == 0)
			{
				base += np;
				surface++;
				if (surface < obj->number_of_surfaces)
				{
					remaining = obj->surfaces[surface].number_of_faces;
				}
			}
		}
	}
	if (a->over_surface)
	{
		a->status = "index_violation";
		a->detail = "face_point_reference";
		return;
	}
	a->cap_surface_points = observed_capacity (c, F_SURFPOINTS);
	if (a->cap_surface_points < 0 || a->cap_surface_points <= a->max_surface_address)
	{
		a->status = a->cap_surface_points < 0 ? "capacity_unobservable" : "capacity_short";
		a->detail = field_name (F_SURFPOINTS);
		return;
	}

	/* pass 2: every surface point must name a point of the object */
	{
		int f, j, surface = 0;
		long remaining = obj->surfaces[0].number_of_faces, base = 0, position = 0;
		for (f = 0; f < obj->number_of_faces; f++)
		{
			long np = obj->surfaces[surface].number_of_points ? obj->surfaces[surface].number_of_points : 256;
			for (j = 0; j < obj->faces[f].number_of_points; j++)
			{
				int ref = obj->object_faces_point_plain_list[position + j].point;
				if (obj->surface_points[base + ref].point >= obj->number_of_points)
				{
					a->over_points++;
				}
			}
			position += obj->faces[f].number_of_points;
			if (--remaining == 0)
			{
				base += np;
				surface++;
				if (surface < obj->number_of_surfaces)
				{
					remaining = obj->surfaces[surface].number_of_faces;
				}
			}
		}
	}
	if (a->over_points)
	{
		a->status = "index_violation";
		a->detail = "surface_point";
		return;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* summary */

int eech_model_summary_run (char *text, int capacity)
{
	struct out out;
	out_begin (&out, text, capacity);
	if (!objects_3d_scene_database || !objects_3d_data)
	{
		return 1;
	}
	emit (&out, "model-summary scene_last=%d raw_objects=%d surface_block=%ld scene_entry_bytes=%d object_bytes=%d surface_bytes=%d page_scenes=%d page_triangles=%d names=compiled_enum\n",
		(int) OBJECT_3D_LAST, total_number_of_raw_3d_objects, surface_block_length (), (int) sizeof (struct OBJECT_3D_SCENE_DATABASE_ENTRY), (int) sizeof (object_3d),
		(int) sizeof (face_surface_description), PAGE_SCENES, PAGE_TRIANGLES);
	return out.overflow ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* catalogue */

static int entity_live (entity *en)
{
	return entities && en >= entities && en < entities + number_of_entities;
}

struct uses
{
	int count;
	int type[USE_TYPE_SLOTS];
	int type_count[USE_TYPE_SLOTS];
	int other;
};

int eech_model_catalogue_run (int first, int count, char *text, int capacity)
{
	struct out out;
	static struct uses uses[PAGE_SCENES];
	entity *en;
	long scanned = 0, with_shape = 0;
	int i;

	out_begin (&out, text, capacity);
	if (!objects_3d_scene_database || !objects_3d_data || first < 1 || count < 1 || count > PAGE_SCENES || first >= OBJECT_3D_LAST)
	{
		return 1;
	}
	if (first + count > OBJECT_3D_LAST)
	{
		count = OBJECT_3D_LAST - first;
	}

	/* the use count: the entities of the loaded campaign, by the shape the ordinary getter reports for each */
	memset (uses, 0, sizeof (uses));
	for (en = get_local_entity_list (); en; en = get_local_entity_succ (en))
	{
		int shape, slot;
		if (!entity_live (en))
		{
			break;
		}
		scanned++;
		shape = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
		if (shape < 1 || shape >= OBJECT_3D_LAST)
		{
			continue;
		}
		with_shape++;
		if (shape >= first && shape < first + count)
		{
			struct uses *u = &uses[shape - first];
			u->count++;
			for (slot = 0; slot < USE_TYPE_SLOTS; slot++)
			{
				if (u->type_count[slot] && u->type[slot] == (int) en->type)
				{
					u->type_count[slot]++;
					break;
				}
				if (!u->type_count[slot])
				{
					u->type[slot] = (int) en->type;
					u->type_count[slot] = 1;
					break;
				}
			}
			if (slot == USE_TYPE_SLOTS)
			{
				u->other++;
			}
		}
	}

	emit (&out, "model-page first=%d count=%d scene_last=%d entities_scanned=%ld entities_with_shape=%ld\n", first, count, (int) OBJECT_3D_LAST, scanned, with_shape);
	for (i = first; i < first + count; i++)
	{
		struct analysis a;
		const struct OBJECT_3D_SCENE_DATABASE_ENTRY *scene = &objects_3d_scene_database[i];
		const char *name = scene_name (i);
		char bb[160], bb2[160], cbb[160];
		struct uses *u = &uses[i - first];
		int slot;
		float radius;
		object_3d_index_numbers index = (object_3d_index_numbers) i;

		analyse (&a, i);
		/* the original scene accessors, once each */
		bounds_text (bb, sizeof (bb), get_object_3d_bounding_box (index));
		bounds_text (bb2, sizeof (bb2), get_object_3d_bounding_box_without_lines (index));
		radius = get_object_3d_radius (index);
		emit (&out, "model-scene scene=%d name=%s main=%d succeeded=%d collision=%d status=%s detail=%s radius=0x%08x bb2=%s bb=%s uses=%d", i, name, scene->index,
			scene->succeeded, a.collision_index, a.status, a.detail, float_bits (radius), bb2, bb, u->count);
		emit (&out, " use_types=");
		if (!u->count)
		{
			emit (&out, "-");
		}
		for (slot = 0; slot < USE_TYPE_SLOTS && u->type_count[slot]; slot++)
		{
			emit (&out, "%s%s:%d", slot ? "," : "", get_entity_type_name ((entity_types) u->type[slot]), u->type_count[slot]);
		}
		emit (&out, " use_other=%d", u->other);
		if (a.collision_index > 0 && a.collision_index <= total_number_of_raw_3d_objects && strcmp (a.status, "null_surfaces"))
		{
			const object_3d *obj = &objects_3d_data[a.collision_index];
			bounds_text (cbb, sizeof (cbb), &obj->bounding_box);
			emit (&out,
				" c_points=%d c_faces=%d c_surfaces=%d c_custom=%d c_radius=0x%08x c_bb=%s surface_faces=%ld polygon_faces=%ld nonpolygon_surfaces=%ld zero_face_surfaces=%ld surface_points=%ld face_points=%ld"
				" face_points_min=%d face_points_max=%d fan_bound=%ld end_read_backed=%d cap_faces=%ld cap_point_list=%ld cap_surface_points=%ld max_depth=%ld",
				a.number_of_points, a.number_of_faces, a.number_of_surfaces, a.custom, float_bits (obj->radius), cbb, a.surface_faces, a.polygon_surface_faces,
				a.nonpolygon_surfaces, a.zero_face_surfaces, a.surface_points_sum, a.face_points_sum, a.face_points_min, a.face_points_max, a.fan_triangles,
				a.end_read_backed, a.cap_faces, a.cap_point_list, a.cap_surface_points, analysis_probeable (&a) ? analysis_max_depth (&a) : -1L);
		}
		emit (&out, "\n");
	}
	return out.overflow ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* geometry: the original accessor, depth by depth */

int eech_model_geometry_run (int scene, int first_depth, int count, char *text, int capacity)
{
	struct out out;
	struct analysis a;
	long max_depth;
	int depth, called = 0;

	out_begin (&out, text, capacity);
	if (!objects_3d_scene_database || !objects_3d_data || !scene_in_range (scene) || first_depth < 0 || count < 1 || count > PAGE_TRIANGLES ||
		first_depth > 0x7fffffff - PAGE_TRIANGLES)
	{
		return 1;
	}
	analyse (&a, scene);
	emit (&out, "model-geometry scene=%d name=%s collision=%d status=%s detail=%s first=%d count=%d fan_bound=%ld end_read_backed=%d\n", scene, scene_name (scene),
		a.collision_index, a.status, a.detail, first_depth, count, a.fan_triangles, a.end_read_backed);

	if (!strcmp (a.status, "absent"))
	{
		/* the original answers the absence itself, before any object is read */
		vec3d buffer[3];
		uint32_t words[9];
		int i, writes = 0, returned;
		for (i = 0; i < 9; i++)
		{
			words[i] = 0x7fc0a5a5u;
		}
		memcpy (buffer, words, sizeof (buffer));
		returned = get_object_3d_collision_object_geometry_triangle ((object_3d_index_numbers) scene, buffer, first_depth);
		memcpy (words, buffer, sizeof (words));
		for (i = 0; i < 9; i++)
		{
			writes += words[i] != 0x7fc0a5a5u;
		}
		emit (&out, "model-tri depth=%d return=%d writes=%d\n", first_depth, returned, writes);
		return out.overflow ? 1 : 0;
	}
	if (!analysis_probeable (&a))
	{
		emit (&out, "model-refused scene=%d status=%s detail=%s\n", scene, a.status, a.detail);
		return out.overflow ? 1 : 0;
	}

	max_depth = analysis_max_depth (&a);
	for (depth = first_depth; depth < first_depth + count; depth++)
	{
		vec3d buffer[3];
		uint32_t words[9];
		int i, writes = 0, returned;
		if (depth > max_depth)
		{
			emit (&out, "model-end scene=%d depth=%d reason=%s\n", scene, depth, (depth == a.fan_triangles) ? "end_probe_unbacked" : "beyond_safe_bound");
			break;
		}
		for (i = 0; i < 9; i++)
		{
			words[i] = 0x7fc0a5a5u;
		}
		memcpy (buffer, words, sizeof (buffer));
		returned = get_object_3d_collision_object_geometry_triangle ((object_3d_index_numbers) scene, buffer, depth);
		called++;
		memcpy (words, buffer, sizeof (words));
		for (i = 0; i < 9; i++)
		{
			writes += words[i] != 0x7fc0a5a5u;
		}
		emit (&out, "model-tri depth=%d return=%d writes=%d", depth, returned, writes);
		if (writes)
		{
			emit (&out, " p=");
			for (i = 0; i < 9; i++)
			{
				emit (&out, "%s0x%08x", i ? "," : "", words[i]);
			}
		}
		emit (&out, "\n");
		if (!returned)
		{
			emit (&out, "model-end scene=%d depth=%d reason=original_false\n", scene, depth);
			break;
		}
	}
	emit (&out, "model-geometry-end scene=%d calls=%d\n", scene, called);
	return out.overflow ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* state digest: every table the entries above read, hashed whole */

static void hash_bytes (uint64_t *hash, const void *data, size_t length)
{
	const unsigned char *p = (const unsigned char *) data;
	size_t i;
	for (i = 0; i < length; i++)
	{
		*hash ^= p[i];
		*hash *= FNV_PRIME;
	}
}

int eech_model_state_run (char *text, int capacity)
{
	struct out out;
	uint64_t tables = FNV_OFFSET, pointers = FNV_OFFSET, content = FNV_OFFSET;
	long safe_objects = 0;
	int i;

	out_begin (&out, text, capacity);
	if (!objects_3d_scene_database || !objects_3d_data || !object_database_surfaces)
	{
		return 1;
	}
	/* the scene table and the object table, every byte; the surface block, every byte */
	hash_bytes (&tables, objects_3d_scene_database, sizeof (struct OBJECT_3D_SCENE_DATABASE_ENTRY) * (size_t) OBJECT_3D_LAST);
	hash_bytes (&tables, objects_3d_data, sizeof (object_3d) * (size_t) (total_number_of_raw_3d_objects + 1));
	hash_bytes (&tables, object_database_surfaces, sizeof (face_surface_description) * (size_t) surface_block_length ());
	/* the table and block addresses, and the bounds */
	{
		uintptr_t addresses[3];
		addresses[0] = (uintptr_t) objects_3d_scene_database;
		addresses[1] = (uintptr_t) objects_3d_data;
		addresses[2] = (uintptr_t) object_database_surfaces;
		hash_bytes (&pointers, addresses, sizeof (addresses));
		hash_bytes (&pointers, &total_number_of_raw_3d_objects, sizeof (total_number_of_raw_3d_objects));
	}
	/* the arrays the original decoder reads, for every scene whose collision model the verdict allows to be called */
	for (i = 1; i < OBJECT_3D_LAST; i++)
	{
		struct analysis a;
		const object_3d *obj;
		analyse (&a, i);
		if (strcmp (a.status, "safe"))
		{
			continue;
		}
		obj = &objects_3d_data[a.collision_index];
		hash_bytes (&content, &i, sizeof (i));
		hash_bytes (&content, obj->points, sizeof (object_short_3d_point) * (size_t) obj->number_of_points);
		hash_bytes (&content, obj->faces, sizeof (object_3d_face) * (size_t) obj->number_of_faces);
		hash_bytes (&content, obj->object_faces_point_plain_list, (size_t) a.face_points_sum);
		hash_bytes (&content, obj->surface_points, sizeof (point_3d_plain_reference) * (size_t) (a.max_surface_address + 1));
		safe_objects++;
	}
	emit (&out, "model-state tables=%016llx pointers=%016llx content=%016llx content_objects=%ld\n", (unsigned long long) tables, (unsigned long long) pointers,
		(unsigned long long) content, safe_objects);
	return out.overflow ? 1 : 0;
}

#endif
