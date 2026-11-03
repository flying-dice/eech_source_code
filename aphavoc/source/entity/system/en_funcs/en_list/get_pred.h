

































































switch (type)
{


	#ifdef LIST_TYPE_AIRCREW_LINK

	case LIST_TYPE_AIRCREW:
	{
		child_pred = raw->aircrew_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_ASSIGNED_TASK_LINK

	case LIST_TYPE_ASSIGNED_TASK:
	{
		child_pred = raw->task_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CAMPAIGN_OBJECTIVE_LINK

	case LIST_TYPE_CAMPAIGN_OBJECTIVE:
	{
		child_pred = raw->campaign_objective_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CARGO_LINK

	case LIST_TYPE_CARGO:
	{
		child_pred = raw->cargo_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CITY_BUILDING_LINK

	case LIST_TYPE_CITY_BUILDING:
	{
		child_pred = raw->city_building_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_COMPLETED_TASK_LINK

	case LIST_TYPE_COMPLETED_TASK:
	{
		child_pred = raw->task_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_CURRENT_WAYPOINT_LINK

	case LIST_TYPE_CURRENT_WAYPOINT:
	{
		child_pred = raw->current_waypoint_link.child_pred;

		break;
	}

	#endif



	// Jabbewock 031107 Designated targets
	
	#ifdef LIST_TYPE_DESIGNATED_TARGET_LINK

	case LIST_TYPE_DESIGNATED_TARGET:
	{
		child_pred = raw->designated_target_link.child_pred;

		break;
	}

	#endif

	// Jabbewock 031107 ends

	#ifdef LIST_TYPE_DIVISION_LINK

	case LIST_TYPE_DIVISION:
	{
		child_pred = raw->division_link.child_pred;

		break;
	}

	#endif
	


	#ifdef LIST_TYPE_DIVISION_HEADQUARTERS_LINK

	case LIST_TYPE_DIVISION_HEADQUARTERS:
	{
		child_pred = raw->division_headquarters_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_FOLLOWER_LINK

	case LIST_TYPE_FOLLOWER:
	{
		child_pred = raw->follower_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_FORCE_LINK

	case LIST_TYPE_FORCE:
	{
		child_pred = raw->force_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GROUP_LINK

	case LIST_TYPE_BUILDING_GROUP:
	case LIST_TYPE_INDEPENDENT_GROUP:
	case LIST_TYPE_KEYSITE_GROUP:
	{
		child_pred = raw->group_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUIDE_LINK

	case LIST_TYPE_GUIDE:
	{
		child_pred = raw->guide_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUIDE_STACK_LINK

	case LIST_TYPE_GUIDE_STACK:
	{
		child_pred = raw->guide_stack_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_GUNSHIP_TARGET_LINK

	case LIST_TYPE_GUNSHIP_TARGET:
	{
		child_pred = raw->gunship_target_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_HIERARCHY_LINK

	case LIST_TYPE_HIERARCHY:
	{
		child_pred = raw->hierarchy_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_KEYSITE_FORCE_LINK

	case LIST_TYPE_KEYSITE_FORCE:
	{
		child_pred = raw->keysite_force_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_LAUNCHED_WEAPON_LINK

	case LIST_TYPE_LAUNCHED_WEAPON:
	{
		child_pred = raw->launched_weapon_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_LANDING_SITE_LINK

	case LIST_TYPE_LANDING_SITE:
	{
		child_pred = raw->landing_site_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_MEMBER_LINK

	case LIST_TYPE_MEMBER:
	{
		child_pred = raw->member_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_MOVEMENT_DEPENDENT_LINK

	case LIST_TYPE_MOVEMENT_DEPENDENT:
	{
		child_pred = raw->movement_dependent_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PADLOCK_LINK

	case LIST_TYPE_PADLOCK:
	{
		child_pred = raw->padlock_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PILOT_LINK

	case LIST_TYPE_PILOT:
	{
		child_pred = raw->pilot_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PILOT_LOCK_LINK

	case LIST_TYPE_PILOT_LOCK:
	{
		child_pred = raw->pilot_lock_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_PLAYER_TASK_LINK

	case LIST_TYPE_PLAYER_TASK:
	{
		child_pred = raw->player_task_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_REGEN_LINK

	case LIST_TYPE_REGEN:
	{
		child_pred = raw->regen_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_REGISTRY_LINK

	case LIST_TYPE_AIR_REGISTRY:
	case LIST_TYPE_GROUND_REGISTRY:
	case LIST_TYPE_SEA_REGISTRY:
	{
		child_pred = raw->registry_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SECTOR_LINK

	case LIST_TYPE_SECTOR:
	{
		child_pred = raw->sector_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SECTOR_TASK_LINK

	case LIST_TYPE_SECTOR_TASK:
	{
		child_pred = raw->sector_task_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SEGMENT_LINK

	case LIST_TYPE_SEGMENT:
	{
		child_pred = raw->segment_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_SPECIAL_EFFECT_LINK

	case LIST_TYPE_SPECIAL_EFFECT:
	{
		child_pred = raw->special_effect_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TAKEOFF_QUEUE_LINK

	case LIST_TYPE_TAKEOFF_QUEUE:
	{
		child_pred = raw->takeoff_queue_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TARGET_LINK

	case LIST_TYPE_TARGET:
	{
		child_pred = raw->target_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_TASK_DEPENDENT_LINK

	case LIST_TYPE_TASK_DEPENDENT:
	{
		child_pred = raw->task_dependent_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_UNASSIGNED_TASK_LINK

	case LIST_TYPE_UNASSIGNED_TASK:
	{
		child_pred = raw->task_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_UPDATE_LINK

	case LIST_TYPE_UPDATE:
	{
		child_pred = raw->update_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_VIEW_LINK

	case LIST_TYPE_VIEW:
	{
		child_pred = raw->view_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_VIEW_WEAPON_LINK

	case LIST_TYPE_VIEW_WEAPON:
	{
		child_pred = raw->view_weapon_link.child_pred;

		break;
	}

	#endif



	#ifdef LIST_TYPE_WAYPOINT_LINK

	case LIST_TYPE_WAYPOINT:
	{
		child_pred = raw->waypoint_link.child_pred;

		break;
	}

	#endif



	default:
	{
		debug_fatal_invalid_list_type (en, type);

		child_pred = NULL;

		break;
	}


}

//
// check list integrity
//

ASSERT (en != child_pred);





