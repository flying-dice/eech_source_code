/* A DIFFERENTIAL check of the area blast (EEDCS-46): the ORIGINAL damage_objects_within_explosion_damage_radius of the campaign (transcribed below from explosn.c at the baseline revision,
   expression for expression, over a fake world) against the NEW path, which is the real aphavoc/source/entity/mobile/mb_hostaim.c driver, the reference host's REAL kernel
   (eech-world csrc/cmb_step.c, linked in) and the campaign's window and notification callbacks (the same code as explosn.c's, over the same fake world).  Thousands of random worlds
   and explosions are run through both, each against its own copy of the world; every candidate examined, every object notified or destroyed, and every effect of a notification on the
   lists (a notification can kill the next object, unlink it, or add one to the end of the list) must come out IDENTICALLY, in the same order, with the same bits.  A notification's
   effect is a deterministic function of the object and the modifier, so both paths see the same world at every step.
   It is NOT a native run: it proves the walk, the radius, the sector rectangle, the strict in-range test, the exclusion and the rocket threshold, and the order of notification against
   the reading of the list, are the original's over the worlds it generates; it does not prove the campaign's own notification handlers, which are untouched.  Run by run-blast-differential.sh. */
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
int get_local_entity_index (entity *en) { return 0; }

/* ---- the reference kernel, linked from eech-world csrc/cmb_step.c (its structures, repeated here field for field) */
#define KERNEL_WINDOW_MAX 16

typedef struct cf_blast_entry
{
	int handle, type, readable, alive, has_position, rocket, excluded;
	float position[3];
} cf_blast_entry;

typedef struct cf_blast_in
{
	float position[3];
	int damage_capability, warhead_type, has_target, sector_side_length, min_x_sector, max_x_sector, min_z_sector, max_z_sector;
	int window_valid, window_sx, window_sz, window_after, window_count, window_end;
	cf_blast_entry window[KERNEL_WINDOW_MAX];
} cf_blast_in;

typedef struct cf_blast_echo
{
	int index, flags;
	float range, radius, modifier;
} cf_blast_echo;

typedef struct cf_blast_out
{
	int state, failed;
	float radius;
	int need_sx, need_sz, need_after, action, handle;
	float modifier;
	int echo_count;
	cf_blast_echo echo[KERNEL_WINDOW_MAX];
} cf_blast_out;

extern void eech_cmb_blast (const cf_blast_in *in, cf_blast_out *out);

/* ---- the fake world: sectors of linked lists; an object is an index + 1 (0 ends a list) */
#define NUM_SECTORS_X 3
#define NUM_SECTORS_Z 3
#define SECTOR_SIDE 100
#define MAX_OBJECTS 512

typedef struct fake_object
{
	int type;
	int alive;
	int has_position;
	int rocket;
	float position[3];
	int next;
	int sx, sz;
} fake_object;

typedef struct fake_world
{
	fake_object objects[MAX_OBJECTS];
	int first[NUM_SECTORS_Z][NUM_SECTORS_X];
	int count;
	int target;			/* index + 1, or 0 */
} fake_world;

static const int examined_types[13] = { 1, 6, 10, 14, 19, 21, 23, 24, 26, 28, 29, 30, 37 };
#define TYPE_WEAPON 37

static int is_examined_type (int type)
{
	int i;

	for (i = 0; i < 13; i++)
	{
		if (examined_types[i] == type)
		{
			return 1;
		}
	}

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

static int add_object (fake_world *w, int sx, int sz, int type, int alive, int has_position, int rocket, float x, float y, float z)
{
	int index = w->count++;
	int *link = &w->first[sz][sx];

	w->objects[index].type = type;
	w->objects[index].alive = alive;
	w->objects[index].has_position = has_position;
	w->objects[index].rocket = rocket;
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

/* the event log both paths write: kind, object (index + 1), a float's bits */
typedef struct event
{
	int kind;
	int object;
	unsigned int bits;
} event;

enum { EV_BEGIN = 1, EV_CANDIDATE, EV_NOTIFY, EV_ROCKET, EV_COLLIDE, EV_DESTROY, EV_END, EV_EFFECT_UNLINK, EV_EFFECT_KILL, EV_EFFECT_ADD };

#define LOG_MAX 20000
static event log_a[LOG_MAX], log_b[LOG_MAX];
static int count_a, count_b;

static unsigned int float_bits (float v)
{
	unsigned int bits;

	memcpy (&bits, &v, sizeof (bits));

	return bits;
}

static void put (event *log, int *count, int kind, int object, unsigned int bits)
{
	if (*count < LOG_MAX)
	{
		log[*count].kind = kind;
		log[*count].object = object;
		log[*count].bits = bits;
	}

	(*count)++;
}

/* the effect of a notification on the world: a function of the object and the modifier only, so that both paths see the same world.  It can unlink the next object of the list, kill
   the next one, or add a new object at the end of the sector's list (which a walk still to come will meet) */
static void effect_of_notification (fake_world *w, event *log, int *count, int object, float modifier)
{
	fake_object *item = &w->objects[object - 1];
	unsigned int code = float_bits (modifier) % 11u + (unsigned int) object % 4u;

	if (code % 5u == 0 && item->next)
	{
		int victim = item->next;

		item->next = w->objects[victim - 1].next;		/* unlinked from the list; the victim keeps its own `next`, as a removed entity's does */
		put (log, count, EV_EFFECT_UNLINK, victim, 0);
	}
	else if (code % 5u == 1 && item->next)
	{
		w->objects[item->next - 1].alive = 0;
		put (log, count, EV_EFFECT_KILL, item->next, 0);
	}
	else if (code % 7u == 2 && w->count < MAX_OBJECTS - 1)
	{
		int added = add_object (w, item->sx, item->sz, 6, 1, 1, 0, 100.0f * (float) item->sx + 10.0f, 5.0f, 100.0f * (float) item->sz + 10.0f);

		put (log, count, EV_EFFECT_ADD, added, 0);
	}
}

/* ---- the ORIGINAL (explosn.c at the baseline), over the fake world.  The accessors are the fake world's; the expressions are the original's, unchanged */
typedef struct vec3
{
	float x, y, z;
} vec3;

#define bound(VALUE,LOWER,UPPER) ( ( VALUE ) < ( LOWER ) ? ( LOWER ) : ( ( VALUE ) > ( UPPER ) ? ( UPPER ) : ( VALUE ) ) )

static float get_3d_range_orig (const vec3 *v1, const vec3 *v2)
{
	float dx, dy, dz, range;

	dx = v1->x - v2->x;
	dy = v1->y - v2->y;
	dz = v1->z - v2->z;

	range = sqrt ((dx * dx) + (dy * dy) + (dz * dz));

	return (range);
}

static void original_blast (fake_world *w, vec3 explosion_position_value, int damage_capability, int warhead_type, int target)
{
	float range, temp_x, temp_z, damage_radius, damage_modifier;
	int sx, sz, min_sector_x, max_sector_x, min_sector_z, max_sector_z, object;
	vec3 *explosion_position = &explosion_position_value, *object_position;

	damage_radius = sqrt ((float) damage_capability);

	put (log_a, &count_a, EV_BEGIN, 0, float_bits (damage_radius));

	if (damage_radius == 0.0)
	{
		put (log_a, &count_a, EV_END, 0, 0);

		return;
	}

	temp_x = explosion_position->x - damage_radius;
	temp_z = explosion_position->z - damage_radius;

	min_sector_x = (int) temp_x;
	min_sector_x /= SECTOR_SIDE;
	min_sector_z = (int) temp_z;
	min_sector_z /= SECTOR_SIDE;

	temp_x = explosion_position->x + damage_radius;
	temp_z = explosion_position->z + damage_radius;

	max_sector_x = (int) temp_x;
	max_sector_x /= SECTOR_SIDE;
	max_sector_z = (int) temp_z;
	max_sector_z /= SECTOR_SIDE;

	min_sector_x = bound (min_sector_x, 0, NUM_SECTORS_X - 1);
	max_sector_x = bound (max_sector_x, 0, NUM_SECTORS_X - 1);

	min_sector_z = bound (min_sector_z, 0, NUM_SECTORS_Z - 1);
	max_sector_z = bound (max_sector_z, 0, NUM_SECTORS_Z - 1);

	for (sz = min_sector_z; sz <= max_sector_z; sz ++)
	{
		for (sx = min_sector_x; sx <= max_sector_x; sx ++)
		{
			object = w->first[sz][sx];

			while (object)
			{
				fake_object *item = &w->objects[object - 1];

				if (object == target)
				{
					object = item->next;

					continue;
				}

				if (is_examined_type (item->type))
				{
					if (item->alive)
					{
						vec3 position;

						position.x = item->position[0];
						position.y = item->position[1];
						position.z = item->position[2];
						object_position = item->has_position ? &position : NULL;

						if (object_position)
						{
							range = get_3d_range_orig (explosion_position, object_position);

							put (log_a, &count_a, EV_CANDIDATE, object, float_bits (range));

							if (range && (range < damage_radius))
							{
								damage_modifier = range / damage_radius; // less value - more damage!

								put (log_a, &count_a, EV_NOTIFY, object, float_bits (damage_modifier));

								if (item->type != TYPE_WEAPON)
								{
									put (log_a, &count_a, EV_COLLIDE, object, float_bits (damage_modifier));
									effect_of_notification (w, log_a, &count_a, object, damage_modifier);
								}
								else if (1 - pow (damage_modifier, pow ((float) (warhead_type - 3) / 3, 1.5f)) > 0.75 && item->rocket)
								{
									put (log_a, &count_a, EV_ROCKET, object, float_bits (damage_modifier));
									put (log_a, &count_a, EV_DESTROY, object, 0);
								}
							}
						}
					}
				}

				object = item->next;		/* the successor, read AFTER the notification */
			}
		}
	}

	put (log_a, &count_a, EV_END, 0, 0);
}

/* ---- the NEW path: the real driver, the real kernel, and the campaign's callbacks over the fake world (explosn.c's, in the fake world's terms) */
typedef struct new_context
{
	fake_world *world;
} new_context;

static int new_window (void *context, int sx, int sz, const void *after, campaign_blast_entry *entries, void **objects, int max, int *end)
{
	fake_world *w = ((new_context *) context)->world;
	int object, count = 0;

	object = after ? w->objects[(int) (intptr_t) after - 1].next : w->first[sz][sx];

	while (object && count < max)
	{
		fake_object *item = &w->objects[object - 1];
		campaign_blast_entry *entry = &entries[count];

		entry->type = item->type;
		entry->excluded = (object == w->target) ? TRUE : FALSE;

		if (is_examined_type (item->type))
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

			if (item->type == TYPE_WEAPON)
			{
				entry->rocket = item->rocket ? TRUE : FALSE;
			}
		}

		objects[count] = (void *) (intptr_t) object;
		count++;
		object = item->next;
	}

	*end = (object == 0) ? TRUE : FALSE;

	return count;
}

static void new_apply (void *context, void *object, int action, float modifier)
{
	fake_world *w = ((new_context *) context)->world;
	int index = (int) (intptr_t) object;

	if (action == CAMPAIGN_BLAST_COLLISION)
	{
		put (log_b, &count_b, EV_COLLIDE, index, float_bits (modifier));
		effect_of_notification (w, log_b, &count_b, index, modifier);
	}
	else
	{
		put (log_b, &count_b, EV_DESTROY, index, 0);
	}
}

static void new_begin (void *context, float radius)
{
	put (log_b, &count_b, EV_BEGIN, 0, float_bits (radius));
}

/* the fixture's replay, in the original's order: the candidate, the notification hook for an object in range, the rocket hook */
static void new_examined (void *context, const campaign_aim_result *result, void *const *objects, int count)
{
	int i;

	for (i = 0; i < result->echo_count; i++)
	{
		const campaign_aim_event *event = &result->echo[i];
		int object = (int) (intptr_t) objects[event->a];

		put (log_b, &count_b, EV_CANDIDATE, object, float_bits (event->f[0]));

		if (event->b & 1)
		{
			put (log_b, &count_b, EV_NOTIFY, object, float_bits (event->f[2]));
		}

		if (event->b & 2)
		{
			put (log_b, &count_b, EV_ROCKET, object, float_bits (event->f[2]));
		}
	}
}

static void new_end (void *context)
{
	put (log_b, &count_b, EV_END, 0, 0);
}

/* the reference host: the real kernel behind the campaign's by-value exchange */
static int reference_host (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	const campaign_blast_request *blast = &step->blast;
	cf_blast_in in;
	cf_blast_out out;
	int i;

	if (step->mode != CAMPAIGN_AIM_BLAST)
	{
		snprintf (error, error_size, "unexpected mode");

		return 1;
	}

	memset (&in, 0, sizeof (in));
	memset (&out, 0, sizeof (out));
	memcpy (in.position, blast->position, sizeof (in.position));
	in.damage_capability = blast->damage_capability;
	in.warhead_type = blast->warhead_type;
	in.has_target = blast->has_target;
	in.sector_side_length = blast->sector_side_length;
	in.min_x_sector = blast->min_x_sector;
	in.max_x_sector = blast->max_x_sector;
	in.min_z_sector = blast->min_z_sector;
	in.max_z_sector = blast->max_z_sector;
	in.window_valid = blast->window_valid;
	in.window_sx = blast->window_sx;
	in.window_sz = blast->window_sz;
	in.window_after = blast->window_after;
	in.window_count = blast->window_count;
	in.window_end = blast->window_end;

	for (i = 0; i < blast->window_count; i++)
	{
		in.window[i].handle = blast->window[i].handle;
		in.window[i].type = blast->window[i].type;
		in.window[i].readable = blast->window[i].readable;
		in.window[i].alive = blast->window[i].alive;
		in.window[i].has_position = blast->window[i].has_position;
		in.window[i].rocket = blast->window[i].rocket;
		in.window[i].excluded = blast->window[i].excluded;
		memcpy (in.window[i].position, blast->window[i].position, sizeof (in.window[i].position));
	}

	eech_cmb_blast (&in, &out);

	if (out.failed)
	{
		snprintf (error, error_size, "the blast cannot be answered");

		return 1;
	}

	memset (result, 0, sizeof (*result));
	result->blast.radius = out.radius;

	if (out.state)
	{
		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->blast.need_sx = out.need_sx;
		result->blast.need_sz = out.need_sz;
		result->blast.need_after = out.need_after;
	}
	else
	{
		result->state = CAMPAIGN_AIM_DONE;
		result->blast.action = out.action;
		result->blast.handle = out.handle;
		result->blast.modifier = out.modifier;
	}

	if (step->echo && step->echo_out)
	{
		for (i = 0; i < out.echo_count; i++)
		{
			campaign_aim_event *event = &step->echo_out[i];

			memset (event, 0, sizeof (*event));
			event->kind = CAMPAIGN_AIM_EVENT_BLAST;
			event->a = out.echo[i].index;
			event->b = out.echo[i].flags;
			event->f[0] = out.echo[i].range;
			event->f[1] = out.echo[i].radius;
			event->f[2] = out.echo[i].modifier;
		}

		result->has_echo = 1;
		result->echo_count = out.echo_count;
		result->echo = step->echo_out;
	}

	return 0;
}

static void new_blast (fake_world *w, vec3 position, int damage_capability, int warhead_type, int target)
{
	new_context context;
	struct campaign_blast_walk walk;
	float p[3];

	context.world = w;
	memset (&walk, 0, sizeof (walk));
	walk.sector_side_length = SECTOR_SIDE;
	walk.min_x_sector = 0;
	walk.max_x_sector = NUM_SECTORS_X - 1;
	walk.min_z_sector = 0;
	walk.max_z_sector = NUM_SECTORS_Z - 1;
	walk.echo = 1;
	walk.window = new_window;
	walk.apply = new_apply;
	walk.begin = new_begin;
	walk.examined = new_examined;
	walk.end = new_end;
	walk.context = &context;
	p[0] = position.x;
	p[1] = position.y;
	p[2] = position.z;

	campaign_aim_blast (p, damage_capability, warhead_type, target != 0, &walk);
}

static void random_world (fake_world *w)
{
	static const int type_pool[] = { 1, 6, 10, 14, 19, 21, 23, 24, 26, 28, 29, 30, 37, 37, 37, 3, 4, 17, 25, 34 };
	int objects = (int) (next_random () % 120u), i;

	memset (w, 0, sizeof (*w));

	for (i = 0; i < objects; i++)
	{
		int sx = (int) (next_random () % NUM_SECTORS_X), sz = (int) (next_random () % NUM_SECTORS_Z);
		int type = type_pool[next_random () % (sizeof (type_pool) / sizeof (type_pool[0]))];
		int alive = (next_random () % 6u) != 0;
		int has_position = (next_random () % 12u) != 0;
		int rocket = (next_random () % 3u) != 0;
		float x = 100.0f * (float) sx + random_float (0.0f, 100.0f), z = 100.0f * (float) sz + random_float (0.0f, 100.0f);

		add_object (w, sx, sz, type, alive, has_position, rocket, x, random_float (0.0f, 30.0f), z);
	}

	if (w->count && next_random () % 2u)
	{
		w->target = (int) (next_random () % (unsigned int) w->count) + 1;
	}
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
		if (log_a[i].kind != log_b[i].kind || log_a[i].object != log_b[i].object || log_a[i].bits != log_b[i].bits)
		{
			return 0;
		}
	}

	return 1;
}

static int failures, scenarios, with_candidates, with_collisions, with_rockets, with_effects, total_events, zero_radius, in_range_weapons, at_radius;

int main (void)
{
	static const int capabilities[] = { 0, 1, 4, 100, 400, 900, 2500, 10000, 22500, 40000, 90000, 160000, 250000 };
	static fake_world world_a, world_b;
	campaign_aim_host host;
	int run;

	memset (&host, 0, sizeof (host));
	host.aim = reference_host;
	campaign_aim_set_host (&host);

	for (run = 0; run < 6000; run++)
	{
		vec3 position;
		int capability, warhead, i;

		rng_state = 12345u + (unsigned int) run * 7919u;
		random_world (&world_a);
		world_b = world_a;
		capability = capabilities[next_random () % (sizeof (capabilities) / sizeof (capabilities[0]))];
		warhead = (int) (next_random () % 10u);
		position.x = random_float (-20.0f, 320.0f);
		position.y = random_float (0.0f, 30.0f);
		position.z = random_float (-20.0f, 320.0f);

		/* an object exactly at the explosion's position (range 0) in a fraction of the runs, and an exact radius in another */
		if (run % 7 == 0 && world_a.count)
		{
			int victim = (int) (next_random () % (unsigned int) world_a.count);

			world_a.objects[victim].has_position = 1;
			world_a.objects[victim].position[0] = position.x;
			world_a.objects[victim].position[1] = position.y;
			world_a.objects[victim].position[2] = position.z;
			world_b = world_a;
		}

		/* an object exactly one radius away, on whole-number coordinates so that the range is exactly the radius (the strict test must leave it out) */
		if (run % 5 == 1 && world_a.count && capability > 0)
		{
			int victim = (int) (next_random () % (unsigned int) world_a.count);

			position.x = floorf (position.x);
			position.y = floorf (position.y);
			position.z = floorf (position.z);
			world_a.objects[victim].has_position = 1;
			world_a.objects[victim].position[0] = position.x + (float) sqrt ((double) capability);
			world_a.objects[victim].position[1] = position.y;
			world_a.objects[victim].position[2] = position.z;
			world_b = world_a;
			at_radius++;
		}

		count_a = count_b = 0;
		original_blast (&world_a, position, capability, warhead, world_a.target);

		harness_message[0] = '\0';

		if (setjmp (harness_fatal))
		{
			failures++;
			if (failures <= 5) printf ("FAIL run %d: the new path stopped: %s\n", run, harness_message);

			continue;
		}

		new_blast (&world_b, position, capability, warhead, world_b.target);

		scenarios++;
		total_events += count_a;

		if (!logs_equal ())
		{
			failures++;
			if (failures <= 5) printf ("FAIL run %d: the paths differ (original %d events, new %d), capability %d warhead %d\n", run, count_a, count_b, capability, warhead);

			for (i = 0; i < count_a && i < count_b && failures <= 5; i++)
			{
				if (log_a[i].kind != log_b[i].kind || log_a[i].object != log_b[i].object || log_a[i].bits != log_b[i].bits)
				{
					printf ("     first difference at event %d: original (%d, %d, %08x) new (%d, %d, %08x)\n", i, log_a[i].kind, log_a[i].object, log_a[i].bits, log_b[i].kind, log_b[i].object,
						log_b[i].bits);

					break;
				}
			}
		}

		for (i = 0; i < count_a && i < LOG_MAX; i++)
		{
			with_candidates += log_a[i].kind == EV_CANDIDATE;
			with_collisions += log_a[i].kind == EV_COLLIDE;
			with_rockets += log_a[i].kind == EV_ROCKET;
			with_effects += log_a[i].kind == EV_EFFECT_UNLINK || log_a[i].kind == EV_EFFECT_KILL || log_a[i].kind == EV_EFFECT_ADD;
			in_range_weapons += log_a[i].kind == EV_NOTIFY && world_a.objects[log_a[i].object - 1].type == TYPE_WEAPON;
		}

		zero_radius += capability == 0;
	}

	/* the harness must have exercised what it claims to compare, or it proves nothing */
	printf ("scenarios %d, events %d, candidates %d, collisions %d, rockets destroyed %d, weapons in range %d, effects on the lists %d, zero radius %d, objects exactly one radius away %d, failures %d\n", scenarios, total_events,
		with_candidates, with_collisions, with_rockets, in_range_weapons, with_effects, zero_radius, at_radius, failures);

	if (scenarios != 6000 || with_candidates < 10000 || with_collisions < 2000 || with_rockets < 20 || in_range_weapons < 100 || with_effects < 500 || zero_radius < 100 || at_radius < 500)
	{
		printf ("FAIL the harness did not exercise enough to be a comparison\n");
		failures++;
	}

	return failures ? 1 : 0;
}
