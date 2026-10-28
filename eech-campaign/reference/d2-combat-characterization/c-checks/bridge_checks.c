/* A focused C-level check of the campaign's half of the BRIDGE collapse questions (EEDCS-46, R5): it compiles the real aphavoc/source/entity/mobile/mb_hostaim.c against a stand-in for the engine's
   project.h and drives campaign_aim_bridge_kill, _shape, _neighbour, _fall, _rest and campaign_aim_segment_vehicles with scripted hosts over a small fake world.
   It asserts what the campaign SENDS (by value: the segment's kind and alive flag, the neighbours, the position and the frame time, the terrain only once the host has asked for it, the segment's length and
   the map's sector figures), that the host's answers are used as given (a changed host kills, shapes, tells and drops otherwise), the ORDER of the vehicles walk (the campaign kills an object BEFORE it
   supplies the rest of the sector's list, and the rest is the list as it is then), that no sample of the random stream is taken, and what the campaign REFUSES before it applies anything: the kill of a segment
   that is not alive or of a client, a neighbour told that is not there, a variant that does not exist, a landing without a fall, a figure that is not finite, an answer beside another, the terrain asked
   for twice, a request for a sample, an object that is not a living object of the window, a continuation that is not where the window ended, a sector's start asked twice or more than the bound, and a
   missing or failing host.  It does NOT veto by the reference's rules: a host that kills a supported segment, tells no neighbour or never lands a segment is accepted.  It checks the campaign's bridge;
   it is NOT a native run and says nothing about the arithmetic (the host's: eech-world src/cmb.rs).  Run by run-bridge-checks.sh. */
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

/* ---- the scripted host */
static campaign_aim_step seen, requests[64];
static int request_count, host_calls, host_fails;
static campaign_aim_result scripted;
static int terrain_asks_remaining;			/* FALL and REST: the host asks for the terrain this many times (1: once; 2: again after it has it) */
static int vehicles_mode;					/* 1: reach the first living object of each window, 0: nothing */
static int vehicles_sx, vehicles_sz;		/* the sector the vehicles host asks for first */
static int sweep;							/* the host asks for a fresh sector every time */
static int fault;
static int asks_draw;						/* the host asks for a sample of the random stream */

enum { FAULT_NONE, FAULT_REVISIT, FAULT_WRONG_AFTER, FAULT_NEGATIVE_AFTER, FAULT_SECTOR_OUTSIDE };

static int host_aim (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	const campaign_bridge_request *bridge = &step->bridge;
	int i;

	host_calls++;

	if (host_fails)
	{
		snprintf (error, error_size, "the host has no model for this bridge");

		return 1;
	}

	seen = *step;

	if (request_count < 64)
	{
		requests[request_count++] = *step;
	}

	memset (result, 0, sizeof (*result));

	if (asks_draw)
	{
		result->state = CAMPAIGN_AIM_NEED_DRAW;

		return 0;
	}

	if ((bridge->op == CAMPAIGN_BRIDGE_FALL || bridge->op == CAMPAIGN_BRIDGE_REST) && terrain_asks_remaining > 0 && (!bridge->terrain_known || terrain_asks_remaining > 1))
	{
		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->bridge.need_terrain = 1;

		return 0;
	}

	if (bridge->op != CAMPAIGN_BRIDGE_VEHICLES)
	{
		result->state = CAMPAIGN_AIM_DONE;
		result->bridge = scripted.bridge;

		return 0;
	}

	if (sweep)
	{
		int n = bridge->window_valid ? (bridge->window_sz * 100 + bridge->window_sx + 1) : 0;

		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->bridge.need_sx = n % 100;
		result->bridge.need_sz = n / 100;

		return 0;
	}

	if (!bridge->window_valid)
	{
		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->bridge.need_sx = vehicles_sx;
		result->bridge.need_sz = vehicles_sz;

		if (fault == FAULT_SECTOR_OUTSIDE)
		{
			result->bridge.need_sx = bridge->max_x_sector + 1;
		}

		return 0;
	}

	result->state = CAMPAIGN_AIM_DONE;

	if (vehicles_mode)
	{
		for (i = 0; i < bridge->window_count; i++)
		{
			if (bridge->window[i].alive)
			{
				result->bridge.action = 1;
				result->bridge.handle = bridge->window[i].handle;

				if (scripted.bridge.handle)
				{
					result->bridge.handle = scripted.bridge.handle;
				}

				return 0;
			}
		}
	}

	if (!bridge->window_end)
	{
		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->bridge.need_sx = bridge->window_sx;
		result->bridge.need_sz = bridge->window_sz;
		result->bridge.need_after = bridge->window[bridge->window_count - 1].handle;

		if (fault == FAULT_WRONG_AFTER)
		{
			result->bridge.need_after = 1;
		}

		if (fault == FAULT_NEGATIVE_AFTER)
		{
			result->bridge.need_after = -1;
		}

		return 0;
	}

	if (fault == FAULT_REVISIT && request_count < 6)
	{
		/* the sector just walked, started again */
		result->state = CAMPAIGN_AIM_NEED_LOOKUP;
		result->bridge.need_sx = bridge->window_sx;
		result->bridge.need_sz = bridge->window_sz;
		result->bridge.need_after = 0;

		return 0;
	}

	return 0;
}

static void host_installed (void)
{
	campaign_aim_host host;

	memset (&host, 0, sizeof (host));
	host.aim = host_aim;
	campaign_aim_set_host (&host);
	memset (&scripted, 0, sizeof (scripted));
	scripted.state = CAMPAIGN_AIM_DONE;
	memset (&seen, 0, sizeof (seen));
	host_calls = request_count = host_fails = 0;
	terrain_asks_remaining = 0;
	vehicles_mode = 1;
	vehicles_sx = vehicles_sz = 2;
	sweep = 0;
	fault = FAULT_NONE;
	asks_draw = 0;
	draws_taken = 0;
	harness_message[0] = '\0';
}

static int terrain_calls;

static void terrain_fn (void *context, float *height, int *water)
{
	terrain_calls++;

	*height = 100.0f;
	*water = TRUE;
}

static int result_int;
static campaign_bridge_result fall_result;
static const float segment_position[3] = { 250.0f, 120.0f, 250.0f };

static int ask_kill (int kind, int alive, int server)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	result_int = campaign_aim_bridge_kill (kind, alive, server);

	return 1;
}

static int ask_shape (int alive, int pp, int pa, int sp, int sa)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	result_int = campaign_aim_bridge_shape (alive, pp, pa, sp, sa);

	return 1;
}

static int ask_neighbour (int which, int present, int kind, int alive)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	result_int = campaign_aim_bridge_neighbour (which, present, kind, alive);

	return 1;
}

static int ask_fall (int kind)
{
	harness_message[0] = '\0';
	terrain_calls = 0;

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	fall_result = campaign_aim_bridge_fall (kind, segment_position, 0.1f, terrain_fn, NULL);

	return 1;
}

static campaign_bridge_result rest_result;

static int ask_rest (int kind, int alive)
{
	harness_message[0] = '\0';
	terrain_calls = 0;

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	rest_result = campaign_aim_bridge_rest (kind, alive, segment_position, terrain_fn, NULL);

	return 1;
}

/* ---- the fake sector list of the vehicles walk: objects are index + 1 */
#define OBJECTS 64

typedef struct fake_object
{
	int alive;
	int next;
} fake_object;

static fake_object world[OBJECTS];
static int first_of[3][3];
static int object_count, windows_supplied, kills_applied, kill_order[64];
static int kill_unlinks_successor;

static void world_clear (void)
{
	memset (world, 0, sizeof (world));
	memset (first_of, 0, sizeof (first_of));
	object_count = windows_supplied = kills_applied = 0;
	kill_unlinks_successor = 0;
}

static int world_add (int sx, int sz, int alive)
{
	int index = object_count++;
	int *link = &first_of[sz][sx];

	world[index].alive = alive;

	while (*link)
	{
		link = &world[*link - 1].next;
	}

	*link = index + 1;

	return index + 1;
}

static int walk_window (void *context, int sx, int sz, const void *after, campaign_blast_entry *entries, void **objects, int max, int *end)
{
	int object, count = 0;

	windows_supplied++;

	object = after ? world[(int) (intptr_t) after - 1].next : ((sx >= 0 && sx <= 2 && sz >= 0 && sz <= 2) ? first_of[sz][sx] : 0);

	while (object && count < max)
	{
		entries[count].type = 23;
		entries[count].readable = 1;
		entries[count].alive = world[object - 1].alive;
		entries[count].has_position = 1;
		entries[count].position[0] = 250.0f;
		entries[count].position[1] = 5.0f;
		entries[count].position[2] = 250.0f;
		objects[count] = (void *) (intptr_t) object;
		count++;
		object = world[object - 1].next;
	}

	*end = (object == 0) ? TRUE : FALSE;

	return count;
}

static void walk_apply (void *context, void *object, int action, float modifier)
{
	int index = (int) (intptr_t) object - 1;

	if (kills_applied < 64)
	{
		kill_order[kills_applied] = index + 1;
	}

	kills_applied++;
	world[index].alive = 0;

	if (kill_unlinks_successor && world[index].next)
	{
		world[index].next = world[world[index].next - 1].next;
	}
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
}

static int ask_vehicles (void)
{
	static const float position[3] = { 250.0f, 5.0f, 250.0f };

	harness_message[0] = '\0';
	windows_supplied = kills_applied = 0;
	walk_reset ();

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	campaign_aim_segment_vehicles (position, 10.0f, &walk);

	return 1;
}

int main (void)
{
	int accepted, i;

	/* ================ KILL ================ */
	host_installed ();
	scripted.bridge.kill = 1;
	accepted = ask_kill (3, 1, 1);
	check (accepted && result_int == 1 && host_calls == 1 && draws_taken == 0, "a kill question is one question that takes no sample, and the host's yes is returned");
	check (seen.mode == CAMPAIGN_AIM_BRIDGE && seen.bridge.op == CAMPAIGN_BRIDGE_KILL && seen.bridge.segment_kind == 3 && seen.bridge.alive == 1 && seen.bridge.server == 1 && !seen.bridge.window_valid,
		"the segment's kind, its alive flag and whether this is a server cross by value");
	scripted.bridge.kill = 0;
	accepted = ask_kill (3, 1, 1);
	check (accepted && result_int == 0, "a changed host that holds the span up is obeyed");
	scripted.bridge.kill = 1;
	accepted = ask_kill (2, 1, 1);
	check (accepted && result_int == 1, "and one that kills a supported segment is not vetoed by the reference's rule (the campaign only refuses what it cannot apply)");
	accepted = ask_kill (3, 0, 1);
	check (!accepted && refused_with ("kill 1"), "the kill of a segment that is not alive is refused (its kill would run again)");
	accepted = ask_kill (3, 1, 0);
	check (!accepted && refused_with ("kill 1"), "and the kill by a client");
	scripted.bridge.kill = 0;
	scripted.bridge.variant = 1;
	accepted = ask_kill (3, 1, 1);
	check (!accepted && refused_with ("beside other answers"), "a kill answer with another answer beside it is refused");
	host_installed ();
	scripted.bridge.kill = 2;
	accepted = ask_kill (3, 1, 1);
	check (!accepted, "a flag that is not a boolean is refused");
	host_installed ();
	host_fails = 1;
	accepted = ask_kill (3, 1, 1);
	check (!accepted && refused_with ("the host has no model for this bridge"), "a failing host stops the engine with its reason");
	campaign_aim_set_host (NULL);
	accepted = ask_kill (3, 1, 1);
	check (!accepted && refused_with ("needs a host aim callback"), "no host stops the engine at the first request");

	/* ================ SHAPE ================ */
	host_installed ();
	scripted.bridge.variant = 3;
	accepted = ask_shape (1, 1, 1, 1, 0);
	check (accepted && result_int == 3 && seen.bridge.op == CAMPAIGN_BRIDGE_SHAPE && seen.bridge.alive == 1 && seen.bridge.pred_present == 1 && seen.bridge.pred_alive == 1 && seen.bridge.succ_present == 1 &&
		seen.bridge.succ_alive == 0, "the segment's own alive flag and each neighbour's presence and alive flag cross by value, and the variant comes back");
	scripted.bridge.variant = 0;
	accepted = ask_shape (0, 0, 0, 0, 0);
	check (accepted && result_int == 0, "a changed host's variant is used as given, whatever the facts");
	scripted.bridge.variant = CAMPAIGN_BRIDGE_VARIANTS;
	accepted = ask_shape (1, 0, 0, 0, 0);
	check (!accepted && refused_with ("variant"), "a variant that does not exist is refused");
	scripted.bridge.variant = -1;
	accepted = ask_shape (1, 0, 0, 0, 0);
	check (!accepted, "and a negative one");
	scripted.bridge.variant = 1;
	scripted.bridge.kill = 1;
	accepted = ask_shape (1, 0, 0, 0, 0);
	check (!accepted && refused_with ("more than a variant"), "a shape answer with more than a variant is refused");
	accepted = ask_shape (1, 0, 1, 0, 0);
	check (!accepted && refused_with ("inconsistent figures"), "a request that says a neighbour that is not there is alive is not sent");

	/* ================ NEIGHBOUR ================ */
	host_installed ();
	scripted.bridge.notify = 1;
	accepted = ask_neighbour (1, 1, 2, 1);
	check (accepted && result_int == 1 && seen.bridge.op == CAMPAIGN_BRIDGE_NEIGHBOUR && seen.bridge.which == 1 && seen.bridge.neighbour_present == 1 && seen.bridge.neighbour_kind == 2 &&
		seen.bridge.neighbour_alive == 1, "which neighbour it is and what the campaign knows of it cross by value");
	scripted.bridge.notify = 0;
	accepted = ask_neighbour (0, 1, 3, 1);
	check (accepted && result_int == 0, "a host that tells no neighbour (the propagation withheld) is obeyed");
	scripted.bridge.notify = 1;
	accepted = ask_neighbour (0, 0, 0, 0);
	check (!accepted && refused_with ("not there"), "a neighbour that is not there cannot be told");

	/* ================ FALL ================ */
	host_installed ();
	terrain_asks_remaining = 1;
	scripted.bridge.fell = 1;
	scripted.bridge.dropped_y = 119.0f;
	scripted.bridge.floor_height = 99.0f;
	accepted = ask_fall (3);
	check (accepted && host_calls == 2 && terrain_calls == 1 && requests[0].bridge.terrain_known == 0 && requests[1].bridge.terrain_known == 1 && requests[1].bridge.terrain_height == 100.0f &&
		requests[1].bridge.terrain_water == 1, "the host asks for the terrain, the campaign reads it ONCE and asks again with it");
	check (fall_result.fell == 1 && fall_result.dropped_y == 119.0f && fall_result.floor_height == 99.0f && !fall_result.landed && !fall_result.remove && seen.bridge.op == CAMPAIGN_BRIDGE_FALL &&
		seen.bridge.segment_kind == 3 && seen.bridge.position[1] == 120.0f && seen.bridge.delta_time == 0.1f && draws_taken == 0, "the segment's kind, position and the frame time cross, no sample is taken, and the fall comes back whole");
	host_installed ();
	scripted.bridge.remove = 1;
	accepted = ask_fall (2);
	check (accepted && terrain_calls == 0 && host_calls == 1 && fall_result.remove == 1 && !fall_result.fell, "a host that needs no terrain is never given it (a supported segment leaves the list)");
	host_installed ();
	scripted.bridge.fell = 1;
	scripted.bridge.dropped_y = 99.5f;
	scripted.bridge.floor_height = 99.0f;
	scripted.bridge.landed = 1;
	scripted.bridge.landed_y = 99.0f;
	scripted.bridge.remove = 1;
	accepted = ask_fall (3);
	check (accepted && fall_result.landed == 1 && fall_result.landed_y == 99.0f && fall_result.remove == 1, "a landing with its position and the leaving of the list comes back whole");
	host_installed ();
	scripted.bridge.fell = 1;
	scripted.bridge.dropped_y = 50.0f;
	scripted.bridge.floor_height = 99.0f;
	accepted = ask_fall (3);
	check (accepted && fall_result.fell == 1 && !fall_result.landed, "a host that drops a segment and never lands it (below the floor it reports) is not vetoed");
	host_installed ();
	scripted.bridge.landed = 1;
	scripted.bridge.landed_y = 1.0f;
	accepted = ask_fall (3);
	check (!accepted && refused_with ("do not agree"), "a landing without a fall is refused");
	host_installed ();
	scripted.bridge.dropped_y = 1.0f;
	accepted = ask_fall (3);
	check (!accepted && refused_with ("do not agree"), "a position for a fall that did not happen is refused");
	host_installed ();
	scripted.bridge.fell = 1;
	scripted.bridge.dropped_y = NAN;
	accepted = ask_fall (3);
	check (!accepted && refused_with ("not finite"), "a drop that is not a number is refused");
	host_installed ();
	scripted.bridge.fell = 1;
	scripted.bridge.dropped_y = 10.0f;
	scripted.bridge.remove = 3;
	accepted = ask_fall (3);
	check (!accepted, "a flag that is not a boolean is refused");
	host_installed ();
	terrain_asks_remaining = 2;
	accepted = ask_fall (3);
	check (!accepted && refused_with ("the terrain again") && terrain_calls == 1, "the terrain asked for a second time is refused (it is read once)");
	host_installed ();
	scripted.state = CAMPAIGN_AIM_DONE;
	scripted.bridge.kill = 1;
	accepted = ask_fall (3);
	check (!accepted, "a fall answered with a kill beside it is refused");

	/* ================ REST (where a collapsed segment lies when a saved world is unpacked) ================ */
	host_installed ();
	terrain_asks_remaining = 1;
	scripted.bridge.landed = 1;
	scripted.bridge.landed_y = 99.0f;
	accepted = ask_rest (3, 0);
	check (accepted && host_calls == 2 && terrain_calls == 1 && requests[0].bridge.terrain_known == 0 && requests[1].bridge.terrain_known == 1 && requests[1].bridge.terrain_height == 100.0f &&
		requests[1].bridge.terrain_water == 1, "a rest request asks for the terrain, the campaign reads it ONCE and asks again with it");
	check (rest_result.landed == 1 && rest_result.landed_y == 99.0f && !rest_result.fell && !rest_result.remove && seen.bridge.op == CAMPAIGN_BRIDGE_REST && seen.bridge.segment_kind == 3 &&
		seen.bridge.alive == 0 && seen.bridge.position[1] == 120.0f && draws_taken == 0, "the segment's kind, whether it is alive and its position cross, no sample is taken, and the height comes back");
	host_installed ();
	accepted = ask_rest (3, 0);
	check (accepted && terrain_calls == 0 && host_calls == 1 && !rest_result.landed && rest_result.landed_y == 0.0f, "a host that leaves the segment where it was saved needs no terrain and moves nothing");
	host_installed ();
	scripted.bridge.landed = 1;
	scripted.bridge.landed_y = 7.0f;
	accepted = ask_rest (3, 1);
	check (!accepted && refused_with ("segment that is alive"), "a segment that is alive is not moved by an unpack");
	host_installed ();
	scripted.bridge.landed_y = 7.0f;
	accepted = ask_rest (3, 0);
	check (!accepted && refused_with ("do not agree"), "a height for a rest that did not happen is refused");
	host_installed ();
	scripted.bridge.landed = 1;
	scripted.bridge.landed_y = NAN;
	accepted = ask_rest (3, 0);
	check (!accepted && refused_with ("not finite"), "a resting height that is not a number is refused");
	host_installed ();
	scripted.bridge.landed = 1;
	scripted.bridge.landed_y = 1.0f;
	scripted.bridge.fell = 1;
	accepted = ask_rest (3, 0);
	check (!accepted && refused_with ("do not agree"), "a rest answered with a fall beside it is refused");
	host_installed ();
	scripted.bridge.landed = 1;
	scripted.bridge.landed_y = 1.0f;
	scripted.bridge.remove = 1;
	accepted = ask_rest (3, 0);
	check (!accepted && refused_with ("do not agree"), "and with the leaving of an update list that it is not in");
	host_installed ();
	terrain_asks_remaining = 2;
	accepted = ask_rest (3, 0);
	check (!accepted && refused_with ("the terrain again") && terrain_calls == 1, "the terrain asked for a second time is refused");
	host_installed ();
	asks_draw = 1;
	accepted = ask_rest (3, 0);
	check (!accepted && refused_with ("asked for a draw") && draws_taken == 0, "a request for a sample in answer to a rest question is refused");

	/* ================ a request for a sample ================ */
	for (i = 0; i < 4; i++)
	{
		host_installed ();
		asks_draw = 1;
		accepted = i == 0 ? ask_kill (3, 1, 1) : (i == 1 ? ask_shape (1, 0, 0, 0, 0) : (i == 2 ? ask_neighbour (0, 1, 0, 1) : ask_fall (3)));
		check (!accepted && refused_with ("asked for a draw") && draws_taken == 0, i == 0 ? "a request for a sample in answer to a kill question is refused" : (i == 1 ? "... to a shape question" : (i == 2 ? "... to a neighbour question" : "... to a fall question")));
	}

	/* ================ VEHICLES ================ */
	host_installed ();
	world_clear ();
	world_add (2, 2, 1);
	world_add (2, 2, 0);
	world_add (2, 2, 1);
	accepted = ask_vehicles ();
	check (accepted && kills_applied == 2 && kill_order[0] == 1 && kill_order[1] == 3 && draws_taken == 0, "the campaign kills each living object the host names, in the host's order, and takes no sample");
	check (requests[0].bridge.op == CAMPAIGN_BRIDGE_VEHICLES && !requests[0].bridge.window_valid && requests[0].bridge.segment_length == 10.0f && requests[0].bridge.position[0] == 250.0f &&
		requests[0].bridge.sector_side_length == 100 && requests[0].bridge.max_x_sector == 2, "the segment's position and length (data) and the map's sector figures cross; the first request has no window");
	/* the object after a kill is read AFTER the kill: one that the kill unlinked is never offered */
	host_installed ();
	world_clear ();
	world_add (2, 2, 1);
	world_add (2, 2, 1);
	world_add (2, 2, 1);
	kill_unlinks_successor = 1;
	accepted = ask_vehicles ();
	check (accepted && kills_applied == 2 && kill_order[0] == 1 && kill_order[1] == 3, "an object a kill unlinked is not offered: the list continues as it is after the kill");
	/* a list of twenty is offered in windows of sixteen and four */
	host_installed ();
	world_clear ();
	for (i = 0; i < 20; i++)
	{
		world_add (2, 2, i == 17);
	}
	accepted = ask_vehicles ();
	check (accepted && kills_applied == 1 && kill_order[0] == 18, "a vehicle in the second window is reached");
	/* a changed host that kills nothing */
	host_installed ();
	vehicles_mode = 0;
	world_clear ();
	world_add (2, 2, 1);
	accepted = ask_vehicles ();
	check (accepted && kills_applied == 0, "a host that kills no vehicle (they ride it out) is obeyed");
	/* refusals */
	host_installed ();
	world_clear ();
	world_add (2, 2, 1);
	scripted.bridge.handle = 99;
	accepted = ask_vehicles ();
	check (!accepted && refused_with ("not a living object of the window") && kills_applied == 0, "an object that is not in the window is refused before anything is killed");
	host_installed ();
	world_clear ();
	world_add (2, 2, 0);
	world_add (2, 2, 1);
	scripted.bridge.handle = 1;
	accepted = ask_vehicles ();
	check (!accepted && refused_with ("not a living object of the window") && kills_applied == 0, "and a dead object (killing it would run its kill again)");
	host_installed ();
	world_clear ();
	fault = FAULT_SECTOR_OUTSIDE;
	accepted = ask_vehicles ();
	check (!accepted && refused_with ("malformed window"), "a window of a sector outside the map is refused");
	host_installed ();
	world_clear ();
	for (i = 0; i < 20; i++)
	{
		world_add (2, 2, 0);
	}
	fault = FAULT_WRONG_AFTER;
	accepted = ask_vehicles ();
	check (!accepted && refused_with ("not where the window ended"), "a continuation after an object other than the last offered is refused");
	host_installed ();
	world_clear ();
	for (i = 0; i < 20; i++)
	{
		world_add (2, 2, 0);
	}
	fault = FAULT_NEGATIVE_AFTER;
	accepted = ask_vehicles ();
	check (!accepted && refused_with ("malformed window"), "and a negative one");
	host_installed ();
	world_clear ();
	fault = FAULT_REVISIT;
	accepted = ask_vehicles ();
	check (!accepted && refused_with ("again"), "the start of the sector just walked, asked a second time, is refused");
	host_installed ();
	world_clear ();
	sweep = 1;
	walk_reset ();
	walk.max_x_sector = 99;
	walk.max_z_sector = 99;
	harness_message[0] = '\0';
	windows_supplied = 0;
	{
		static const float position[3] = { 250.0f, 5.0f, 250.0f };

		if (setjmp (harness_fatal))
		{
			accepted = 0;
		}
		else
		{
			campaign_aim_segment_vehicles (position, 10.0f, &walk);
			accepted = 1;
		}
	}
	check (!accepted && refused_with ("more than 4096 sectors") && windows_supplied == 4096, "a walk of more than 4096 fresh sectors is refused at the 4097th");
	host_installed ();
	host_fails = 1;
	world_clear ();
	accepted = ask_vehicles ();
	check (!accepted && refused_with ("the host has no model for this bridge"), "a failing host stops a vehicles walk with its reason");

	printf ("%d checks, %d failures\n", checks, failures);

	return failures ? 1 : 0;
}
