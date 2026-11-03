

































































#include "project.h"





#define DEBUG_MODULE 0





const char
	*entity_message_names [] =
	{
		"ADJUST_LAND_URGENCY",
		"ARTICULATE_LOADING_DOORS",
		"ARTICULATE_UNDERCARRIAGE",
		"CHECK_CAMPAIGN_OBJECTIVES",
		"CHECK_MOBILE_REACHED_GUIDE",
		"CHILD_KILLED",
		"CHILD_MOVE",
		"COLLISION",
		"DECOY_ACTIVE",
		"DECOY_EXPIRED",
		"DEPENDENT_CHILD_TASK_TERMINATED",
		"DEPENDENT_PARENT_TASK_TERMINATED",
		"ENEMY_ACTIVITY",
		"ENTITY_FIRED_AT",
		"ENTITY_TARGETED",
		"ESCORT_TASK_START",
		"ESCORT_TASK_STOP",
		"FORCE_ARMOUR_ADVANCING",
		"FORCE_ARMOUR_RESISTING",
		"FORCE_ARMOUR_RETREATING",
		"FORCE_ARMOUR_STUCK",
		"FORCE_DESTROYED",
		"FORCE_ENTERED_SECTOR",
		"FORCE_INEFFICIENT_KEYSITE",
		"FORCE_LOW_ON_SUPPLIES",
		"FORCE_SIDE_SURRENDERED",
		"FORCE_SPECIAL_KILL",
		"FORCE_VACANT_FARP",
		"FORCE_WAYPOINT_REACHED",
		"GROUND_FORCE_ADVANCE",
		"GROUND_FORCE_RETREAT",
		"GROUP_KILLED",
		"GROUP_TAKEN_OFF",
		"GUIDE_COVER_POSITION_REACHED",
		"LANDING_SITE_REQUEST",
		"LINK_CHILD",
		"LINK_PARENT",
		"LOCK_LANDING_ROUTE",
		"LOCK_LANDING_SITE",
		"LOCK_TAKEOFF_ROUTE",
		"MOBILE_KILLED",
		"PROTOTYPE_1",
		"PROTOTYPE_2",
		"PROTOTYPE_3",
		"PROTOTYPE_4",
		"REQUEST_ASSISTANCE",
		"RESERVE_LANDING_SITE",
		"SET_CAMERA_ACTION",
		"SET_ENTITY_POSITION",
   	"TASK_ASSIGNED",
		"TASK_COMPLETED",
		"TASK_CREATED",
   	"TASK_OBJECTIVE_MET",
   	"TASK_OBJECTIVE_FAILED",
   	"TASK_TERMINATED",
		"UNLINK_CHILD",
		"UNLINK_PARENT",
		"UNLOCK_LANDING_ROUTE",
		"UNLOCK_LANDING_SITE",
		"UNLOCK_TAKEOFF_ROUTE",
		"UNRESERVE_LANDING_SITE",
		"WAYPOINT_ACTION",
		"WAYPOINT_APPROACH_ACTION",
		"WAYPOINT_ATTACK_REACHED",
		"WAYPOINT_CONVOY_REACHED",
		"WAYPOINT_DEFEND_REACHED",
		"WAYPOINT_DROP_OFF_REACHED",
		"WAYPOINT_END_REACHED",
		"WAYPOINT_FINISH_DROP_OFF_REACHED",
		"WAYPOINT_FARP_RECON_REACHED",
		"WAYPOINT_LAND_REACHED",
		"WAYPOINT_LANDED_REACHED",
		"WAYPOINT_LIFT_OFF_REACHED",
		"WAYPOINT_LOOP_REACHED",
		"WAYPOINT_LOWER_UNDERCARRIAGE_ACTION",
		"WAYPOINT_NAVIGATION_REACHED",
		"WAYPOINT_PREPARE_FOR_DROP_OFF_REACHED",
		"WAYPOINT_PICK_UP_REACHED",
		"WAYPOINT_RAISE_UNDERCARRIAGE_REACHED",
		"WAYPOINT_REACHED",
		"WAYPOINT_RECON_ACTION",
		"WAYPOINT_RECON_REACHED",
		"WAYPOINT_REPAIR_REACHED",
		"WAYPOINT_REVERSE_CONVOY_REACHED",
		"WAYPOINT_START_UP_REACHED",
		"WAYPOINT_SUB_ROUTE_NAVIGATION_REACHED",
		"WAYPOINT_TAKEN_OFF_REACHED",
		"WAYPOINT_TAXI_REACHED",
		"WAYPOINT_TOUCH_DOWN_REACHED",
		"WAYPOINT_TROOP_CAPTURE_REACHED",
		"WAYPOINT_TROOP_DEFEND_REACHED",
		"WAYPOINT_TROOP_EXTRACT_REACHED",
		"WAYPOINT_TROOP_INSERT_REACHED",
		"WAYPOINT_TROOP_PICKUP_POINT_END_REACHED",
		"WAYPOINT_TROOP_PICKUP_POINT_START_REACHED",
		"WAYPOINT_TROOP_PUTDOWN_POINT_REACHED",
		"WAYPOINT_WAIT_REACHED",
		"WAYPOINT_PREPARE_FOR_INSERTION_REACHED",
		"NUM_ENTITY_MESSAGES"
	};





int notify_local_entity (entity_messages message, entity *receiver, entity *sender, ...)
{
	va_list
		pargs;

	int
		acknowledge;

	ASSERT ((message >= 0) && (message < NUM_ENTITY_MESSAGES));

	ASSERT (receiver);

	va_start (pargs, sender);

	acknowledge = message_responses[receiver->type][message] (message, receiver, sender, pargs);

	va_end (pargs);

	return (acknowledge);
}





int (*message_responses[NUM_ENTITY_TYPES][NUM_ENTITY_MESSAGES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);





static int default_message_response (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	return (FALSE);
}





void initialise_entity_message_responses (void)
{
	int
		i,
		j;

	//
	// set default message responses
	//

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		for (j = 0; j < NUM_ENTITY_MESSAGES; j++)
		{
			message_responses[i][j] = default_message_response;
		}
	}

	//
	// overload default message responses
	//

	overload_anti_aircraft_message_responses ();

	overload_bridge_message_responses ();

	overload_camera_message_responses ();

	overload_cargo_message_responses ();

	overload_city_message_responses ();

	overload_city_building_message_responses ();

	overload_crater_message_responses ();

	overload_division_message_responses ();

	overload_explosion_message_responses ();

	overload_fixed_wing_message_responses ();

	overload_force_message_responses ();

	overload_group_message_responses ();

	overload_guide_message_responses ();

	overload_helicopter_message_responses ();

	overload_keysite_message_responses ();

	overload_landing_message_responses ();

	overload_object_message_responses ();

	overload_particle_message_responses ();

	overload_person_message_responses ();

	overload_pilot_message_responses ();

	overload_pylon_message_responses ();

	overload_regen_message_responses ();

	overload_routed_vehicle_message_responses ();

	overload_scenic_message_responses ();

	overload_sector_message_responses ();

	overload_segment_message_responses ();

	overload_session_message_responses ();

	overload_ship_vehicle_message_responses ();

	overload_site_message_responses ();

	overload_site_updatable_message_responses ();

	overload_smoke_list_message_responses ();

	overload_sound_effect_message_responses ();

	overload_sprite_message_responses ();

	overload_task_message_responses ();

	overload_update_message_responses ();

	overload_waypoint_message_responses ();

	overload_weapon_message_responses ();
}




