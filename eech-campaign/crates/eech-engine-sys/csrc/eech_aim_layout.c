/*
 * The layout of the aim exchange as the campaign compiled it (mb_hostaim.h).  The Rust mirrors of campaign_aim_step, campaign_aim_result, campaign_weapon_figures and
 * campaign_damage_figures (eech-engine-sys src/lib.rs) are written by hand, field for field; a mirror that drifts from the C structure is memory corruption at the first request, so
 * the engine's tests read the sizes and the offsets of the fields that were appended last from here and compare them with the mirrors' (EEDCS-46).  Reads nothing, changes nothing.
 */
#include <stddef.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

size_t eech_engine_aim_layout (int which)
{
	switch (which)
	{
		case 0: return sizeof (campaign_aim_step);
		case 1: return sizeof (campaign_aim_result);
		case 2: return sizeof (campaign_weapon_figures);
		case 3: return sizeof (campaign_damage_figures);
		case 4: return offsetof (campaign_aim_step, release_not_ai);
		case 5: return offsetof (campaign_aim_step, animate_motion_y);
		case 6: return offsetof (campaign_aim_step, forecast_figures);
		case 7: return offsetof (campaign_aim_step, forecast_armour_level);
		case 8: return offsetof (campaign_aim_step, forecast_velocity);
		case 9: return offsetof (campaign_aim_result, release_last_shot);
		case 10: return offsetof (campaign_aim_result, animate_lifetime);
		case 11: return offsetof (campaign_aim_result, forecast_capability);
		case 12: return offsetof (campaign_damage_figures, burst_duration);
		case 13: return offsetof (campaign_aim_step, weapon_state);
		case 14: return offsetof (campaign_aim_step, answers);
		case 15: return sizeof (campaign_damage_request);
		case 16: return sizeof (campaign_damage_result);
		case 17: return offsetof (campaign_aim_step, damage);
		case 18: return offsetof (campaign_aim_result, damage);
		case 19: return offsetof (campaign_damage_request, weapon_speed);
		case 20: return offsetof (campaign_damage_request, armour_level);
		case 21: return offsetof (campaign_damage_request, armour_front);
		case 22: return offsetof (campaign_damage_request, target_attitude);
		case 23: return offsetof (campaign_damage_request, target_not_ai);
		case 24: return offsetof (campaign_damage_request, player_modifier);
		case 25: return offsetof (campaign_damage_request, critical_model);
		case 26: return offsetof (campaign_damage_result, critical_before);
	}

	return (size_t) -1;
}
