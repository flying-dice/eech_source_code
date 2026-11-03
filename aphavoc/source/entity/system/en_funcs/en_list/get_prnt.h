

































































switch (type)
{


	#ifdef LIST_TYPE_AIRCREW_LINK

	case LIST_TYPE_AIRCREW:
	{
		parent = raw->aircrew_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_ASSIGNED_TASK_LINK

	case LIST_TYPE_ASSIGNED_TASK:
	{
		parent = raw->task_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CAMPAIGN_OBJECTIVE_LINK

	case LIST_TYPE_CAMPAIGN_OBJECTIVE:
	{
		parent = raw->campaign_objective_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CARGO_LINK

	case LIST_TYPE_CARGO:
	{
		parent = raw->cargo_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CITY_BUILDING_LINK

	case LIST_TYPE_CITY_BUILDING:
	{
		parent = raw->city_building_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_COMPLETED_TASK_LINK

	case LIST_TYPE_COMPLETED_TASK:
	{
		parent = raw->task_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CURRENT_WAYPOINT_LINK

	case LIST_TYPE_CURRENT_WAYPOINT:
	{
		parent = raw->current_waypoint_link.parent;

		break;
	}

	#endif



	// Jabbewock 031107 Designated targets
	
	#ifdef LIST_TYPE_DESIGNATED_TARGET_LINK

	case LIST_TYPE_DESIGNATED_TARGET:
	{
		parent = raw->designated_target_link.parent;

		break;
	}

	#endif

	// Jabbewock 031107 ends

	#ifdef LIST_TYPE_DIVISION_LINK

	case LIST_TYPE_DIVISION:
	{
		parent = raw->division_link.parent;

		break;
	}

	#endif
	


	#ifdef LIST_TYPE_DIVISION_HEADQUARTERS_LINK

	case LIST_TYPE_DIVISION_HEADQUARTERS:
	{
		parent = raw->division_headquarters_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_FOLLOWER_LINK

	case LIST_TYPE_FOLLOWER:
	{
		parent = raw->follower_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_FORCE_LINK

	case LIST_TYPE_FORCE:
	{
		parent = raw->force_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GROUP_LINK

	case LIST_TYPE_BUILDING_GROUP:
	case LIST_TYPE_INDEPENDENT_GROUP:
	case LIST_TYPE_KEYSITE_GROUP:
	{
		parent = raw->group_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUIDE_LINK

	case LIST_TYPE_GUIDE:
	{
		parent = raw->guide_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUIDE_STACK_LINK

	case LIST_TYPE_GUIDE_STACK:
	{
		parent = raw->guide_stack_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUNSHIP_TARGET_LINK

	case LIST_TYPE_GUNSHIP_TARGET:
	{
		parent = raw->gunship_target_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_HIERARCHY_LINK

	case LIST_TYPE_HIERARCHY:
	{
		parent = raw->hierarchy_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_KEYSITE_FORCE_LINK

	case LIST_TYPE_KEYSITE_FORCE:
	{
		parent = raw->keysite_force_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_LAUNCHED_WEAPON_LINK

	case LIST_TYPE_LAUNCHED_WEAPON:
	{
		parent = raw->launched_weapon_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_LANDING_SITE_LINK

	case LIST_TYPE_LANDING_SITE:
	{
		parent = raw->landing_site_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_MEMBER_LINK

	case LIST_TYPE_MEMBER:
	{
		parent = raw->member_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_MOVEMENT_DEPENDENT_LINK

	case LIST_TYPE_MOVEMENT_DEPENDENT:
	{
		parent = raw->movement_dependent_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PADLOCK_LINK

	case LIST_TYPE_PADLOCK:
	{
		parent = raw->padlock_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PILOT_LINK

	case LIST_TYPE_PILOT:
	{
		parent = raw->pilot_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PILOT_LOCK_LINK

	case LIST_TYPE_PILOT_LOCK:
	{
		parent = raw->pilot_lock_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PLAYER_TASK_LINK

	case LIST_TYPE_PLAYER_TASK:
	{
		parent = raw->player_task_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_REGEN_LINK

	case LIST_TYPE_REGEN:
	{
		parent = raw->regen_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_REGISTRY_LINK

	case LIST_TYPE_AIR_REGISTRY:
	case LIST_TYPE_GROUND_REGISTRY:
	case LIST_TYPE_SEA_REGISTRY:
	{
		parent = raw->registry_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SECTOR_LINK

	case LIST_TYPE_SECTOR:
	{
		parent = raw->sector_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SECTOR_TASK_LINK

	case LIST_TYPE_SECTOR_TASK:
	{
		parent = raw->sector_task_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SEGMENT_LINK

	case LIST_TYPE_SEGMENT:
	{
		parent = raw->segment_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SPECIAL_EFFECT_LINK

	case LIST_TYPE_SPECIAL_EFFECT:
	{
		parent = raw->special_effect_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TAKEOFF_QUEUE_LINK

	case LIST_TYPE_TAKEOFF_QUEUE:
	{
		parent = raw->takeoff_queue_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TARGET_LINK

	case LIST_TYPE_TARGET:
	{
		parent = raw->target_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TASK_DEPENDENT_LINK

	case LIST_TYPE_TASK_DEPENDENT:
	{
		parent = raw->task_dependent_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_UNASSIGNED_TASK_LINK

	case LIST_TYPE_UNASSIGNED_TASK:
	{
		parent = raw->task_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_UPDATE_LINK

	case LIST_TYPE_UPDATE:
	{
		parent = raw->update_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_VIEW_LINK

	case LIST_TYPE_VIEW:
	{
		parent = raw->view_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_VIEW_WEAPON_LINK

	case LIST_TYPE_VIEW_WEAPON:
	{
		parent = raw->view_weapon_link.parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_WAYPOINT_LINK

	case LIST_TYPE_WAYPOINT:
	{
		parent = raw->waypoint_link.parent;

		break;
	}

	#endif



	default:
	{
		debug_fatal_invalid_list_type (en, type);

		parent = NULL;

		break;
	}


}

//
// check list integrity
//

ASSERT (en != parent);





