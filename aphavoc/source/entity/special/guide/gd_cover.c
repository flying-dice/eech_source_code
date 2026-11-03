























































































#define SEEK_COVER_CURRENT_POSITION_THRESHOLD	100





enum GUIDE_SEEK_COVER_MODES
{
	SEEK_COVER_MODE_CURRENT_POSITION,
	SEEK_COVER_MODE_CHECK_SCORE1,

	SEEK_COVER_MODE_BUILDINGS,
	SEEK_COVER_MODE_BRIDGES,
	SEEK_COVER_MODE_CITY_BUILDINGS,
	SEEK_COVER_MODE_CHECK_SCORE2,
	
	SEEK_COVER_MODE_FOREST,
	SEEK_COVER_MODE_RIVER,
	SEEK_COVER_MODE_CHECK_SCORE3,

	SEEK_COVER_MODE_DEFAULT_TERRAIN,
	SEEK_COVER_MODE_CHECK_SCORE4,

	NUM_GUIDE_SEEK_COVER_MODES
};

typedef enum GUIDE_SEEK_COVER_MODES guide_seek_cover_modes;



















void reset_guide_seek_cover_results (entity *en)
{
	guide
		*raw;

	ASSERT (en);

	raw = (guide *) get_local_entity_data (en);

	memset (&(raw->current_seek_cover_result), 0, sizeof (guide_seek_cover_result_type));

	memset (&(raw->best_seek_cover_result), 0, sizeof (guide_seek_cover_result_type));

	if (raw->terrain_polygon_list)
	{
		free_mem (raw->terrain_polygon_list);

		raw->terrain_polygon_list = NULL;

		raw->terrain_polygon_count = 0;
	}
}





int update_guide_seek_cover_results (entity *en)
{
	guide
		*raw;

	entity
		*group,
		*guide,
		*member,
		*aggressor;

	int
		loop;

	ASSERT (en);

	//
	// wait till other *AI* members have found cover
	//

	aggressor = get_local_entity_first_child (en, LIST_TYPE_FOLLOWER);

	ASSERT (aggressor);

	group = get_local_entity_parent (aggressor, LIST_TYPE_MEMBER);

	ASSERT (group);

	member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

	while (member)
	{
		if (member == aggressor)
		{
			break;
		}

		if (get_local_entity_int_value (member, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
		{
			guide = get_local_entity_parent (member, LIST_TYPE_FOLLOWER);

			if (guide)
			{
				if (get_local_entity_int_value (guide, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_SEEK_COVER)
				{
					return FALSE;
				}
			}
		}

		member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
	}

	//
	// seek cover
	//

	raw = (struct GUIDE *) get_local_entity_data (en);

	for (loop = 0; loop < NUM_COVER_CHECKS_PER_FRAME; loop ++)
	{
		switch (raw->current_seek_cover_result.search_mode)
		{
			case SEEK_COVER_MODE_CURRENT_POSITION:
			{
				update_guide_seek_cover_current_position_search (en);
	
				break;
			}
	
			case SEEK_COVER_MODE_BUILDINGS:
			case SEEK_COVER_MODE_BRIDGES:
			{
				update_guide_seek_cover_fixed_structure_search (en);
	
				break;
			}
	
			case SEEK_COVER_MODE_CITY_BUILDINGS:
			{
				update_guide_seek_cover_fixed_structure_search (en);

				loop += 2;		// less checks per frame for cities (check is slow)
	
				break;
			}
	
			case SEEK_COVER_MODE_FOREST:
			{
				update_guide_seek_cover_forest_search (en);
	
				break;
			}
	
			case SEEK_COVER_MODE_RIVER:
			{
				set_guide_seek_cover_next_search_mode (en);
	
				break;
			}
	
			case SEEK_COVER_MODE_DEFAULT_TERRAIN:
			{
				update_guide_seek_cover_default_terrain_search (en);
	
				break;
			}
	
			case SEEK_COVER_MODE_CHECK_SCORE1:
			case SEEK_COVER_MODE_CHECK_SCORE2:
			case SEEK_COVER_MODE_CHECK_SCORE3:
			case SEEK_COVER_MODE_CHECK_SCORE4:
			{
				//
				// Check current score and early out if good
				//
				
				if (raw->best_seek_cover_result.score > SEEK_COVER_CURRENT_POSITION_THRESHOLD)
				{
					//
					// set cover position to best result
					//
	
					set_attack_guide_fly_to_cover_position (en);
	
					reset_guide_seek_cover_results (en);

					return TRUE;
				}
				else
				{
					set_guide_seek_cover_next_search_mode (en);
				}
	
				break;
			}
	
			case NUM_GUIDE_SEEK_COVER_MODES:
			{
				if (raw->best_seek_cover_result.score > 0)
				{
					//
					// set cover position to best result
					//
	
					set_attack_guide_fly_to_cover_position (en);
				}
				else
				{
					debug_colour_log (DEBUG_COLOUR_DARK_GREEN, "GD_COVER: No Cover found for %s", get_local_entity_string (aggressor, STRING_TYPE_FULL_NAME));
	
					attack_guide_no_cover_found (en);
				}
	
				reset_guide_seek_cover_results (en);
	
				return TRUE;
			}
	
			default:
			{
				debug_fatal ("GD_COVER: Invalid seek mode %d", raw->current_seek_cover_result.search_mode);
			}
		}
	}
	
	return TRUE;
}





void set_guide_seek_cover_next_search_mode (entity *en)
{
	guide
		*raw;

	ASSERT (en);

	raw = (guide *) get_local_entity_data (en);

	raw->current_seek_cover_result.search_mode ++;

	raw->current_seek_cover_result.sector = 0;
	
	raw->current_seek_cover_result.count = 0;

	if (raw->terrain_polygon_list)
	{
		free_mem (raw->terrain_polygon_list);

		raw->terrain_polygon_list = NULL;

		raw->terrain_polygon_count = 0;
	}














































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































