/* Bounded checks of the controlled-case driver's own guards (EEDCS-46), against the REAL csrc/eech_cmb_case.c compiled over a stand-in engine (case-stub/project.h) whose entity
   list the check models.  What is asserted is what the driver DECIDES: it refuses before it launches when it could not tell new weapons from old, it retires exactly the weapons the
   launch created and never an old one, it validates a blast's resulting position before it launches or mutates anything, and it sets a vehicle's health only where the original
   critical predicate accepts it (a person is never critical, a non-explosive vehicle has no fire roll, a level that is not critical is put back).
   NOT asserted, and not claimed: anything about the campaign's own physics, the host's birth, or a native run.  Run by run-case-checks.sh. */
#include "project.h"

#include <stdlib.h>

#define N 2048
#define MAXX 10000.0f

static entity pool[N];
static int alive_[N], level_[N], initial_[N], quality_[N], kill_[N], capability_[N], launcher_of[N], target_of[N], destroyed_[N];
static vec3d pos_[N];
static char data_marker[N];
static const char *host_id_[N];

entity *entities;
int number_of_entities;

/* counters the checks read */
static int launch_calls, query_calls, sets, destroys, notifies, effects, last_sender, last_receiver, launch_creates, launch_fail, query_mode;
static double last_modifier;
static int launch_launcher;

static int idx (entity *en) { return (int) (en - entities); }

entity *get_local_entity_list (void) { return &pool[1]; }

entity *get_local_entity_succ (entity *en)
{
	return (idx (en) + 1 < number_of_entities) ? en + 1 : NULL;
}

int get_local_entity_int_value (entity *en, int_types type)
{
	int i = idx (en);

	switch (type)
	{
		case INT_TYPE_ALIVE: return alive_[i];
		case INT_TYPE_DAMAGE_LEVEL: return level_[i];
		case INT_TYPE_INITIAL_DAMAGE_LEVEL: return initial_[i];
		case INT_TYPE_EXPLOSIVE_QUALITY: return quality_[i];
		case INT_TYPE_WEAPON_KILL_CODE: return kill_[i];
		case INT_TYPE_DAMAGE_CAPABILITY: return capability_[i];
		default: return 0;
	}
}

void set_client_server_entity_int_value (entity *en, int_types type, int value)
{
	sets++;

	if (type == INT_TYPE_DAMAGE_LEVEL)
	{
		level_[idx (en)] = value;
	}
}

void set_local_entity_int_value (entity *en, int_types type, int value)
{
	sets++;

	if (type == INT_TYPE_WEAPON_KILL_CODE)
	{
		kill_[idx (en)] = value;
	}
}

const char *campaign_roster_host_id (entity *en) { return host_id_[idx (en)]; }

const char *get_entity_type_name (entity_types type)
{
	static const char *names[NUM_ENTITY_TYPES] = { "ENTITY_TYPE_UNKNOWN", "ENTITY_TYPE_ANTI_AIRCRAFT", "ENTITY_TYPE_CITY_BUILDING", "ENTITY_TYPE_FIXED_WING", "ENTITY_TYPE_HELICOPTER",
		"ENTITY_TYPE_OBJECT", "ENTITY_TYPE_PERSON", "ENTITY_TYPE_PYLON", "ENTITY_TYPE_ROUTED_VEHICLE", "ENTITY_TYPE_SCENIC", "ENTITY_TYPE_SEGMENT", "ENTITY_TYPE_SHIP_VEHICLE", "ENTITY_TYPE_SITE",
		"ENTITY_TYPE_SITE_UPDATABLE", "ENTITY_TYPE_WEAPON" };

	return names[type];
}

static int is_child (int i, int launcher)
{
	return pool[i].type == ENTITY_TYPE_WEAPON && launcher_of[i] == launcher && !destroyed_[i];
}

entity *stub_first_child (entity *parent, list_types list)
{
	int i;

	for (i = 1; i < number_of_entities; i++)
	{
		if (is_child (i, idx (parent)))
		{
			return &pool[i];
		}
	}

	return NULL;
}

entity *stub_child_succ (entity *child, list_types list)
{
	int i, launcher = launcher_of[idx (child)];

	for (i = idx (child) + 1; i < number_of_entities; i++)
	{
		if (is_child (i, launcher))
		{
			return &pool[i];
		}
	}

	return NULL;
}

entity *get_local_entity_parent (entity *en, list_types list)
{
	if (list == LIST_TYPE_LAUNCHED_WEAPON)
	{
		return launcher_of[idx (en)] ? &pool[launcher_of[idx (en)]] : NULL;
	}

	return target_of[idx (en)] ? &pool[target_of[idx (en)]] : NULL;
}

void stub_vec3d_get (entity *en, vec3d_types type, vec3d *v) { *v = pos_[idx (en)]; }

void stub_vec3d_set (entity *en, vec3d_types type, vec3d *v)
{
	/* the real setter stops the engine for an off-map position; the stand-in does the same, so a driver that lets one through is caught */
	if (!point_inside_map_area (v))
	{
		printf ("ENGINE STOP: position off map in the setter\n");
		abort ();
	}

	pos_[idx (en)] = *v;
}

void stub_destroy (entity *en)
{
	destroys++;
	destroyed_[idx (en)] = 1;
	alive_[idx (en)] = 0;
}

void set_client_server_entity_parent (entity *en, list_types list, entity *parent) { target_of[idx (en)] = parent ? idx (parent) : 0; }

void notify_local_entity (int message, entity *receiver, entity *sender, ...)
{
	va_list arguments;

	va_start (arguments, sender);
	last_modifier = va_arg (arguments, double);
	va_end (arguments);

	notifies++;
	last_receiver = idx (receiver);
	last_sender = sender ? idx (sender) : 0;
}

void create_client_server_weapon_killed_explosion_effect (entity *en) { effects++; }

/* the original predicate (vehicle.c): FALSE for a person, else strictly below 15 percent in integer arithmetic */
int vehicle_critically_damaged (entity *en)
{
	int i = idx (en);

	if (en->type == ENTITY_TYPE_PERSON)
	{
		return 0;
	}

	return (100 * level_[i] / initial_[i]) < 15;
}

/* the Task45 read-only query, as the driver sees it.  The answers are the REAL helper's own shapes (csrc/eech_wpn_fixture.c eech_wpn_case_query_run; the literals are pinned against that
   file by run-case-checks.sh): an unknown host id and an unsupported launcher kind are -1 with a wpn-case-error reason; otherwise a state line whose `alive` is the launcher's and whose
   `warhead` is -1 for a weapon sub type that is not valid.  query_mode: 0 normal, 1 unsupported launcher kind, 2 unknown host id. */
int eech_wpn_case_query_run (const char *host_id, int weapon, char *text, int capacity)
{
	int i;

	query_calls++;

	if (query_mode == 2)
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=no_such_host_id host_id=%s\n", host_id);

		return -1;
	}

	if (query_mode == 1)
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=subject_kind_unsupported kind=ENTITY_TYPE_SCENIC\n");

		return -1;
	}

	for (i = 1; i < number_of_entities; i++)
	{
		if (host_id_[i] && strcmp (host_id_[i], host_id) == 0)
		{
			int valid = weapon > 0 && weapon < 200;

			snprintf (text, (size_t) capacity, "wpn-case-state host_id=%s type=helicopter alive=%d position=0,0,0 has_target=0 target_type=none weapon=%d weapon_count=%d warhead=%d guidance=0 aiming=0 decoy=0\n",
				host_id, alive_[i], weapon, valid ? 6 : -1, valid ? 4 : -1);

			return 0;
		}
	}

	snprintf (text, (size_t) capacity, "wpn-case-error reason=no_such_host_id host_id=%s\n", host_id);

	return -1;
}

/* the Task45 case launch, as the driver sees it: it creates `launch_creates` new weapon children of the launcher; `launch_fail` is the helper rejecting a launch whose launcher and
   weapon were valid (its real reason `target_link_not_set` stands for any such rejection) */
int eech_wpn_case_launch_run (const char *launcher_id, const char *target_id, int weapon, int salvo_count, char *text, int capacity)
{
	int i, made = 0;

	launch_calls++;

	if (launch_fail)
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=target_link_not_set\n");

		return -1;
	}

	for (i = 1; i < number_of_entities && made < launch_creates; i++)
	{
		if (pool[i].type == ENTITY_TYPE_UNKNOWN)
		{
			pool[i].type = ENTITY_TYPE_WEAPON;
			pool[i].data = &data_marker[i];
			alive_[i] = 1;
			launcher_of[i] = launch_launcher;
			destroyed_[i] = 0;
			capability_[i] = 100;
			pos_[i].x = 1.0f;
			pos_[i].y = 1.0f;
			pos_[i].z = 1.0f;
			made++;
		}
	}

	snprintf (text, (size_t) capacity, "wpn-case-launch ok\n");

	return 0;
}

extern int eech_cmb_case_census_run (char *text, int capacity);
extern int eech_cmb_case_health_run (const char *kind, int ordinal, int percent, char *text, int capacity);
extern int eech_cmb_case_collide_run (const char *kind, int ordinal, const char *launcher_id, int weapon, double modifier, int weaponless, char *text, int capacity);
extern int eech_cmb_case_blast_run (const char *kind, int ordinal, double offset, int kill_code, const char *launcher_id, int weapon, char *text, int capacity);

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

static void reset_world (void)
{
	memset (pool, 0, sizeof (pool));
	memset (alive_, 0, sizeof (alive_));
	memset (level_, 0, sizeof (level_));
	memset (initial_, 0, sizeof (initial_));
	memset (quality_, 0, sizeof (quality_));
	memset (kill_, 0, sizeof (kill_));
	memset (capability_, 0, sizeof (capability_));
	memset (launcher_of, 0, sizeof (launcher_of));
	memset (target_of, 0, sizeof (target_of));
	memset (destroyed_, 0, sizeof (destroyed_));
	memset (pos_, 0, sizeof (pos_));
	memset (host_id_, 0, sizeof (host_id_));
	entities = pool;
	number_of_entities = N;
	launch_calls = query_calls = sets = destroys = notifies = effects = launch_creates = launch_fail = query_mode = 0;
	last_sender = last_receiver = 0;
	launch_launcher = 0;
}

static int add (int slot, entity_types type, int level, int initial, int quality)
{
	pool[slot].type = type;
	pool[slot].data = &data_marker[slot];
	alive_[slot] = 1;
	level_[slot] = level;
	initial_[slot] = initial;
	quality_[slot] = quality;
	pos_[slot].x = 5000.0f;
	pos_[slot].y = 100.0f;
	pos_[slot].z = 5000.0f;

	return slot;
}

/* a launcher (slot 1, host id "L1") that already has `old` weapon children */
static void launcher_with (int old)
{
	int i;

	add (1, ENTITY_TYPE_HELICOPTER, 10, 10, 0);
	host_id_[1] = "L1";
	launch_launcher = 1;

	for (i = 0; i < old; i++)
	{
		add (100 + i, ENTITY_TYPE_WEAPON, 1, 1, 0);
		launcher_of[100 + i] = 1;
	}
}

static int old_destroyed (int old)
{
	int i, n = 0;

	for (i = 0; i < old; i++)
	{
		n += destroyed_[100 + i];
	}

	return n;
}

int main (void)
{
	char text[1024];
	int rc;

	setvbuf (stdout, NULL, _IONBF, 0);

	/* ---- the health case: the original critical predicate decides, and a person is never critical */
	reset_world ();
	add (2, ENTITY_TYPE_PERSON, 10, 10, 0);
	rc = eech_cmb_case_health_run ("PERSON", 0, 14, text, sizeof (text));
	check(rc == 0 && strstr (text, "eligible=0") && strstr (text, "reason=person_never_critical_in_source") && strstr (text, "predicate=0"), "a person is reported never critical (the original predicate asked once, FALSE) and not eligible");
	check(sets == 0 && level_[2] == 10, "and its health was not touched, not even to 1 percent");

	reset_world ();
	add (2, ENTITY_TYPE_ANTI_AIRCRAFT, 100, 100, 0);
	rc = eech_cmb_case_health_run ("ANTI_AIRCRAFT", 0, 14, text, sizeof (text));
	check(rc == 0 && strstr (text, "eligible=0") && strstr (text, "reason=not_explosive_or_flammable"), "an anti-aircraft vehicle that is neither explosive nor flammable has no fire roll: not eligible");
	check(sets == 0 && level_[2] == 100, "and its health was not touched");

	reset_world ();
	add (2, ENTITY_TYPE_ANTI_AIRCRAFT, 100, 100, EXPLOSIVE_QUALITY_EXPLOSIVE);
	rc = eech_cmb_case_health_run ("ANTI_AIRCRAFT", 0, 20, text, sizeof (text));
	check(rc == 0 && strstr (text, "eligible=0") && strstr (text, "reason=level_not_critical") && strstr (text, "restored=100"), "20 percent is NOT critical (strictly below 15 is): not eligible, and the health is put back");
	check(level_[2] == 100, "the health is exactly what it was");

	reset_world ();
	add (2, ENTITY_TYPE_ANTI_AIRCRAFT, 100, 100, EXPLOSIVE_QUALITY_FLAMMABLE);
	rc = eech_cmb_case_health_run ("ANTI_AIRCRAFT", 0, 14, text, sizeof (text));
	check(rc == 0 && strstr (text, "eligible=1") && strstr (text, "set=14") && strstr (text, "critical=1") && level_[2] == 14, "14 percent of a flammable vehicle is critical by the original predicate: eligible, health set to 14");

	reset_world ();
	add (2, ENTITY_TYPE_ANTI_AIRCRAFT, 5, 5, EXPLOSIVE_QUALITY_EXPLOSIVE);
	rc = eech_cmb_case_health_run ("ANTI_AIRCRAFT", 0, 14, text, sizeof (text));
	check(rc == 0 && strstr (text, "eligible=0") && strstr (text, "reason=level_not_critical") && level_[2] == 5, "a vehicle too small to have a positive health below 15 percent (initial 5: one point is 20 percent) is not eligible and is restored");

	reset_world ();
	add (2, ENTITY_TYPE_ANTI_AIRCRAFT, 100, 100, EXPLOSIVE_QUALITY_EXPLOSIVE);
	add (3, ENTITY_TYPE_ANTI_AIRCRAFT, 100, 100, EXPLOSIVE_QUALITY_EXPLOSIVE);
	rc = eech_cmb_case_health_run ("ANTI_AIRCRAFT", 1, 14, text, sizeof (text));
	check(rc == 0 && level_[3] == 14 && level_[2] == 100, "ordinal 1 is the second living vehicle of the kind and only that one");
	rc = eech_cmb_case_health_run ("ANTI_AIRCRAFT", 5, 14, text, sizeof (text));
	check(rc == 0 && strstr (text, "found=0") && strstr (text, "eligible=0"), "an ordinal past the living entities is not found and not eligible");
	rc = eech_cmb_case_health_run ("HELICOPTER", 0, 14, text, sizeof (text));
	check(rc == -1 && strstr (text, "kind_has_no_vehicle_critical_predicate"), "a kind with no vehicle critical predicate is refused");

	/* ---- launch accounting: refuse before the launch when new could not be told from old */
	reset_world ();
	launcher_with (257);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 1;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.0, 0, text, sizeof (text));
	check(rc == -1 && strstr (text, "launcher_has_too_many_children_to_tell_new_from_old") && strstr (text, "nothing was launched"), "257 existing children: refused BEFORE the launch (the snapshot cannot be complete)");
	check(launch_calls == 0 && destroys == 0 && notifies == 0, "nothing was launched, retired or delivered");

	reset_world ();
	launcher_with (256);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 1;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.0, 0, text, sizeof (text));
	check(rc == 0 && strstr (text, "launched=1") && launch_calls == 1 && notifies == 1, "exactly 256 existing children is the boundary the snapshot holds: the case runs");
	check(old_destroyed (256) == 0 && destroys == 1, "and only the one weapon the launch created was retired, none of the 256 old ones");

	/* ---- created overflow: every provably new weapon retired, no old one, an explicit error */
	reset_world ();
	launcher_with (3);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 300;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.0, 0, text, sizeof (text));
	check(rc == -1 && strstr (text, "launch_created_more_weapons_than_the_table_holds") && strstr (text, "created=300") && strstr (text, "retired_new=300") && strstr (text, "unrelated_retired=0"), "a launch that creates 300 weapons is an explicit error with the counts");
	check(old_destroyed (3) == 0 && destroys == 300 && notifies == 0, "all 300 new weapons were retired, none of the 3 old ones, and nothing was delivered");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 256;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.5, 0, text, sizeof (text));
	check(rc == 0 && strstr (text, "launched=256") && notifies == 1 && destroys == 256 && last_modifier == 0.5, "a launch that creates exactly 256 weapons is accounted in full: delivered once, all 256 retired");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_fail = 1;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.0, 0, text, sizeof (text));
	check(rc == -1 && strstr (text, "launch_rejected_after_validation") && strstr (text, "target_link_not_set"), "a launch the helper rejects although the launcher and the weapon were valid is an ERROR carrying the helper's own reason (not data, not discarded)");
	check(notifies == 0 && destroys == 0, "and nothing is delivered or retired");

	/* a launch that returns normally but creates no weapon (no ammunition, the release gate, a launcher that is no longer alive) is the same: data, with the launch text and its counts */
	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 0;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.5, 0, text, sizeof (text));
	check(rc == 0 && strstr (text, "launched=0") && strstr (text, "refused=1") && strstr (text, "launch=wpn-case-launch|ok") && notifies == 0 && destroys == 0, "a launch that creates no weapon is data too, with the launch text, and delivers and retires nothing");

	/* a refused launch for a blast: the placement was valid, the case is not done, and the reason is the result; nothing is exploded or destroyed */
	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_CITY_BUILDING, 10, 10, 0);
	launch_fail = 1;
	rc = eech_cmb_case_blast_run ("CITY_BUILDING", 0, 5.0, WEAPON_KILL_CODE_HIT_LAND, "L1", 5, text, sizeof (text));
	check(rc == -1 && strstr (text, "launch_rejected_after_validation") && strstr (text, "target_link_not_set") && effects == 0 && destroys == 0 && sets == 0, "a launch the helper rejects after validation is an error for a blast too; nothing exploded, destroyed or set");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_CITY_BUILDING, 10, 10, 0);
	launch_creates = 0;
	rc = eech_cmb_case_blast_run ("CITY_BUILDING", 0, 5.0, WEAPON_KILL_CODE_HIT_LAND, "L1", 5, text, sizeof (text));
	check(rc == 0 && strstr (text, "launched=0") && strstr (text, "refused=1") && effects == 0 && destroys == 0, "a launch that creates no weapon for a blast is data too");

	/* ---- validation BEFORE the launch: an invalid argument is an error and launches and changes nothing; a dead launcher is a valid campaign outcome and is data */
	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 1;
	query_mode = 1;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.0, 0, text, sizeof (text));
	check(rc == -1 && strstr (text, "launcher_cannot_be_used_for_a_case_launch") && strstr (text, "subject_kind_unsupported"), "an unsupported launcher kind is an ERROR with the helper's reason, not refused data");
	check(launch_calls == 0 && notifies == 0 && destroys == 0 && sets == 0 && effects == 0, "and nothing was launched, delivered, retired or set");
	rc = eech_cmb_case_blast_run ("ROUTED_VEHICLE", 0, 5.0, WEAPON_KILL_CODE_HIT_LAND, "L1", 5, text, sizeof (text));
	check(rc == -1 && strstr (text, "launcher_cannot_be_used_for_a_case_launch") && launch_calls == 0 && effects == 0 && destroys == 0, "the same for a blast: an error, no launch, no explosion");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 1;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 0, 0.0, 0, text, sizeof (text));
	check(rc == -1 && strstr (text, "weapon_sub_type_not_valid") && strstr (text, "weapon=0") && launch_calls == 0 && notifies == 0 && destroys == 0 && sets == 0, "an invalid weapon sub type (0) is an ERROR with no launch and no mutation");
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 999, 0.0, 0, text, sizeof (text));
	check(rc == -1 && strstr (text, "weapon_sub_type_not_valid") && launch_calls == 0, "and so is one past the table (999)");
	rc = eech_cmb_case_blast_run ("ROUTED_VEHICLE", 0, 5.0, WEAPON_KILL_CODE_HIT_LAND, "L1", 999, text, sizeof (text));
	check(rc == -1 && strstr (text, "weapon_sub_type_not_valid") && launch_calls == 0 && effects == 0 && destroys == 0, "and for a blast: an error, no launch, no explosion");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 1;
	query_mode = 2;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.0, 0, text, sizeof (text));
	check(rc == -1 && strstr (text, "launcher_cannot_be_used_for_a_case_launch") && strstr (text, "no_such_host_id") && launch_calls == 0, "a host id the helper does not know is an ERROR with its reason (the case never launches)");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 1;
	alive_[1] = 0;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.0, 0, text, sizeof (text));
	check(rc == 0 && strstr (text, "launched=0") && strstr (text, "refused=1") && strstr (text, "launcher_not_alive") && launch_calls == 0 && notifies == 0 && destroys == 0, "a launcher that is not alive is a valid campaign refusal: DATA (refused=1, launcher_not_alive), and the launch helper is not called");
	rc = eech_cmb_case_blast_run ("ROUTED_VEHICLE", 0, 5.0, WEAPON_KILL_CODE_HIT_LAND, "L1", 5, text, sizeof (text));
	check(rc == 0 && strstr (text, "placed=1") && strstr (text, "refused=1") && strstr (text, "launcher_not_alive") && launch_calls == 0 && effects == 0 && destroys == 0, "and for a blast: data, placed, nothing exploded or retired");

	/* a launch that works says so, with the launch text */
	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	launch_creates = 1;
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "L1", 5, 0.0, 0, text, sizeof (text));
	check(rc == 0 && strstr (text, "launched=1") && strstr (text, "refused=0") && strstr (text, "launch=wpn-case-launch|ok"), "a launch that works reports refused=0 and the launch text");

	reset_world ();
	add (2, ENTITY_TYPE_ROUTED_VEHICLE, 10, 10, 0);
	rc = eech_cmb_case_collide_run ("ROUTED_VEHICLE", 0, "", 0, 0.0, 1, text, sizeof (text));
	check(rc == 0 && launch_calls == 0 && notifies == 1 && last_sender == 0, "a weaponless collision launches nothing and delivers with no sender");

	/* ---- blast placement: validated before anything is launched or mutated */
	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_CITY_BUILDING, 10, 10, 0);
	pos_[2].x = MAXX - 2.0f;
	launch_creates = 1;
	rc = eech_cmb_case_blast_run ("CITY_BUILDING", 0, 5.0, WEAPON_KILL_CODE_HIT_LAND, "L1", 5, text, sizeof (text));
	check(rc == 0 && strstr (text, "placed=0") && strstr (text, "reason=placement_off_map"), "the default 5 m offset from an entity 2 m inside the east edge is off the map: reported not done");
	check(launch_calls == 0 && sets == 0 && destroys == 0 && effects == 0, "nothing was launched, set, destroyed or exploded (and the setter's engine stop was not reached)");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_CITY_BUILDING, 10, 10, 0);
	pos_[2].x = 2.0f;
	launch_creates = 1;
	rc = eech_cmb_case_blast_run ("CITY_BUILDING", 0, -5.0, WEAPON_KILL_CODE_HIT_LAND, "L1", 5, text, sizeof (text));
	check(rc == 0 && strstr (text, "placed=0") && launch_calls == 0, "a negative offset past the west edge is off the map too");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_CITY_BUILDING, 10, 10, 0);
	launch_creates = 1;
	rc = eech_cmb_case_blast_run ("CITY_BUILDING", 0, (double) NAN, WEAPON_KILL_CODE_HIT_LAND, "L1", 5, text, sizeof (text));
	check(rc == -1 && launch_calls == 0, "a non-finite offset is refused before anything happens");

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_CITY_BUILDING, 10, 10, 0);
	launch_creates = 1;
	rc = eech_cmb_case_blast_run ("CITY_BUILDING", 0, 5.0, WEAPON_KILL_CODE_HIT_LAND, "L1", 5, text, sizeof (text));
	check(rc == 0 && strstr (text, "placed=1") && launch_calls == 1 && effects == 1 && destroys == 1, "an on-map placement launches once, runs the killed-weapon explosion once and destroys the weapon once");

	{
		int w = -1, i;

		for (i = 3; i < N; i++)
		{
			if (pool[i].type == ENTITY_TYPE_WEAPON && launcher_of[i] == 1)
			{
				w = i;
			}
		}

		check(w > 0 && pos_[w].x == 5005.0f && pos_[w].z == 5000.0f && kill_[w] == WEAPON_KILL_CODE_HIT_LAND, "the weapon was placed 5 m east of the receiver with the chosen kill code");
	}

	reset_world ();
	launcher_with (0);
	add (2, ENTITY_TYPE_CITY_BUILDING, 10, 10, 0);
	launch_creates = 1;
	rc = eech_cmb_case_blast_run ("CITY_BUILDING", 0, 5.0, WEAPON_KILL_CODE_HIT_TARGET, "L1", 5, text, sizeof (text));
	{
		int linked = 0, i;

		for (i = 3; i < N; i++)
		{
			linked += (pool[i].type == ENTITY_TYPE_WEAPON && target_of[i] == 2);
		}

		check(rc == 0 && linked == 1, "a hit on target links the exploding weapon to the receiver before the explosion");
	}

	printf ("%s: %d checks, %d failures\n", failures ? "CASE_CHECKS=FAILED" : "CASE_CHECKS=ok", checks, failures);

	return failures ? 1 : 0;
}
