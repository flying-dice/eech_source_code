/* A DIFFERENTIAL check of the bridge collapse (EEDCS-46, R5): the ORIGINAL decisions of the campaign (transcribed below at the baseline revision, expression for expression, from br_msgs.c
   response_to_collision, sg_dbase.c get_bridge_segment_damaged_object, sg_updt.c update_server and sg_dstry.c kill_routed_vehicles_on_segment) against the NEW path, which is the real
   aphavoc/source/entity/mobile/mb_hostaim.c driver, the reference host's REAL kernel (eech-world csrc/cmb_step.c, linked in) and callbacks over the same fake world.
     - the kill, shape, neighbour, fall and rest questions over every combination of the discrete facts and tens of thousands of random continuous ones (heights, frame times, terrain heights, water,
       the boundary of a landing), compared bit for bit;
     - the vehicles walk over thousands of random sectors of routed vehicles and other objects, with the effects a kill can have on the list (the next object killed, unlinked or an object added), in the
       same order with the same bits.
   It is NOT a native run: it proves the decisions and the walk are the original's over what it generates; it does not prove the campaign's handlers, which are untouched.  Run by run-bridge-differential.sh. */
#include "project.h"

#include <stdint.h>
#include <stdlib.h>

#include "mb_hostaim.h"

jmp_buf harness_fatal;
char harness_message[512];

void debug_fatal (const char *format, ...)
{
	va_list arguments;

	va_start (arguments, format);
	vsnprintf (harness_message, sizeof (harness_message), format, arguments);
	va_end (arguments);
	longjmp (harness_fatal, 1);
}

int rand16 (void) { return 0; }
float get_delta_time (void) { return 0.1f; }
unsigned int get_system_time (void) { return 0; }
const char *campaign_roster_host_id (entity *en) { return NULL; }
const char *campaign_object_identify (entity *en) { return campaign_roster_host_id (en); }
int get_local_entity_index (entity *en) { return 0; }

/* ---- the reference kernel, linked from eech-world csrc/cmb_step.c (its structures, repeated here field for field) */
#define KERNEL_WINDOW_MAX 16

typedef struct cf_blast_entry
{
	int handle, type, readable, alive, has_position, rocket, excluded;
	float position[3];
} cf_blast_entry;

typedef struct cf_bridge_in
{
	int op, segment_kind, alive, server, pred_present, pred_alive, succ_present, succ_alive, which, neighbour_present, neighbour_kind, neighbour_alive;
	float position[3];
	float delta_time;
	int terrain_known;
	float terrain_height;
	int terrain_water;
	float segment_length;
	int sector_side_length, min_x_sector, max_x_sector, min_z_sector, max_z_sector;
	int window_valid, window_sx, window_sz, window_after, window_count, window_end;
	cf_blast_entry window[KERNEL_WINDOW_MAX];
} cf_bridge_in;

typedef struct cf_bridge_out
{
	int state, failed, kill, variant, notify, need_terrain, fell;
	float dropped_y, floor_height;
	int landed;
	float landed_y;
	int remove, need_sx, need_sz, need_after, action, handle;
} cf_bridge_out;

extern void eech_cmb_bridge (const cf_bridge_in *in, cf_bridge_out *out);

static int reference_host (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	const campaign_bridge_request *b = &step->bridge;
	cf_bridge_in in;
	cf_bridge_out out;
	int i;

	if (step->mode != CAMPAIGN_AIM_BRIDGE)
	{
		snprintf (error, error_size, "unexpected mode");

		return 1;
	}

	memset (&in, 0, sizeof (in));
	memset (&out, 0, sizeof (out));
	in.op = b->op;
	in.segment_kind = b->segment_kind;
	in.alive = b->alive;
	in.server = b->server;
	in.pred_present = b->pred_present;
	in.pred_alive = b->pred_alive;
	in.succ_present = b->succ_present;
	in.succ_alive = b->succ_alive;
	in.which = b->which;
	in.neighbour_present = b->neighbour_present;
	in.neighbour_kind = b->neighbour_kind;
	in.neighbour_alive = b->neighbour_alive;
	memcpy (in.position, b->position, sizeof (in.position));
	in.delta_time = b->delta_time;
	in.terrain_known = b->terrain_known;
	in.terrain_height = b->terrain_height;
	in.terrain_water = b->terrain_water;
	in.segment_length = b->segment_length;
	in.sector_side_length = b->sector_side_length;
	in.min_x_sector = b->min_x_sector;
	in.max_x_sector = b->max_x_sector;
	in.min_z_sector = b->min_z_sector;
	in.max_z_sector = b->max_z_sector;
	in.window_valid = b->window_valid;
	in.window_sx = b->window_sx;
	in.window_sz = b->window_sz;
	in.window_after = b->window_after;
	in.window_count = b->window_count;
	in.window_end = b->window_end;

	for (i = 0; i < b->window_count; i++)
	{
		in.window[i].handle = b->window[i].handle;
		in.window[i].type = b->window[i].type;
		in.window[i].readable = b->window[i].readable;
		in.window[i].alive = b->window[i].alive;
		in.window[i].has_position = b->window[i].has_position;
		in.window[i].rocket = b->window[i].rocket;
		in.window[i].excluded = b->window[i].excluded;
		memcpy (in.window[i].position, b->window[i].position, sizeof (in.window[i].position));
	}

	eech_cmb_bridge (&in, &out);

	if (out.failed)
	{
		snprintf (error, error_size, "the bridge question cannot be answered");

		return 1;
	}

	memset (result, 0, sizeof (*result));
	result->state = out.state ? CAMPAIGN_AIM_NEED_LOOKUP : CAMPAIGN_AIM_DONE;
	result->bridge.kill = out.kill;
	result->bridge.variant = out.variant;
	result->bridge.notify = out.notify;
	result->bridge.need_terrain = out.need_terrain;
	result->bridge.fell = out.fell;
	result->bridge.dropped_y = out.dropped_y;
	result->bridge.floor_height = out.floor_height;
	result->bridge.landed = out.landed;
	result->bridge.landed_y = out.landed_y;
	result->bridge.remove = out.remove;
	result->bridge.need_sx = out.need_sx;
	result->bridge.need_sz = out.need_sz;
	result->bridge.need_after = out.need_after;
	result->bridge.action = out.action;
	result->bridge.handle = out.handle;

	return 0;
}

static unsigned int rng_state;

static unsigned int next_random (void)
{
	rng_state = rng_state * 1664525u + 1013904223u;

	return rng_state >> 8;
}

static float random_float (float low, float high)
{
	return low + (high - low) * ((float) (next_random () % 100000u) / 100000.0f);
}

static unsigned int float_bits (float v)
{
	unsigned int bits;

	memcpy (&bits, &v, sizeof (bits));

	return bits;
}

static int failures, compared;

static void differ (const char *what, int run)
{
	failures++;

	if (failures <= 5)
	{
		printf ("FAIL %s (run %d)\n", what, run);
	}
}

/* ---- the segment sub types of the campaign (en_sbtyp.h) */
#define END_SECTION 1
#define SUPPORTED_MID 2
#define UNSUPPORTED_MID 3

typedef struct vec3
{
	float x, y, z;
} vec3;

/* ---- the ORIGINAL decisions */

/* br_msgs.c response_to_collision: kills a living unsupported mid section on a server */
static int original_kill (int sub_type, int alive, int server)
{
	if (sub_type == UNSUPPORTED_MID)
	{
		if (alive)
		{
			if (server)
			{
				return 1;
			}
		}
	}

	return 0;
}

/* sg_dbase.c get_bridge_segment_damaged_object, as the variant it selects (0 normal, 1 destroyed, 2 pred destroyed, 3 succ destroyed, 4 both) */
static int original_variant (int alive, int pred_present, int pred_alive_flag, int succ_present, int succ_alive_flag)
{
	int pred_alive = ((!pred_present) || pred_alive_flag), succ_alive = ((!succ_present) || succ_alive_flag);

	if (!alive)
	{
		return 1;
	}
	else if ((!pred_alive) && (succ_alive))
	{
		return 2;
	}
	else if ((pred_alive) && (!succ_alive))
	{
		return 3;
	}
	else if ((!pred_alive) && (!succ_alive))
	{
		return 4;
	}

	return 0;
}

typedef struct fall_outcome
{
	int fell, landed, remove;
	float dropped_y, floor_height, landed_y;
} fall_outcome;

/* sg_updt.c update_server, the fall: the terrain is the caller's */
static fall_outcome original_fall (int sub_type, float y, float delta_time, float terrain_height, int water)
{
	fall_outcome o;
	float pos_y = y;

	memset (&o, 0, sizeof (o));

	if (sub_type == UNSUPPORTED_MID)
	{
		pos_y -= (10.0 * delta_time);

		if (water)
		{
			terrain_height -= 1.0;
		}

		o.fell = 1;
		o.dropped_y = pos_y;
		o.floor_height = terrain_height;

		if (pos_y <= terrain_height)
		{
			pos_y = terrain_height;
			o.landed = 1;
			o.landed_y = pos_y;
			o.remove = 1;
		}
	}
	else
	{
		o.remove = 1;
	}

	return o;
}

/* sg_pack.c unpack_local_segment_data: a segment that is not alive and is an unsupported mid section is positioned on the ground (a metre lower over water); any other keeps its position */
static float original_rest (int sub_type, int alive, float y, float terrain_height, int water)
{
	if ((!alive) && (sub_type == UNSUPPORTED_MID))
	{
		y = terrain_height;

		if (water)
		{
			y -= 1.0;
		}
	}

	return y;
}

/* ---- the NEW path's terrain */
static float terrain_value;
static int terrain_water_value;

static void terrain_fn (void *context, float *height, int *water)
{
	*height = terrain_value;
	*water = terrain_water_value;
}

/* ---- the fake world of the vehicles walk: sectors of linked lists; an object is an index + 1 */
#define NUM_SECTORS 3
#define SECTOR_SIDE 100
#define MAX_OBJECTS 512
#define ROUTED_VEHICLE 23

typedef struct fake_object
{
	int type;
	int alive;
	int has_position;
	float position[3];
	int next;
	int sx, sz;
} fake_object;

typedef struct fake_world
{
	fake_object objects[MAX_OBJECTS];
	int first[NUM_SECTORS][NUM_SECTORS];
	int count;
} fake_world;

enum { EV_KILL = 1, EV_EFFECT_UNLINK, EV_EFFECT_KILL, EV_EFFECT_ADD };

typedef struct event
{
	int kind;
	int object;
} event;

#define LOG_MAX 4000
static event log_a[LOG_MAX], log_b[LOG_MAX];
static int count_a, count_b;

static void put (event *log, int *count, int kind, int object)
{
	if (*count < LOG_MAX)
	{
		log[*count].kind = kind;
		log[*count].object = object;
	}

	(*count)++;
}

static int add_object (fake_world *w, int sx, int sz, int type, int alive, int has_position, float x, float y, float z)
{
	int index = w->count++;
	int *link = &w->first[sz][sx];

	w->objects[index].type = type;
	w->objects[index].alive = alive;
	w->objects[index].has_position = has_position;
	w->objects[index].position[0] = has_position ? x : 0.0f;
	w->objects[index].position[1] = has_position ? y : 0.0f;
	w->objects[index].position[2] = has_position ? z : 0.0f;
	w->objects[index].next = 0;
	w->objects[index].sx = sx;
	w->objects[index].sz = sz;

	while (*link)
	{
		link = &w->objects[*link - 1].next;
	}

	*link = index + 1;

	return index + 1;
}

/* what the death of a vehicle does to the list: a function of the object only, so both paths see the same world */
static void effect_of_kill (fake_world *w, event *log, int *count, int object)
{
	fake_object *item = &w->objects[object - 1];
	unsigned int code = (unsigned int) object * 2654435761u >> 7;

	item->alive = 0;

	if (code % 5u == 0 && item->next)
	{
		int victim = item->next;

		item->next = w->objects[victim - 1].next;
		put (log, count, EV_EFFECT_UNLINK, victim);
	}
	else if (code % 5u == 1 && item->next)
	{
		w->objects[item->next - 1].alive = 0;
		put (log, count, EV_EFFECT_KILL, item->next);
	}
	else if (code % 7u == 2 && w->count < MAX_OBJECTS - 1)
	{
		int added = add_object (w, item->sx, item->sz, ROUTED_VEHICLE, 1, 1, 100.0f * (float) item->sx + 10.0f, 5.0f, 100.0f * (float) item->sz + 10.0f);

		put (log, count, EV_EFFECT_ADD, added);
	}
}

static float get_approx_3d_range_orig (const vec3 *v1, const vec3 *v2)
{
	float dx, dy, dz, tmp, range;

	dx = fabs (v1->x - v2->x);
	dy = fabs (v1->y - v2->y);
	dz = fabs (v1->z - v2->z);

	if (dx < dy)
	{
		tmp = dx; dx = dy; dy = tmp;
	}

	if (dx < dz)
	{
		tmp = dx; dx = dz; dz = tmp;
	}

	range = dx + ((dy + dz) * (1.0 / 4.0));

	return (range);
}

/* sg_dstry.c kill_routed_vehicles_on_segment at the baseline */
static void original_vehicles (fake_world *w, vec3 segment_position_value, float length)
{
	float range, damage_radius;
	int sx, sz, object;
	vec3 *segment_position = &segment_position_value;

	damage_radius = length;

	if (damage_radius == 0.0)
	{
		return;
	}

	damage_radius *= 1.414213562f;

	sx = (int) segment_position->x;
	sx /= SECTOR_SIDE;
	sz = (int) segment_position->z;
	sz /= SECTOR_SIDE;

	object = w->first[sz][sx];

	while (object)
	{
		fake_object *item = &w->objects[object - 1];

		if (item->type == ROUTED_VEHICLE)
		{
			if (item->alive)
			{
				vec3 position;

				position.x = item->position[0];
				position.y = item->position[1];
				position.z = item->position[2];

				if (item->has_position)
				{
					range = get_approx_3d_range_orig (segment_position, &position);

					if (range < damage_radius)
					{
						put (log_a, &count_a, EV_KILL, object);
						effect_of_kill (w, log_a, &count_a, object);
					}
				}
			}
		}

		object = item->next;		/* the successor, read AFTER the kill */
	}
}

/* ---- the NEW path of the vehicles walk */
static int new_window (void *context, int sx, int sz, const void *after, campaign_blast_entry *entries, void **objects, int max, int *end)
{
	fake_world *w = (fake_world *) context;
	int object, count = 0;

	object = after ? w->objects[(int) (intptr_t) after - 1].next : w->first[sz][sx];

	while (object && count < max)
	{
		fake_object *item = &w->objects[object - 1];
		campaign_blast_entry *entry = &entries[count];

		entry->type = item->type;

		if (item->type == ROUTED_VEHICLE || item->type == 19 || item->type == 37)
		{
			entry->readable = TRUE;
			entry->alive = item->alive ? TRUE : FALSE;

			if (item->has_position)
			{
				entry->has_position = TRUE;
				entry->position[0] = item->position[0];
				entry->position[1] = item->position[1];
				entry->position[2] = item->position[2];
			}
		}

		objects[count] = (void *) (intptr_t) object;
		count++;
		object = item->next;
	}

	*end = (object == 0) ? TRUE : FALSE;

	return count;
}

static fake_world *new_world;

static void new_apply (void *context, void *object, int action, float modifier)
{
	int index = (int) (intptr_t) object;

	put (log_b, &count_b, EV_KILL, index);
	effect_of_kill (new_world, log_b, &count_b, index);
}

static void new_vehicles (fake_world *w, vec3 position, float length)
{
	struct campaign_blast_walk walk;
	float p[3];

	new_world = w;
	memset (&walk, 0, sizeof (walk));
	walk.sector_side_length = SECTOR_SIDE;
	walk.min_x_sector = 0;
	walk.max_x_sector = NUM_SECTORS - 1;
	walk.min_z_sector = 0;
	walk.max_z_sector = NUM_SECTORS - 1;
	walk.window = new_window;
	walk.apply = new_apply;
	walk.context = w;
	p[0] = position.x;
	p[1] = position.y;
	p[2] = position.z;

	campaign_aim_segment_vehicles (p, length, &walk);
}

static int logs_equal (void)
{
	int i;

	if (count_a != count_b || count_a > LOG_MAX)
	{
		return 0;
	}

	for (i = 0; i < count_a; i++)
	{
		if (log_a[i].kind != log_b[i].kind || log_a[i].object != log_b[i].object)
		{
			return 0;
		}
	}

	return 1;
}

int main (void)
{
	campaign_aim_host host;
	int run, kind, alive, server, pp, pa, sp, sa, kinds[4] = { 0, END_SECTION, SUPPORTED_MID, UNSUPPORTED_MID };
	int rest_moved = 0, rest_water = 0, fell_runs = 0, landed_runs = 0, water_runs = 0, boundary_runs = 0, vehicle_kills = 0, vehicle_effects = 0, vehicle_runs = 0, zero_length = 0;

	setvbuf (stdout, NULL, _IONBF, 0);

	/* a refusal outside the vehicles walk is a defect of this harness or of the campaign, never a crash */
	if (setjmp (harness_fatal))
	{
		printf ("FAIL the campaign refused a question the harness asked: %s\n", harness_message);

		return 1;
	}

	memset (&host, 0, sizeof (host));
	host.aim = reference_host;
	campaign_aim_set_host (&host);

	printf ("stage: the discrete facts\n");

	/* ---- every combination of the discrete facts: kill, shape, neighbour */
	for (kind = 0; kind < 4; kind++)
	{
		for (alive = 0; alive < 2; alive++)
		{
			for (server = 0; server < 2; server++)
			{
				compared++;

				if (campaign_aim_bridge_kill (kinds[kind], alive, server) != original_kill (kinds[kind], alive, server))
				{
					differ ("kill", kind);
				}
			}
		}
	}

	for (alive = 0; alive < 2; alive++)
	{
		for (pp = 0; pp < 2; pp++)
		{
			for (pa = 0; pa < 2; pa++)
			{
				for (sp = 0; sp < 2; sp++)
				{
					for (sa = 0; sa < 2; sa++)
					{
						if ((pa && !pp) || (sa && !sp))
						{
							continue;
						}

						compared++;

						if (campaign_aim_bridge_shape (alive, pp, pa, sp, sa) != original_variant (alive, pp, pa, sp, sa))
						{
							differ ("shape", alive);
						}
					}
				}
			}
		}
	}

	for (pp = 0; pp < 2; pp++)
	{
		compared++;

		if (campaign_aim_bridge_neighbour (0, pp, 2, pp) != pp || campaign_aim_bridge_neighbour (1, pp, 3, 0) != pp)
		{
			differ ("neighbour", pp);
		}
	}

	printf ("stage: the fall\n");

	/* ---- the fall over random continuous facts, bit for bit */
	for (run = 0; run < 60000; run++)
	{
		float y, dt, floor;
		float position[3];
		int water;
		campaign_bridge_result n;
		fall_outcome o;
		int sub_type;

		rng_state = 777u + (unsigned int) run * 6007u;
		sub_type = kinds[1 + next_random () % 3];
		y = random_float (-50.0f, 400.0f);
		dt = (run % 5 == 0) ? 0.1f : ((run % 7 == 0) ? 0.0f : random_float (0.0f, 1.5f));
		floor = random_float (-20.0f, 300.0f);
		water = (int) (next_random () % 3u) == 0;

		/* on the boundary of a landing: the drop lands the segment exactly at the floor (or a hair above it) */
		if (run % 11 == 0)
		{
			y = (float) ((double) floor - (water ? 1.0 : 0.0) + 10.0 * (double) dt);
			boundary_runs++;
		}

		position[0] = 250.0f;
		position[1] = y;
		position[2] = 250.0f;
		terrain_value = floor;
		terrain_water_value = water;

		n = campaign_aim_bridge_fall (sub_type, position, dt, terrain_fn, NULL);
		o = original_fall (sub_type, y, dt, floor, water);
		compared++;

		if (n.fell != o.fell || n.landed != o.landed || n.remove != o.remove || float_bits (n.dropped_y) != float_bits (o.dropped_y) || float_bits (n.floor_height) != float_bits (o.floor_height) ||
			float_bits (n.landed_y) != float_bits (o.landed_y))
		{
			differ ("fall", run);
		}

		fell_runs += o.fell;
		landed_runs += o.landed;
		water_runs += o.fell && water;
	}

	printf ("stage: the rest of a collapsed segment\n");

	/* ---- where a segment rests when a saved world is unpacked, bit for bit; the campaign applies the host's height only if it landed */
	for (run = 0; run < 20000; run++)
	{
		float y, floor;
		float position[3];
		int water, alive, sub_type;
		campaign_bridge_result n;
		float applied, expected;

		rng_state = 991u + (unsigned int) run * 4801u;
		sub_type = kinds[1 + next_random () % 3];
		y = random_float (-50.0f, 400.0f);
		floor = random_float (-20.0f, 300.0f);
		water = (int) (next_random () % 3u) == 0;
		alive = (int) (next_random () % 2u);

		position[0] = 250.0f;
		position[1] = y;
		position[2] = 250.0f;
		terrain_value = floor;
		terrain_water_value = water;

		n = campaign_aim_bridge_rest (sub_type, alive, position, terrain_fn, NULL);
		applied = n.landed ? n.landed_y : y;
		expected = original_rest (sub_type, alive, y, floor, water);
		compared++;

		if (float_bits (applied) != float_bits (expected) || n.fell || n.remove)
		{
			differ ("rest", run);
		}

		rest_moved += (float_bits (expected) != float_bits (y));
		rest_water += (float_bits (expected) != float_bits (y)) && water;
	}

	printf ("stage: the vehicles walk\n");

	/* ---- the vehicles walk over random worlds, with the effects of a kill on the list */
	for (run = 0; run < 4000; run++)
	{
		static fake_world world_a, world_b;
		static const int types[] = { ROUTED_VEHICLE, ROUTED_VEHICLE, ROUTED_VEHICLE, 19, 37, 3, 29 };
		vec3 position;
		float length;
		int objects, i, sx, sz;

		rng_state = 4242u + (unsigned int) run * 7919u;
		memset (&world_a, 0, sizeof (world_a));
		sx = (int) (next_random () % NUM_SECTORS);
		sz = (int) (next_random () % NUM_SECTORS);
		position.x = 100.0f * (float) sx + random_float (1.0f, 99.0f);
		position.z = 100.0f * (float) sz + random_float (1.0f, 99.0f);
		position.y = random_float (0.0f, 40.0f);
		length = (run % 13 == 0) ? 0.0f : ((run % 3 == 0) ? 10.0f : random_float (1.0f, 40.0f));
		zero_length += length == 0.0f;
		objects = (int) (next_random () % 60u);

		for (i = 0; i < objects; i++)
		{
			int osx = (next_random () % 4u) ? sx : (int) (next_random () % NUM_SECTORS), osz = (next_random () % 4u) ? sz : (int) (next_random () % NUM_SECTORS);
			int type = types[next_random () % (sizeof (types) / sizeof (types[0]))];
			float x = position.x + random_float (-30.0f, 30.0f), y = position.y + random_float (-12.0f, 12.0f), z = position.z + random_float (-30.0f, 30.0f);

			add_object (&world_a, osx, osz, type, (next_random () % 6u) != 0, (next_random () % 12u) != 0, x, y, z);
		}

		/* a vehicle exactly at the radius along one axis (the strict test leaves it out) */
		if (run % 5 == 1 && world_a.count && length > 0.0f)
		{
			int victim = (int) (next_random () % (unsigned int) world_a.count);

			world_a.objects[victim].type = ROUTED_VEHICLE;
			world_a.objects[victim].alive = 1;
			world_a.objects[victim].has_position = 1;
			world_a.objects[victim].position[0] = position.x + length * 1.414213562f;
			world_a.objects[victim].position[1] = position.y;
			world_a.objects[victim].position[2] = position.z;
		}

		world_b = world_a;
		count_a = count_b = 0;
		original_vehicles (&world_a, position, length);

		harness_message[0] = '\0';

		if (setjmp (harness_fatal))
		{
			differ ("the new vehicles walk stopped", run);

			continue;
		}

		new_vehicles (&world_b, position, length);
		compared++;
		vehicle_runs++;

		if (!logs_equal ())
		{
			differ ("vehicles walk", run);
		}

		for (i = 0; i < count_a && i < LOG_MAX; i++)
		{
			vehicle_kills += log_a[i].kind == EV_KILL;
			vehicle_effects += log_a[i].kind != EV_KILL;
		}
	}

	printf ("compared %d, fell %d, landed %d, over water %d, on the boundary %d, vehicle walks %d, vehicles killed %d, effects on the lists %d, zero length %d, rested %d, rested over water %d, failures %d\n", compared, fell_runs, landed_runs,
		water_runs, boundary_runs, vehicle_runs, vehicle_kills, vehicle_effects, zero_length, rest_moved, rest_water, failures);

	/* the harness must have exercised what it claims to compare, or it proves nothing */
	if (landed_runs < 1000 || fell_runs < 20000 || water_runs < 3000 || boundary_runs < 4000 || vehicle_kills < 1500 || vehicle_effects < 200 || zero_length < 100 || vehicle_runs != 4000 || rest_moved < 1000 || rest_water < 200)
	{
		printf ("FAIL the harness did not exercise enough to be a comparison\n");
		failures++;
	}

	return failures ? 1 : 0;
}
