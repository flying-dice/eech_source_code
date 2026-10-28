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
 * What is not here: which weapons and targets meet, the sounds a hit makes, the setting of the health and its side effects, the credit and the kill, the player-setting table, the
 * multiplayer multiplier's record and the debug display of a hit (the campaign's; it sends the settings as values), and the frame's timed health rolls, the explosion and its blast
 * walk and the bridge, which are the original campaign code in this step.  The only random sample a hit takes, the original's `frand1 ()`, is the campaign's: this step asks for it
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
