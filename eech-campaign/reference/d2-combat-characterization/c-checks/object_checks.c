/* Checks of the host-issued identity of the physical objects that are not rostered mobiles and of the physical facts the host reports about them (EEDCS-46, R3): the REAL
   aphavoc/source/entity/mobile/mb_hostobj.c (the structures' roster and the report application) and the REAL mb_hostaim.c (the IDENTIFY exchange and its validation), compiled against object-stub/project.h
   and driven with a scripted host over a small fake entity heap.  The engine operations the registry applies (set a damage level, kill an object) are recorded, so the order and the once-only effect of a report
   are observable; the entity list is a plain array whose slots the checks release and reuse.  It is NOT a native run.  Run by run-object-checks.sh. */
#include "project.h"

#include "entity/mobile/mb_hostaim.h"

#include "entity/mobile/mb_hostobj.h"

jmp_buf harness_fatal;
char harness_message[512];

struct ENTITY *fake_entities = NULL;
int number_of_entities = 0;
int out_of_range_reads = 0;
static struct ENTITY fake_dummy;

struct ENTITY *fake_entity_at (int index)
{
	if ((index < 0) || (index >= number_of_entities))
	{
		out_of_range_reads++;

		return &fake_dummy;
	}

	return &fake_entities[index];
}
comms_model_types fake_comms_model = COMMS_MODEL_SERVER;

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

int get_local_entity_index (entity *en)
{
	return (int) (en - fake_entities);
}

/* the instrumented entity access: a slot outside the capacity the run allocated is COUNTED and not touched (so a check reports the access that would have been out of bounds, whatever the memory behind it) */
int type_reads_outside = 0, entity_touches_outside = 0;

static int entity_outside (entity *en)
{
	long slot = (long) (en - fake_entities);

	return (slot < 0) || (slot >= number_of_entities);
}

entity_types get_local_entity_type (entity *en)
{
	if (entity_outside (en))
	{
		type_reads_outside++;

		return ENTITY_TYPE_UNKNOWN;
	}

	return (entity_types) en->type;
}

int get_local_entity_int_value (entity *en, int_types type)
{
	if (entity_outside (en))
	{
		entity_touches_outside++;

		return 0;
	}

	switch (type)
	{
		case INT_TYPE_ALIVE: return en->alive;
		case INT_TYPE_DAMAGE_LEVEL: return en->damage_level;
		case INT_TYPE_INITIAL_DAMAGE_LEVEL: return en->initial_damage_level;
		case INT_TYPE_ENTITY_SUB_TYPE: return en->sub_type;
		case INT_TYPE_SIDE: return en->side;
	}

	return 0;
}

vec3d *get_local_entity_vec3d_ptr (entity *en, vec3d_types type)
{
	if (entity_outside (en))
	{
		entity_touches_outside++;

		return NULL;
	}

	return en->has_position ? &en->position : NULL;
}

/* what the registry applied, in order: 'S' a damage level set, 'K' a kill */
static char applied[64];
static int applied_count, applied_levels[64], applied_index[64];

void set_client_server_entity_int_value (entity *en, int_types type, int value)
{
	if (entity_outside (en))
	{
		entity_touches_outside++;

		return;
	}

	en->damage_level = value;
	applied[applied_count] = 'S';
	applied_levels[applied_count] = value;
	applied_index[applied_count++] = get_local_entity_index (en);
}

void kill_client_server_entity (entity *en)
{
	if (entity_outside (en))
	{
		entity_touches_outside++;

		return;
	}

	en->alive = 0;
	applied[applied_count] = 'K';
	applied_levels[applied_count] = 0;
	applied_index[applied_count++] = get_local_entity_index (en);
}

/* The roster, as a table the checks can grow: the identities the roster issued, each live (state 1, on an entity) or ended (state 2).  It starts with one rostered mobile (entity 7, "mobile-1") and one ended
   incarnation ("mobile-0").  The binding of a later member goes through the REAL gp_rosterid.c (campaign_roster_accept_host_id, the function campaign_roster_link_member calls before it commits a record);
   `roster_has` below is the roster's own record scan, and `roster_commit` is the commit that follows an accepted identity. */
#define ROSTER_MAX 16
static char roster_id[ROSTER_MAX][CAMPAIGN_OBJECT_ID_MAX + 1];
static int roster_state[ROSTER_MAX], roster_entity[ROSTER_MAX], roster_count;

static void roster_commit (const char *id, int state, int entity_index)
{
	snprintf (roster_id[roster_count], sizeof (roster_id[roster_count]), "%s", id);
	roster_state[roster_count] = state;
	roster_entity[roster_count] = entity_index;
	roster_count++;
}

static void roster_reset (void)
{
	roster_count = 0;
	roster_commit ("mobile-1", 1, 7);
	roster_commit ("mobile-0", 2, -1);
}

const char *campaign_roster_host_id (entity *en)
{
	int i;

	for (i = 0; i < roster_count; i++)
	{
		if (roster_state[i] == 1 && roster_entity[i] == get_local_entity_index (en))
		{
			return roster_id[i];
		}
	}

	return NULL;
}

int campaign_roster_find_host_id (const char *host_id, entity **member)
{
	int i;

	if (member)
	{
		*member = NULL;
	}

	for (i = 0; i < roster_count; i++)
	{
		if (strcmp (roster_id[i], host_id) == 0)
		{
			if (roster_state[i] == 1 && member)
			{
				*member = &fake_entities[roster_entity[i]];
			}

			return roster_state[i];
		}
	}

	return 0;
}

static int roster_has (const char *host_id)
{
	return campaign_roster_find_host_id (host_id, NULL) != 0;
}

/* the production function the roster calls before it commits a binding (gp_rosterid.c) */
void campaign_roster_accept_host_id (const char *host_id, int (*roster_has_fn) (const char *host_id));

static int checks, failures;

static void check (int ok, const char *what)
{
	checks++;

	if (!ok)
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

/* ---- the scripted host: it issues "obj-<n>" (its own count) unless a check scripts something else */
static int host_calls, host_serial_seen[16], host_serial_count;
static campaign_identify_request last_request;
static const char *scripted_id;
static int asks_draw;
static int issued_count;

static int host_aim (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	host_calls++;

	memset (result, 0, sizeof (*result));

	if (step->mode != CAMPAIGN_AIM_IDENTIFY)
	{
		snprintf (error, error_size, "unexpected mode %d", step->mode);

		return 1;
	}

	last_request = step->identify;

	if (host_serial_count < 16)
	{
		host_serial_seen[host_serial_count++] = step->identify.serial;
	}

	if (asks_draw)
	{
		result->state = CAMPAIGN_AIM_NEED_DRAW;

		return 0;
	}

	result->state = CAMPAIGN_AIM_DONE;

	if (scripted_id)
	{
		/* memcpy of a field's width, so that a check can hand over an identity that does not end inside it */
		memcpy (result->identify.host_id, scripted_id, strlen (scripted_id) < sizeof (result->identify.host_id) ? strlen (scripted_id) + 1 : sizeof (result->identify.host_id));
	}
	else
	{
		snprintf (result->identify.host_id, sizeof (result->identify.host_id), "obj-%d", ++issued_count);
	}

	return 0;
}

/* a run with a heap of `capacity` slots: allocated exactly that big (initialise_entity_heap), far below the compile-time ceiling MAX_NUM_ENTITIES, and the registry and the roster start afresh */
static void world_reset_with (int capacity)
{
	campaign_aim_host host;

	free (fake_entities);
	fake_entities = (struct ENTITY *) calloc ((size_t) capacity, sizeof (struct ENTITY));
	number_of_entities = capacity;
	out_of_range_reads = 0;
	type_reads_outside = entity_touches_outside = 0;
	roster_reset ();
	campaign_object_reset ();
	memset (&host, 0, sizeof (host));
	host.aim = host_aim;
	campaign_aim_set_host (&host);
	host_calls = host_serial_count = issued_count = 0;
	scripted_id = NULL;
	asks_draw = 0;
	applied_count = 0;
	fake_comms_model = COMMS_MODEL_SERVER;
	harness_message[0] = '\0';
	memset (&last_request, 0, sizeof (last_request));
}

static void world_reset (void)
{
	world_reset_with (64);
}

static void place (int index, entity_types type, int alive, int damage, int initial)
{
	fake_entities[index].type = type;
	fake_entities[index].alive = alive;
	fake_entities[index].damage_level = damage;
	fake_entities[index].initial_damage_level = initial;
	fake_entities[index].sub_type = 40 + index;
	fake_entities[index].side = 2;
	fake_entities[index].has_position = TRUE;
	fake_entities[index].position.x = 100.0f * index;
	fake_entities[index].position.y = 5.0f;
	fake_entities[index].position.z = 250.0f;
}

static const char *identified;

static int try_identify (int index)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	identified = campaign_object_identify (&fake_entities[index]);

	return 1;
}

/* the cached identity of an entity, looked up for the slot `index` (which may be the capacity: only the address is formed); 1 if it returned (the identity in `identified`, NULL if none), 0 if refused */
static int try_host_id (int index)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	identified = campaign_object_host_id (&fake_entities[index]);

	return 1;
}

/* a later bind: the host returned `id` for a new group member; the REAL acceptance runs, and only an accepted identity is committed to the roster (as campaign_roster_link_member does) */
static int try_bind_member (const char *id, int entity_index)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	campaign_roster_accept_host_id (id, roster_has);
	roster_commit (id, 1, entity_index);

	return 1;
}

static campaign_object_report_receipt receipt;

static int try_report (const char *id, int has_level, int level, int destroyed)
{
	campaign_object_report report;

	memset (&report, 0, sizeof (report));
	snprintf (report.host_id, sizeof (report.host_id), "%s", id);
	report.has_damage_level = has_level;
	report.damage_level = level;
	report.destroyed = destroyed;
	memset (&receipt, 0, sizeof (receipt));
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	campaign_object_report_apply (&report, &receipt);

	return 1;
}

static int collected, collected_alive;
static char collected_first[CAMPAIGN_OBJECT_ID_MAX + 1];
static int sorted_subtypes[256], sorted_count;

static void collect (const campaign_object_snapshot *object, void *user)
{
	if (collected == 0)
	{
		snprintf (collected_first, sizeof (collected_first), "%s", object->host_id);
	}

	if (sorted_count < 256)
	{
		sorted_subtypes[sorted_count++] = object->sub_type;
	}

	collected++;
	collected_alive += object->alive;
}

int main (void)
{
	char first[CAMPAIGN_OBJECT_ID_MAX + 1];
	char second[CAMPAIGN_OBJECT_ID_MAX + 1];
	int accepted;

	/* ================ the identity is asked of the host once, from the object's facts by value ================ */
	world_reset ();
	place (1, ENTITY_TYPE_SITE, 1, 80, 100);
	accepted = try_identify (1);
	check (accepted && identified && strcmp (identified, "obj-1") == 0 && host_calls == 1, "a structure is identified by asking the host");
	check (last_request.entity_type == ENTITY_TYPE_SITE && last_request.sub_type == 41 && last_request.side == 2 && last_request.alive == 1 && last_request.damage_level == 80 &&
		last_request.initial_damage_level == 100 && last_request.position[0] == 100.0f && last_request.position[2] == 250.0f && last_request.serial == 1, "the facts cross by value, with the campaign's own count and no heap index");
	snprintf (first, sizeof (first), "%s", identified);
	accepted = try_identify (1);
	check (accepted && strcmp (identified, first) == 0 && host_calls == 1, "and once: the identity is kept, the host is not asked again");
	check (campaign_object_host_id (&fake_entities[1]) && strcmp (campaign_object_host_id (&fake_entities[1]), first) == 0, "the identity is readable without calling the host");

	/* ================ the objects that are not structures ================ */
	place (5, ENTITY_TYPE_WEAPON, 1, 1, 1);
	place (6, ENTITY_TYPE_BRIDGE, 1, 0, 0);
	place (7, ENTITY_TYPE_ROUTED_VEHICLE, 1, 50, 50);
	place (8, ENTITY_TYPE_ROUTED_VEHICLE, 1, 50, 50);
	host_calls = 0;
	accepted = try_identify (5);
	check (accepted && identified == NULL && host_calls == 0, "a weapon has no identity and the host is not asked");
	accepted = try_identify (6);
	check (accepted && identified == NULL && host_calls == 0, "a bridge has none either (its segments do)");
	accepted = try_identify (7);
	check (accepted && identified && strcmp (identified, "mobile-1") == 0 && host_calls == 0, "a rostered mobile keeps the identity of the roster: it is not bound twice");
	accepted = try_identify (8);
	check (accepted && identified == NULL && host_calls == 0, "an unrostered mobile has none");

	/* ================ what the host may issue ================ */
	place (2, ENTITY_TYPE_PYLON, 1, 10, 10);
	scripted_id = first;
	accepted = try_identify (2);
	check (!accepted && refused_with ("which it issued before") && campaign_object_host_id (&fake_entities[2]) == NULL, "an identity the host issued before is refused and binds nothing");
	scripted_id = "mobile-1";
	accepted = try_identify (2);
	check (!accepted && refused_with ("of a rostered member"), "so is the identity of a rostered member");
	scripted_id = "mobile-0";
	accepted = try_identify (2);
	check (!accepted && refused_with ("of a rostered member"), "and one a roster member had once");
	scripted_id = "";
	accepted = try_identify (2);
	check (!accepted && refused_with ("issued no identity"), "an empty identity is refused");
	{
		char long_id[200];

		memset (long_id, 'x', sizeof (long_id));
		long_id[199] = '\0';
		scripted_id = long_id;
		accepted = try_identify (2);
		check (!accepted && refused_with ("issued no identity"), "one that does not end inside its 128 bytes is refused");
	}
	scripted_id = "bad\nidentity";
	accepted = try_identify (2);
	check (!accepted && refused_with ("control character"), "one with a control character is refused");
	scripted_id = NULL;
	asks_draw = 1;
	accepted = try_identify (2);
	check (!accepted && refused_with ("identify request with state"), "a sample asked for in answer to an identify question is refused");
	asks_draw = 0;
	accepted = try_identify (2);
	check (accepted && identified && strcmp (identified, "obj-2") == 0, "after the refusals the next structure is identified normally");

	/* ================ the incarnation ends with the entity ================ */
	world_reset ();
	place (1, ENTITY_TYPE_SEGMENT, 1, 500, 500);
	try_identify (1);
	snprintf (first, sizeof (first), "%s", identified);
	campaign_object_release_entity (&fake_entities[1]);
	check (campaign_object_host_id (&fake_entities[1]) == NULL, "a released structure has no identity");
	accepted = try_report (first, 1, 0, 0);
	check (!accepted && refused_with ("no longer exists") && applied_count == 0, "a report with the identity of an ended incarnation is refused and applies nothing");
	/* the slot is reused by another object */
	place (1, ENTITY_TYPE_SEGMENT, 1, 500, 500);
	accepted = try_report (first, 1, 0, 0);
	check (!accepted && refused_with ("no longer exists") && applied_count == 0 && fake_entities[1].alive == 1, "and so is one whose entity slot now holds another object: that object is untouched");
	try_identify (1);
	snprintf (second, sizeof (second), "%s", identified);
	check (strcmp (first, second) != 0, "the new incarnation is issued a new identity");
	accepted = try_report (second, 1, 400, 0);
	check (accepted && fake_entities[1].damage_level == 400, "which does name it");

	/* ================ a report applies the collision steps, once ================ */
	world_reset ();
	place (1, ENTITY_TYPE_SEGMENT, 1, 500, 500);
	place (2, ENTITY_TYPE_SITE, 1, 100, 100);
	place (3, ENTITY_TYPE_PYLON, 1, 60, 60);
	place (7, ENTITY_TYPE_ROUTED_VEHICLE, 1, 50, 50);
	try_identify (1);
	snprintf (first, sizeof (first), "%s", identified);
	try_identify (2);
	snprintf (second, sizeof (second), "%s", identified);
	accepted = try_report (first, 0, 0, 1);
	check (accepted && applied_count == 1 && applied[0] == 'K' && applied_index[0] == 1 && fake_entities[1].damage_level == 500 && receipt.killed && receipt.damage_level_before == 500 && receipt.damage_level_after == 500,
		"a destruction kills the structure and leaves its stored damage level as it was (a dead segment keeps 500)");
	accepted = try_report (first, 0, 0, 1);
	check (!accepted && refused_with ("already destroyed") && applied_count == 1, "the fact is applied once: a second report of it is refused");
	applied_count = 0;
	accepted = try_report (second, 1, 40, 0);
	check (accepted && applied_count == 1 && applied[0] == 'S' && applied_levels[0] == 40 && !receipt.killed && fake_entities[2].alive == 1 && receipt.damage_level_after == 40, "a damage level above zero is set and the structure lives");
	accepted = try_report (second, 1, 41, 0);
	check (!accepted && refused_with ("cannot be repaired") && fake_entities[2].damage_level == 40, "a report cannot raise a damage level");
	accepted = try_report (second, 1, -1, 0);
	check (!accepted && refused_with ("cannot be repaired"), "or lower it below zero");
	applied_count = 0;
	accepted = try_report (second, 1, 0, 0);
	check (accepted && applied_count == 2 && applied[0] == 'S' && applied[1] == 'K' && applied_levels[0] == 0 && receipt.killed && fake_entities[2].alive == 0, "a damage level of zero is set and then the structure is killed, in that order");
	accepted = try_report (second, 1, 0, 0);
	check (!accepted && refused_with ("already destroyed"), "and it is destroyed once");
	accepted = try_report ("never-issued", 1, 0, 0);
	check (!accepted && refused_with ("was never issued"), "an identity nobody issued is refused");
	accepted = try_report ("", 1, 0, 0);
	check (!accepted && refused_with ("without an identity"), "a report without an identity is refused");
	accepted = try_report (first, 0, 0, 0);
	check (!accepted && refused_with ("states neither"), "a report that says nothing is refused");
	accepted = try_report (first, 0, 5, 1);
	check (!accepted && refused_with ("states neither"), "and one with a level but no statement of it");
	applied_count = 0;
	accepted = try_report ("mobile-1", 1, 20, 0);
	check (accepted && receipt.rostered && applied_count == 1 && applied[0] == 'S' && applied_index[0] == 7, "a rostered mobile is reported by the identity of the roster, through the same steps");
	accepted = try_report ("mobile-0", 1, 0, 0);
	check (!accepted && refused_with ("no longer exists"), "a rostered member that ended is refused");
	place (4, ENTITY_TYPE_PYLON, 1, 30, 30);
	try_identify (4);
	fake_comms_model = COMMS_MODEL_CLIENT;
	applied_count = 0;
	accepted = try_report (identified, 1, 0, 0);
	check (!accepted && refused_with ("server only") && applied_count == 0, "a client applies nothing");
	fake_comms_model = COMMS_MODEL_SERVER;

	/* ================ the structures, identified now if they have no identity yet ================ */
	world_reset ();
	place (1, ENTITY_TYPE_SITE, 1, 100, 100);
	place (2, ENTITY_TYPE_SCENIC, 0, 0, 10);
	place (3, ENTITY_TYPE_CITY_BUILDING, 1, 20, 20);
	place (4, ENTITY_TYPE_WEAPON, 1, 1, 1);
	place (5, ENTITY_TYPE_ROUTED_VEHICLE, 1, 50, 50);
	collected = collected_alive = 0;
	check (campaign_object_structures (collect, NULL) == 3 && collected == 3 && collected_alive == 2 && host_calls == 3 && strcmp (collected_first, "obj-1") == 0, "the structures are listed, in entity-list order, each identified by the host once");
	collected = 0;
	check (campaign_object_structures (collect, NULL) == 3 && host_calls == 3, "and listed again without asking the host");

	/* ================ ONE namespace, both orders (the roster's binding goes through the real gp_rosterid.c) ================ */
	world_reset ();
	place (1, ENTITY_TYPE_SITE, 1, 100, 100);
	place (2, ENTITY_TYPE_SITE, 1, 100, 100);
	place (3, ENTITY_TYPE_SITE, 1, 100, 100);
	place (7, ENTITY_TYPE_ROUTED_VEHICLE, 1, 50, 50);
	place (8, ENTITY_TYPE_ROUTED_VEHICLE, 1, 50, 50);
	try_identify (1);
	snprintf (first, sizeof (first), "%s", identified);
	/* a member bound LATER may not take the identity of a LIVE structure */
	accepted = try_bind_member (first, 8);
	check (!accepted && refused_with ("already issued for a structure") && roster_count == 2 && campaign_object_host_id (&fake_entities[8]) == NULL, "a member bound later cannot take the identity of a live structure, and nothing is committed");
	accepted = try_report (first, 1, 60, 0);
	check (accepted && fake_entities[1].damage_level == 60 && applied_count == 1 && applied_index[0] == 1, "and a report by that identity still reaches the structure, never the refused member");
	/* ... nor that of a structure whose incarnation ENDED */
	campaign_object_release_entity (&fake_entities[1]);
	accepted = try_bind_member (first, 8);
	check (!accepted && refused_with ("already issued for a structure") && roster_count == 2, "nor the identity of a structure that ended (no identity is ever reused)");
	accepted = try_report (first, 1, 0, 0);
	check (!accepted && refused_with ("no longer exists"), "so the ended identity still reads as ended, not as the member's");
	/* ... nor, from the same entity slot, once another structure lives there */
	place (1, ENTITY_TYPE_SITE, 1, 100, 100);
	try_identify (1);
	snprintf (second, sizeof (second), "%s", identified);
	check (strcmp (first, second) != 0, "the structure that now lives in the slot is issued another identity");
	accepted = try_bind_member (first, 1);
	check (!accepted && refused_with ("already issued for a structure") && roster_count == 2, "a member cannot take the old identity even in the slot its structure had");
	accepted = try_bind_member (second, 8);
	check (!accepted && refused_with ("already issued for a structure"), "nor the live identity of the structure now in that slot");
	/* the converse order: a structure may not take any identity of the roster, live or ended */
	accepted = try_bind_member ("member-2", 8);
	check (accepted && roster_count == 3 && campaign_object_host_id (&fake_entities[8]) && strcmp (campaign_object_host_id (&fake_entities[8]), "member-2") == 0, "a fresh identity binds the member");
	scripted_id = "member-2";
	accepted = try_identify (2);
	check (!accepted && refused_with ("of a rostered member") && campaign_object_host_id (&fake_entities[2]) == NULL, "a structure cannot take the identity of a live member");
	fake_entities[8].type = ENTITY_TYPE_UNKNOWN;
	roster_state[2] = 2;
	accepted = try_identify (2);
	check (!accepted && refused_with ("of a rostered member"), "nor that of a member that ended");
	scripted_id = NULL;
	/* the existing refusals of the roster's acceptance are the same ones as before */
	accepted = try_bind_member ("mobile-1", 8);
	check (!accepted && refused_with ("was reused for a new physical incarnation"), "a member cannot take a member's identity (live)");
	accepted = try_bind_member ("mobile-0", 8);
	check (!accepted && refused_with ("was reused for a new physical incarnation"), "or an ended member's");
	accepted = try_bind_member ("", 8);
	check (!accepted && refused_with ("invalid identity length"), "an empty identity is refused");
	accepted = try_bind_member ("caf\xc3(", 8);
	check (!accepted && refused_with ("non-UTF-8"), "and one that is not UTF-8");
	accepted = try_bind_member ("caf\xc3\xa9", 8);
	check (accepted, "while a UTF-8 identity is accepted");

	/* ================ the common source/target envelope (the REAL campaign_aim_identify over the registry and the roster) ================ */
	world_reset ();
	place (1, ENTITY_TYPE_SITE, 1, 100, 100);
	place (5, ENTITY_TYPE_WEAPON, 1, 1, 1);
	place (7, ENTITY_TYPE_ROUTED_VEHICLE, 1, 50, 50);
	{
		campaign_aim_step envelope;

		memset (&envelope, 0, sizeof (envelope));
		campaign_aim_identify (&envelope, &fake_entities[7], &fake_entities[1]);
		check (strcmp (envelope.source_host_id, "mobile-1") == 0 && strcmp (envelope.target_host_id, "obj-1") == 0 && host_calls == 1, "a rostered launcher keeps the roster's identity and a structure target is identified (asked of the host once, at its first physical need)");
		memset (&envelope, 0, sizeof (envelope));
		campaign_aim_identify (&envelope, &fake_entities[7], &fake_entities[1]);
		check (strcmp (envelope.target_host_id, "obj-1") == 0 && host_calls == 1, "and carried again without asking the host");
		memset (&envelope, 0, sizeof (envelope));
		campaign_aim_identify (&envelope, &fake_entities[5], NULL);
		check (envelope.source_host_id[0] == '\0' && envelope.target_host_id[0] == '\0' && host_calls == 1, "a weapon has no identity and no subject is none, and neither asks the host");
		campaign_object_release_entity (&fake_entities[1]);
		place (1, ENTITY_TYPE_SITE, 1, 100, 100);
		memset (&envelope, 0, sizeof (envelope));
		campaign_aim_identify (&envelope, NULL, &fake_entities[1]);
		check (strcmp (envelope.target_host_id, "obj-2") == 0 && host_calls == 2, "an object that took the released slot is a new incarnation with a new identity, never the old one");
	}

	/* ================ the heap a run allocated is SMALLER than the compile-time ceiling (EEDCS-46, R3: the real Windows runs read past it) ================ */
	{
		static const int capacities[] = { 8, 20, 37, 64, 129 };
		int c;

		for (c = 0; c < (int) (sizeof (capacities) / sizeof (capacities[0])); c++)
		{
			int capacity = capacities[c], last = capacity - 1, middle = capacity / 2 > 5 ? capacity / 2 : 5, listed, i, in_order;
			char what[220];

			world_reset_with (capacity);
			place (1, ENTITY_TYPE_SITE, 1, 90, 90);
			place (middle, ENTITY_TYPE_SEGMENT, 1, 500, 500);
			place (last, ENTITY_TYPE_PYLON, 0, 0, 10);
			place (2, ENTITY_TYPE_WEAPON, 1, 1, 1);
			place (4, ENTITY_TYPE_ROUTED_VEHICLE, 1, 50, 50);
			sorted_count = 0;
			collected = collected_alive = 0;
			listed = campaign_object_structures (collect, NULL);
			in_order = 1;

			for (i = 1; i < sorted_count; i++)
			{
				in_order = in_order && sorted_subtypes[i - 1] < sorted_subtypes[i];
			}

			snprintf (what, sizeof (what), "heap of %d slots (ceiling %d): every structure in range is listed, in slot order, and the last slot is reached", capacity, MAX_NUM_ENTITIES);
			check (listed == 3 && collected == 3 && sorted_count == 3 && in_order && sorted_subtypes[0] == 41 && sorted_subtypes[2] == 40 + last, what);
			snprintf (what, sizeof (what), "heap of %d slots: no slot outside the allocated capacity was read", capacity);
			check (out_of_range_reads == 0, what);
			collected = 0;
			listed = campaign_object_structures (collect, NULL);
			snprintf (what, sizeof (what), "heap of %d slots: listed again, the identities held, still inside the capacity and without asking the host again", capacity);
			check (listed == 3 && out_of_range_reads == 0 && host_calls == 3, what);
			campaign_object_release_entity (&fake_entities[last]);
			snprintf (what, sizeof (what), "heap of %d slots: the identity of the last slot ends with it, inside the capacity", capacity);
			check (campaign_object_host_id (&fake_entities[last]) == NULL && out_of_range_reads == 0, what);
		}

		/* more than one runtime capacity, with the reset the engine does at boot: the previous heap's identities and map are forgotten */
		world_reset_with (20);
		place (19, ENTITY_TYPE_SITE, 1, 90, 90);
		check (try_identify (19) && campaign_object_host_id (&fake_entities[19]) != NULL, "a heap of 20 slots identifies its last slot");
		world_reset_with (37);
		place (36, ENTITY_TYPE_SITE, 1, 90, 90);
		check (try_identify (36) && campaign_object_host_id (&fake_entities[36]) != NULL && out_of_range_reads == 0, "after a reset a heap of 37 slots identifies its last slot, the earlier map forgotten");
		/* a heap that changes size under live identities is refused, not re-read */
		number_of_entities = 20;
		harness_message[0] = '\0';
		if (setjmp (harness_fatal))
		{
			accepted = 0;
		}
		else
		{
			collected = 0;
			campaign_object_structures (collect, NULL);
			accepted = 1;
		}
		check (!accepted && refused_with ("changed from 37 to 20 slots"), "a heap whose size changes while identities exist is refused, not re-read");
		/* ... on the identify path too: a structure in range of the new size is not identified into a map of the old one */
		number_of_entities = 20;
		place (5, ENTITY_TYPE_SITE, 1, 90, 90);
		accepted = try_identify (5);
		check (!accepted && refused_with ("changed from 37 to 20 slots"), "an identity is not issued into a map of another heap size either");
		number_of_entities = 0;
		harness_message[0] = '\0';
		if (setjmp (harness_fatal))
		{
			accepted = 0;
		}
		else
		{
			campaign_object_structures (collect, NULL);
			accepted = 1;
		}
		check (!accepted && refused_with ("not a heap this run allocated"), "and a heap of no slots is not a heap");
		/* a slot index AT the capacity is outside the heap.  The heap is allocated EXACTLY that big (no spare slot behind it), the address one past it is only formed, and the entity access functions COUNT any
		   read of it: the registry must refuse before the type read, before the host is asked and before anything is applied */
		world_reset_with (10);
		accepted = try_identify (10);
		check (!accepted && refused_with ("outside the entity heap of 10 slots") && type_reads_outside == 0 && entity_touches_outside == 0 && host_calls == 0,
			"an entity whose slot index is the capacity is refused before its type is read, before the host is asked");
		check (try_host_id (10) == 0 && refused_with ("outside the entity heap of 10 slots") && type_reads_outside == 0 && entity_touches_outside == 0,
			"and its cached identity is not looked up either: the slot is refused before the type read");
		/* a rostered member whose recorded slot is the capacity: the report is refused before the member is read or anything applied */
		roster_commit ("member-edge", 1, 10);
		accepted = try_report ("member-edge", 1, 0, 0);
		check (!accepted && refused_with ("outside the entity heap of 10 slots") && type_reads_outside == 0 && entity_touches_outside == 0 && applied_count == 0,
			"a report whose rostered target is at the capacity is refused before the target is read, and applies nothing");
	}

	/* ================ identities that exist when the heap is no longer the one the registry was built for (no live resizing is supported: refuse, never re-read) ================ */
	{
		int host_before;

		world_reset_with (37);
		place (36, ENTITY_TYPE_SITE, 1, 90, 90);
		place (3, ENTITY_TYPE_PYLON, 1, 60, 60);
		try_identify (36);
		snprintf (first, sizeof (first), "%s", identified);
		try_identify (3);
		snprintf (second, sizeof (second), "%s", identified);
		host_before = host_calls;
		applied_count = 0;
		number_of_entities = 20;
		type_reads_outside = entity_touches_outside = 0;
		/* the cached identity of a slot outside the new range, and of one still inside it: neither is returned */
		check (try_host_id (36) == 0 && refused_with ("changed from 37 to 20 slots") && type_reads_outside == 0 && entity_touches_outside == 0, "the cached identity of a slot outside the new capacity is refused, with no type read");
		check (try_host_id (3) == 0 && refused_with ("changed from 37 to 20 slots") && type_reads_outside == 0 && entity_touches_outside == 0, "and so is one of a slot still inside it");
		accepted = try_identify (36);
		check (!accepted && refused_with ("changed from 37 to 20 slots") && type_reads_outside == 0 && entity_touches_outside == 0 && host_calls == host_before, "identify does not return a cached identity of the old heap, and does not ask the host");
		accepted = try_identify (3);
		check (!accepted && refused_with ("changed from 37 to 20 slots") && host_calls == host_before, "nor for a slot still inside the new capacity");
		place (5, ENTITY_TYPE_SITE, 1, 90, 90);
		accepted = try_identify (5);
		check (!accepted && refused_with ("changed from 37 to 20 slots") && host_calls == host_before, "nor does a fresh structure get an identity asked for");
		/* a report by the stale identities, the old slot outside the range and the old slot inside it, and by the roster's: refused before any record is followed, any entity read or anything applied */
		accepted = try_report (first, 1, 0, 0);
		check (!accepted && refused_with ("changed from 37 to 20 slots") && type_reads_outside == 0 && entity_touches_outside == 0 && applied_count == 0, "a report by the identity of a slot outside the new capacity is refused before it is followed");
		accepted = try_report (second, 1, 0, 0);
		check (!accepted && refused_with ("changed from 37 to 20 slots") && type_reads_outside == 0 && entity_touches_outside == 0 && applied_count == 0 && fake_entities[3].damage_level == 60, "and one by the identity of a slot still inside it, which stays as it was");
		accepted = try_report ("mobile-1", 1, 0, 0);
		check (!accepted && refused_with ("changed from 37 to 20 slots") && applied_count == 0, "and a report by a rostered member's identity");
		/* the heap is the registry's again: the same identities work, nothing was lost by the refusals */
		number_of_entities = 37;
		check (try_host_id (36) == 1 && strcmp (identified, first) == 0, "back at the original capacity the cached identity is returned");
		accepted = try_report (second, 1, 40, 0);
		check (accepted && fake_entities[3].damage_level == 40 && applied_count == 1, "and a report by the identity applies as it always did");
	}

	printf ("%d checks, %d failures\n", checks, failures);

	return failures ? 1 : 0;
}
