





































































ASSERT (en != parent);

switch (type)
{


	#ifdef LIST_TYPE_AIRCREW_LINK

	case LIST_TYPE_AIRCREW:
	{
		raw->aircrew_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_ASSIGNED_TASK_LINK

	case LIST_TYPE_ASSIGNED_TASK:
	{
		raw->task_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CAMPAIGN_OBJECTIVE_LINK

	case LIST_TYPE_CAMPAIGN_OBJECTIVE:
	{
		raw->campaign_objective_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CARGO_LINK

	case LIST_TYPE_CARGO:
	{
		raw->cargo_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CITY_BUILDING_LINK

	case LIST_TYPE_CITY_BUILDING:
	{
		raw->city_building_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_COMPLETED_TASK_LINK

	case LIST_TYPE_COMPLETED_TASK:
	{
		raw->task_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CURRENT_WAYPOINT_LINK

	case LIST_TYPE_CURRENT_WAYPOINT:
	{
		raw->current_waypoint_link.parent = parent;

		break;
	}

	#endif




	// Jabbewock 031107 Designated targets
	
	#ifdef LIST_TYPE_DESIGNATED_TARGET_LINK

	case LIST_TYPE_DESIGNATED_TARGET:
	{
		raw->designated_target_link.parent = parent;

		break;
	}

	#endif
	
	// Jabbewock 031107 ends
	
	#ifdef LIST_TYPE_DIVISION_LINK

	case LIST_TYPE_DIVISION:
	{
		raw->division_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_DIVISION_HEADQUARTERS_LINK

	case LIST_TYPE_DIVISION_HEADQUARTERS:
	{
		raw->division_headquarters_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_FOLLOWER_LINK

	case LIST_TYPE_FOLLOWER:
	{
		raw->follower_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_FORCE_LINK

	case LIST_TYPE_FORCE:
	{
		raw->force_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GROUP_LINK

	case LIST_TYPE_BUILDING_GROUP:
	case LIST_TYPE_INDEPENDENT_GROUP:
	case LIST_TYPE_KEYSITE_GROUP:
	{
		raw->group_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUIDE_LINK

	case LIST_TYPE_GUIDE:
	{
		raw->guide_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUIDE_STACK_LINK

	case LIST_TYPE_GUIDE_STACK:
	{
		raw->guide_stack_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUNSHIP_TARGET_LINK

	case LIST_TYPE_GUNSHIP_TARGET:
	{
		raw->gunship_target_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_HIERARCHY_LINK

	case LIST_TYPE_HIERARCHY:
	{
		raw->hierarchy_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_KEYSITE_FORCE_LINK

	case LIST_TYPE_KEYSITE_FORCE:
	{
		raw->keysite_force_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_LAUNCHED_WEAPON_LINK

	case LIST_TYPE_LAUNCHED_WEAPON:
	{
		raw->launched_weapon_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_LANDING_SITE_LINK

	case LIST_TYPE_LANDING_SITE:
	{
		raw->landing_site_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_MEMBER_LINK

	case LIST_TYPE_MEMBER:
	{
		raw->member_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_MOVEMENT_DEPENDENT_LINK

	case LIST_TYPE_MOVEMENT_DEPENDENT:
	{
		raw->movement_dependent_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PADLOCK_LINK

	case LIST_TYPE_PADLOCK:
	{
		raw->padlock_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PILOT_LINK

	case LIST_TYPE_PILOT:
	{
		raw->pilot_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PILOT_LOCK_LINK

	case LIST_TYPE_PILOT_LOCK:
	{
		raw->pilot_lock_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PLAYER_TASK_LINK

	case LIST_TYPE_PLAYER_TASK:
	{
		raw->player_task_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_REGEN_LINK

	case LIST_TYPE_REGEN:
	{
		raw->regen_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_REGISTRY_LINK

	case LIST_TYPE_AIR_REGISTRY:
	case LIST_TYPE_GROUND_REGISTRY:
	case LIST_TYPE_SEA_REGISTRY:
	{
		raw->registry_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SECTOR_LINK

	case LIST_TYPE_SECTOR:
	{
		raw->sector_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SECTOR_TASK_LINK

	case LIST_TYPE_SECTOR_TASK:
	{
		raw->sector_task_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SEGMENT_LINK

	case LIST_TYPE_SEGMENT:
	{
		raw->segment_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SPECIAL_EFFECT_LINK

	case LIST_TYPE_SPECIAL_EFFECT:
	{
		raw->special_effect_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TAKEOFF_QUEUE_LINK

	case LIST_TYPE_TAKEOFF_QUEUE:
	{
		raw->takeoff_queue_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TARGET_LINK

	case LIST_TYPE_TARGET:
	{
		raw->target_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TASK_DEPENDENT_LINK

	case LIST_TYPE_TASK_DEPENDENT:
	{
		raw->task_dependent_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_UNASSIGNED_TASK_LINK

	case LIST_TYPE_UNASSIGNED_TASK:
	{
		raw->task_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_UPDATE_LINK

	case LIST_TYPE_UPDATE:
	{
		raw->update_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_VIEW_LINK

	case LIST_TYPE_VIEW:
	{
		raw->view_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_VIEW_WEAPON_LINK

	case LIST_TYPE_VIEW_WEAPON:
	{
		raw->view_weapon_link.parent = parent;

		break;
	}

	#endif



	#ifdef LIST_TYPE_WAYPOINT_LINK

	case LIST_TYPE_WAYPOINT:
	{
		raw->waypoint_link.parent = parent;

		break;
	}

	#endif



	default:
	{
		debug_fatal_invalid_list_type (en, type);

		break;
	}


}





