/*
 * TEST-ONLY physical combat-resolution characterisation (EEDCS-46). Compiled only with the `weapon-fixture` feature (the same fixture build as the physical
 * weapon-execution observer; its reports are separate); never part of the shipping module.
 *
 * entity/system/en_msgs/en_msgs.c, entity/mobile/weapon/damage.c, entity/special/effect/explosn/explosn.c and entity/mobile/mb_msgs.c, built with
 * EECH_WEAPON_FIXTURE, call the eech_cmb_* hooks declared in entity/mobile/mb_cmbfx.h from inside the original code. The hooks only observe (see that header).
 *
 *   damage_hash     every weapon_damage_capability call, in order: whether a real weapon or the no-weapon forecast, the weapon type and warhead, which arithmetic ran
 *                   (armour penetrated, high-explosive splash, none), the penetration, the armour and the random modifier as exact float bits, the result before and after
 *                   the player and multiplayer modifiers, and the number of draws it took from the shared random stream
 *   collision_hash  every ENTITY_MESSAGE_COLLISION delivery, in order: the receiver's kind, the sender (weapon type or entity kind), the modifier, the damage level and the
 *                   alive flag before and after the handler, whether the handler destroyed the receiver, and what damage_client_server_entity applied
 *   blast_hash      every damage_objects_within_explosion_damage_radius call, in order: the capability and radius, each candidate examined in the order it was visited
 *                   (kind, range, modifier) and what was done to it (collision notification, rocket destroyed, nothing)
 *   repair_hash     every write of the damage level by the repair handler (a D4 coupling, observed so it is classified, not lost)
 *
 * No hash, event or counter carries an entity pointer, index, heap address or seed. The shared random stream is observed only by the number of draws between two
 * points. A hook reads only fields the original has already stored; after a collision handler an entity whose slot no longer holds the receiver is reported as
 * destroyed and read for nothing else.
 */

#ifdef EECH_WEAPON_FIXTURE

#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"
#include "entity/mobile/mb_cmbfx.h"

#define LOG_EVENTS 32768
#define LOG_TEXT 400

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

#define CAP_PATHS 3		/* 0 none, 1 armour penetrated, 2 high-explosive splash */
#define WARHEADS 16

struct kind_row
{
	uint64_t collisions, from_weapon, from_other, direct, blast, applied, no_damage, invulnerable, fatal, destroyed, already_dead;
	uint64_t candidates, in_range, notified, rocket_kills;
	double damage_sum;
	int max_damage;
};

struct fixture_trace
{
	uint64_t damage_hash, collision_hash, blast_hash, repair_hash;
	uint64_t capability_calls, capability_real, capability_forecast, capability_paths[CAP_PATHS], capability_draws, capability_player, capability_multiplier;
	uint64_t warhead_real[WARHEADS], warhead_forecast[WARHEADS], warhead_direct[WARHEADS], warhead_splash[WARHEADS];
	uint64_t damage_calls, damage_no_weapon, damage_invulnerable_calls, damage_written, damage_unwritten;
	uint64_t blast_calls, blast_zero_radius, blast_target_skipped, blast_repeats, blast_max_candidates, blast_candidates, blast_by_kill[NUM_WEAPON_KILL_CODES];
	uint64_t collision_nested_max, repair_writes, repair_zero_to_positive;
	uint64_t log_overflow, installed;
	struct kind_row kinds[NUM_ENTITY_TYPES];
};

static struct fixture_trace trace;
static int primed;

static char log_pool[LOG_EVENTS][LOG_TEXT];
static int log_count, log_drained;
static uint64_t log_seq;

/* the calls in flight */
#define NEST_MAX 8
static struct
{
	int active, type, level, alive, sender_weapon, sender_type, sender_sub;
	double modifier;
	int cap_calls, cap_result, applied, damage_end;
	int invulnerable;
} collisions[NEST_MAX];
static int depth;

static struct
{
	int active, rng0, path, raw, real, written, level_written;
} cur_cap;

static struct
{
	int active, kill_code, candidates, notified, rockets;
	uint64_t seq;
	void *weapon;
} cur_blast;
static void *last_blast_weapon;
static int last_blast_kill_code = -1;

static void fold (uint64_t *hash, uint32_t word)
{
	int byte;

	for (byte = 0; byte < 4; byte++)
	{
		*hash ^= (uint64_t) ((word >> (8 * byte)) & 0xffu);
		*hash *= FNV_PRIME;
	}
}

static uint32_t float_bits (float value)
{
	uint32_t bits;

	memcpy (&bits, &value, sizeof (bits));

	return bits;
}

static void fold_f (uint64_t *hash, float value)
{
	fold (hash, float_bits (value));
}

static void primed_reset (void)
{
	memset (&trace, 0, sizeof (trace));
	trace.damage_hash = trace.collision_hash = trace.blast_hash = trace.repair_hash = FNV_OFFSET;
	primed = 1;
}

static void ready (void)
{
	if (!primed)
	{
		primed_reset ();
	}
}

static int draws_between (int before, int after)
{
	uint32_t s = (uint32_t) before;
	int k;

	if (before == after)
	{
		return 0;
	}

	for (k = 1; k <= 64; k++)
	{
		s = s * 1103515245u + 12345u;

		if ((int) s == after)
		{
			return k;
		}
	}

	return 65;
}

static int valid_type (int type)
{
	return (type >= 0) && (type < NUM_ENTITY_TYPES);
}

static const char *type_name (int type)
{
	return valid_type (type) ? get_entity_type_name ((entity_types) type) : "invalid";
}

static int warhead_index (int warhead)
{
	return ((warhead >= 0) && (warhead < WARHEADS)) ? warhead : WARHEADS - 1;
}

static void log_event (const char *format, ...) __attribute__ ((format (printf, 1, 2)));

static void log_event (const char *format, ...)
{
	va_list args;
	int used, n;

	log_seq++;

	if (log_count >= LOG_EVENTS)
	{
		trace.log_overflow++;

		return;
	}

	used = snprintf (log_pool[log_count], LOG_TEXT, "cmb-ev seq=%llu ", (unsigned long long) log_seq);

	va_start (args, format);
	n = vsnprintf (log_pool[log_count] + used, (size_t) (LOG_TEXT - used), format, args);
	va_end (args);

	if (n < 0 || n >= LOG_TEXT - used)
	{
		/* an event that does not fit is a defect of the fixture, never silently cut */
		trace.log_overflow++;

		snprintf (log_pool[log_count], LOG_TEXT, "cmb-ev seq=%llu kind=truncated", (unsigned long long) log_seq);
	}

	log_count++;
}

static int entity_alive (entity *en)
{
	return get_local_entity_int_value (en, INT_TYPE_ALIVE) ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* damage capability */

void eech_cmb_capability_begin (void)
{
	ready ();

	memset (&cur_cap, 0, sizeof (cur_cap));

	cur_cap.active = 1;
	cur_cap.rng0 = random_number_seed;
}

void eech_cmb_capability_path (int path)
{
	if (cur_cap.active)
	{
		cur_cap.path = path;
	}
}

void eech_cmb_capability_raw (int result)
{
	if (cur_cap.active)
	{
		cur_cap.raw = result;
	}
}

void eech_cmb_capability_end (entity *wpn, entity *target, float modifier, int weapon_type, int warhead_type, float penetration, float armour, float random_modifier, int result)
{
	int draws, w, player, multiplier;

	ready ();

	if (!cur_cap.active)
	{
		return;
	}

	cur_cap.active = 0;

	draws = draws_between (cur_cap.rng0, random_number_seed);
	player = (get_local_entity_int_value (target, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI) ? 1 : 0;
	/* the multiplayer multiplier acts only for a real weapon that has a launcher: observed as whether the result moved between the raw and the final value */
	multiplier = (wpn && (result != cur_cap.raw)) ? 1 : 0;
	w = warhead_index (warhead_type);

	trace.capability_calls++;
	trace.capability_paths[cur_cap.path]++;
	trace.capability_draws += (uint64_t) draws;
	trace.capability_player += (uint64_t) player;

	if (wpn)
	{
		trace.capability_real++;
		trace.warhead_real[w]++;
	}
	else
	{
		trace.capability_forecast++;
		trace.warhead_forecast[w]++;
	}

	if (cur_cap.path == 1)
	{
		trace.warhead_direct[w]++;
	}
	else if (cur_cap.path == 2)
	{
		trace.warhead_splash[w]++;
	}

	trace.capability_multiplier += (uint64_t) multiplier;

	fold (&trace.damage_hash, wpn ? 1u : 0u);
	fold (&trace.damage_hash, (uint32_t) weapon_type);
	fold (&trace.damage_hash, (uint32_t) warhead_type);
	fold (&trace.damage_hash, (uint32_t) cur_cap.path);
	fold_f (&trace.damage_hash, modifier);
	fold_f (&trace.damage_hash, penetration);
	fold_f (&trace.damage_hash, armour);
	fold_f (&trace.damage_hash, random_modifier);
	fold (&trace.damage_hash, (uint32_t) cur_cap.raw);
	fold (&trace.damage_hash, (uint32_t) result);
	fold (&trace.damage_hash, (uint32_t) draws);
	fold (&trace.damage_hash, (uint32_t) target->type);
	fold (&trace.damage_hash, (uint32_t) player);

	if (depth > 0)
	{
		collisions[depth - 1].cap_calls++;
		collisions[depth - 1].cap_result = result;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* damage_client_server_entity */

void eech_cmb_damage_invulnerable (entity *en)
{
	(void) en;

	ready ();

	trace.damage_invulnerable_calls++;

	if (depth > 0)
	{
		collisions[depth - 1].invulnerable = 1;
	}
}

void eech_cmb_damage_written (entity *en, int new_level)
{
	(void) en;
	(void) new_level;

	ready ();

	trace.damage_written++;

	if (depth > 0)
	{
		collisions[depth - 1].applied = 1;
	}
}

void eech_cmb_damage_end (entity *en, entity *weapon, float modifier, int capability)
{
	(void) en;
	(void) modifier;

	ready ();

	trace.damage_calls++;

	if (!weapon)
	{
		trace.damage_no_weapon++;
	}

	if (capability > 0 && depth > 0 && !collisions[depth - 1].applied)
	{
		trace.damage_unwritten++;
	}

	if (depth > 0)
	{
		collisions[depth - 1].damage_end = capability;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* collisions */

static void collision_begin (entity *receiver, entity *sender, double modifier)
{
	ready ();

	if (depth >= NEST_MAX)
	{
		depth++;

		return;
	}

	memset (&collisions[depth], 0, sizeof (collisions[depth]));

	collisions[depth].active = 1;
	collisions[depth].type = receiver->type;
	collisions[depth].level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);
	collisions[depth].alive = entity_alive (receiver);
	collisions[depth].modifier = modifier;

	if (sender && (sender->type == ENTITY_TYPE_WEAPON))
	{
		collisions[depth].sender_weapon = 1;
		collisions[depth].sender_type = sender->type;
		collisions[depth].sender_sub = get_local_entity_int_value (sender, INT_TYPE_ENTITY_SUB_TYPE);
	}
	else
	{
		collisions[depth].sender_type = sender ? sender->type : -1;
	}

	depth++;

	if ((uint64_t) depth > trace.collision_nested_max)
	{
		trace.collision_nested_max = (uint64_t) depth;
	}
}

static void collision_end (entity *receiver, entity *sender)
{
	int index, destroyed, level1, alive1, fatal, type;
	struct kind_row *row;

	(void) sender;

	ready ();

	if (depth <= 0)
	{
		return;
	}

	depth--;

	if (depth >= NEST_MAX)
	{
		return;
	}

	index = depth;
	type = collisions[index].type;
	/* an entity the handler destroyed no longer holds its slot: nothing but that fact is read from it */
	destroyed = (receiver->type != (entity_types) type) || !receiver->data;
	level1 = destroyed ? 0 : get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);
	alive1 = destroyed ? 0 : entity_alive (receiver);
	fatal = destroyed || (level1 <= 0);
	row = valid_type (type) ? &trace.kinds[type] : &trace.kinds[0];

	row->collisions++;

	if (collisions[index].sender_weapon)
	{
		row->from_weapon++;
	}
	else
	{
		row->from_other++;
	}

	if (collisions[index].modifier == 0.0)
	{
		row->direct++;
	}
	else
	{
		row->blast++;
	}

	if (collisions[index].applied)
	{
		row->applied++;
		row->damage_sum += (double) (collisions[index].level - level1);

		if (collisions[index].level - level1 > row->max_damage)
		{
			row->max_damage = collisions[index].level - level1;
		}
	}
	else
	{
		row->no_damage++;
	}

	if (collisions[index].invulnerable)
	{
		row->invulnerable++;
	}

	if (!collisions[index].alive)
	{
		row->already_dead++;
	}

	if (fatal)
	{
		row->fatal++;
	}

	if (destroyed)
	{
		row->destroyed++;
	}

	fold (&trace.collision_hash, (uint32_t) type);
	fold (&trace.collision_hash, collisions[index].sender_weapon ? 1u : 0u);
	fold (&trace.collision_hash, (uint32_t) (collisions[index].sender_weapon ? collisions[index].sender_sub : collisions[index].sender_type));
	fold_f (&trace.collision_hash, (float) collisions[index].modifier);
	fold (&trace.collision_hash, (uint32_t) collisions[index].level);
	fold (&trace.collision_hash, (uint32_t) level1);
	fold (&trace.collision_hash, (uint32_t) collisions[index].alive);
	fold (&trace.collision_hash, (uint32_t) alive1);
	fold (&trace.collision_hash, (uint32_t) destroyed);
	fold (&trace.collision_hash, (uint32_t) collisions[index].cap_calls);
	fold (&trace.collision_hash, (uint32_t) collisions[index].cap_result);
	fold (&trace.collision_hash, (uint32_t) collisions[index].applied);
	fold (&trace.collision_hash, (uint32_t) collisions[index].invulnerable);

	log_event ("kind=collision receiver=%s sender=%s sub=%d modifier=%.9g level0=%d level1=%d alive0=%d alive1=%d destroyed=%d capability=%d applied=%d invulnerable=%d depth=%d",
		type_name (type), collisions[index].sender_weapon ? "weapon" : type_name (collisions[index].sender_type), collisions[index].sender_weapon ? collisions[index].sender_sub : -1,
		collisions[index].modifier, collisions[index].level, level1, collisions[index].alive, alive1, destroyed, collisions[index].damage_end, collisions[index].applied,
		collisions[index].invulnerable, depth);
}

/* the original handlers, by receiver kind, and the one wrapper every wrapped kind is registered with: it reads the receiver, runs the original handler on the same
   arguments (the modifier is read from a COPY of the argument list), reads the receiver again, and returns the original's answer */
extern int (*message_responses[NUM_ENTITY_TYPES][NUM_ENTITY_MESSAGES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

static int (*original_collision[NUM_ENTITY_TYPES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

static int collision_wrapper (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	va_list copy;
	double modifier;
	int acknowledge, type;

	va_copy (copy, pargs);
	modifier = va_arg (copy, double);
	va_end (copy);

	type = receiver->type;

	collision_begin (receiver, sender, modifier);

	acknowledge = original_collision[type] (message, receiver, sender, pargs);

	collision_end (receiver, sender);

	return acknowledge;
}

void eech_cmb_install (void *default_response)
{
	int type;

	ready ();

	for (type = 0; type < NUM_ENTITY_TYPES; type++)
	{
		if ((void *) message_responses[type][ENTITY_MESSAGE_COLLISION] != default_response)
		{
			/* installed once: a second initialisation would otherwise wrap the wrapper */
			if (message_responses[type][ENTITY_MESSAGE_COLLISION] != collision_wrapper)
			{
				original_collision[type] = message_responses[type][ENTITY_MESSAGE_COLLISION];

				message_responses[type][ENTITY_MESSAGE_COLLISION] = collision_wrapper;

				trace.installed++;
			}
		}
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* explosion blast */

void eech_cmb_blast_begin (entity *en, entity *target, int capability, float radius)
{
	int kill_code;

	ready ();

	memset (&cur_blast, 0, sizeof (cur_blast));

	cur_blast.active = 1;
	kill_code = get_local_entity_int_value (en, INT_TYPE_WEAPON_KILL_CODE);
	cur_blast.kill_code = kill_code;
	cur_blast.weapon = en;

	trace.blast_calls++;

	if ((kill_code >= 0) && (kill_code < NUM_WEAPON_KILL_CODES))
	{
		trace.blast_by_kill[kill_code]++;
	}

	/* the same weapon asked to blast twice in a row with the same kill code: the land hit's deliberate fall-through */
	if (last_blast_weapon == (void *) en && last_blast_kill_code == kill_code)
	{
		trace.blast_repeats++;
	}

	last_blast_weapon = (void *) en;
	last_blast_kill_code = kill_code;

	if (radius == 0.0f)
	{
		trace.blast_zero_radius++;
	}

	fold (&trace.blast_hash, (uint32_t) kill_code);
	fold (&trace.blast_hash, (uint32_t) get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE));
	fold (&trace.blast_hash, (uint32_t) get_local_entity_int_value (en, INT_TYPE_WEAPON_WARHEAD_TYPE));
	fold (&trace.blast_hash, (uint32_t) capability);
	fold_f (&trace.blast_hash, radius);
	fold (&trace.blast_hash, target ? 1u : 0u);

	log_event ("kind=blast-begin kill_code=%d sub=%d capability=%d radius=%.9g has_target=%d", kill_code, get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE), capability,
		(double) radius, target ? 1 : 0);
}

void eech_cmb_blast_candidate (entity *object, float range, float radius)
{
	int type = object->type;
	struct kind_row *row = valid_type (type) ? &trace.kinds[type] : &trace.kinds[0];
	int in_range = (range != 0.0f) && (range < radius);

	ready ();

	cur_blast.candidates++;
	trace.blast_candidates++;
	row->candidates++;

	if (in_range)
	{
		row->in_range++;
	}

	fold (&trace.blast_hash, (uint32_t) type);
	fold_f (&trace.blast_hash, range);
	fold (&trace.blast_hash, (uint32_t) in_range);
}

/* an object inside the radius, with its modifier: everything but a weapon is sent a collision (and the collision hook then records the damage it did); a weapon in range is
   only a rocket-destruction candidate */
void eech_cmb_blast_notify (entity *object, float modifier)
{
	int type = object->type;
	struct kind_row *row = valid_type (type) ? &trace.kinds[type] : &trace.kinds[0];
	int notified = (type != ENTITY_TYPE_WEAPON);

	ready ();

	if (notified)
	{
		cur_blast.notified++;
		row->notified++;
	}

	fold (&trace.blast_hash, notified ? 0xC0111510u : 0x0A11D3A5u);
	fold (&trace.blast_hash, (uint32_t) type);
	fold_f (&trace.blast_hash, modifier);
}

void eech_cmb_blast_rocket (entity *object, float modifier)
{
	int type = object->type;
	struct kind_row *row = valid_type (type) ? &trace.kinds[type] : &trace.kinds[0];

	ready ();

	cur_blast.rockets++;
	row->rocket_kills++;

	fold (&trace.blast_hash, 0x0C4E7000u);
	fold (&trace.blast_hash, (uint32_t) type);
	fold_f (&trace.blast_hash, modifier);
}

void eech_cmb_blast_end (void)
{
	ready ();

	if ((uint64_t) cur_blast.candidates > trace.blast_max_candidates)
	{
		trace.blast_max_candidates = (uint64_t) cur_blast.candidates;
	}

	fold (&trace.blast_hash, (uint32_t) cur_blast.candidates);
	fold (&trace.blast_hash, (uint32_t) cur_blast.notified);
	fold (&trace.blast_hash, (uint32_t) cur_blast.rockets);

	log_event ("kind=blast-end kill_code=%d candidates=%d notified=%d rockets=%d", cur_blast.kill_code, cur_blast.candidates, cur_blast.notified, cur_blast.rockets);

	cur_blast.active = 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* repair */

void eech_cmb_repair (entity *en, int old_level, int new_level)
{
	ready ();

	trace.repair_writes++;

	if (old_level <= 0 && new_level > 0)
	{
		trace.repair_zero_to_positive++;
	}

	fold (&trace.repair_hash, (uint32_t) en->type);
	fold (&trace.repair_hash, (uint32_t) old_level);
	fold (&trace.repair_hash, (uint32_t) new_level);

	log_event ("kind=repair entity=%s level0=%d level1=%d", type_name (en->type), old_level, new_level);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* report */

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

static int report (char *text, int capacity)
{
	struct out out;
	int t, w;

	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	text[0] = 0;

	emit (&out, "cmb-hash damage=%016llx collision=%016llx blast=%016llx repair=%016llx\n", (unsigned long long) trace.damage_hash, (unsigned long long) trace.collision_hash,
		(unsigned long long) trace.blast_hash, (unsigned long long) trace.repair_hash);
	emit (&out, "cmb-capability calls=%llu real=%llu forecast=%llu none=%llu penetrated=%llu splash=%llu draws=%llu player_target=%llu multiplier_moved=%llu\n",
		(unsigned long long) trace.capability_calls, (unsigned long long) trace.capability_real, (unsigned long long) trace.capability_forecast,
		(unsigned long long) trace.capability_paths[0], (unsigned long long) trace.capability_paths[1], (unsigned long long) trace.capability_paths[2],
		(unsigned long long) trace.capability_draws, (unsigned long long) trace.capability_player, (unsigned long long) trace.capability_multiplier);
	emit (&out, "cmb-damage calls=%llu no_weapon=%llu invulnerable=%llu written=%llu positive_unwritten=%llu\n", (unsigned long long) trace.damage_calls,
		(unsigned long long) trace.damage_no_weapon, (unsigned long long) trace.damage_invulnerable_calls, (unsigned long long) trace.damage_written,
		(unsigned long long) trace.damage_unwritten);
	emit (&out, "cmb-blast calls=%llu zero_radius=%llu candidates=%llu max_candidates=%llu repeats_same_weapon=%llu nested_collision_max=%llu\n", (unsigned long long) trace.blast_calls,
		(unsigned long long) trace.blast_zero_radius, (unsigned long long) trace.blast_candidates, (unsigned long long) trace.blast_max_candidates,
		(unsigned long long) trace.blast_repeats, (unsigned long long) trace.collision_nested_max);

	for (w = 0; w < NUM_WEAPON_KILL_CODES; w++)
	{
		if (trace.blast_by_kill[w])
		{
			emit (&out, "cmb-blast-kill code=%d calls=%llu\n", w, (unsigned long long) trace.blast_by_kill[w]);
		}
	}

	for (w = 0; w < WARHEADS; w++)
	{
		if (trace.warhead_real[w] || trace.warhead_forecast[w])
		{
			emit (&out, "cmb-warhead type=%d real=%llu forecast=%llu penetrated=%llu splash=%llu\n", w, (unsigned long long) trace.warhead_real[w],
				(unsigned long long) trace.warhead_forecast[w], (unsigned long long) trace.warhead_direct[w], (unsigned long long) trace.warhead_splash[w]);
		}
	}

	for (t = 0; t < NUM_ENTITY_TYPES; t++)
	{
		const struct kind_row *r = &trace.kinds[t];

		if (r->collisions || r->candidates)
		{
			emit (&out,
				"cmb-kind type=%s collisions=%llu from_weapon=%llu from_other=%llu direct=%llu blast=%llu applied=%llu no_damage=%llu invulnerable=%llu fatal=%llu destroyed=%llu "
				"already_dead=%llu damage_sum=%.9g max_damage=%d candidates=%llu in_range=%llu notified=%llu rocket_kills=%llu\n",
				type_name (t), (unsigned long long) r->collisions, (unsigned long long) r->from_weapon, (unsigned long long) r->from_other, (unsigned long long) r->direct,
				(unsigned long long) r->blast, (unsigned long long) r->applied, (unsigned long long) r->no_damage, (unsigned long long) r->invulnerable,
				(unsigned long long) r->fatal, (unsigned long long) r->destroyed, (unsigned long long) r->already_dead, r->damage_sum, r->max_damage,
				(unsigned long long) r->candidates, (unsigned long long) r->in_range, (unsigned long long) r->notified, (unsigned long long) r->rocket_kills);
		}
	}

	emit (&out, "cmb-install wrapped_collision_handlers=%llu\n", (unsigned long long) trace.installed);
	emit (&out, "cmb-repair writes=%llu zero_to_positive=%llu\n", (unsigned long long) trace.repair_writes, (unsigned long long) trace.repair_zero_to_positive);
	emit (&out, "cmb-log events=%llu stored=%d drained=%d overflow=%llu\n", (unsigned long long) log_seq, log_count, log_drained, (unsigned long long) trace.log_overflow);

	return out.overflow ? -1 : 0;
}

int eech_cmb_trace_get (char *text, int capacity)
{
	ready ();

	return report (text, capacity);
}

/* drains the ordered event log from the cursor: whole events only; returns 0 when everything stored has been drained and 1 when more remains */
int eech_cmb_events_get (char *text, int capacity)
{
	int used = 0;

	ready ();

	text[0] = 0;

	while (log_drained < log_count)
	{
		int n = (int) strlen (log_pool[log_drained]);

		if (used + n + 2 > capacity)
		{
			break;
		}

		memcpy (text + used, log_pool[log_drained], (size_t) n);
		used += n;
		text[used++] = '\n';
		text[used] = 0;
		log_drained++;
	}

	return log_drained < log_count ? 1 : 0;
}

#endif
