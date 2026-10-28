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
		case 27: return sizeof (campaign_health_request);
		case 28: return sizeof (campaign_health_result);
		case 29: return offsetof (campaign_aim_step, health);
		case 30: return offsetof (campaign_aim_result, health);
		case 31: return offsetof (campaign_health_request, delta_time);
		case 32: return offsetof (campaign_health_request, aircraft_class);
		case 33: return offsetof (campaign_health_request, target_not_ai);
		case 34: return offsetof (campaign_health_result, eligible);
		case 35: return offsetof (campaign_health_result, fatal);
		/* IMPACT and BLAST (EEDCS-46) */
		case 36: return sizeof (campaign_impact_request);
		case 37: return sizeof (campaign_impact_result);
		case 38: return sizeof (campaign_blast_entry);
		case 39: return sizeof (campaign_blast_request);
		case 40: return sizeof (campaign_blast_result);
		case 41: return offsetof (campaign_aim_step, impact);
		case 42: return offsetof (campaign_aim_step, blast);
		case 43: return offsetof (campaign_aim_result, impact);
		case 44: return offsetof (campaign_aim_result, blast);
		case 45: return offsetof (campaign_blast_request, window);
		case 46: return offsetof (campaign_blast_entry, position);
		case 47: return offsetof (campaign_blast_result, modifier);
		case 48: return offsetof (campaign_impact_request, position);
		case 49: return offsetof (campaign_blast_request, window_valid);
		/* BRIDGE (EEDCS-46, R5) */
		case 50: return sizeof (campaign_bridge_request);
		case 51: return sizeof (campaign_bridge_result);
		case 52: return offsetof (campaign_aim_step, bridge);
		case 53: return offsetof (campaign_aim_result, bridge);
		case 54: return offsetof (campaign_bridge_request, position);
		case 55: return offsetof (campaign_bridge_request, terrain_known);
		case 56: return offsetof (campaign_bridge_request, segment_length);
		case 57: return offsetof (campaign_bridge_request, window);
		case 58: return offsetof (campaign_bridge_result, dropped_y);
		case 59: return offsetof (campaign_bridge_result, handle);
		/* the campaign's enum values the reference host's kernel hard-codes: the thirteen kinds a blast examines and the weapon kill codes */
		case 60: return (size_t) ENTITY_TYPE_ANTI_AIRCRAFT;
		case 61: return (size_t) ENTITY_TYPE_CITY_BUILDING;
		case 62: return (size_t) ENTITY_TYPE_FIXED_WING;
		case 63: return (size_t) ENTITY_TYPE_HELICOPTER;
		case 64: return (size_t) ENTITY_TYPE_PERSON;
		case 65: return (size_t) ENTITY_TYPE_PYLON;
		case 66: return (size_t) ENTITY_TYPE_ROUTED_VEHICLE;
		case 67: return (size_t) ENTITY_TYPE_SCENIC;
		case 68: return (size_t) ENTITY_TYPE_SEGMENT;
		case 69: return (size_t) ENTITY_TYPE_SHIP_VEHICLE;
		case 70: return (size_t) ENTITY_TYPE_SITE;
		case 71: return (size_t) ENTITY_TYPE_SITE_UPDATABLE;
		case 72: return (size_t) ENTITY_TYPE_WEAPON;
		case 90: return (size_t) ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;
		case 91: return (size_t) ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;
		case 92: return (size_t) ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION;
		/* the bridge questions (campaign_bridge_op): the numbers the host speaks, which the reference host and the DC module name */
		case 93: return (size_t) CAMPAIGN_BRIDGE_KILL;
		case 94: return (size_t) CAMPAIGN_BRIDGE_SHAPE;
		case 95: return (size_t) CAMPAIGN_BRIDGE_NEIGHBOUR;
		case 96: return (size_t) CAMPAIGN_BRIDGE_FALL;
		case 97: return (size_t) CAMPAIGN_BRIDGE_VEHICLES;
		case 98: return (size_t) CAMPAIGN_BRIDGE_REST;
		case 80: return (size_t) WEAPON_KILL_CODE_OK;
		case 81: return (size_t) WEAPON_KILL_CODE_EXHAUSTED;
		case 82: return (size_t) WEAPON_KILL_CODE_OUT_OF_BOUNDS;
		case 83: return (size_t) WEAPON_KILL_CODE_HIT_LAND;
		case 84: return (size_t) WEAPON_KILL_CODE_HIT_WATER;
		case 85: return (size_t) WEAPON_KILL_CODE_HIT_TARGET;
		case 86: return (size_t) WEAPON_KILL_CODE_OVERSHOT_TARGET;
		case 87: return (size_t) WEAPON_KILL_CODE_SELF_DESTRUCT;
	}

	return (size_t) -1;
}
