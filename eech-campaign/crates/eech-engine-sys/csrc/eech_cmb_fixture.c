/*
 * TEST-ONLY physical combat-resolution characterisation (EEDCS-46). Compiled only with the `weapon-fixture` feature (the same fixture build as the physical
 * weapon-execution observer; its reports are separate); never part of the shipping module.
 *
 * entity/system/en_msgs/en_msgs.c, entity/system/en_main/en_heap.c, entity/mobile/weapon/damage.c, entity/special/effect/explosn/explosn.c, entity/mobile/mb_msgs.c,
 * entity/fixed/fixed.c and the six per-frame update files, built with EECH_WEAPON_FIXTURE, call the eech_cmb_* hooks declared in entity/mobile/mb_cmbfx.h from inside the
 * original code. The hooks only observe (see that header).
 *
 *   damage_hash     every weapon_damage_capability call, in order: whether a real weapon or the no-weapon forecast, the weapon type and warhead, which arithmetic ran
 *                   (armour penetrated, high-explosive splash, none), the penetration, the armour and the random modifier as exact float bits, the result before and after
 *                   the player and multiplayer modifiers, and the number of draws it took from the shared random stream
 *   collision_hash  every ENTITY_MESSAGE_COLLISION delivery to one of the thirteen kinds that hold a damage level (six mobile, seven fixed), in order: the receiver's kind,
 *                   the sender (weapon type or entity kind), the modifier, the damage level and the alive flag before and after the handler, whether the receiver was
 *                   destroyed during the handler, and what damage_client_server_entity applied
 *   blast_hash      every damage_objects_within_explosion_damage_radius call, in order: the capability and radius, each candidate examined in the order it was visited
 *                   (kind, range, modifier) and what was done to it (collision notification, rocket destroyed, nothing); and the explosion invocation each call belongs to
 *   repair_hash     every write of the damage level that restores health (the mobile resupply repair and the fixed-structure restore: D4 coupling, observed so it is
 *                   classified, not lost)
 *   timed_hash      every per-frame timed health attempt (helicopter and fixed-wing fire, a person's bleeding, an anti-aircraft, routed or ship vehicle's fire): the kind,
 *                   the draws the original's condition took from the shared stream, whether it fired and the health it left
 *   order_hash      the order in which the categories above happened
 *
 * No hash, event or counter carries an entity pointer, index, heap address or seed. The shared random stream is observed only by the number of draws between two
 * points. IDENTITY: the receiver of a collision is identified across its handler by a fixture-private generation of its slot, advanced when the entity heap frees the slot
 * (eech_cmb_release); a slot freed during the handler is reported as destroyed and read for nothing else, so a slot reused by another entity is never read as the receiver.
 * COVERAGE LIMITS, declared in the report: deliveries to kinds that hold no damage level (a bridge, a cargo, a weapon) are only counted, never read (a bridge is notified
 * by its segments with no fourth argument); nesting deeper than NEST_MAX frames and any release of a slot outside the entity array are counted and make the trace untrusted.
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
#define NEST_MAX 8
#define GEN_SLOTS 262144

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

#define CAP_PATHS 3		/* 0 none, 1 armour penetrated, 2 high-explosive splash */
#define WARHEADS 16

enum category
{
	CAT_CAPABILITY = 1,
	CAT_DAMAGE,
	CAT_COLLISION,
	CAT_BLAST,
	CAT_EXPLOSION,
	CAT_REPAIR,
	CAT_TIMED
};

struct kind_row
{
	uint64_t collisions, from_weapon, from_other, direct, blast, applied, no_damage, invulnerable, fatal, destroyed, already_dead;
	uint64_t candidates, in_range, notified, rocket_kills;
	double damage_sum;
	int max_damage;
};

struct timed_row
{
	uint64_t attempts, draws, hits, fatal;
};

struct fixture_trace
{
	uint64_t damage_hash, collision_hash, blast_hash, repair_hash, timed_hash, order_hash;
	uint64_t capability_calls, capability_real, capability_forecast, capability_paths[CAP_PATHS], capability_draws, capability_player, capability_multiplier;
	uint64_t warhead_real[WARHEADS], warhead_forecast[WARHEADS], warhead_direct[WARHEADS], warhead_splash[WARHEADS];
	uint64_t damage_entries, damage_exit_no_weapon, damage_exit_invulnerable, damage_exit_done, damage_written, damage_unwritten;
	uint64_t explosions, explosion_blast_calls_max, explosions_with_repeated_blast, blast_outside_explosion;
	uint64_t blast_calls, blast_zero_radius, blast_candidates, blast_max_candidates, blast_by_kill[NUM_WEAPON_KILL_CODES];
	uint64_t repair_writes[3], repair_zero_to_positive;
	uint64_t depth_max, depth_overflow, depth_unbalanced, identity_unknown, release_outside, releases, observed_deliveries, unobserved_deliveries[NUM_ENTITY_TYPES];
	uint64_t installed, log_overflow;
	struct kind_row kinds[NUM_ENTITY_TYPES];
	struct timed_row timed[EECH_CMB_TIMEDS];
};

static struct fixture_trace trace;
static int primed;

static uint32_t generation[GEN_SLOTS];

static char log_pool[LOG_EVENTS][LOG_TEXT];
static int log_count, log_drained;
static uint64_t log_seq;

/* the collision handlers in flight (a handler can notify another entity, so they nest) */
struct collision_frame
{
	int type, level, alive, sender_weapon, sender_type, sender_sub, slot;
	uint32_t generation;
	double modifier;
	int cap_calls, cap_result, applied, damage_end, invulnerable;
};

static struct collision_frame collision_frames[NEST_MAX];
static int depth;

static struct
{
	int active, rng0, path, raw;
} cur_cap;

static struct
{
	int active, kill_code, candidates, notified, rockets;
} cur_blast;

static struct
{
	int active, kill_code, blast_calls;
} cur_explosion;

static struct
{
	int active, kind, rng0;
} cur_timed;

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
	trace.damage_hash = trace.collision_hash = trace.blast_hash = trace.repair_hash = trace.timed_hash = trace.order_hash = FNV_OFFSET;
	primed = 1;
}

static void ready (void)
{
	if (!primed)
	{
		primed_reset ();
	}
}

static void order (enum category category)
{
	fold (&trace.order_hash, (uint32_t) category);
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

/* the thirteen kinds whose collision handlers read the damage modifier and which hold a damage level */
static int observed_kind (int type)
{
	switch (type)
	{
		case ENTITY_TYPE_ANTI_AIRCRAFT:
		case ENTITY_TYPE_CITY_BUILDING:
		case ENTITY_TYPE_FIXED_WING:
		case ENTITY_TYPE_HELICOPTER:
		case ENTITY_TYPE_OBJECT:
		case ENTITY_TYPE_PERSON:
		case ENTITY_TYPE_PYLON:
		case ENTITY_TYPE_ROUTED_VEHICLE:
		case ENTITY_TYPE_SCENIC:
		case ENTITY_TYPE_SEGMENT:
		case ENTITY_TYPE_SHIP_VEHICLE:
		case ENTITY_TYPE_SITE:
		case ENTITY_TYPE_SITE_UPDATABLE:
		{
			return 1;
		}
	}

	return 0;
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

/* the slot of an entity in the entity array, or -1 when it is not in it (or beyond the generation table) */
static int slot_of (entity *en)
{
	if (entities && en >= entities && en < entities + number_of_entities && (en - entities) < GEN_SLOTS)
	{
		return (int) (en - entities);
	}

	return -1;
}

/* the frame of the collision handler now running, or NULL when none is or it is deeper than the table */
static struct collision_frame *current_frame (void)
{
	if (depth > 0 && depth <= NEST_MAX)
	{
		return &collision_frames[depth - 1];
	}

	return NULL;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* slot generations */

void eech_cmb_release (entity *en)
{
	int slot;

	ready ();

	trace.releases++;

	slot = slot_of (en);

	if (slot < 0)
	{
		trace.release_outside++;

		return;
	}

	generation[slot]++;
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
	if (cur_cap.active && path >= 0 && path < CAP_PATHS)
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

/* the damage forecast is the host's (EEDCS-46): the reference host echoes the original's hooks of it in ONE event, and this replays them in the order the original made them
   (begin, the path when one ran, the raw capability, end).  A forecast draws nothing, so the observer sees none.  A host that echoes nothing (a model of its own) is not observed. */
void eech_cmb_capability_forecast (entity *target, int weapon_type, const campaign_aim_event *events, int count, int capability)
{
	const campaign_aim_event *event;

	if (!events || count != 1)
	{
		return;
	}

	event = &events[0];

	eech_cmb_capability_begin ();

	if (event->a)
	{
		eech_cmb_capability_path (event->a);
	}

	eech_cmb_capability_raw (event->b);

	eech_cmb_capability_end (NULL, target, event->f[0], weapon_type, event->c, event->f[1], event->f[2], event->f[3], capability);
}

/* the capability of a REAL weapon is the host's (EEDCS-46): the campaign began the original's hooks before the host took its one sample, and the reference host echoes the rest
   in ONE event; this replays the path when one ran, the raw capability and the end, in the order the original made them.  A host that echoes nothing is not observed. */
void eech_cmb_capability_damage (entity *wpn, entity *target, int weapon_type, const campaign_aim_event *events, int count, int capability)
{
	const campaign_aim_event *event;

	if (!events || count != 1)
	{
		return;
	}

	event = &events[0];

	if (event->a)
	{
		eech_cmb_capability_path (event->a);
	}

	eech_cmb_capability_raw (event->b);

	/* the four intermediate floats arrive as raw bits (they can legitimately be a NaN: a hit with no direction) */
	eech_cmb_capability_end (wpn, target, campaign_aim_damage_echo_float (event, 0), weapon_type, event->c, campaign_aim_damage_echo_float (event, 1), campaign_aim_damage_echo_float (event, 2),
		campaign_aim_damage_echo_float (event, 3), capability);
}

void eech_cmb_capability_end (entity *wpn, entity *target, float modifier, int weapon_type, int warhead_type, float penetration, float armour, float random_modifier, int result)
{
	int draws, w, player, multiplier;
	struct collision_frame *frame;

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

	order (CAT_CAPABILITY);

	frame = current_frame ();

	if (frame)
	{
		frame->cap_calls++;
		frame->cap_result = result;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* damage_client_server_entity: every entry has exactly one exit (no weapon, invulnerable, or done) */

void eech_cmb_damage_begin (entity *en, entity *weapon)
{
	(void) en;
	(void) weapon;

	ready ();

	trace.damage_entries++;
}

void eech_cmb_damage_exit (int reason)
{
	struct collision_frame *frame;

	ready ();

	if (reason == EECH_CMB_DAMAGE_EXIT_NO_WEAPON)
	{
		trace.damage_exit_no_weapon++;
	}
	else
	{
		trace.damage_exit_invulnerable++;

		frame = current_frame ();

		if (frame)
		{
			frame->invulnerable = 1;
		}
	}

	order (CAT_DAMAGE);
}

void eech_cmb_damage_written (entity *en, int new_level)
{
	struct collision_frame *frame;

	(void) en;
	(void) new_level;

	ready ();

	trace.damage_written++;

	frame = current_frame ();

	if (frame)
	{
		frame->applied = 1;
	}
}

void eech_cmb_damage_end (entity *en, entity *weapon, float modifier, int capability)
{
	struct collision_frame *frame;

	(void) en;
	(void) weapon;
	(void) modifier;

	ready ();

	trace.damage_exit_done++;

	frame = current_frame ();

	if (capability > 0 && frame && !frame->applied)
	{
		trace.damage_unwritten++;
	}

	if (frame)
	{
		frame->damage_end = capability;
	}

	order (CAT_DAMAGE);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* collisions */

static void collision_begin (entity *receiver, entity *sender, double modifier)
{
	struct collision_frame *frame;
	int slot;

	ready ();

	depth++;

	if ((uint64_t) depth > trace.depth_max)
	{
		trace.depth_max = (uint64_t) depth;
	}

	if (depth > NEST_MAX)
	{
		/* beyond the table: counted, makes the trace untrusted, and no frame is written or read for it */
		trace.depth_overflow++;

		return;
	}

	frame = &collision_frames[depth - 1];

	memset (frame, 0, sizeof (*frame));

	slot = slot_of (receiver);

	frame->slot = slot;
	frame->generation = (slot >= 0) ? generation[slot] : 0;

	if (slot < 0)
	{
		trace.identity_unknown++;
	}

	frame->type = receiver->type;
	frame->level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);
	frame->alive = entity_alive (receiver);
	frame->modifier = modifier;

	if (sender && (sender->type == ENTITY_TYPE_WEAPON))
	{
		frame->sender_weapon = 1;
		frame->sender_type = sender->type;
		frame->sender_sub = get_local_entity_int_value (sender, INT_TYPE_ENTITY_SUB_TYPE);
	}
	else
	{
		frame->sender_type = sender ? sender->type : -1;
	}
}

static void collision_end (entity *receiver)
{
	int index, destroyed, level1, alive1, fatal, type;
	struct kind_row *row;
	struct collision_frame *frame;

	ready ();

	if (depth <= 0)
	{
		trace.depth_unbalanced++;

		return;
	}

	index = depth - 1;

	depth--;

	if (index >= NEST_MAX)
	{
		return;
	}

	frame = &collision_frames[index];
	type = frame->type;
	/* the receiver is the same entity only if its slot has not been freed since the handler began (and, as a second check, still holds its kind); an entity that is not
	   provably the receiver is reported as destroyed and read for nothing else */
	destroyed = (frame->slot < 0) || (generation[frame->slot] != frame->generation) || (receiver->type != (entity_types) type) || !receiver->data;
	level1 = destroyed ? 0 : get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);
	alive1 = destroyed ? 0 : entity_alive (receiver);
	fatal = destroyed || (level1 <= 0);
	row = valid_type (type) ? &trace.kinds[type] : &trace.kinds[0];

	trace.observed_deliveries++;

	row->collisions++;

	if (frame->sender_weapon)
	{
		row->from_weapon++;
	}
	else
	{
		row->from_other++;
	}

	if (frame->modifier == 0.0)
	{
		row->direct++;
	}
	else
	{
		row->blast++;
	}

	if (frame->applied)
	{
		row->applied++;
		row->damage_sum += (double) (frame->level - level1);

		if (frame->level - level1 > row->max_damage)
		{
			row->max_damage = frame->level - level1;
		}
	}
	else
	{
		row->no_damage++;
	}

	if (frame->invulnerable)
	{
		row->invulnerable++;
	}

	if (!frame->alive)
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
	fold (&trace.collision_hash, frame->sender_weapon ? 1u : 0u);
	fold (&trace.collision_hash, (uint32_t) (frame->sender_weapon ? frame->sender_sub : frame->sender_type));
	fold_f (&trace.collision_hash, (float) frame->modifier);
	fold (&trace.collision_hash, (uint32_t) frame->level);
	fold (&trace.collision_hash, (uint32_t) level1);
	fold (&trace.collision_hash, (uint32_t) frame->alive);
	fold (&trace.collision_hash, (uint32_t) alive1);
	fold (&trace.collision_hash, (uint32_t) destroyed);
	fold (&trace.collision_hash, (uint32_t) frame->cap_calls);
	fold (&trace.collision_hash, (uint32_t) frame->cap_result);
	fold (&trace.collision_hash, (uint32_t) frame->applied);
	fold (&trace.collision_hash, (uint32_t) frame->invulnerable);

	order (CAT_COLLISION);

	log_event ("kind=collision receiver=%s sender=%s sub=%d modifier=%.9g level0=%d level1=%d alive0=%d alive1=%d destroyed=%d capability=%d applied=%d invulnerable=%d depth=%d",
		type_name (type), frame->sender_weapon ? "weapon" : type_name (frame->sender_type), frame->sender_weapon ? frame->sender_sub : -1, frame->modifier, frame->level, level1,
		frame->alive, alive1, destroyed, frame->damage_end, frame->applied, frame->invulnerable, depth);
}

/* the original handlers, by receiver kind, and the one wrapper every registered kind is routed through: for the observed kinds it reads the receiver and the damage modifier
   (from a COPY of the argument list), runs the original on the same arguments and reads the receiver again; for every other kind it only counts the delivery and runs the
   original, never touching the argument list (a bridge is notified with no fourth argument) */
extern int (*message_responses[NUM_ENTITY_TYPES][NUM_ENTITY_MESSAGES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

static int (*original_collision[NUM_ENTITY_TYPES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

static int collision_wrapper (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	va_list copy;
	double modifier;
	int acknowledge, type;

	type = receiver->type;

	if (!observed_kind (type))
	{
		trace.unobserved_deliveries[valid_type (type) ? type : 0]++;

		return original_collision[type] (message, receiver, sender, pargs);
	}

	va_copy (copy, pargs);
	modifier = va_arg (copy, double);
	va_end (copy);

	collision_begin (receiver, sender, modifier);

	acknowledge = original_collision[type] (message, receiver, sender, pargs);

	collision_end (receiver);

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
/* explosion blast: a blast call belongs to the explosion invocation it runs inside, not to a weapon's address */

void eech_cmb_explosion_begin (int kill_code)
{
	ready ();

	memset (&cur_explosion, 0, sizeof (cur_explosion));

	cur_explosion.active = 1;
	cur_explosion.kill_code = kill_code;

	trace.explosions++;
}

void eech_cmb_explosion_end (void)
{
	ready ();

	if (!cur_explosion.active)
	{
		return;
	}

	if ((uint64_t) cur_explosion.blast_calls > trace.explosion_blast_calls_max)
	{
		trace.explosion_blast_calls_max = (uint64_t) cur_explosion.blast_calls;
	}

	/* a land hit blasts twice in one invocation (the crater branch falls through into the case that blasts again) */
	if (cur_explosion.blast_calls >= 2)
	{
		trace.explosions_with_repeated_blast++;
	}

	fold (&trace.blast_hash, 0xE0710510u);
	fold (&trace.blast_hash, (uint32_t) cur_explosion.kill_code);
	fold (&trace.blast_hash, (uint32_t) cur_explosion.blast_calls);

	order (CAT_EXPLOSION);

	log_event ("kind=explosion kill_code=%d blast_calls=%d", cur_explosion.kill_code, cur_explosion.blast_calls);

	cur_explosion.active = 0;
}

void eech_cmb_blast_begin (entity *en, entity *target, int capability, float radius)
{
	int kill_code;

	ready ();

	memset (&cur_blast, 0, sizeof (cur_blast));

	cur_blast.active = 1;
	kill_code = get_local_entity_int_value (en, INT_TYPE_WEAPON_KILL_CODE);
	cur_blast.kill_code = kill_code;

	trace.blast_calls++;

	if ((kill_code >= 0) && (kill_code < NUM_WEAPON_KILL_CODES))
	{
		trace.blast_by_kill[kill_code]++;
	}

	if (cur_explosion.active)
	{
		cur_explosion.blast_calls++;
	}
	else
	{
		trace.blast_outside_explosion++;
	}

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

	log_event ("kind=blast-begin kill_code=%d sub=%d capability=%d radius=%.9g has_target=%d in_explosion=%d", kill_code, get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE),
		capability, (double) radius, target ? 1 : 0, cur_explosion.active);
}

static void eech_cmb_blast_candidate (entity *object, float range, float radius)
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

/* an object inside the radius, with its modifier: everything but a weapon is sent a collision (and the collision wrapper then records the damage it did); a weapon in range
   is only a rocket-destruction candidate */
static void eech_cmb_blast_notify (entity *object, float modifier)
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

static void eech_cmb_blast_rocket (entity *object, float modifier)
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

/* The blast walk is the host's (EEDCS-46): the reference host echoes one event per candidate it examined in the window it was given, in the order examined, and this replays the
   original's hooks for them (the candidate, the notification of an object inside the radius, the destruction of a rocket) in the order the original made them.  The campaign calls
   it just before it acts on the object the host reached, so the hooks of the candidates before that object, and of that object itself, precede the collision it causes, as they did.
   A host that echoes nothing is not observed. */
void eech_cmb_blast_examined (void *context, const campaign_aim_result *result, void *const *objects, int count)
{
	int i;

	(void) context;

	for (i = 0; i < result->echo_count; i++)
	{
		const campaign_aim_event *event = &result->echo[i];
		entity *object;

		if ((event->kind != CAMPAIGN_AIM_EVENT_BLAST) || (event->a < 0) || (event->a >= count))
		{
			continue;
		}

		object = (entity *) objects[event->a];

		eech_cmb_blast_candidate (object, event->f[0], event->f[1]);

		if (event->b & 1)
		{
			eech_cmb_blast_notify (object, event->f[2]);
		}

		if (event->b & 2)
		{
			eech_cmb_blast_rocket (object, event->f[2]);
		}
	}
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

	order (CAT_BLAST);

	log_event ("kind=blast-end kill_code=%d candidates=%d notified=%d rockets=%d", cur_blast.kill_code, cur_blast.candidates, cur_blast.notified, cur_blast.rockets);

	cur_blast.active = 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the controlled bridge case (EEDCS-46, R5): the ORIGINAL bridge and segment code is observed by CMB_FX_BRIDGE only while csrc/eech_cmb_case.c has armed it; outside the case every hook
   returns before it reads or writes anything (no event, no sequence number), so a natural run's report and event log are untouched.  A hook only reads. */

static int bridge_case_armed;

void eech_cmb_bridge_case_set (int active)
{
	ready ();

	bridge_case_armed = active ? 1 : 0;
}

static const char *bridge_event_name (int kind)
{
	switch (kind)
	{
		case EECH_CMB_BRIDGE_SEGMENT_UPDATE: return "segment-update";
		case EECH_CMB_BRIDGE_COLLISION: return "collision";
		case EECH_CMB_BRIDGE_UNSUPPORTED_KILL: return "unsupported-kill";
		case EECH_CMB_BRIDGE_SHAPE: return "shape";
		case EECH_CMB_BRIDGE_CHILD_KILLED: return "child-killed";
		case EECH_CMB_BRIDGE_LINK_BROKEN: return "link-broken";
		case EECH_CMB_BRIDGE_REPAIR_TASK: return "repair-task";
		case EECH_CMB_BRIDGE_UPDATE_INSERTED: return "update-inserted";
		case EECH_CMB_BRIDGE_FALL: return "fall";
		case EECH_CMB_BRIDGE_LANDED: return "landed";
		case EECH_CMB_BRIDGE_SUPPORTED_REMOVED: return "supported-removed";
	}

	return "unknown";
}

void eech_cmb_bridge_event (int kind, entity *bridge, entity *segment, int a, int b, float f)
{
	entity *en;
	vec3d *position;
	int ordinal = -1, bridges = 0, index = -1, segments = 0;

	if (!bridge_case_armed)
	{
		return;
	}

	ready ();

	for (en = get_local_entity_list (); en; en = get_local_entity_succ (en))
	{
		if (!(entities && en >= entities && en < entities + number_of_entities))
		{
			break;
		}

		if (en->type == ENTITY_TYPE_BRIDGE)
		{
			if (en == bridge)
			{
				ordinal = bridges;
			}

			bridges++;
		}
	}

	for (en = get_local_entity_first_child (bridge, LIST_TYPE_SEGMENT); en; en = get_local_entity_child_succ (en, LIST_TYPE_SEGMENT))
	{
		if (en == segment)
		{
			index = segments;
		}

		segments++;
	}

	position = get_local_entity_vec3d_ptr (segment, VEC3D_TYPE_POSITION);

	log_event ("kind=bridge-%s bridge=%d seg=%d of=%d sub=%d alive=%d shape=%d y=%.9g a=%d b=%d f=%.9g", bridge_event_name (kind), ordinal, index, segments,
		get_local_entity_int_value (segment, INT_TYPE_ENTITY_SUB_TYPE), get_local_entity_int_value (segment, INT_TYPE_ALIVE) ? 1 : 0, get_local_entity_int_value (segment, INT_TYPE_OBJECT_3D_SHAPE),
		position ? (double) position->y : 0.0, a, b, (double) f);
}
/* ---------------------------------------------------------------------------------------------------------------------------- */
/* repair and restore */

void eech_cmb_repair (entity *en, int old_level, int new_level, int source)
{
	ready ();

	if (source < 1 || source > 2)
	{
		source = 0;
	}

	trace.repair_writes[source]++;

	if (old_level <= 0 && new_level > 0)
	{
		trace.repair_zero_to_positive++;
	}

	fold (&trace.repair_hash, (uint32_t) source);
	fold (&trace.repair_hash, (uint32_t) en->type);
	fold (&trace.repair_hash, (uint32_t) old_level);
	fold (&trace.repair_hash, (uint32_t) new_level);

	order (CAT_REPAIR);

	log_event ("kind=repair source=%s entity=%s level0=%d level1=%d", source == EECH_CMB_REPAIR_RESUPPLY ? "resupply" : source == EECH_CMB_REPAIR_FIXED_RESTORE ? "fixed_restore" : "unknown",
		type_name (en->type), old_level, new_level);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* timed health loss: the per-frame fire and bleeding rolls. BEGIN is before the original's condition (which may or may not draw), HIT is inside the body that lowers health,
   END after the condition when the body did not run */

static void timed_finish (int hit, int new_level, entity *en)
{
	int draws = draws_between (cur_timed.rng0, random_number_seed);
	struct timed_row *row = &trace.timed[cur_timed.kind];

	row->attempts++;
	row->draws += (uint64_t) draws;

	if (hit)
	{
		row->hits++;

		if (new_level <= 0)
		{
			row->fatal++;
		}
	}

	fold (&trace.timed_hash, (uint32_t) cur_timed.kind);
	fold (&trace.timed_hash, (uint32_t) draws);
	fold (&trace.timed_hash, (uint32_t) hit);

	if (hit)
	{
		fold (&trace.timed_hash, (uint32_t) new_level);
		fold (&trace.timed_hash, (uint32_t) en->type);

		order (CAT_TIMED);

		log_event ("kind=timed type=%d receiver=%s draws=%d new_level=%d", cur_timed.kind, type_name (en->type), draws, new_level);
	}

	cur_timed.active = 0;
}

void eech_cmb_timed_begin (int kind)
{
	ready ();

	cur_timed.active = 0;

	if (kind < 1 || kind >= EECH_CMB_TIMEDS)
	{
		return;
	}

	cur_timed.active = 1;
	cur_timed.kind = kind;
	cur_timed.rng0 = random_number_seed;
}

void eech_cmb_timed_hit (entity *en, int new_level)
{
	ready ();

	if (cur_timed.active)
	{
		timed_finish (1, new_level, en);
	}
}

void eech_cmb_timed_end (void)
{
	ready ();

	if (cur_timed.active)
	{
		timed_finish (0, 0, NULL);
	}
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
	uint64_t exits;

	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	text[0] = 0;

	exits = trace.damage_exit_no_weapon + trace.damage_exit_invulnerable + trace.damage_exit_done;

	emit (&out, "cmb-hash damage=%016llx collision=%016llx blast=%016llx repair=%016llx timed=%016llx order=%016llx\n", (unsigned long long) trace.damage_hash,
		(unsigned long long) trace.collision_hash, (unsigned long long) trace.blast_hash, (unsigned long long) trace.repair_hash, (unsigned long long) trace.timed_hash,
		(unsigned long long) trace.order_hash);
	emit (&out, "cmb-integrity depth_max=%llu depth_overflow=%llu depth_unbalanced=%llu identity_unknown=%llu release_outside=%llu log_overflow=%llu releases=%llu trusted=%d\n",
		(unsigned long long) trace.depth_max, (unsigned long long) trace.depth_overflow, (unsigned long long) trace.depth_unbalanced, (unsigned long long) trace.identity_unknown,
		(unsigned long long) trace.release_outside, (unsigned long long) trace.log_overflow, (unsigned long long) trace.releases,
		(trace.depth_overflow || trace.depth_unbalanced || trace.identity_unknown || trace.release_outside || trace.log_overflow) ? 0 : 1);
	emit (&out, "cmb-capability calls=%llu real=%llu forecast=%llu none=%llu penetrated=%llu splash=%llu draws=%llu player_target=%llu multiplier_moved=%llu\n",
		(unsigned long long) trace.capability_calls, (unsigned long long) trace.capability_real, (unsigned long long) trace.capability_forecast,
		(unsigned long long) trace.capability_paths[0], (unsigned long long) trace.capability_paths[1], (unsigned long long) trace.capability_paths[2],
		(unsigned long long) trace.capability_draws, (unsigned long long) trace.capability_player, (unsigned long long) trace.capability_multiplier);
	emit (&out, "cmb-damage entries=%llu exit_no_weapon=%llu exit_invulnerable=%llu exit_done=%llu balanced=%d written=%llu positive_unwritten=%llu\n",
		(unsigned long long) trace.damage_entries, (unsigned long long) trace.damage_exit_no_weapon, (unsigned long long) trace.damage_exit_invulnerable,
		(unsigned long long) trace.damage_exit_done, trace.damage_entries == exits ? 1 : 0, (unsigned long long) trace.damage_written, (unsigned long long) trace.damage_unwritten);
	emit (&out, "cmb-explosion invocations=%llu max_blast_calls=%llu with_repeated_blast=%llu blast_outside_invocation=%llu\n", (unsigned long long) trace.explosions,
		(unsigned long long) trace.explosion_blast_calls_max, (unsigned long long) trace.explosions_with_repeated_blast, (unsigned long long) trace.blast_outside_explosion);
	emit (&out, "cmb-blast calls=%llu zero_radius=%llu candidates=%llu max_candidates=%llu\n", (unsigned long long) trace.blast_calls, (unsigned long long) trace.blast_zero_radius,
		(unsigned long long) trace.blast_candidates, (unsigned long long) trace.blast_max_candidates);

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

		if (trace.unobserved_deliveries[t])
		{
			emit (&out, "cmb-unobserved type=%s deliveries=%llu note=counted_only_no_damage_level_or_no_modifier_argument\n", type_name (t), (unsigned long long) trace.unobserved_deliveries[t]);
		}
	}

	for (t = 1; t < EECH_CMB_TIMEDS; t++)
	{
		emit (&out, "cmb-timed kind=%d attempts=%llu draws=%llu hits=%llu fatal=%llu\n", t, (unsigned long long) trace.timed[t].attempts, (unsigned long long) trace.timed[t].draws,
			(unsigned long long) trace.timed[t].hits, (unsigned long long) trace.timed[t].fatal);
	}

	emit (&out, "cmb-install wrapped_collision_handlers=%llu observed_deliveries=%llu\n", (unsigned long long) trace.installed, (unsigned long long) trace.observed_deliveries);
	emit (&out, "cmb-repair resupply=%llu fixed_restore=%llu other=%llu zero_to_positive=%llu\n", (unsigned long long) trace.repair_writes[EECH_CMB_REPAIR_RESUPPLY],
		(unsigned long long) trace.repair_writes[EECH_CMB_REPAIR_FIXED_RESTORE], (unsigned long long) trace.repair_writes[0], (unsigned long long) trace.repair_zero_to_positive);
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
