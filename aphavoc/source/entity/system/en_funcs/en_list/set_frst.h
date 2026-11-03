





































































ASSERT (en != first_child);

switch (type)
{


	#ifdef LIST_TYPE_AIRCREW_ROOT

	case LIST_TYPE_AIRCREW:
	{
		raw->aircrew_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_AIR_REGISTRY_ROOT

	case LIST_TYPE_AIR_REGISTRY:
	{
		raw->air_registry_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_ASSIGNED_TASK_ROOT

	case LIST_TYPE_ASSIGNED_TASK:
	{
		raw->assigned_task_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_BUILDING_GROUP_ROOT

	case LIST_TYPE_BUILDING_GROUP:
	{
		raw->building_group_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CAMPAIGN_OBJECTIVE_ROOT

	case LIST_TYPE_CAMPAIGN_OBJECTIVE:
	{
		raw->campaign_objective_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CARGO_ROOT

	case LIST_TYPE_CARGO:
	{
		raw->cargo_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CITY_BUILDING_ROOT

	case LIST_TYPE_CITY_BUILDING:
	{
		raw->city_building_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_COMPLETED_TASK_ROOT

	case LIST_TYPE_COMPLETED_TASK:
	{
		raw->completed_task_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CURRENT_WAYPOINT_ROOT

	case LIST_TYPE_CURRENT_WAYPOINT:
	{
		raw->current_waypoint_root.first_child = first_child;

		break;
	}

	#endif



	// Jabbewock 031107 Designated targets
		
	#ifdef LIST_TYPE_DESIGNATED_TARGET_ROOT

	case LIST_TYPE_DESIGNATED_TARGET:
	{
		raw->designated_target_root.first_child = first_child;

		break;
	}

	#endif
	
	// Jabbewock 031107 ends

	#ifdef LIST_TYPE_DIVISION_ROOT

	case LIST_TYPE_DIVISION:
	{
		raw->division_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_DIVISION_HEADQUARTERS_ROOT

	case LIST_TYPE_DIVISION_HEADQUARTERS:
	{
		raw->division_headquarters_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_FOLLOWER_ROOT

	case LIST_TYPE_FOLLOWER:
	{
		raw->follower_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_FORCE_ROOT

	case LIST_TYPE_FORCE:
	{
		raw->force_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GROUND_REGISTRY_ROOT

	case LIST_TYPE_GROUND_REGISTRY:
	{
		raw->ground_registry_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUIDE_ROOT

	case LIST_TYPE_GUIDE:
	{
		raw->guide_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUIDE_STACK_ROOT

	case LIST_TYPE_GUIDE_STACK:
	{
		raw->guide_stack_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUNSHIP_TARGET_ROOT

	case LIST_TYPE_GUNSHIP_TARGET:
	{
		raw->gunship_target_root.first_child = first_child;

		break;
	}

	#endif
	


	#ifdef LIST_TYPE_INDEPENDENT_GROUP_ROOT

	case LIST_TYPE_INDEPENDENT_GROUP:
	{
		raw->independent_group_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_HIERARCHY_ROOT

	case LIST_TYPE_HIERARCHY:
	{
		raw->hierarchy_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_KEYSITE_FORCE_ROOT

	case LIST_TYPE_KEYSITE_FORCE:
	{
		raw->keysite_force_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_KEYSITE_GROUP_ROOT

	case LIST_TYPE_KEYSITE_GROUP:
	{
		raw->keysite_group_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_LAUNCHED_WEAPON_ROOT

	case LIST_TYPE_LAUNCHED_WEAPON:
	{
		raw->launched_weapon_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_LANDING_SITE_ROOT

	case LIST_TYPE_LANDING_SITE:
	{
		raw->landing_site_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_MEMBER_ROOT

	case LIST_TYPE_MEMBER:
	{
		raw->member_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_MOVEMENT_DEPENDENT_ROOT

	case LIST_TYPE_MOVEMENT_DEPENDENT:
	{
		raw->movement_dependent_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PADLOCK_ROOT

	case LIST_TYPE_PADLOCK:
	{
		raw->padlock_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PILOT_ROOT

	case LIST_TYPE_PILOT:
	{
		raw->pilot_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PILOT_LOCK_ROOT

	case LIST_TYPE_PILOT_LOCK:
	{
		raw->pilot_lock_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PLAYER_TASK_ROOT

	case LIST_TYPE_PLAYER_TASK:
	{
		raw->player_task_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_REGEN_ROOT

	case LIST_TYPE_REGEN:
	{
		raw->regen_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SEA_REGISTRY_ROOT

	case LIST_TYPE_SEA_REGISTRY:
	{
		raw->sea_registry_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SECTOR_ROOT

	case LIST_TYPE_SECTOR:
	{
		raw->sector_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SECTOR_TASK_ROOT

	case LIST_TYPE_SECTOR_TASK:
	{
		raw->sector_task_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SEGMENT_ROOT

	case LIST_TYPE_SEGMENT:
	{
		raw->segment_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SPECIAL_EFFECT_ROOT

	case LIST_TYPE_SPECIAL_EFFECT:
	{
		raw->special_effect_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TAKEOFF_QUEUE_ROOT

	case LIST_TYPE_TAKEOFF_QUEUE:
	{
		raw->takeoff_queue_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TARGET_ROOT

	case LIST_TYPE_TARGET:
	{
		raw->target_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TASK_DEPENDENT_ROOT

	case LIST_TYPE_TASK_DEPENDENT:
	{
		raw->task_dependent_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_UNASSIGNED_TASK_ROOT

	case LIST_TYPE_UNASSIGNED_TASK:
	{
		raw->unassigned_task_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_UPDATE_ROOT

	case LIST_TYPE_UPDATE:
	{
		raw->update_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_VIEW_ROOT

	case LIST_TYPE_VIEW:
	{
		raw->view_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_VIEW_WEAPON_ROOT

	case LIST_TYPE_VIEW_WEAPON:
	{
		raw->view_weapon_root.first_child = first_child;

		break;
	}

	#endif



	#ifdef LIST_TYPE_WAYPOINT_ROOT

	case LIST_TYPE_WAYPOINT:
	{
		raw->waypoint_root.first_child = first_child;

		break;
	}

	#endif



	default:
	{
		debug_fatal_invalid_list_type (en, type);

		break;
	}


}





