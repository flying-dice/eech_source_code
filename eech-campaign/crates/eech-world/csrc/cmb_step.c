/*
 * The physical damage arithmetic of EECH's weapons (EEDCS-46, D2): the damage a weapon is capable of against a target, with no weapon in flight (the FORECAST the target
 * selection asks) and with a real weapon (the DAMAGE of a hit that reached a damage handler), and what a hit does to the target's health.  It is EECH's own arithmetic, moved here
 * expression for expression from
 *   - aphavoc/source/entity/mobile/weapon/damage.c: weapon_damage_capability (both paths: the conventional-munitions relationship, the armour of the target, the two
 *     high-explosive warhead branches, the integer truncations, the player's damage setting and the multiplayer multiplier applied last) and damage_client_server_entity (the
 *     player's invulnerability, the cheat guard, the subtraction and the clamp at zero);
 *   - aphavoc/source/entity/mobile/weapon/wn_cllsn.c: get_local_entity_armour_thickness (the easiest-difficulty armour, the table level and the aspect armour of a vehicle or an
 *     aircraft, from the impact vector in the target's frame);
 *   - aphavoc/source/entity/mobile/vehicle/vehicle.c and aircraft/aircraft.c: the critical-state classification (vehicle_critically_damaged, aircraft_critically_damaged);
 *   - modules/maths/matrix.c: multiply_matrix3x3_vec3d;
 * so the host and the original reference produce the same bits.  Neither a rounding mode nor a type is chosen here: the engine runs a host callback under its own floating point
 * mode and every float and double expression below is the engine's own, in its own order (`min`, `max`, `pow`, `fabs` and the casts are the engine's).
 *
 * It also answers the HEALTH questions that are not a hit (aphavoc/source/entity/mobile/vehicle/vehicle.c, aircraft/aircraft.c and the six timed rolls of the vehicle and aircraft
 * updates, expression for expression): the critical class of a health, what a change of health crosses (assess_vehicle_damage_level, assess_aircraft_damage_level: the float
 * percentages), the frame's timed fire/bleeding/degradation roll (its eligibility, the campaign's one sample, the hit and the health it leaves) and the helicopter's refusal of its
 * own hit.
 *
 * It also answers what a weapon's DEATH does to the world (aphavoc/source/entity/special/effect/explosn/explosn.c, expression for expression): the dispatch on its kill code (no blast for\n * a weapon that ran out, left the map or hit water; a land hit's crater where the terrain takes one and its deliberate TWO blasts; one blast for an overshot or self-destroyed weapon; a hit on\n * the target's direct hit and one blast that leaves it out) and the area blast (the damage radius, the rectangle of sectors it covers bounded to the map, the walk over the sectors and over each\n * sector's list in order, the candidates, the range and the strict in-range test, the modifier, the rocket interception).  The blast is asked one object at a time: the campaign supplies\n * the windows of the sectors' lists and acts on each object this host reaches BEFORE the walk continues from the list as it is then.\n *\n * What is not here: which weapons and targets meet, the sounds a hit makes, the setting of the health and its side effects, the credit and the kill, the player-setting table, the
 * multiplayer multiplier's record and the debug display of a hit (the campaign's; it sends the settings as values), the observer's hooks and the kill after a timed hit, the
 * effects of an explosion and the notification each object receives (the campaign's), and the bridge (the original campaign code until its own step).  Which target rolls is THIS file's: the reference eligibility of a timed roll
 * (never a person, an aircraft in a class above none, a vehicle with health that can burn in the critical class) lives here only; the campaign validates the shape of the exchange.  The only random sample a hit takes, the original's `frand1 ()`, is the campaign's: this step asks for it
 * (NEED_DRAW) and is given the raw integer, from which it forms the double with the macro's own expression.  A forecast takes none.
 *
 * The functions are pure: they keep nothing between questions.
 */
#include <stddef.h>

#include "eech_arith.h"

#define TRUE 1
#define FALSE 0

/* the warhead types of the weapon database, as the figures carry them */
#define CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_TANK 4
#define CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_AIRCRAFT 7
#define CF_WARHEAD_CONVENTIONAL_MUNITIONS 8

/* how the armour of a target is modelled (campaign_damage_request armour_model) */
#define CF_ARMOUR_PLAIN 0
#define CF_ARMOUR_VEHICLE 1
#define CF_ARMOUR_AIRCRAFT 2

/* how the critical state of a target is classified (campaign_damage_request critical_model) */
#define CF_CRITICAL_NONE 0
#define CF_CRITICAL_VEHICLE 1
#define CF_CRITICAL_PERSON 2
#define CF_CRITICAL_AIRCRAFT 3

#define CF_VEHICLE_CRITICAL_DAMAGE_LEVEL 15
#define CF_AIRCRAFT_MORTAL_DAMAGE_LEVEL 30
#define CF_AIRCRAFT_CRITICAL_DAMAGE_LEVEL 5

/* what the original's observer hooks of a capability saw, in the values they were given */
typedef struct cf_trace
{
	int path;
	int raw;
	int warhead;
	float damage_modifier;
	float penetration;
	float armour;
	float random_modifier;
} cf_trace;

/* an integer conversion the original makes with `(int)`: defined only for a finite value that fits */
static int convertible(float value)
{
	return isfinite(value) && value > -2147483648.0f && value < 2147483648.0f;
}

/* The arithmetic the forecast and the real weapon share (weapon_damage_capability from the penetration to the capability before the player's setting).  `strict` refuses a value
   the original could not convert to an integer (its result there is undefined: the forecast does), where the real weapon performs the conversion as the original does.
   Returns the capability before the player's setting, or sets *failed. */
static int capability_arithmetic(int warhead_type, float damage_capability, float armor_penetration_capability, float target_armor_thickness, float random_modifier, float velocity,
	int high_explosive, float damage_modifier, int strict, cf_trace *trace, int *failed)
{
	int result = 0;
	float weapon_armor_penetration_capability, armor_modifier, value;

	trace->path = 0;

	weapon_armor_penetration_capability = armor_penetration_capability * random_modifier * velocity;

	if (weapon_armor_penetration_capability >= target_armor_thickness && damage_modifier < 0.01) // weapon pierces through the armor
	{
		value = damage_capability * random_modifier * velocity;

		if (strict && !convertible(value))
		{
			*failed = 1;

			return 0;
		}

		result = (int)value;
		trace->path = 1;
	}
	else if (high_explosive)
	{
		float he_warheads = CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_AIRCRAFT - CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_TANK; // subtract 1

		damage_modifier = 1 - pow(damage_modifier, pow((float)(warhead_type - CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_TANK + 1) / he_warheads, 1.5f));
		armor_modifier = min(1.0f, damage_capability * damage_modifier * random_modifier / pow(target_armor_thickness, 1.5f));

		value = damage_capability * damage_modifier * armor_modifier * random_modifier;

		if (strict && !convertible(value))
		{
			*failed = 1;

			return 0;
		}

		result = (int)value;
		trace->path = 2;
	}

	trace->raw = result;
	trace->warhead = warhead_type;
	trace->damage_modifier = damage_modifier;
	trace->penetration = weapon_armor_penetration_capability;
	trace->armour = target_armor_thickness;
	trace->random_modifier = random_modifier;

	return result;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the forecast */

typedef struct cf_forecast_in
{
	int weapon_warhead;
	int warhead;
	int damage_capability;
	int armor_penetration_capability;
	float burst_duration;
	int armour_level;
	int not_ai;
	float player_modifier;
	float damage_modifier;
	float velocity;
} cf_forecast_in;

/* the capability, and the original observer's values of the call (the fixture build replays them): the arithmetic path (0 none, 1 pierces, 2 splash), the capability before the
   player's setting, the warhead the arithmetic used, the damage modifier as it stood at the end, the penetration capability, the armour thickness and the random modifier.
   `failed` is nonzero when an intermediate value was not a number the original could convert to an integer (the original's result there is undefined). */
typedef struct cf_forecast_out
{
	int capability;
	int path;
	int raw;
	int warhead;
	float damage_modifier;
	float penetration;
	float armour;
	float random_modifier;
	int failed;
} cf_forecast_out;

void eech_cmb_forecast(const cf_forecast_in *in, cf_forecast_out *out)
{
	int result, high_explosive = FALSE, warhead_type, failed = 0;

	float random_modifier = 1, target_armor_thickness, damage_capability, armor_penetration_capability, velocity = in->velocity, value;

	cf_trace trace;

	memset(out, 0, sizeof(*out));
	memset(&trace, 0, sizeof(trace));

	switch (in->weapon_warhead)
	{
		case CF_WARHEAD_CONVENTIONAL_MUNITIONS:
			warhead_type = in->warhead;
			damage_capability = in->damage_capability * in->burst_duration / 3;
			armor_penetration_capability = in->armor_penetration_capability;
			break;
		default:
			warhead_type = in->warhead;
			damage_capability = in->damage_capability;
			armor_penetration_capability = in->armor_penetration_capability;
	}

	target_armor_thickness = (float)in->armour_level;

	if (warhead_type >= CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_TANK && warhead_type <= CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_AIRCRAFT)
	{
		velocity = 1;
		high_explosive = TRUE;
	}

	result = capability_arithmetic(warhead_type, damage_capability, armor_penetration_capability, target_armor_thickness, random_modifier, velocity, high_explosive,
		in->damage_modifier, 1, &trace, &failed);

	if (failed)
	{
		out->failed = 1;

		return;
	}

	if (in->not_ai)
	{
		value = result * in->player_modifier;

		if (!convertible(value))
		{
			out->failed = 1;

			return;
		}

		result = (int)value;
	}

	out->capability = result;
	out->path = trace.path;
	out->raw = trace.raw;
	out->warhead = trace.warhead;
	out->damage_modifier = trace.damage_modifier;
	out->penetration = trace.penetration;
	out->armour = trace.armour;
	out->random_modifier = trace.random_modifier;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the damage of a hit */

typedef struct cf_damage_in
{
	/* the weapon: the figures the capability is made from (as the forecast's), its speed and its own definition's cruise velocity */
	int weapon_warhead;
	int warhead;
	int damage_capability;
	int armor_penetration_capability;
	float burst_duration;
	float weapon_speed;
	float weapon_cruise_velocity;
	float damage_modifier;
	/* the armour of the target */
	int armour_level;
	int armour_easy;
	int armour_model;
	float armour_front;
	float armour_rear;
	float armour_side;
	float weapon_position[3];
	float target_position[3];
	float target_attitude[9];
	/* the settings */
	int not_ai;
	int player_modifier_valid;
	float player_modifier;
	int has_launcher;
	int multiplier;
	int invulnerable;
	int cheat_guard;
	/* the target's health */
	int damage_level;
	int initial_damage_level;
	int critical_model;
	/* the sample of the campaign's stream the arithmetic asked for, when it has been given */
	int has_draw;
	int draw;
} cf_damage_in;

/* state: 0 done, 1 the arithmetic needs its one sample (and nothing else of the answer is valid); failed nonzero when the question has no answer (a critical classification
   with no initial level to divide by) */
typedef struct cf_damage_out
{
	int state;
	int exit;
	int capability;
	int written;
	int new_level;
	int critical_before;
	int critical_after;
	int bad_modifier;
	int failed;
	cf_trace trace;
} cf_damage_out;

/* the critical-state classification of vehicle_critically_damaged and aircraft_critically_damaged: a vehicle is critical (1) below 15 percent of its initial health and a
   person never is; an aircraft is critical (2) below 5 percent and mortal (1) below 30.  The percentage is the original's integer arithmetic. */
static int classify(int critical_model, int damage_level, int initial_damage_level)
{
	int percent_damaged;

	switch (critical_model)
	{
		case CF_CRITICAL_VEHICLE:
			percent_damaged = 100 * damage_level / initial_damage_level;

			return percent_damaged < CF_VEHICLE_CRITICAL_DAMAGE_LEVEL ? 1 : 0;
		case CF_CRITICAL_AIRCRAFT:
			percent_damaged = (100 * damage_level) / initial_damage_level;

			if (percent_damaged < CF_AIRCRAFT_CRITICAL_DAMAGE_LEVEL)
			{
				return 2;
			}
			else if (percent_damaged < CF_AIRCRAFT_MORTAL_DAMAGE_LEVEL)
			{
				return 1;
			}

			return 0;
		default:
			return 0;
	}
}

/* get_local_entity_armour_thickness */
static float armour_thickness(const cf_damage_in *in)
{
	float armor_thickness, result = 0;

	// the launcher is a player at the easiest difficulty
	if (in->armour_easy)
	{
		return (0.1);
	}

	armor_thickness = (float)in->armour_level;

	if (in->armour_model == CF_ARMOUR_VEHICLE || in->armour_model == CF_ARMOUR_AIRCRAFT)
	{
		float front_armor, rear_armor, side_armor, top_armor, norm;
		vec3d impact_vector, temp;

		if (in->armour_model == CF_ARMOUR_VEHICLE)
		{
			front_armor = in->armour_front;
			rear_armor = in->armour_rear;
			side_armor = in->armour_side;
			top_armor = (rear_armor + side_armor) / 3.0;
		}
		else
		{
			front_armor = 1;
			rear_armor = 0.25;
			side_armor = 0.75;
			top_armor = 0.33;
		}

		impact_vector.x = in->weapon_position[0] - in->target_position[0];
		impact_vector.y = in->weapon_position[1] - in->target_position[1];
		impact_vector.z = in->weapon_position[2] - in->target_position[2];

		// multiply_matrix3x3_vec3d (&impact_vector, *target_attitude, &impact_vector)
		temp.x = impact_vector.x * in->target_attitude[0];
		temp.x += impact_vector.y * in->target_attitude[3];
		temp.x += impact_vector.z * in->target_attitude[6];

		temp.y = impact_vector.x * in->target_attitude[1];
		temp.y += impact_vector.y * in->target_attitude[4];
		temp.y += impact_vector.z * in->target_attitude[7];

		temp.z = impact_vector.x * in->target_attitude[2];
		temp.z += impact_vector.y * in->target_attitude[5];
		temp.z += impact_vector.z * in->target_attitude[8];

		impact_vector.x = temp.x;
		impact_vector.y = temp.y;
		impact_vector.z = temp.z;

		norm = fabs(impact_vector.x) + fabs(impact_vector.y) + fabs(impact_vector.z);

		impact_vector.x /= norm;
		impact_vector.y /= norm;
		impact_vector.z /= norm;

		if (impact_vector.x >= 0)
			result += side_armor * impact_vector.x;
		else
			result -= side_armor * impact_vector.x;

		if (impact_vector.y >= 0)
			result += top_armor * impact_vector.y;
		else
			result -= top_armor * impact_vector.y;

		if (impact_vector.z >= 0)
			result += front_armor * impact_vector.z;
		else
			result -= rear_armor * impact_vector.z;

		armor_thickness *= result;
	}

	return armor_thickness;
}

void eech_cmb_damage(const cf_damage_in *in, cf_damage_out *out)
{
	int result, high_explosive = FALSE, warhead_type, failed = 0, damage_level;

	float random_modifier, target_armor_thickness, damage_capability, armor_penetration_capability, velocity = 1;

	memset(out, 0, sizeof(*out));

	if ((in->critical_model == CF_CRITICAL_VEHICLE || in->critical_model == CF_CRITICAL_AIRCRAFT) && in->initial_damage_level <= 0)
	{
		out->failed = 1;

		return;
	}

	out->new_level = in->damage_level;
	out->critical_before = classify(in->critical_model, in->damage_level, in->initial_damage_level);
	out->critical_after = out->critical_before;

	// invulnerability option for players: a hit does nothing and takes no sample
	if (in->not_ai && in->invulnerable)
	{
		out->exit = 1;

		return;
	}

	// the arithmetic's one sample, which is the campaign's
	if (!in->has_draw)
	{
		out->state = 1;

		return;
	}

	switch (in->weapon_warhead)
	{
		case CF_WARHEAD_CONVENTIONAL_MUNITIONS:
			warhead_type = in->warhead;
			damage_capability = in->damage_capability * in->burst_duration / 3;
			armor_penetration_capability = in->armor_penetration_capability;
			break;
		default:
			warhead_type = in->warhead;
			damage_capability = in->damage_capability;
			armor_penetration_capability = in->armor_penetration_capability;
	}

	target_armor_thickness = armour_thickness(in);
	random_modifier = ((double)in->draw) * (1.0 / 32767.0); // frand1 ()

	if (warhead_type >= CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_TANK && warhead_type <= CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_AIRCRAFT)
	{
		velocity = 1;
		high_explosive = TRUE;
		random_modifier = 0.75 + 2 * random_modifier * random_modifier * random_modifier; // random chance from 75% to 275%, average 100%
	}
	else
	{
		velocity = in->weapon_speed / in->weapon_cruise_velocity;
		random_modifier = 0.75 + 0.5 * random_modifier; // random chance from 75% to 125%, average 100%
	}

	result = capability_arithmetic(warhead_type, damage_capability, armor_penetration_capability, target_armor_thickness, random_modifier, velocity, high_explosive,
		in->damage_modifier, 0, &out->trace, &failed);

	if (in->not_ai)
	{
		if (!in->player_modifier_valid)
		{
			out->bad_modifier = 1;

			return;
		}

		result *= in->player_modifier;
	}

	if (in->has_launcher) // damage multiplier for clients
		result *= in->multiplier;

	out->capability = result;

	if (result > 0 && !in->cheat_guard)
	{
		damage_level = in->damage_level;

		damage_level -= result;

		damage_level = max(damage_level, 0);

		out->written = 1;
		out->new_level = damage_level;
		out->critical_after = classify(in->critical_model, damage_level, in->initial_damage_level);
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* health that is not a hit */

#define CF_HEALTH_CLASSIFY 1
#define CF_HEALTH_ASSESS 2
#define CF_HEALTH_TIMED 3
#define CF_HEALTH_HIT_ELIGIBLE 4

typedef struct cf_health_in
{
	int op;
	int critical_model;
	int damage_level;
	int initial_damage_level;
	int old_damage_level;
	int explosive;
	int flammable;
	float delta_time;
	int aircraft_class;
	int self_hit;
	int target_not_ai;
	/* the sample of the campaign's stream a timed roll asked for, when it has been given */
	int has_draw;
	int draw;
} cf_health_in;

/* state: 1 the roll needs its one sample; failed nonzero when the question has no answer (a classified health with no initial level) */
typedef struct cf_health_out
{
	int state;
	int critical_class;
	int crossed;
	int recovered;
	int eject;
	int eligible;
	int rolled;
	int hit;
	int new_level;
	int fatal;
	int failed;
} cf_health_out;

void eech_cmb_health(const cf_health_in *in, cf_health_out *out)
{
	memset(out, 0, sizeof(*out));
	out->new_level = in->damage_level;

	if ((in->critical_model == CF_CRITICAL_VEHICLE || in->critical_model == CF_CRITICAL_AIRCRAFT) && in->initial_damage_level <= 0 && in->op != CF_HEALTH_HIT_ELIGIBLE)
	{
		out->failed = 1;

		return;
	}

	switch (in->op)
	{
		case CF_HEALTH_CLASSIFY:
			out->critical_class = classify(in->critical_model, in->damage_level, in->initial_damage_level);
			break;

		case CF_HEALTH_ASSESS:
		{
			// assess_vehicle_damage_level and assess_aircraft_damage_level: the float percentages of the original
			float initial_damage, old_percent_damaged, new_percent_damaged;

			if (in->initial_damage_level <= 0)
			{
				out->failed = 1;

				return;
			}

			initial_damage = (float)(in->initial_damage_level);

			old_percent_damaged = (100.0 * (float)in->old_damage_level) / initial_damage;
			new_percent_damaged = (100.0 * (float)in->damage_level) / initial_damage;

			if (in->critical_model == CF_CRITICAL_AIRCRAFT)
			{
				out->eject = (new_percent_damaged < 0.75 * CF_AIRCRAFT_CRITICAL_DAMAGE_LEVEL && in->damage_level > 10) ? 1 : 0;

				out->crossed = ((new_percent_damaged < CF_AIRCRAFT_CRITICAL_DAMAGE_LEVEL && old_percent_damaged >= CF_AIRCRAFT_CRITICAL_DAMAGE_LEVEL) ||
					(new_percent_damaged < CF_AIRCRAFT_MORTAL_DAMAGE_LEVEL && old_percent_damaged >= CF_AIRCRAFT_MORTAL_DAMAGE_LEVEL)) ? 1 : 0;

				out->recovered = (new_percent_damaged >= CF_AIRCRAFT_MORTAL_DAMAGE_LEVEL && old_percent_damaged < CF_AIRCRAFT_MORTAL_DAMAGE_LEVEL) ? 1 : 0;
			}
			else if (in->explosive || in->flammable)
			{
				out->crossed = (new_percent_damaged < CF_VEHICLE_CRITICAL_DAMAGE_LEVEL && old_percent_damaged >= CF_VEHICLE_CRITICAL_DAMAGE_LEVEL) ? 1 : 0;
			}

			break;
		}

		case CF_HEALTH_TIMED:
		{
			// the frame's timed roll, in the original's short-circuit order: a vehicle that can burn burns when it has health and is critical, a person would bleed (never: a person is
			// never critical), an aircraft is degraded when the class the update computed earlier is above none; only an eligible target takes the sample, whatever the odds
			int eligible, hit = 0, level = in->damage_level;
			double roll;

			if (in->critical_model == CF_CRITICAL_AIRCRAFT)
			{
				eligible = in->aircraft_class > 0;
			}
			else
			{
				eligible = in->damage_level && (in->critical_model == CF_CRITICAL_PERSON || in->explosive || in->flammable) &&
					classify(in->critical_model, in->damage_level, in->initial_damage_level);
			}

			out->eligible = eligible ? 1 : 0;

			if (eligible)
			{
				if (!in->has_draw)
				{
					out->state = 1;

					return;
				}

				roll = ((double)in->draw) * (1.0 / 32767.0); // frand1 ()

				if (in->critical_model == CF_CRITICAL_AIRCRAFT)
				{
					hit = roll < pow((float)in->aircraft_class, 4) * in->delta_time;
				}
				else
				{
					hit = roll < 2 * in->delta_time;
				}

				out->rolled = 1;
			}

			if (hit)
			{
				level = max(0, in->damage_level - 1);

				out->hit = 1;
				out->new_level = level;
				out->fatal = (in->damage_level - 1 <= 0) ? 1 : 0;
			}

			out->critical_class = classify(in->critical_model, level, in->initial_damage_level);

			break;
		}

		case CF_HEALTH_HIT_ELIGIBLE:
			// a helicopter that is an AI ignores its own hit (the original's fix against helicopters killing themselves with missiles fired into the ground)
			out->eligible = (in->self_hit && !in->target_not_ai) ? 0 : 1;
			break;

		default:
			out->failed = 1;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the impact of a weapon's death and the area blast (EEDCS-46) */

/* the campaign's weapon kill codes (weapon.h WEAPON_KILL_CODES) and entity types (en_types.h ENTITY_TYPES): the layout probe of the engine's tests reads them from the campaign as
   compiled and compares them with these */
#define CF_KILL_EXHAUSTED 1
#define CF_KILL_OUT_OF_BOUNDS 2
#define CF_KILL_HIT_LAND 3
#define CF_KILL_HIT_WATER 4
#define CF_KILL_HIT_TARGET 5
#define CF_KILL_OVERSHOT_TARGET 6
#define CF_KILL_SELF_DESTRUCT 7

#define CF_ET_ANTI_AIRCRAFT 1
#define CF_ET_CITY_BUILDING 6
#define CF_ET_FIXED_WING 10
#define CF_ET_HELICOPTER 14
#define CF_ET_PERSON 19
#define CF_ET_PYLON 21
#define CF_ET_ROUTED_VEHICLE 23
#define CF_ET_SCENIC 24
#define CF_ET_SEGMENT 26
#define CF_ET_SHIP_VEHICLE 28
#define CF_ET_SITE 29
#define CF_ET_SITE_UPDATABLE 30
#define CF_ET_WEAPON 37

#define CF_BLAST_WINDOW_MAX 16

typedef struct cf_impact_in
{
	int kill_code;
	int has_target;
	int target_alive;
	int terrain_known;
	int terrain_draws_crater;
} cf_impact_in;

typedef struct cf_impact_out
{
	int need_terrain;
	int invalid;
	int crater;
	int direct_hit;
	int blasts;
	int exclude_target;
} cf_impact_out;

/* The dispatch of create_client_server_weapon_killed_explosion_effect on the weapon's kill code, in the original's order (the crater, the direct hit, the blasts): no blast for an
   exhausted weapon, one that left the map or one that hit water; a land hit makes a crater where the terrain takes one and blasts TWICE (the original's deliberate fall-through); an
   overshot or self-destroyed weapon blasts once; a hit on the target sends the target a direct hit if it is there and alive and then blasts once without it; any other code is not
   known.  Only a land hit needs the terrain, and it is asked for once. */
void eech_cmb_impact(const cf_impact_in *in, cf_impact_out *out)
{
	memset(out, 0, sizeof(*out));

	switch (in->kill_code)
	{
		case CF_KILL_EXHAUSTED:
		case CF_KILL_OUT_OF_BOUNDS:
		case CF_KILL_HIT_WATER:
			break;

		case CF_KILL_HIT_LAND:
			if (!in->terrain_known)
			{
				out->need_terrain = 1;

				return;
			}

			out->crater = in->terrain_draws_crater ? 1 : 0;
			out->blasts = 2;
			break;

		case CF_KILL_OVERSHOT_TARGET:
		case CF_KILL_SELF_DESTRUCT:
			out->blasts = 1;
			break;

		case CF_KILL_HIT_TARGET:
			out->direct_hit = (in->has_target && in->target_alive) ? 1 : 0;
			out->blasts = 1;
			out->exclude_target = in->has_target ? 1 : 0;
			break;

		default:
			out->invalid = 1;
	}
}

typedef struct cf_blast_entry
{
	int handle;
	int type;
	int readable;
	int alive;
	int has_position;
	int rocket;
	int excluded;
	float position[3];
} cf_blast_entry;

typedef struct cf_blast_in
{
	float position[3];
	int damage_capability;
	int warhead_type;
	int has_target;
	int sector_side_length;
	int min_x_sector;
	int max_x_sector;
	int min_z_sector;
	int max_z_sector;
	int window_valid;
	int window_sx;
	int window_sz;
	int window_after;
	int window_count;
	int window_end;
	cf_blast_entry window[CF_BLAST_WINDOW_MAX];
} cf_blast_in;

/* one object examined as a candidate: its index in the window, the flags (1 strictly inside the radius, 2 a rocket destroyed), the range, the radius and the modifier */
typedef struct cf_blast_echo
{
	int index;
	int flags;
	float range;
	float radius;
	float modifier;
} cf_blast_echo;

/* state: 0 the answer is final (`action`: 0 the walk is over, 1 a collision, 2 a rocket destroyed), 1 the host needs the window at (need_sx, need_sz) after `need_after`; failed nonzero when
   the question has no answer */
typedef struct cf_blast_out
{
	int state;
	int failed;
	float radius;
	int need_sx;
	int need_sz;
	int need_after;
	int action;
	int handle;
	float modifier;
	int echo_count;
	cf_blast_echo echo[CF_BLAST_WINDOW_MAX];
} cf_blast_out;

static int blast_examines(int type)
{
	switch (type)
	{
		case CF_ET_ANTI_AIRCRAFT:
		case CF_ET_CITY_BUILDING:
		case CF_ET_FIXED_WING:
		case CF_ET_HELICOPTER:
		case CF_ET_PERSON:
		case CF_ET_PYLON:
		case CF_ET_ROUTED_VEHICLE:
		case CF_ET_SCENIC:
		case CF_ET_SEGMENT:
		case CF_ET_SHIP_VEHICLE:
		case CF_ET_SITE:
		case CF_ET_SITE_UPDATABLE:
		case CF_ET_WEAPON:
			return 1;

		default:
			return 0;
	}
}

/* modules/maths/range.c get_3d_range */
static float blast_range(const float *v1, const float *v2)
{
	float dx, dy, dz, range;

	dx = v1[0] - v2[0];
	dy = v1[1] - v2[1];
	dz = v1[2] - v2[2];

	range = sqrt((dx * dx) + (dy * dy) + (dz * dz));

	return range;
}

/* One question of an area blast (damage_objects_within_explosion_damage_radius, expression for expression): the damage radius, the rectangle of sectors the radius covers bounded to the
   map's, the walk over the sectors (rows, each from its first sector) and over each sector's list in order, the target left out, the kinds that are examined, an object that is alive
   and has a position, its range, the strict in-range test, the modifier, and the interception of a rocket.  It asks for the window of the first sector, examines the window it is given
   up to the first object that needs the campaign to act (a collision, or a rocket's destruction), and otherwise asks for the next window: the rest of the list the window did not hold, or
   the first window of the next sector, or ends the walk.  It keeps nothing between questions: the window carries where the walk is. */
void eech_cmb_blast(const cf_blast_in *in, cf_blast_out *out)
{
	float damage_radius, temp_x, temp_z;
	int min_sector_x, max_sector_x, min_sector_z, max_sector_z, i;

	memset(out, 0, sizeof(*out));

	damage_radius = sqrt((float)in->damage_capability);
	out->radius = damage_radius;

	if (damage_radius == 0.0)
	{
		return;
	}

	temp_x = in->position[0] - damage_radius;
	temp_z = in->position[2] - damage_radius;

	if (!convertible(temp_x) || !convertible(temp_z) || in->sector_side_length <= 0)
	{
		out->failed = 1;

		return;
	}

	min_sector_x = (int)temp_x;
	min_sector_x /= in->sector_side_length;
	min_sector_z = (int)temp_z;
	min_sector_z /= in->sector_side_length;

	temp_x = in->position[0] + damage_radius;
	temp_z = in->position[2] + damage_radius;

	if (!convertible(temp_x) || !convertible(temp_z))
	{
		out->failed = 1;

		return;
	}

	max_sector_x = (int)temp_x;
	max_sector_x /= in->sector_side_length;
	max_sector_z = (int)temp_z;
	max_sector_z /= in->sector_side_length;

	min_sector_x = bound(min_sector_x, in->min_x_sector, in->max_x_sector);
	max_sector_x = bound(max_sector_x, in->min_x_sector, in->max_x_sector);

	min_sector_z = bound(min_sector_z, in->min_z_sector, in->max_z_sector);
	max_sector_z = bound(max_sector_z, in->min_z_sector, in->max_z_sector);

	if (min_sector_x > max_sector_x || min_sector_z > max_sector_z)
	{
		return;
	}

	if (!in->window_valid)
	{
		out->state = 1;
		out->need_sx = min_sector_x;
		out->need_sz = min_sector_z;
		out->need_after = 0;

		return;
	}

	if (in->window_sx < min_sector_x || in->window_sx > max_sector_x || in->window_sz < min_sector_z || in->window_sz > max_sector_z || in->window_count < 0 ||
		in->window_count > CF_BLAST_WINDOW_MAX || (!in->window_end && in->window_count == 0))
	{
		out->failed = 1;

		return;
	}

	for (i = 0; i < in->window_count; i++)
	{
		const cf_blast_entry *object = &in->window[i];
		float range, damage_modifier;
		int in_range;

		if (object->excluded) // skip target, as it is dealt with elsewhere
		{
			continue;
		}

		if (!blast_examines(object->type) || !object->readable || !object->alive || !object->has_position)
		{
			continue;
		}

		range = blast_range(in->position, object->position);

		in_range = (range && (range < damage_radius)) ? 1 : 0;

		out->echo[out->echo_count].index = i;
		out->echo[out->echo_count].flags = in_range;
		out->echo[out->echo_count].range = range;
		out->echo[out->echo_count].radius = damage_radius;
		out->echo[out->echo_count].modifier = 0.0f;

		if (in_range)
		{
			damage_modifier = range / damage_radius; // less value - more damage!

			out->echo[out->echo_count].modifier = damage_modifier;

			if (object->type != CF_ET_WEAPON)
			{
				out->echo_count++;
				out->action = 1;
				out->handle = object->handle;
				out->modifier = damage_modifier;

				return;
			}

			if (1 - pow(damage_modifier, pow((float)(in->warhead_type - 3) / 3, 1.5f)) > 0.75 && object->rocket)
			{
				out->echo[out->echo_count].flags = 3;
				out->echo_count++;
				out->action = 2;
				out->handle = object->handle;
				out->modifier = damage_modifier;

				return;
			}
		}

		out->echo_count++;
	}

	out->state = 1;

	if (!in->window_end)
	{
		// the rest of this sector's list, after the last object the window held
		out->need_sx = in->window_sx;
		out->need_sz = in->window_sz;
		out->need_after = in->window[in->window_count - 1].handle;

		return;
	}

	if (in->window_sx < max_sector_x)
	{
		out->need_sx = in->window_sx + 1;
		out->need_sz = in->window_sz;
		out->need_after = 0;

		return;
	}

	if (in->window_sz < max_sector_z)
	{
		out->need_sx = min_sector_x;
		out->need_sz = in->window_sz + 1;
		out->need_after = 0;

		return;
	}

	// the last sector's list has ended: the walk is over
	out->state = 0;
}
