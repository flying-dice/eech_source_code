/* Bounded checks of the combat observer's own safety (EEDCS-46), against the REAL csrc/eech_cmb_fixture.c compiled over a stand-in engine (stub/project.h): the collision
   wrapper never reads the argument list or the health of a kind that has neither (a bridge is notified with no fourth argument), nesting past the frame table is counted and
   never written, a slot freed during a handler is not read as the receiver, the damage entry and exit counters balance, a repeated blast is told by the explosion invocation
   rather than by an address, and the timed health rolls are counted with the draws the original's condition took.  Run by run-observer-checks.sh (AddressSanitizer and
   UndefinedBehaviorSanitizer when the compiler has them). */
#include "project.h"

#include <stdlib.h>

#include "entity/mobile/mb_cmbfx.h"

#define POOL 16

static entity pool[POOL];
static int levels[POOL], alives[POOL], reads[POOL];
static char data_marker[POOL];

entity *entities;
int number_of_entities;
int random_number_seed;

int (*message_responses[NUM_ENTITY_TYPES][NUM_ENTITY_MESSAGES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

int get_local_entity_int_value (entity *en, int_types type)
{
	int slot = (int) (en - entities);

	reads[slot]++;

	switch (type)
	{
		case INT_TYPE_DAMAGE_LEVEL: return levels[slot];
		case INT_TYPE_ALIVE: return alives[slot];
		case INT_TYPE_PLAYER: return ENTITY_PLAYER_AI;
		case INT_TYPE_ENTITY_SUB_TYPE: return 7;
		case INT_TYPE_WEAPON_KILL_CODE: return 3;
		case INT_TYPE_WEAPON_WARHEAD_TYPE: return 2;
		default: return 0;
	}
}

const char *get_entity_type_name (entity_types type)
{
	static const char *names[NUM_ENTITY_TYPES] = { "UNKNOWN", "ANTI_AIRCRAFT", "BRIDGE", "CARGO", "CITY_BUILDING", "FIXED_WING", "HELICOPTER", "OBJECT", "PERSON", "PYLON",
		"ROUTED_VEHICLE", "SCENIC", "SEGMENT", "SHIP_VEHICLE", "SITE", "SITE_UPDATABLE", "WEAPON" };

	return names[type];
}

extern int eech_cmb_trace_get (char *text, int capacity);

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

/* the value of `key` on the report line that starts with `prefix` (and, for a kind row, names `type`), or -1 */
static long field (const char *prefix, const char *type, const char *key)
{
	static char report[200000];
	char needle[96];
	char *line, *at;

	eech_cmb_trace_get (report, sizeof (report));

	for (line = report; line && *line; line = strchr (line, '\n') ? strchr (line, '\n') + 1 : NULL)
	{
		if (strncmp (line, prefix, strlen (prefix)) != 0)
		{
			continue;
		}

		if (type)
		{
			/* a plain name is a kind row's type; a "key=value" selects a row by that field (a timed row by its kind) */
			snprintf (needle, sizeof (needle), strchr (type, '=') ? " %s " : " type=%s ", type);

			if (!strstr (line, needle) || strstr (line, needle) > strchr (line, '\n'))
			{
				continue;
			}
		}

		snprintf (needle, sizeof (needle), " %s=", key);

		at = strstr (line, needle);

		if (at && at < strchr (line, '\n'))
		{
			return strtol (at + strlen (needle), NULL, 10);
		}
	}

	return -1;
}

static int default_response (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	return 0;
}

static void (*scenario) (entity *receiver, entity *sender, double modifier);
static int bridge_called;

/* a handler of one of the thirteen kinds reads the damage modifier like the real ones, then does whatever the scenario says */
static int observed_response (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	double modifier = va_arg (pargs, double);

	if (scenario)
	{
		scenario (receiver, sender, modifier);
	}

	return 1;
}

/* a bridge's handler ignores the argument list (the segments notify it with no fourth argument) */
static int ignoring_response (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	bridge_called++;

	return 1;
}

static int notify (entity_messages message, entity *receiver, entity *sender, ...)
{
	va_list arguments;
	int acknowledge;

	va_start (arguments, sender);

	acknowledge = message_responses[receiver->type][message] (message, receiver, sender, arguments);

	va_end (arguments);

	return acknowledge;
}

static void set_entity (int slot, entity_types type, int level, int alive)
{
	pool[slot].type = type;
	pool[slot].data = &data_marker[slot];
	levels[slot] = level;
	alives[slot] = alive;
}

/* ---- scenarios ---- */

static void lethal_hit (entity *receiver, entity *sender, double modifier)
{
	eech_cmb_damage_begin (receiver, sender);
	eech_cmb_capability_begin ();
	eech_cmb_capability_end (sender, receiver, (float) modifier, 7, 2, 1.0f, 1.0f, 1.0f, 5);
	eech_cmb_damage_written (receiver, 0);

	levels[receiver - entities] = 0;
	alives[receiver - entities] = 0;

	eech_cmb_damage_end (receiver, sender, (float) modifier, 5);
}

static int next_nested;

static void nested (entity *receiver, entity *sender, double modifier)
{
	/* hooks that find the current frame run at every depth, including past the table */
	eech_cmb_capability_begin ();
	eech_cmb_capability_end (sender, receiver, 0.0f, 7, 2, 1.0f, 1.0f, 1.0f, 1);
	eech_cmb_damage_written (receiver, 1);

	if (next_nested < POOL - 1)
	{
		entity *next = &pool[next_nested++];

		notify (ENTITY_MESSAGE_COLLISION, next, sender, 0.0);
	}
}

static int reads_at_release;

/* the slot is freed during the handler and reused by another entity of the SAME kind, with the SAME data representation: type and data alone cannot tell it from the
   receiver, only the slot's generation can */
static void freed_and_reused_same_kind (entity *receiver, entity *sender, double modifier)
{
	int slot = (int) (receiver - entities);

	eech_cmb_release (receiver);

	reads_at_release = reads[slot];

	levels[slot] = 77;
	alives[slot] = 1;
}

/* an ordinary handler: the receiver loses health and keeps its slot */
static void damaged_in_place (entity *receiver, entity *sender, double modifier)
{
	levels[receiver - entities] -= 1;
}

static void freed_and_reused (entity *receiver, entity *sender, double modifier)
{
	int slot = (int) (receiver - entities);

	eech_cmb_release (receiver);

	reads_at_release = reads[slot];

	/* the slot is reused by another entity of another kind, with a health of its own */
	receiver->type = ENTITY_TYPE_SCENIC;
	levels[slot] = 99;
	alives[slot] = 1;
}

int main (void)
{
	int slot, type;

	/* unbuffered, so that a crash cannot swallow the lines already printed */
	setvbuf (stdout, NULL, _IONBF, 0);

	entities = pool;
	number_of_entities = POOL;
	random_number_seed = 12345;

	for (type = 0; type < NUM_ENTITY_TYPES; type++)
	{
		int message;

		for (message = 0; message < NUM_ENTITY_MESSAGES; message++)
		{
			message_responses[type][message] = default_response;
		}
	}

	for (type = 0; type < NUM_ENTITY_TYPES; type++)
	{
		if (type != ENTITY_TYPE_UNKNOWN)
		{
			message_responses[type][ENTITY_MESSAGE_COLLISION] = observed_response;
		}
	}

	message_responses[ENTITY_TYPE_BRIDGE][ENTITY_MESSAGE_COLLISION] = ignoring_response;

	eech_cmb_install ((void *) default_response);
	eech_cmb_install ((void *) default_response);

	check (field ("cmb-install", NULL, "wrapped_collision_handlers") == 16, "every one of the sixteen registered collision handlers is wrapped once, and installing twice does not wrap the wrapper");

	/* a bridge is notified with no fourth argument: the observer neither reads the argument list nor the bridge's health, and the original handler still runs */
	set_entity (1, ENTITY_TYPE_BRIDGE, 0, 1);
	set_entity (2, ENTITY_TYPE_SEGMENT, 3, 1);
	notify (ENTITY_MESSAGE_COLLISION, &pool[1], &pool[2]);
	check (bridge_called == 1, "the bridge's own handler ran with the original arguments");
	check (reads[1] == 0, "no health or flag of the bridge was read");
	check (field ("cmb-unobserved", "BRIDGE", "deliveries") == 1, "the bridge delivery is counted");
	check (field ("cmb-install", NULL, "observed_deliveries") == 0, "and it is not counted as an observed delivery, so it is never a false fatal");

	set_entity (3, ENTITY_TYPE_CARGO, 0, 1);
	notify (ENTITY_MESSAGE_COLLISION, &pool[3], &pool[2], 0.5);
	check (reads[3] == 0 && field ("cmb-unobserved", "CARGO", "deliveries") == 1, "a cargo (no damage level) is counted and never read");

	/* a lethal direct hit on a helicopter */
	set_entity (4, ENTITY_TYPE_HELICOPTER, 5, 1);
	set_entity (5, ENTITY_TYPE_WEAPON, 0, 1);
	scenario = lethal_hit;
	notify (ENTITY_MESSAGE_COLLISION, &pool[4], &pool[5], 0.0);
	scenario = NULL;
	check (field ("cmb-kind", "HELICOPTER", "collisions") == 1 && field ("cmb-kind", "HELICOPTER", "applied") == 1, "a hit on a helicopter is observed and its damage applied");
	check (field ("cmb-kind", "HELICOPTER", "fatal") == 1 && field ("cmb-kind", "HELICOPTER", "damage_sum") == 5, "its health went from 5 to 0: fatal, five points");
	check (field ("cmb-kind", "HELICOPTER", "destroyed") == 0, "and the receiver was still the same entity afterwards");
	check (field ("cmb-damage", NULL, "balanced") == 1 && field ("cmb-damage", NULL, "entries") == 1, "one damage entry, one exit");

	/* nesting: eight frames, then deeper; the ninth and beyond are counted, not written, and the trace says it is untrusted */
	for (slot = 6; slot < POOL; slot++)
	{
		set_entity (slot, ENTITY_TYPE_PERSON, 3, 1);
	}

	next_nested = 7;
	scenario = nested;
	notify (ENTITY_MESSAGE_COLLISION, &pool[6], &pool[5], 0.0);
	scenario = NULL;
	check (field ("cmb-integrity", NULL, "depth_max") >= 9, "the real nesting depth is retained past the table (at least 9)");
	check (field ("cmb-integrity", NULL, "depth_overflow") >= 1, "the frames past the table are counted");
	check (field ("cmb-integrity", NULL, "trusted") == 0, "and the trace says it is not trustworthy");
	check (field ("cmb-install", NULL, "observed_deliveries") == 1 + 8, "only the frames inside the table were recorded (the eight inner ones and the first hit)");

	/* a slot freed during the handler, and reused by another entity of another kind, is not read as the receiver */
	set_entity (15, ENTITY_TYPE_SITE, 4, 1);
	scenario = freed_and_reused;
	notify (ENTITY_MESSAGE_COLLISION, &pool[15], &pool[5], 0.25);
	scenario = NULL;
	check (reads[15] == reads_at_release, "nothing was read from the slot after it was freed");
	check (field ("cmb-kind", "SITE", "destroyed") == 1 && field ("cmb-kind", "SITE", "fatal") == 1, "the receiver is reported destroyed");
	check (field ("cmb-kind", "SCENIC", "collisions") == 0 || field ("cmb-kind", "SCENIC", "collisions") == -1, "and the entity that reused the slot is not credited with the collision");

	/* the same, when the reusing entity has the same kind and the same data: an ordinary handler leaves the receiver in place (not destroyed); a handler that frees the slot is
	   reported destroyed although the slot's type and data are exactly what they were */
	set_entity (0, ENTITY_TYPE_OBJECT, 6, 1);
	scenario = damaged_in_place;
	notify (ENTITY_MESSAGE_COLLISION, &pool[0], &pool[5], 0.5);
	check (field ("cmb-kind", "OBJECT", "collisions") == 1 && field ("cmb-kind", "OBJECT", "destroyed") == 0 && field ("cmb-kind", "OBJECT", "fatal") == 0,
		"an object that kept its slot is the same receiver afterwards: not destroyed, not fatal");
	scenario = freed_and_reused_same_kind;
	notify (ENTITY_MESSAGE_COLLISION, &pool[0], &pool[5], 0.5);
	scenario = NULL;
	check (pool[0].type == ENTITY_TYPE_OBJECT && pool[0].data == &data_marker[0], "(the slot still holds the same kind and the same data after the handler)");
	check (field ("cmb-kind", "OBJECT", "collisions") == 2 && field ("cmb-kind", "OBJECT", "destroyed") == 1 && field ("cmb-kind", "OBJECT", "fatal") == 1,
		"a slot freed and reused by an entity of the same kind and data is still reported destroyed: the generation, not the type and data, decides");
	check (reads[0] == reads_at_release, "and nothing was read from that slot after it was freed");

	/* every damage entry has exactly one exit */
	eech_cmb_damage_begin (&pool[4], NULL);
	eech_cmb_damage_exit (EECH_CMB_DAMAGE_EXIT_NO_WEAPON);
	eech_cmb_damage_begin (&pool[4], &pool[5]);
	eech_cmb_damage_exit (EECH_CMB_DAMAGE_EXIT_INVULNERABLE);
	check (field ("cmb-damage", NULL, "balanced") == 1 && field ("cmb-damage", NULL, "exit_no_weapon") == 1 && field ("cmb-damage", NULL, "exit_invulnerable") == 1,
		"the no-weapon and invulnerable exits are counted as entries with exits");
	eech_cmb_damage_begin (&pool[4], &pool[5]);
	check (field ("cmb-damage", NULL, "balanced") == 0, "an entry with no exit unbalances the counters");

	/* a repeated blast is told by the explosion invocation, not by a weapon's address */
	eech_cmb_explosion_begin (3);
	eech_cmb_blast_begin (&pool[5], NULL, 10, 3.0f);
	eech_cmb_blast_end ();
	eech_cmb_blast_begin (&pool[5], NULL, 10, 3.0f);
	eech_cmb_blast_end ();
	eech_cmb_explosion_end ();
	eech_cmb_explosion_begin (3);
	eech_cmb_blast_begin (&pool[5], NULL, 10, 3.0f);
	eech_cmb_blast_end ();
	eech_cmb_explosion_end ();
	eech_cmb_blast_begin (&pool[5], NULL, 10, 3.0f);
	eech_cmb_blast_end ();
	check (field ("cmb-explosion", NULL, "invocations") == 2 && field ("cmb-explosion", NULL, "with_repeated_blast") == 1 && field ("cmb-explosion", NULL, "max_blast_calls") == 2,
		"two blasts in one invocation are a repetition; one blast in each of two invocations by the same weapon address are not");
	check (field ("cmb-explosion", NULL, "blast_outside_invocation") == 1, "a blast outside any invocation is counted");

	/* timed health rolls: the attempt, the draws the original's condition took, and the hit */
	eech_cmb_timed_begin (EECH_CMB_TIMED_SHIP_FIRE);
	eech_cmb_timed_end ();
	eech_cmb_timed_begin (EECH_CMB_TIMED_ANTI_AIRCRAFT_FIRE);
	random_number_seed = (int) ((uint32_t) random_number_seed * 1103515245u + 12345u);
	eech_cmb_timed_hit (&pool[4], 0);
	eech_cmb_timed_end ();
	/* a roll that drew but did not fire is an attempt with a draw and no hit */
	eech_cmb_timed_begin (EECH_CMB_TIMED_HELICOPTER);
	random_number_seed = (int) ((uint32_t) random_number_seed * 1103515245u + 12345u);
	eech_cmb_timed_end ();

	check (field ("cmb-timed", "kind=6", "attempts") == 1 && field ("cmb-timed", "kind=6", "draws") == 0 && field ("cmb-timed", "kind=6", "hits") == 0 && field ("cmb-timed", "kind=6", "fatal") == 0,
		"a roll whose condition took no draw and did not fire: one attempt, no draw, no hit, not fatal");
	check (field ("cmb-timed", "kind=4", "attempts") == 1 && field ("cmb-timed", "kind=4", "draws") == 1 && field ("cmb-timed", "kind=4", "hits") == 1 && field ("cmb-timed", "kind=4", "fatal") == 1,
		"a roll that drew once and fired to zero health: one attempt (the END after the hit does not count it twice), one draw, one fatal hit");
	check (field ("cmb-timed", "kind=1", "attempts") == 1 && field ("cmb-timed", "kind=1", "draws") == 1 && field ("cmb-timed", "kind=1", "hits") == 0,
		"a roll that drew once and did not fire: one attempt, one draw, no hit");
	check (field ("cmb-timed", "kind=3", "attempts") == 0 && field ("cmb-timed", "kind=2", "attempts") == 0, "the kinds that were not rolled have no attempts");

	printf ("%s: %d checks, %d failures\n", failures ? "OBSERVER_CHECKS=FAILED" : "OBSERVER_CHECKS=ok", checks, failures);

	return failures ? 1 : 0;
}
