/* A focused C-level check of the campaign's half of the IMPACT dispatch and the area BLAST (EEDCS-46): it compiles the real aphavoc/source/entity/mobile/mb_hostaim.c against a
   stand-in for the engine's project.h and drives campaign_aim_impact and campaign_aim_blast with scripted hosts over a small fake world (sectors of linked lists of objects).
   It asserts what the campaign SENDS (by value: the explosion, the map's sector figures, the windows of the sectors' lists with handles that are numbers for this blast only), that
   the host's answers are used as given (a changed host reaches other objects, none, or all), the ORDER the original had (the campaign acts on each object the host reaches BEFORE
   it supplies the rest of the list, and the rest is the list as it is then: a notification that unlinks the next object takes it out of the walk), that no sample of the random
   stream is taken, and what the campaign REFUSES before it applies anything: an object that is not in the window, a rocket that is not one, a modifier outside [0, 1], a radius that
   is not a length or changes, a window outside the map, a sector's start asked twice (the ORDER the host walks the sectors in is the host's: a healthy host that walks them in another order than the reference's is accepted), a continuation that is not where the window ended, a request for a sample,
   a plan that contradicts the request, and a missing or failing host.  It checks the campaign's bridge; it is NOT a native run and says nothing about the arithmetic (the host's:
   eech-world src/cmb.rs).  Run by run-blast-checks.sh. */
#include "project.h"

#include <stdint.h>

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

static int draws_taken;

int rand16 (void) { draws_taken++; return 4321; }
float get_delta_time (void) { return 0.1f; }
unsigned int get_system_time (void) { return 0; }
const char *campaign_roster_host_id (entity *en) { return NULL; }
int get_local_entity_index (entity *en) { return 0; }

static int failures, checks;

static void check (int condition, const char *what)
{
	checks++;

	if (!condition)
	{
		failures++;
		printf ("FAIL %s\n", what);
	}
	else
	{
		printf ("ok   %s\n", what);
	}
}

static int refused_with (const char *text)
{
	return strstr (harness_message, text) != NULL;
}

/* ---- the fake world: three by three sectors, each a linked list of objects (index + 1 is the pointer the campaign passes around; 0 ends a list) */
#define OBJECTS 64

typedef struct fake_object
{
	int type;
	int readable;
	int alive;
	int has_position;
	int rocket;
	int excluded;
	float position[3];
	int next;
	char host_id[CAMPAIGN_AIM_HOST_ID_MAX + 1];	/* the identity the campaign offers the object under: every object of this world has one ("obj-<index + 1>") */
} fake_object;

static fake_object world[OBJECTS];
static int first_of[3][3];
static int object_count;

static void world_clear (void)
{
	memset (world, 0, sizeof (world));
	memset (first_of, 0, sizeof (first_of));
	object_count = 0;
}

/* appends an object to the list of sector (sx, sz); returns its index + 1 */
static int world_add (int sx, int sz, int type, int alive, int rocket)
{
	int index = object_count++;
	int *link = &first_of[sz][sx];

	world[index].type = type;
	world[index].readable = 1;
	world[index].alive = alive;
	world[index].has_position = 1;
	world[index].rocket = rocket;
	world[index].position[0] = 100.0f * (float) sx + 50.0f;
	world[index].position[2] = 100.0f * (float) sz + 50.0f;
	snprintf (world[index].host_id, sizeof (world[index].host_id), "obj-%d", index + 1);

	while (*link)
	{
		link = &world[*link - 1].next;
	}

	*link = index + 1;

	return index + 1;
}

static int windows_supplied, apply_calls, begin_calls, end_calls, examined_calls;
static int supplied_counts[64];
static int applied[64];
static int applied_actions[64];
static float applied_modifiers[64];
static int apply_unlinks_successor;

static int walk_window (void *context, int sx, int sz, const void *after, campaign_blast_entry *entries, void **objects, int max, int *end)
{
	int object, count = 0;

	windows_supplied++;

	if (after)
	{
		object = world[(int) (intptr_t) after - 1].next;
	}
	else
	{
		object = (sx >= 0 && sx <= 2 && sz >= 0 && sz <= 2) ? first_of[sz][sx] : 0;
	}

	while (object && count < max)
	{
		fake_object *item = &world[object - 1];

		entries[count].type = item->type;
		entries[count].readable = item->readable;
		entries[count].alive = item->alive;
		entries[count].has_position = item->has_position;
		entries[count].rocket = item->rocket;
		entries[count].excluded = item->excluded;
		entries[count].position[0] = item->position[0];
		entries[count].position[1] = item->position[1];
		entries[count].position[2] = item->position[2];
		memcpy (entries[count].host_id, item->host_id, sizeof (entries[count].host_id));
		objects[count] = (void *) (intptr_t) object;
		count++;
		object = item->next;
	}

	*end = (object == 0) ? TRUE : FALSE;

	if (windows_supplied <= 64)
	{
		supplied_counts[windows_supplied - 1] = count;
	}

	return count;
}

static void walk_apply (void *context, void *object, int action, float modifier)
{
	int index = (int) (intptr_t) object - 1;

	if (apply_calls < 64)
	{
		applied[apply_calls] = index + 1;
		applied_actions[apply_calls] = action;
		applied_modifiers[apply_calls] = modifier;
	}

	apply_calls++;

	if (apply_unlinks_successor && world[index].next)
	{
		/* a notification that takes the next object out of the sector's list, as a kill can */
		world[index].next = world[world[index].next - 1].next;
	}
}

static float begun_radius;

static void walk_begin (void *context, float radius)
{
	begin_calls++;
	begun_radius = radius;
}

static void walk_examined (void *context, const campaign_aim_result *result, void *const *objects, int count)
{
	examined_calls += result->echo_count;
}

static void walk_end (void *context)
{
	end_calls++;
}

static struct campaign_blast_walk walk;

static void walk_reset (void)
{
	memset (&walk, 0, sizeof (walk));
	walk.sector_side_length = 100;
	walk.min_x_sector = 0;
	walk.max_x_sector = 2;
	walk.min_z_sector = 0;
	walk.max_z_sector = 2;
	walk.window = walk_window;
	walk.apply = walk_apply;
	walk.begin = walk_begin;
	walk.examined = walk_examined;
	walk.end = walk_end;
	windows_supplied = apply_calls = begin_calls = end_calls = examined_calls = 0;
	apply_unlinks_successor = 0;
	begun_radius = -1.0f;
	draws_taken = 0;
}

/* ---- scripted hosts */
enum host_kind
{
	HOST_IMPACT = 1,		/* answers with `plan`, and for a land hit asks for the terrain once */
	HOST_WALK,				/* a walk of the rectangle given: reaches the objects marked reach, in order */
	HOST_NOTHING,			/* the radius, and no object */
	HOST_ORDERED,			/* walks the sectors in the order of order (a healthy host with another order than the reference's), reaching the first live object of each */
	HOST_SWEEP				/* asks for the start of a fresh sector every time, in rows of a map of a hundred by a hundred, and reaches nothing */
};

static int host_kind, host_calls, host_fails;
static campaign_impact_result plan;
static int plan_needs_terrain;
static campaign_aim_step seen_first, seen, requests[128];
static int request_count;
static float walk_radius = 20.0f;
static int fault;			/* a fault the walking host commits, once */
static int order_sx[8], order_sz[8], order_count;	/* HOST_ORDERED: the sectors in the order the host walks them */
static int fault_armed;

enum faults
{
	FAULT_NONE,
	FAULT_UNKNOWN_HANDLE,
	FAULT_IDENTITY_WRONG,
	FAULT_IDENTITY_MISSING,
	FAULT_IDENTITY_BESIDE_END,
	FAULT_MODIFIER_HIGH,
	FAULT_MODIFIER_NAN,
	FAULT_MODIFIER_NEGATIVE,
	FAULT_RADIUS_CHANGES,
	FAULT_RADIUS_NEGATIVE,
	FAULT_RADIUS_NAN,
	FAULT_DRAW,
	FAULT_SECTOR_OUTSIDE,
	FAULT_SECTOR_AGAIN,
	FAULT_CONTINUE_WRONG_AFTER,
	FAULT_CONTINUE_WRONG_SECTOR,
	FAULT_INTERCEPT_NON_ROCKET,
	FAULT_UNKNOWN_ACTION,
	FAULT_ECHO_UNASKED,
	FAULT_ECHO_BAD_INDEX,
	FAULT_ACTION_WITHOUT_WINDOW,
	FAULT_DONE_WITH_HANDLE,
	FAULT_NEGATIVE_AFTER,
	FAULT_SECTOR_REVISIT,		/* an earlier sector (not the last) started again */
	FAULT_RADIUS_CHANGES_ECHO	/* the radius changes, and the echo is consistent with the NEW radius (the diagnostic envelope is not what is wrong) */
};

/* the walking host: asks for the window of the first sector, reaches the first live object that is not excluded in each window (a collision with 0.5, a rocket destroyed), asks for
   the rest of a list that did not fit, and for the next sector (rows, each from its first); it ends after sector (2, 2) */
static int host_aim_inner (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	const campaign_blast_request *blast = &step->blast;
	int i, k;

	host_calls++;

	if (host_fails)
	{
		snprintf (error, error_size, "the host has no model for this blast");

		return 1;
	}

	if (host_calls == 1)
	{
		seen_first = *step;
	}

	seen = *step;

	if (request_count < 128)
	{
		requests[request_count++] = *step;
	}

	memset (result, 0, sizeof (*result));

	if (host_kind == HOST_IMPACT)
	{
		if ((plan_needs_terrain == 1 && !step->impact.terrain_known) || plan_needs_terrain == 2)
		{
			result->state = CAMPAIGN_AIM_NEED_LOOKUP;
			result->impact.need_terrain = 1;

			return 0;
		}

		result->state = CAMPAIGN_AIM_DONE;
		result->impact = plan;

		return 0;
	}

	result->state = CAMPAIGN_AIM_DONE;
	result->blast.radius = walk_radius;

	if (host_kind == HOST_NOTHING)
	{
		return 0;
	}

	if (fault == FAULT_DRAW && fault_armed)
	{
		fault_armed = 0;
		result->state = CAMPAIGN_AIM_NEED_DRAW;

		return 0;
	}

	if (fault == FAULT_RADIUS_NEGATIVE && fault_armed)
	{
		fault_armed = 0;
		result->blast.radius = -1.0f;
	}

	if (fault == FAULT_RADIUS_NAN && fault_armed)
	{
		fault_armed = 0;
		result->blast.radius = NAN;
	}

	if (fault == FAULT_RADIUS_CHANGES && fault_armed && blast->window_valid)
	{
		fault_armed = 0;
		result->blast.radius = walk_radius + 1.0f;
	}

	if (host_kind == HOST_SWEEP)
	{
		/* a fresh sector every time: sector number n of rows of a hundred; the first question has no window */
		int n = blast->window_valid ? (blast->window_sz * 100 + blast->window_sx + 1) : 0;

		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->blast.need_sx = n % 100;
		result->blast.need_sz = n / 100;
		result->blast.need_after = 0;

		return 0;
	}

	if (!blast->window_valid)
	{
		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->blast.need_sx = (host_kind == HOST_ORDERED) ? order_sx[0] : 0;
		result->blast.need_sz = (host_kind == HOST_ORDERED) ? order_sz[0] : 0;
		result->blast.need_after = 0;

		if (fault == FAULT_ACTION_WITHOUT_WINDOW && fault_armed)
		{
			fault_armed = 0;
			result->state = CAMPAIGN_AIM_DONE;
			result->blast.action = CAMPAIGN_BLAST_COLLISION;
			result->blast.handle = 1;
			result->blast.modifier = 0.5f;
		}

		if (fault == FAULT_SECTOR_OUTSIDE && fault_armed)
		{
			fault_armed = 0;
			result->blast.need_sx = 3;
		}

		return 0;
	}

	for (i = 0; i < blast->window_count; i++)
	{
		const campaign_blast_entry *entry = &blast->window[i];

		if (entry->alive && !entry->excluded)
		{
			result->blast.handle = entry->handle;
			result->blast.modifier = 0.5f;
			result->blast.action = entry->rocket ? CAMPAIGN_BLAST_INTERCEPT : CAMPAIGN_BLAST_COLLISION;

			/* the host repeats the identity of the object it reaches, as the campaign offered it */
			memcpy (result->blast.target_host_id, entry->host_id, sizeof (result->blast.target_host_id));

			if (fault == FAULT_IDENTITY_WRONG && fault_armed)
			{
				fault_armed = 0;
				snprintf (result->blast.target_host_id, sizeof (result->blast.target_host_id), "another-%s", entry->host_id);
			}

			if (fault == FAULT_IDENTITY_MISSING && fault_armed)
			{
				fault_armed = 0;
				memset (result->blast.target_host_id, 0, sizeof (result->blast.target_host_id));
			}

			if (fault == FAULT_UNKNOWN_HANDLE && fault_armed)
			{
				fault_armed = 0;
				result->blast.handle = 999;
			}

			if (fault == FAULT_MODIFIER_HIGH && fault_armed)
			{
				fault_armed = 0;
				result->blast.modifier = 1.5f;
			}

			if (fault == FAULT_MODIFIER_NAN && fault_armed)
			{
				fault_armed = 0;
				result->blast.modifier = NAN;
			}

			if (fault == FAULT_MODIFIER_NEGATIVE && fault_armed)
			{
				fault_armed = 0;
				result->blast.modifier = -0.25f;
			}

			if (fault == FAULT_INTERCEPT_NON_ROCKET && fault_armed && !entry->rocket)
			{
				fault_armed = 0;
				result->blast.action = CAMPAIGN_BLAST_INTERCEPT;
			}

			if (fault == FAULT_UNKNOWN_ACTION && fault_armed)
			{
				fault_armed = 0;
				result->blast.action = 7;
			}

			if (fault == FAULT_ECHO_UNASKED && fault_armed)
			{
				fault_armed = 0;
				result->has_echo = 1;
				result->echo_count = 1;
				result->echo = step->echo_out;
			}

			if (fault == FAULT_IDENTITY_BESIDE_END && fault_armed)
			{
				fault_armed = 0;
				result->blast.action = CAMPAIGN_BLAST_DONE;
				result->blast.handle = 0;
				result->blast.modifier = 0.0f;
			}

			if (fault == FAULT_DONE_WITH_HANDLE && fault_armed)
			{
				fault_armed = 0;
				result->blast.action = CAMPAIGN_BLAST_DONE;
				/* the walk ends and names an object; the identity is not repeated, so this check is of the handle alone (an identity beside an end is refused too, by its own check) */
				memset (result->blast.target_host_id, 0, sizeof (result->blast.target_host_id));
			}

			if (step->echo && step->echo_out)
			{
				campaign_aim_event *event = step->echo_out;

				memset (event, 0, sizeof (*event));
				event->kind = CAMPAIGN_AIM_EVENT_BLAST;
				event->a = (fault == FAULT_ECHO_BAD_INDEX) ? 40 : i;
				event->b = 1;
				event->f[0] = 10.0f;
				event->f[1] = walk_radius;
				event->f[2] = 0.5f;
				result->has_echo = 1;
				result->echo_count = 1;
				result->echo = step->echo_out;
			}

			return 0;
		}
	}

	if (!blast->window_end)
	{
		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->blast.need_sx = blast->window_sx;
		result->blast.need_sz = blast->window_sz;
		result->blast.need_after = blast->window[blast->window_count - 1].handle;

		if (fault == FAULT_CONTINUE_WRONG_AFTER && fault_armed)
		{
			fault_armed = 0;
			result->blast.need_after = 1;
		}

		if (fault == FAULT_CONTINUE_WRONG_SECTOR && fault_armed)
		{
			fault_armed = 0;
			result->blast.need_sx = (blast->window_sx + 1) % 3;
		}

		return 0;
	}

	result->state = CAMPAIGN_AIM_NEED_LOOKUP;
	result->blast.need_after = 0;

	if (host_kind == HOST_ORDERED)
	{
		/* the next sector in the host's own order, or the end of the walk */
		for (k = 0; k < order_count && !(order_sx[k] == blast->window_sx && order_sz[k] == blast->window_sz); k++)
		{
		}

		if (k + 1 >= order_count)
		{
			result->state = CAMPAIGN_AIM_DONE;

			return 0;
		}

		result->blast.need_sx = order_sx[k + 1];
		result->blast.need_sz = order_sz[k + 1];
	}
	else if (blast->window_sx < 2)
	{
		result->blast.need_sx = blast->window_sx + 1;
		result->blast.need_sz = blast->window_sz;
	}
	else if (blast->window_sz < 2)
	{
		result->blast.need_sx = 0;
		result->blast.need_sz = blast->window_sz + 1;
	}
	else
	{
		result->state = CAMPAIGN_AIM_DONE;

		return 0;
	}

	if (fault == FAULT_SECTOR_AGAIN && fault_armed)
	{
		fault_armed = 0;
		result->blast.need_sx = blast->window_sx;
		result->blast.need_sz = blast->window_sz;
	}

	if (fault == FAULT_SECTOR_REVISIT && fault_armed && blast->window_sx == 2 && blast->window_sz == 0)
	{
		/* the third sector of the row has just been walked: start the FIRST again */
		fault_armed = 0;
		result->blast.need_sx = 0;
		result->blast.need_sz = 0;
	}

	if (fault == FAULT_NEGATIVE_AFTER && fault_armed)
	{
		fault_armed = 0;
		result->blast.need_after = -1;
	}

	return 0;
}

static int host_aim (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	int code = host_aim_inner (step, result, error, error_size, user);

	if (code == 0 && fault == FAULT_RADIUS_CHANGES_ECHO && step->mode == CAMPAIGN_AIM_BLAST && step->blast.window_valid && step->blast.window_count > 0 && step->echo && step->echo_out)
	{
		/* the radius changes at the first window, and the optional echo of the candidate examined says the NEW radius, so the envelope is consistent and only the change is wrong */
		campaign_aim_event *event = step->echo_out;

		result->blast.radius = walk_radius + 1.0f;
		memset (event, 0, sizeof (*event));
		event->kind = CAMPAIGN_AIM_EVENT_BLAST;
		event->a = 0;
		event->b = 0;
		event->f[0] = 30.0f;
		event->f[1] = walk_radius + 1.0f;
		result->has_echo = 1;
		result->echo_count = 1;
		result->echo = step->echo_out;
	}

	return code;
}

static void host_installed (int kind)
{
	campaign_aim_host host;

	memset (&host, 0, sizeof (host));
	host.aim = host_aim;
	campaign_aim_set_host (&host);
	host_kind = kind;
	host_calls = 0;
	host_fails = 0;
	request_count = 0;
	fault = FAULT_NONE;
	fault_armed = 0;
	walk_radius = 20.0f;
	plan_needs_terrain = 0;
	memset (&plan, 0, sizeof (plan));
	memset (&seen_first, 0, sizeof (seen_first));
	memset (&seen, 0, sizeof (seen));
	harness_message[0] = '\0';
	walk_reset ();
}

static int terrain_calls, terrain_answer;

static int terrain_fn (void *context)
{
	terrain_calls++;

	return terrain_answer;
}

static campaign_impact_result impact_result;
static const float impact_position[3] = { 250.0f, 12.0f, 750.0f };

static int impact_ask (int kill_code, int has_target, int target_alive)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	impact_result = campaign_aim_impact (kill_code, has_target, target_alive, impact_position, terrain_fn, NULL);

	return 1;
}

static const float blast_position[3] = { 150.0f, 5.0f, 150.0f };

static int blast_ask (void)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	campaign_aim_blast (blast_position, 400, 7, 1, &walk);

	return 1;
}

/* a world of two objects in sector (0,0), none in (1,0), one in (2,0), and the other rows empty */
static void small_world (void)
{
	world_clear ();
	world_add (0, 0, 6, 1, 0);
	world_add (0, 0, 19, 1, 0);
	world_add (2, 0, 6, 1, 0);
}

int main (void)
{
	int accepted, i;

	/* ================ IMPACT ================ */
	host_installed (HOST_IMPACT);
	plan.blasts = 1;
	plan.direct_hit = 1;
	plan.exclude_target = 1;
	terrain_calls = 0;
	accepted = impact_ask (5, 1, 1);
	check (accepted && host_calls == 1 && draws_taken == 0 && terrain_calls == 0, "an impact is one question that takes no sample and reads no terrain unless the host asks");
	check (seen.mode == CAMPAIGN_AIM_IMPACT && seen.impact.kill_code == 5 && seen.impact.has_target == 1 && seen.impact.target_alive == 1 && seen.impact.terrain_known == 0 &&
		seen.impact.position[0] == 250.0f && seen.impact.position[1] == 12.0f && seen.impact.position[2] == 750.0f, "the kill code, the target facts and the position cross by value");
	check (impact_result.blasts == 1 && impact_result.direct_hit == 1 && impact_result.exclude_target == 1 && !impact_result.crater && !impact_result.invalid, "the host's plan is returned whole");

	host_installed (HOST_IMPACT);
	plan.blasts = 2;
	plan.crater = 1;
	plan_needs_terrain = 1;
	terrain_calls = 0;
	terrain_answer = 1;
	accepted = impact_ask (3, 0, 0);
	check (accepted && host_calls == 2 && terrain_calls == 1 && seen.impact.terrain_known == 1 && seen.impact.terrain_draws_crater == 1 && seen_first.impact.terrain_known == 0 &&
		impact_result.crater == 1 && impact_result.blasts == 2, "a host that needs the terrain is given it ONCE (read by the campaign on request) and is asked again with it");
	terrain_calls = 0;
	terrain_answer = 0;
	plan.crater = 0;
	accepted = impact_ask (3, 0, 0);
	check (accepted && terrain_calls == 1 && seen.impact.terrain_known == 1 && seen.impact.terrain_draws_crater == 0 && impact_result.crater == 0, "the terrain that takes no crater is given as it is");

	/* a changed host: another plan, used as given */
	host_installed (HOST_IMPACT);
	plan.blasts = 0;
	accepted = impact_ask (3, 0, 0);
	check (accepted && impact_result.blasts == 0 && impact_result.crater == 0, "a host that plans no blast for a land hit gives none, and never asks for the terrain");
	host_installed (HOST_IMPACT);
	plan.invalid = 1;
	accepted = impact_ask (99, 0, 0);
	check (accepted && impact_result.invalid == 1 && impact_result.blasts == 0, "a code the host does not know is reported as such (the campaign then stops with the original's message)");

	/* refusals */
	host_installed (HOST_IMPACT);
	plan.crater = 1;
	plan.blasts = 1;
	accepted = impact_ask (5, 1, 1);
	check (!accepted && refused_with ("inconsistent plan"), "a crater on a terrain the campaign has not read is refused");
	host_installed (HOST_IMPACT);
	plan.blasts = 3;
	accepted = impact_ask (5, 1, 1);
	check (!accepted && refused_with ("inconsistent plan"), "three blasts are refused");
	host_installed (HOST_IMPACT);
	plan.blasts = -1;
	accepted = impact_ask (5, 1, 1);
	check (!accepted, "a negative number of blasts is refused");
	host_installed (HOST_IMPACT);
	plan.direct_hit = 1;
	plan.blasts = 1;
	accepted = impact_ask (5, 0, 0);
	check (!accepted && refused_with ("inconsistent plan"), "a direct hit on a target there is none of is refused");
	host_installed (HOST_IMPACT);
	plan.exclude_target = 1;
	accepted = impact_ask (5, 1, 1);
	check (!accepted, "leaving a target out of no blast is refused");
	host_installed (HOST_IMPACT);
	plan.invalid = 1;
	plan.blasts = 1;
	accepted = impact_ask (99, 0, 0);
	check (!accepted, "a code that is not known cannot also plan a blast");
	host_installed (HOST_IMPACT);
	plan.blasts = 1;
	plan.crater = 2;
	accepted = impact_ask (5, 1, 1);
	check (!accepted, "a flag that is not a boolean is refused");
	host_installed (HOST_IMPACT);
	plan.blasts = 2;
	plan_needs_terrain = 2;		/* the host keeps asking for the terrain, even once it has it */
	terrain_calls = 0;
	terrain_answer = 1;
	accepted = impact_ask (3, 0, 0);
	check (!accepted && refused_with ("for the terrain again") && terrain_calls == 1, "a host that asks for the terrain a second time is refused (it is read once)");

	host_installed (HOST_IMPACT);
	plan.blasts = 1;
	host_fails = 1;
	accepted = impact_ask (5, 1, 1);
	check (!accepted && refused_with ("the host has no model for this blast"), "a failing host stops the engine with its reason");
	campaign_aim_set_host (NULL);
	accepted = impact_ask (5, 1, 1);
	check (!accepted && refused_with ("needs a host aim callback"), "no host stops the engine at the first request");
	host_installed (HOST_IMPACT);
	accepted = impact_ask (5, 0, 1);
	check (!accepted && refused_with ("inconsistent kill code") && host_calls == 0, "a request that says a target there is none of is alive is not sent");

	/* ================ BLAST ================ */
	host_installed (HOST_NOTHING);
	small_world ();
	accepted = blast_ask ();
	check (accepted && host_calls == 1 && begin_calls == 1 && end_calls == 1 && apply_calls == 0 && windows_supplied == 0 && draws_taken == 0 && begun_radius == 20.0f,
		"a host that ends the walk at once is asked once; the radius it computed is reported once, before and without any object, and no window is read");
	check (seen_first.mode == CAMPAIGN_AIM_BLAST && !seen_first.blast.window_valid && seen_first.blast.window_count == 0 && seen_first.blast.damage_capability == 400 &&
		seen_first.blast.warhead_type == 7 && seen_first.blast.has_target == 1 && seen_first.blast.position[0] == 150.0f && seen_first.blast.position[2] == 150.0f &&
		seen_first.blast.sector_side_length == 100 && seen_first.blast.min_x_sector == 0 && seen_first.blast.max_x_sector == 2 && seen_first.blast.min_z_sector == 0 &&
		seen_first.blast.max_z_sector == 2, "the explosion and the map's sector figures cross by value; the first request carries no window");

	/* the walking host over the whole rectangle: every object that is alive is reached, in the order of the walk */
	host_installed (HOST_WALK);
	small_world ();
	accepted = blast_ask ();
	check (accepted && apply_calls == 3 && applied[0] == 1 && applied[1] == 2 && applied[2] == 3 && begin_calls == 1 && end_calls == 1 && draws_taken == 0,
		"the campaign acts on every object the host reaches, in the host's order (sector by sector, each list in order), and takes no sample");
	check (applied_actions[0] == CAMPAIGN_BLAST_COLLISION && applied_modifiers[0] == 0.5f, "the action and the modifier are the host's");
	check (windows_supplied >= 9, "a window is supplied for every sector of the rectangle");
	/* the handles are numbers for this blast only: positive, increasing, never repeated, and every window repeats what it offers */
	{
		int strictly_increasing = 1, last = 0, k;

		for (i = 0; i < request_count; i++)
		{
			for (k = 0; k < requests[i].blast.window_count; k++)
			{
				if (requests[i].blast.window[k].handle <= 0)
				{
					strictly_increasing = 0;
				}
			}
		}

		for (i = 0; i < request_count; i++)
		{
			if (requests[i].blast.window_count)
			{
				if (requests[i].blast.window[0].handle <= last)
				{
					strictly_increasing = 0;
				}

				last = requests[i].blast.window[requests[i].blast.window_count - 1].handle;
			}
		}

		check (strictly_increasing, "a handle is positive and only grows within a blast (an object offered again after a notification has a new one)");
	}

	/* the successor of an object is read AFTER the campaign has acted on it: a notification that unlinks the next object takes it out of the walk (the original read the successor
	   after notifying) */
	host_installed (HOST_WALK);
	small_world ();
	apply_unlinks_successor = 1;
	accepted = blast_ask ();
	check (accepted && apply_calls == 2 && applied[0] == 1 && applied[1] == 3, "an object a notification unlinked is never offered: the list continues as it is after the notification");
	host_installed (HOST_WALK);
	small_world ();
	accepted = blast_ask ();
	check (accepted && apply_calls == 3, "and the same world, with no notification that changes it, offers all three (the control)");

	/* a list longer than a window is offered in windows, the second following the sixteenth object */
	host_installed (HOST_WALK);
	world_clear ();
	for (i = 0; i < 20; i++)
	{
		world_add (0, 0, 6, 0, 0);
	}
	accepted = blast_ask ();
	check (accepted && apply_calls == 0 && supplied_counts[0] == 16 && supplied_counts[1] == 4 && requests[2].blast.window_after == requests[1].blast.window[15].handle &&
		requests[2].blast.window[0].handle == requests[1].blast.window[15].handle + 1 && !requests[1].blast.window_end && requests[2].blast.window_end,
		"a list of twenty is offered as sixteen and then four, the second following the last handle of the first, and says where the list ends");
	host_installed (HOST_WALK);
	world_clear ();
	for (i = 0; i < 20; i++)
	{
		world_add (0, 0, 6, i == 17, 0);
	}
	accepted = blast_ask ();
	check (accepted && apply_calls == 1 && applied[0] == 18, "an object in the second window is reached and the walk goes on after it");

	/* a rocket is destroyed by the host's say; the campaign passes it on and a non-rocket may not be */
	host_installed (HOST_WALK);
	world_clear ();
	world_add (0, 0, 37, 1, 1);
	accepted = blast_ask ();
	check (accepted && apply_calls == 1 && applied_actions[0] == CAMPAIGN_BLAST_INTERCEPT, "a rocket the host destroys is passed on as a destruction");

	/* the echo of the candidates examined is passed to the observer, only when it was asked for */
	host_installed (HOST_WALK);
	small_world ();
	walk.echo = 1;
	accepted = blast_ask ();
	check (accepted && examined_calls == 3, "an echo that was asked for reaches the observer before the campaign acts");
	host_installed (HOST_WALK);
	small_world ();
	accepted = blast_ask ();
	check (accepted && examined_calls == 0, "and none is asked for in the shipping build");

	/* refusals, each at the point the host commits it, before anything else is applied */
	{
		struct
		{
			int kind;
			const char *text;
			const char *what;
		} faults[] =
		{
			{ FAULT_UNKNOWN_HANDLE, "not in the window", "an object that is not in the window" },
			{ FAULT_IDENTITY_WRONG, "but the campaign offered it as", "another identity than the one the object was offered under" },
			{ FAULT_IDENTITY_MISSING, "but the campaign offered it as", "no identity for an object that was offered with one" },
			{ FAULT_MODIFIER_HIGH, "modifier", "a modifier above one" },
			{ FAULT_MODIFIER_NAN, "modifier", "a modifier that is not a number" },
			{ FAULT_MODIFIER_NEGATIVE, "modifier", "a negative modifier" },
			{ FAULT_RADIUS_CHANGES, "changed the damage radius", "a radius that changes within a blast" },
			{ FAULT_RADIUS_NEGATIVE, "damage radius", "a negative radius" },
			{ FAULT_RADIUS_NAN, "damage radius", "a radius that is not a number" },
			{ FAULT_DRAW, "asked for a draw", "a request for a sample of the random stream" },
			{ FAULT_SECTOR_OUTSIDE, "malformed window", "a window of a sector outside the map" },
			{ FAULT_SECTOR_AGAIN, "again", "the start of the sector just walked asked again" },
			{ FAULT_SECTOR_REVISIT, "again", "the start of an earlier sector asked again" },
			{ FAULT_UNKNOWN_ACTION, "action", "an action the campaign does not know" },
			{ FAULT_ECHO_UNASKED, "nobody asked for", "an echo nobody asked for" },
			{ FAULT_NEGATIVE_AFTER, "malformed window", "a window after a negative handle" },
			{ FAULT_DONE_WITH_HANDLE, "action 0", "an end of the walk that names an object" },
			{ FAULT_IDENTITY_BESIDE_END, "identity beside an answer that names no object", "an identity beside an end of the walk" }
		};

		for (i = 0; i < (int) (sizeof (faults) / sizeof (faults[0])); i++)
		{
			char what[160];

			host_installed (HOST_WALK);
			small_world ();
			fault = faults[i].kind;
			fault_armed = 1;
			accepted = blast_ask ();
			snprintf (what, sizeof (what), "refused before it is applied: %s", faults[i].what);
			check (!accepted && refused_with (faults[i].text), what);
		}
	}

	/* the ORDER the host walks the sectors in is the host's: a healthy host that visits the fresh sector (2,1) and then the fresh sector (1,1) is accepted (the reference's row order would
	   visit them the other way), through the real driver, and so is a walk that crosses rows backwards */
	host_installed (HOST_ORDERED);
	world_clear ();
	world_add (1, 1, 6, 1, 0);
	world_add (2, 1, 6, 1, 0);
	world_add (2, 2, 6, 1, 0);
	order_count = 2;
	order_sx[0] = 2; order_sz[0] = 1;
	order_sx[1] = 1; order_sz[1] = 1;
	accepted = blast_ask ();
	check (accepted && apply_calls == 2 && applied[0] == 2 && applied[1] == 1 && begin_calls == 1 && end_calls == 1, "a healthy host that walks fresh sector (2, 1) and then fresh sector (1, 1) is accepted, and its objects are reached in ITS order");
	host_installed (HOST_ORDERED);
	world_clear ();
	world_add (0, 0, 6, 1, 0);
	world_add (1, 1, 6, 1, 0);
	world_add (2, 2, 6, 1, 0);
	order_count = 3;
	order_sx[0] = 2; order_sz[0] = 2;
	order_sx[1] = 0; order_sz[1] = 0;
	order_sx[2] = 1; order_sz[2] = 1;
	accepted = blast_ask ();
	check (accepted && apply_calls == 3 && applied[0] == 3 && applied[1] == 1 && applied[2] == 2, "a walk from the far corner back to the near one and then the middle is accepted too");
	host_installed (HOST_ORDERED);
	world_clear ();
	world_add (1, 1, 6, 1, 0);
	world_add (2, 1, 6, 1, 0);
	order_count = 3;
	order_sx[0] = 2; order_sz[0] = 1;
	order_sx[1] = 1; order_sz[1] = 1;
	order_sx[2] = 2; order_sz[2] = 1;
	accepted = blast_ask ();
	check (!accepted && refused_with ("again") && apply_calls == 2, "but the same fresh sector started a second time is refused, after the objects already reached were acted on");
	/* a walk of more sectors than a blast may walk is refused, whatever the order */
	host_installed (HOST_SWEEP);
	world_clear ();
	walk.max_x_sector = 99;
	walk.max_z_sector = 99;
	accepted = blast_ask ();
	check (!accepted && refused_with ("more than 4096 sectors") && windows_supplied == 4096 && end_calls == 0, "a host that walks more than 4096 fresh sectors is refused at the 4097th");
	/* a changed radius is refused as such even with the optional echo turned on: the echo's own radius agrees with the new one, so the diagnostic envelope is not what is wrong */
	host_installed (HOST_WALK);
	small_world ();
	world_clear ();
	world_add (0, 0, 6, 0, 0);
	world_add (0, 0, 6, 0, 0);
	walk.echo = 1;
	fault = FAULT_RADIUS_CHANGES_ECHO;
	accepted = blast_ask ();
	check (!accepted && refused_with ("changed the damage radius") && !refused_with ("malformed blast echo") && begin_calls == 1 && apply_calls == 0,
		"with the echo on and a window of candidates (nothing reached, no second sector), a changed radius is refused as a changed radius and not as a malformed echo");

	/* the faults that need a world of their own */	host_installed (HOST_WALK);
	world_clear ();
	for (i = 0; i < 20; i++)
	{
		world_add (0, 0, 6, 0, 0);
	}
	fault = FAULT_CONTINUE_WRONG_AFTER;
	fault_armed = 1;
	accepted = blast_ask ();
	check (!accepted && refused_with ("not where the window ended"), "a continuation after an object other than the last offered is refused");
	host_installed (HOST_WALK);
	world_clear ();
	for (i = 0; i < 20; i++)
	{
		world_add (0, 0, 6, 0, 0);
	}
	fault = FAULT_CONTINUE_WRONG_SECTOR;
	fault_armed = 1;
	accepted = blast_ask ();
	check (!accepted && refused_with ("not where the window ended"), "a continuation in another sector than the window's is refused");
	host_installed (HOST_WALK);
	world_clear ();
	world_add (0, 0, 6, 1, 0);
	fault = FAULT_INTERCEPT_NON_ROCKET;
	fault_armed = 1;
	accepted = blast_ask ();
	check (!accepted && refused_with ("does not know as one"), "a destruction of an object the campaign does not know as a rocket is refused");
	host_installed (HOST_WALK);
	small_world ();
	fault = FAULT_ACTION_WITHOUT_WINDOW;
	fault_armed = 1;
	accepted = blast_ask ();
	check (!accepted && refused_with ("not in the window"), "an object reached before any window was supplied is refused");
	host_installed (HOST_WALK);
	small_world ();
	fault = FAULT_ECHO_BAD_INDEX;
	walk.echo = 1;
	accepted = blast_ask ();
	check (!accepted && refused_with ("malformed blast echo"), "an echo event for an object that is not in the window is refused");

	/* nothing is applied by a blast that is refused before it reached anything */
	host_installed (HOST_WALK);
	small_world ();
	fault = FAULT_RADIUS_NEGATIVE;
	fault_armed = 1;
	accepted = blast_ask ();
	check (!accepted && apply_calls == 0 && begin_calls == 0, "a radius that is not a length is refused before the observer or any object");

	/* a missing or failing host */
	host_installed (HOST_WALK);
	small_world ();
	host_fails = 1;
	accepted = blast_ask ();
	check (!accepted && refused_with ("the host has no model for this blast") && apply_calls == 0, "a failing host stops the engine with its reason");
	campaign_aim_set_host (NULL);
	accepted = blast_ask ();
	check (!accepted && refused_with ("needs a host aim callback"), "no host stops the engine at the first request");

	/* a request the campaign would not send */
	host_installed (HOST_WALK);
	small_world ();
	walk.sector_side_length = 0;
	accepted = blast_ask ();
	check (!accepted && refused_with ("inconsistent figures"), "a map with a sector of no length is not sent");

	printf ("%d checks, %d failures\n", checks, failures);

	return failures ? 1 : 0;
}
