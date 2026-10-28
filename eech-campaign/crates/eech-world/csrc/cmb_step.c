/*
 * The physical damage FORECAST of EECH's weapons (EEDCS-46, D2): the damage a weapon is capable of against a target with no weapon in flight, which the target selection
 * (wn_tgt.c) asks to decide whether a package can hurt a target at all and to weigh packages against each other.  It is EECH's own arithmetic, moved here expression for
 * expression from
 *   - aphavoc/source/entity/mobile/weapon/damage.c: weapon_damage_capability, the `wpn == NULL` path (the conventional-munitions relationship, the armour of the target as its
 *     table level, the two high-explosive warhead branches, the integer truncations), and the player's damage setting applied to the result;
 * so the host and the original reference produce the same bits.  Neither a rounding mode nor a type is chosen here: the engine runs a host callback under its own floating
 * point mode and every float and double expression below is the engine's own, in its own order (`min`, `pow` and the casts are the engine's).
 *
 * What is not here: which weapon is weighed against which target, the salvo and rate-of-fire multiplier and the comparison (wn_tgt.c, the campaign's), the REAL weapon's
 * capability (its draw from the shared random stream, its velocity, the aspect armour of the target and the multiplayer multiplier), the reading of the target's armour level
 * and player setting (the campaign's entity), and the database relationship that a conventional-munitions weapon carries its damage in the next database entry (the campaign
 * sends the munition's figures).  A forecast takes NO sample of any random stream.
 *
 * The function is pure: it keeps nothing between questions.
 */
#include <stddef.h>

#include "eech_arith.h"

#define TRUE 1
#define FALSE 0

/* the warhead types of the weapon database, as the figures carry them */
#define CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_TANK 4
#define CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_AIRCRAFT 7
#define CF_WARHEAD_CONVENTIONAL_MUNITIONS 8

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

/* an integer conversion the original makes with `(int)`: defined only for a finite value that fits */
static int convertible(float value)
{
	return isfinite(value) && value > -2147483648.0f && value < 2147483648.0f;
}

void eech_cmb_forecast(const cf_forecast_in *in, cf_forecast_out *out)
{
	int result = 0, high_explosive = FALSE, warhead_type;

	float weapon_armor_penetration_capability, random_modifier = 1, target_armor_thickness, armor_modifier, damage_capability, armor_penetration_capability, damage_modifier = in->damage_modifier,
		velocity = in->velocity, value;

	memset(out, 0, sizeof(*out));

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

	weapon_armor_penetration_capability = armor_penetration_capability * random_modifier * velocity;

	if (weapon_armor_penetration_capability >= target_armor_thickness && damage_modifier < 0.01) // weapon pierces through the armor
	{
		value = damage_capability * random_modifier * velocity;

		if (!convertible(value))
		{
			out->failed = 1;

			return;
		}

		result = (int)value;
		out->path = 1;
	}
	else if (high_explosive)
	{
		float he_warheads = CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_AIRCRAFT - CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_TANK; // subtract 1

		damage_modifier = 1 - pow(damage_modifier, pow((float)(warhead_type - CF_WARHEAD_HIGH_EXPLOSIVE_ANTI_TANK + 1) / he_warheads, 1.5f));
		armor_modifier = min(1.0f, damage_capability * damage_modifier * random_modifier / pow(target_armor_thickness, 1.5f));

		value = damage_capability * damage_modifier * armor_modifier * random_modifier;

		if (!convertible(value))
		{
			out->failed = 1;

			return;
		}

		result = (int)value;
		out->path = 2;
	}

	out->raw = result;

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
	out->warhead = warhead_type;
	out->damage_modifier = damage_modifier;
	out->penetration = weapon_armor_penetration_capability;
	out->armour = target_armor_thickness;
	out->random_modifier = random_modifier;
}
