/*
 * EEDCS-11, TEST ONLY: observes waypoint arrival on SUPPLY tasks and its
 * campaign consequences, for characterisation. It is never part of a product
 * build: tools/build-observed-windows.sh copies this file into csrc/ and adds
 * the hooks in tools/arrival-observe/patches.json, in a temporary worktree.
 *
 * Observational only. It reads entity state through EECH's own getters, never
 * sets entity state, never calls a message handler, a random number or a
 * campaign/world function with effects, and returns every verdict it is given
 * unchanged. Its only effect is a JSON-lines file:
 *
 *   EECH_ARRIVAL_TRACE=<existing absolute directory>
 *     -> <directory>/arrivals-<pid>.jsonl   (nothing is written when unset)
 *
 * Identities are [entity index, generation]. EECH reuses entity indices; the
 * generation is the number of times the index had been freed when the entity
 * was seen (hook A2 counts set_free_entity), so [index, generation] names one
 * entity for the whole run. Times are the session's elapsed seconds (the "t"
 * of lua/observations.lua) and the host's simulated milliseconds.
 *
 * Events (field "e"); see eech-campaign/docs/m5-arrival-characterisation.md:
 *   header      the run: install root, pid
 *   task        a SUPPLY task created (create_supply_task): cargo, supplier,
 *               requester, its waypoints in order
 *   verdict     a guide's arrival check returned TRUE (gd_updt.c) for a
 *               waypoint of a SUPPLY task, or for one of the four SUPPLY
 *               waypoint types on any task: who is task leader, the guide's
 *               criteria, every group member's state, the waypoint's
 *               dependent and the leader's load; opens a span
 *   handler     one of the four SUPPLY waypoint responses entered (mb_msgs.c)
 *   completion  a SUPPLY task was sent TASK_COMPLETED (ts_msgs.c), and why
 *   completed   a SUPPLY task's completion state was set (ts_int.c)
 *   effect      the span closed: the same entities after the consequences
 *   freed       an entity named in an earlier event was freed
 *   counters    at process exit: every arrival check and verdict, by kind
 */

#include "project.h"

#include <stdio.h>
#include <stdlib.h>
#ifdef _WIN32
#include <process.h>
#define eech_observe_pid() ((int) _getpid ())
#else
#include <unistd.h>
#define eech_observe_pid() ((int) getpid ())
#endif

#include "eech_headless.h"

#define MAX_INDEX (1 << 20) /* NUM_ENTITY_INDEX_PACK_BITS */

static int state; /* 0 not yet initialised, 1 enabled, -1 disabled */
static FILE *out;
static unsigned generation[MAX_INDEX];
static unsigned tracked[MAX_INDEX]; /* generation + 1 of an entity named in an event */
static int check_wp[MAX_INDEX], check_wp_gen[MAX_INDEX], checks[MAX_INDEX];

static struct
{
	unsigned long checks_supply_four, checks_supply_other, checks_other_four, checks_other;
	unsigned long leaderless_supply_four, leaderless_supply_other;
	unsigned long verdicts_supply_four, verdicts_supply_other, verdicts_other_four, verdicts_other;
	unsigned long handlers_in_span, handlers_outside_span, events;
} count;

static struct
{
	int open, id;
	entity *guide, *wp, *task, *group, *leader, *dependent;
	unsigned guide_gen, wp_gen, task_gen, group_gen, leader_gen, dependent_gen;
} span;
static int spans;

static void finish (void)
{
	if (state != 1 || !out)
	{
		return;
	}
	fprintf (out, "{\"e\":\"counters\",\"checks\":{\"supply_four\":%lu,\"supply_other\":%lu,\"other_four\":%lu,\"other\":%lu},"
		"\"leaderless_checks\":{\"supply_four\":%lu,\"supply_other\":%lu},"
		"\"verdicts\":{\"supply_four\":%lu,\"supply_other\":%lu,\"other_four\":%lu,\"other\":%lu},"
		"\"handlers\":{\"in_span\":%lu,\"outside_span\":%lu},\"spans\":%d,\"events\":%lu}\n",
		count.checks_supply_four, count.checks_supply_other, count.checks_other_four, count.checks_other,
		count.leaderless_supply_four, count.leaderless_supply_other,
		count.verdicts_supply_four, count.verdicts_supply_other, count.verdicts_other_four, count.verdicts_other,
		count.handlers_in_span, count.handlers_outside_span, spans, count.events);
	fclose (out);
	out = NULL;
}

static int enabled (void)
{
	if (state == 0)
	{
		const char *dir = getenv ("EECH_ARRIVAL_TRACE");
		state = -1;
		if (dir && *dir)
		{
			char path[1024];
			snprintf (path, sizeof (path), "%s/arrivals-%d.jsonl", dir, eech_observe_pid ());
			out = fopen (path, "w");
			if (out)
			{
				state = 1;
				atexit (finish);
			}
			else
			{
				eech_log (0, "EECH_ARRIVAL_TRACE: cannot write %s", path);
			}
		}
	}
	return state == 1;
}

static int index_of (entity *en)
{
	int i = get_local_entity_index (en);
	return (i >= 0 && i < MAX_INDEX) ? i : -1;
}

static unsigned gen_of (entity *en)
{
	int i = index_of (en);
	return i >= 0 ? generation[i] : 0;
}

static void track (entity *en)
{
	int i = index_of (en);
	if (i >= 0)
	{
		tracked[i] = generation[i] + 1;
	}
}

/* still the same entity: not freed since it was seen (its index not reused) */
static int same (entity *en, unsigned gen)
{
	return en && get_local_entity_type (en) != ENTITY_TYPE_UNKNOWN && gen_of (en) == gen;
}

static double now (void)
{
	entity *session = get_session_entity ();
	return session ? get_local_entity_float_value (session, FLOAT_TYPE_ELAPSED_TIME) : -1.0;
}

static void text (const char *s)
{
	fputc ('"', out);
	for (; s && *s; s++)
	{
		unsigned char c = (unsigned char) *s;
		if (c == '"' || c == '\\')
		{
			fprintf (out, "\\%c", c);
		}
		else if (c < 0x20 || c >= 0x7f)
		{
			fprintf (out, "\\u%04x", c);
		}
		else
		{
			fputc (c, out);
		}
	}
	fputc ('"', out);
}

static void number (double v)
{
	if (v != v)
	{
		fputs ("null", out);
	}
	else
	{
		fprintf (out, "%.3f", v);
	}
}

static void begin (const char *event)
{
	count.events++;
	fprintf (out, "{\"e\":\"%s\",\"t\":", event);
	number (now ());
	fprintf (out, ",\"ms\":%u", eech_headless_time_ms ());
}

static void end_line (void)
{
	fputs ("}\n", out);
	fflush (out);
}

/* [index, generation] */
static void ref (const char *key, entity *en)
{
	fprintf (out, ",\"%s\":", key);
	if (!en)
	{
		fputs ("null", out);
		return;
	}
	track (en);
	fprintf (out, "[%d,%u]", get_local_entity_index (en), gen_of (en));
}

static int is_four (int waypoint_type)
{
	return waypoint_type == ENTITY_SUB_TYPE_WAYPOINT_PICK_UP || waypoint_type == ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF
		|| waypoint_type == ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF || waypoint_type == ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF;
}

static int is_supply_task (entity *task)
{
	return task && get_local_entity_type (task) == ENTITY_TYPE_TASK && get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_SUPPLY;
}

static void waypoint (const char *key, entity *wp)
{
	ref (key, wp);
	if (wp)
	{
		int type = get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE);
		fprintf (out, ",\"%s_type\":", key);
		text (entity_sub_type_waypoint_names[type]);
	}
}

/* a keysite or group's supply, and a keysite's crates */
static void holder (const char *key, entity *en)
{
	entity *cargo;
	int first = 1, type;
	fprintf (out, ",\"%s\":", key);
	if (!en)
	{
		fputs ("null", out);
		return;
	}
	track (en);
	type = get_local_entity_type (en);
	fprintf (out, "{\"id\":[%d,%u],\"type\":", get_local_entity_index (en), gen_of (en));
	text (entity_type_names[type]);
	if (type == ENTITY_TYPE_KEYSITE || type == ENTITY_TYPE_GROUP)
	{
		fputs (",\"name\":", out);
		text (get_local_entity_string (en, type == ENTITY_TYPE_KEYSITE ? STRING_TYPE_KEYSITE_NAME : STRING_TYPE_GROUP_CALLSIGN));
		fputs (",\"ammo\":", out);
		number (get_local_entity_float_value (en, FLOAT_TYPE_AMMO_SUPPLY_LEVEL));
		fputs (",\"fuel\":", out);
		number (get_local_entity_float_value (en, FLOAT_TYPE_FUEL_SUPPLY_LEVEL));
	}
	if (type == ENTITY_TYPE_KEYSITE)
	{
		fputs (",\"crates\":[", out);
		for (cargo = get_local_entity_first_child (en, LIST_TYPE_CARGO); cargo; cargo = get_local_entity_child_succ (cargo, LIST_TYPE_CARGO))
		{
			if (get_local_entity_type (cargo) == ENTITY_TYPE_CARGO)
			{
				track (cargo);
				fprintf (out, "%s[%d,%u,", first ? "" : ",", get_local_entity_index (cargo), gen_of (cargo));
				text (entity_sub_type_cargo_names[get_local_entity_int_value (cargo, INT_TYPE_ENTITY_SUB_TYPE)]);
				fputc (']', out);
				first = 0;
			}
		}
		fputc (']', out);
	}
	fputc ('}', out);
}

/* what a mobile carries: its MOVEMENT_DEPENDENT list, as the drop-off reads it */
static void aboard (const char *key, entity *mobile)
{
	entity *child;
	int first = 1;
	fprintf (out, ",\"%s\":", key);
	if (!mobile)
	{
		fputs ("null", out);
		return;
	}
	fputc ('[', out);
	for (child = get_local_entity_first_child (mobile, LIST_TYPE_MOVEMENT_DEPENDENT); child; child = get_local_entity_child_succ (child, LIST_TYPE_MOVEMENT_DEPENDENT))
	{
		int type = get_local_entity_type (child);
		track (child);
		fprintf (out, "%s{\"id\":[%d,%u],\"type\":", first ? "" : ",", get_local_entity_index (child), gen_of (child));
		text (entity_type_names[type]);
		fprintf (out, ",\"identify_cargo\":%d", get_local_entity_int_value (child, INT_TYPE_IDENTIFY_CARGO));
		if (type == ENTITY_TYPE_CARGO)
		{
			fputs (",\"cargo\":", out);
			text (entity_sub_type_cargo_names[get_local_entity_int_value (child, INT_TYPE_ENTITY_SUB_TYPE)]);
		}
		fputc ('}', out);
		first = 0;
	}
	fputc (']', out);
}

static void task_state (entity *task)
{
	if (task && get_local_entity_type (task) == ENTITY_TYPE_TASK)
	{
		fprintf (out, ",\"task_completed\":%d,\"task_state\":%d", get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED),
			get_local_entity_int_value (task, INT_TYPE_TASK_STATE));
	}
}

static void header (void)
{
	static int written;
	if (!written)
	{
		written = 1;
		fprintf (out, "{\"e\":\"header\",\"version\":1,\"pid\":%d,\"root\":", eech_observe_pid ());
		text (comanche_hokum_installation_path);
		fputs (",\"note\":\"EEDCS-11 test-only arrival observation; tools/arrival-observe\"}\n", out);
	}
}

/*
 * A2: set_free_entity. Counts the index's generation; reports an entity an
 * earlier event named. Reads only the entity's type, before EECH clears it.
 */
void eech_observe_entity_freed (entity *en)
{
	int i;
	if (!enabled ())
	{
		return;
	}
	i = index_of (en);
	if (i < 0)
	{
		return;
	}
	if (tracked[i] == generation[i] + 1)
	{
		header ();
		begin ("freed");
		fprintf (out, ",\"id\":[%d,%u],\"type\":", i, generation[i]);
		text (entity_type_names[get_local_entity_type (en)]);
		fprintf (out, ",\"span\":%d", span.open ? span.id : 0);
		end_line ();
		tracked[i] = 0;
	}
	generation[i]++;
}

/* A3: create_supply_task, after the task and its waypoints exist */
void eech_observe_supply_task_created (entity *task, entity *requester, entity *supplier, entity *cargo)
{
	entity *wp;
	int first = 1;
	if (!enabled () || !task)
	{
		return;
	}
	header ();
	begin ("task");
	ref ("task", task);
	fputs (",\"cargo_type\":", out);
	text (cargo ? entity_sub_type_cargo_names[get_local_entity_int_value (cargo, INT_TYPE_ENTITY_SUB_TYPE)] : "");
	ref ("cargo", cargo);
	holder ("supplier", supplier);
	holder ("requester", requester);
	fprintf (out, ",\"side\":%d,\"waypoints\":[", get_local_entity_int_value (task, INT_TYPE_SIDE));
	for (wp = get_local_entity_first_child (task, LIST_TYPE_WAYPOINT); wp; wp = get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT))
	{
		entity *dependent = get_local_entity_parent (wp, LIST_TYPE_TASK_DEPENDENT);
		track (wp);
		fprintf (out, "%s{\"id\":[%d,%u],\"type\":", first ? "" : ",", get_local_entity_index (wp), gen_of (wp));
		text (entity_sub_type_waypoint_names[get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE)]);
		if (dependent)
		{
			track (dependent);
			fprintf (out, ",\"dependent\":[%d,%u]", get_local_entity_index (dependent), gen_of (dependent));
		}
		fputc ('}', out);
		first = 0;
	}
	fputc (']', out);
	end_line ();
}

static void members (entity *guide, entity *group, entity *leader)
{
	entity *member;
	int first = 1;
	vec3d *leader_pos = leader ? get_local_entity_vec3d_ptr (leader, VEC3D_TYPE_POSITION) : NULL;
	fputs (",\"members\":[", out);
	for (member = group ? get_local_entity_first_child (group, LIST_TYPE_MEMBER) : NULL; member; member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER))
	{
		int type = get_local_entity_type (member);
		int aircraft = type == ENTITY_TYPE_HELICOPTER || type == ENTITY_TYPE_FIXED_WING;
		track (member);
		fprintf (out, "%s{\"id\":[%d,%u],\"type\":", first ? "" : ",", get_local_entity_index (member), gen_of (member));
		text (entity_type_names[type]);
		fprintf (out, ",\"number\":%d,\"follows_guide\":%d,\"is_task_leader\":%d", get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER),
			get_local_entity_parent (member, LIST_TYPE_FOLLOWER) == guide, member == leader);
		if (aircraft)
		{
			vec3d *pos = get_local_entity_vec3d_ptr (member, VEC3D_TYPE_POSITION);
			fprintf (out, ",\"alive\":%d,\"player\":%d,\"landed\":%d,\"task_leader_flag\":%d", get_local_entity_int_value (member, INT_TYPE_ALIVE),
				get_local_entity_int_value (member, INT_TYPE_PLAYER), get_local_entity_int_value (member, INT_TYPE_LANDED),
				get_local_entity_int_value (member, INT_TYPE_TASK_LEADER));
			fputs (",\"distance\":", out);
			number (get_local_entity_float_value (member, FLOAT_TYPE_DISTANCE));
			if (type == ENTITY_TYPE_FIXED_WING)
			{
				fputs (",\"velocity\":", out);
				number (get_local_entity_float_value (member, FLOAT_TYPE_VELOCITY));
			}
			else
			{
				fputs (",\"rotor_spin_up\":", out);
				number (get_local_entity_float_value (member, FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER));
			}
			fputs (",\"to_leader_2d\":", out);
			number (leader_pos ? sqrt (get_sqr_2d_range (pos, leader_pos)) : -1.0);
			fputs (",\"pos\":[", out);
			number (pos->x);
			fputc (',', out);
			number (pos->y);
			fputc (',', out);
			number (pos->z);
			fputc (']', out);
		}
		fputc ('}', out);
		first = 0;
	}
	fputc (']', out);
}

/*
 * A1: gd_updt.c, every server guide update: the verdict of
 * check_guide_reached_waypoint, returned unchanged. A TRUE verdict for a
 * SUPPLY task's waypoint, or for one of the four types, is reported in full
 * and opens a span that eech_observe_arrival_end closes after the
 * WAYPOINT_REACHED notification returns.
 */
int eech_observe_arrival_checked (entity *guide, int verdict)
{
	entity *wp, *task, *leader = NULL, *group, *dependent;
	int wp_type, supply, four, gi, wi;
	if (!enabled ())
	{
		return verdict;
	}
	wp = get_local_entity_parent (guide, LIST_TYPE_CURRENT_WAYPOINT);
	if (!wp)
	{
		count.checks_other++;
		if (verdict)
		{
			count.verdicts_other++;
		}
		return verdict;
	}
	wp_type = get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE);
	task = get_local_entity_parent (wp, LIST_TYPE_WAYPOINT);
	supply = is_supply_task (task);
	four = is_four (wp_type);

	/* checks since this waypoint became the guide's current one */
	gi = index_of (guide);
	wi = index_of (wp);
	if (gi >= 0)
	{
		if (check_wp[gi] != wi + 1 || check_wp_gen[gi] != (int) gen_of (wp))
		{
			check_wp[gi] = wi + 1;
			check_wp_gen[gi] = (int) gen_of (wp);
			checks[gi] = 0;
		}
		checks[gi]++;
	}
	if (supply || four)
	{
		leader = (entity *) get_local_entity_ptr_value (guide, PTR_TYPE_TASK_LEADER);
	}
	if (supply)
	{
		if (four)
		{
			count.checks_supply_four++;
			count.leaderless_supply_four += !leader;
		}
		else
		{
			count.checks_supply_other++;
			count.leaderless_supply_other += !leader;
		}
	}
	else if (four)
	{
		count.checks_other_four++;
	}
	else
	{
		count.checks_other++;
	}
	if (!verdict)
	{
		return verdict;
	}
	if (!(supply || four))
	{
		count.verdicts_other++;
		return verdict;
	}
	if (supply)
	{
		if (four)
		{
			count.verdicts_supply_four++;
		}
		else
		{
			count.verdicts_supply_other++;
		}
	}
	else
	{
		count.verdicts_other_four++;
	}

	group = get_local_entity_parent (guide, LIST_TYPE_GUIDE_STACK);
	dependent = get_local_entity_parent (wp, LIST_TYPE_TASK_DEPENDENT);

	header ();
	span.open = 1;
	span.id = ++spans;
	span.guide = guide;
	span.wp = wp;
	span.task = task;
	span.group = group;
	span.leader = leader;
	span.dependent = dependent;
	span.guide_gen = gen_of (guide);
	span.wp_gen = gen_of (wp);
	span.task_gen = task ? gen_of (task) : 0;
	span.group_gen = group ? gen_of (group) : 0;
	span.leader_gen = leader ? gen_of (leader) : 0;
	span.dependent_gen = dependent ? gen_of (dependent) : 0;

	begin ("verdict");
	fprintf (out, ",\"span\":%d,\"supply_task\":%d,\"four\":%d", span.id, supply, four);
	ref ("guide", guide);
	fputs (",\"guide_type\":", out);
	text (entity_sub_type_guide_names[get_local_entity_int_value (guide, INT_TYPE_ENTITY_SUB_TYPE)]);
	waypoint ("wp", wp);
	ref ("task", task);
	if (task && get_local_entity_type (task) == ENTITY_TYPE_TASK)
	{
		fputs (",\"task_type\":", out);
		text (entity_sub_type_task_names[get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE)]);
	}
	ref ("guide_task", get_local_entity_parent (guide, LIST_TYPE_GUIDE));
	task_state (task);
	ref ("group", group);
	if (group)
	{
		fputs (",\"callsign\":", out);
		text (get_local_entity_string (group, STRING_TYPE_GROUP_CALLSIGN));
		ref ("group_leader", (entity *) get_local_entity_ptr_value (group, PTR_TYPE_GROUP_LEADER));
	}
	ref ("task_leader", leader);
	fprintf (out, ",\"last_to_reach\":%d,\"radius_valid\":%d,\"radius\":", get_guide_criteria_valid (guide, GUIDE_CRITERIA_LAST_TO_REACH),
		get_guide_criteria_valid (guide, GUIDE_CRITERIA_RADIUS));
	number (get_guide_criteria_valid (guide, GUIDE_CRITERIA_RADIUS) ? get_guide_criteria_value (guide, GUIDE_CRITERIA_RADIUS) : -1.0);
	fprintf (out, ",\"check_action\":%d,\"valid_members\":%d,\"checks\":%d", get_local_entity_int_value (guide, INT_TYPE_CHECK_WAYPOINT_ACTION),
		get_local_entity_int_value (guide, INT_TYPE_VALID_GUIDE_MEMBERS), gi >= 0 ? checks[gi] : -1);
	members (guide, group, leader);
	holder ("dependent", dependent);
	aboard ("leader_aboard", leader);
	end_line ();
	return verdict;
}

/* A1: after the WAYPOINT_REACHED notification has returned */
void eech_observe_arrival_end (void)
{
	int guide_alive, task_alive, leader_alive;
	if (!enabled () || !span.open)
	{
		return;
	}
	guide_alive = same (span.guide, span.guide_gen);
	task_alive = span.task && same (span.task, span.task_gen);
	leader_alive = span.leader && same (span.leader, span.leader_gen);
	begin ("effect");
	fprintf (out, ",\"span\":%d,\"guide_alive\":%d,\"wp_alive\":%d,\"task_alive\":%d,\"group_alive\":%d,\"leader_alive\":%d", span.id, guide_alive,
		same (span.wp, span.wp_gen), task_alive, span.group && same (span.group, span.group_gen), leader_alive);
	if (guide_alive)
	{
		waypoint ("guide_wp", get_local_entity_parent (span.guide, LIST_TYPE_CURRENT_WAYPOINT));
		ref ("task_leader_now", (entity *) get_local_entity_ptr_value (span.guide, PTR_TYPE_TASK_LEADER));
	}
	if (task_alive)
	{
		task_state (span.task);
	}
	holder ("dependent", span.dependent && same (span.dependent, span.dependent_gen) ? span.dependent : NULL);
	aboard ("leader_aboard", leader_alive ? span.leader : NULL);
	end_line ();
	span.open = 0;
}

/* A6-A9: the four SUPPLY waypoint responses of a mobile (mb_msgs.c), on entry */
void eech_observe_supply_handler (int which, entity *receiver, entity *sender)
{
	static const char *names[] = {"PICK_UP", "PREPARE_FOR_DROP_OFF", "DROP_OFF", "FINISH_DROP_OFF"};
	if (!enabled ())
	{
		return;
	}
	if (span.open)
	{
		count.handlers_in_span++;
	}
	else
	{
		count.handlers_outside_span++;
	}
	header ();
	begin ("handler");
	fputs (",\"handler\":", out);
	text (names[which & 3]);
	fprintf (out, ",\"span\":%d", span.open ? span.id : 0);
	ref ("receiver", receiver);
	waypoint ("wp", sender);
	if (span.open)
	{
		fprintf (out, ",\"receiver_is_span_leader\":%d,\"wp_is_span_wp\":%d", receiver == span.leader, sender == span.wp);
		if (same (span.guide, span.guide_gen))
		{
			waypoint ("guide_wp_now", get_local_entity_parent (span.guide, LIST_TYPE_CURRENT_WAYPOINT));
		}
	}
	end_line ();
}

/* A4: TASK_COMPLETED received by a task (ts_msgs.c), on entry */
void eech_observe_task_completion_request (entity *task, int terminated)
{
	if (!enabled () || !is_supply_task (task))
	{
		return;
	}
	header ();
	begin ("completion");
	fprintf (out, ",\"span\":%d", span.open ? span.id : 0);
	ref ("task", task);
	fputs (",\"reason\":", out);
	text (terminated >= 0 && terminated < NUM_TASK_TERMINATED_TYPES ? task_terminated_names[terminated] : "?");
	task_state (task);
	end_line ();
}

/* A5: a task's completion state set (ts_int.c), for SUPPLY tasks, when not INCOMPLETE */
void eech_observe_task_completed_set (entity *task, int sub_type, int value)
{
	if (!enabled () || sub_type != ENTITY_SUB_TYPE_TASK_SUPPLY || value == TASK_INCOMPLETE)
	{
		return;
	}
	header ();
	begin ("completed");
	fprintf (out, ",\"span\":%d", span.open ? span.id : 0);
	ref ("task", task);
	fprintf (out, ",\"value\":%d", value);
	end_line ();
}
