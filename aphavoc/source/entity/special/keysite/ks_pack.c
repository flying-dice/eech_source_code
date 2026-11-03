


























































































































































































































































































void unpack_local_keysite_data (pack_modes mode)
{

























	en = get_local_entity_list ();

	while (en)
	{

		if (get_local_entity_type (en) == ENTITY_TYPE_KEYSITE)
		{

















			raw->alive = unpack_int_value (en, INT_TYPE_ALIVE);

			side = unpack_int_value (en, INT_TYPE_SIDE);

			if (side != raw->side)
			{
				//
				// update I-Maps (remove old side)
				//

				if (raw->in_use)
				{
					update_imap_sector_side (en, FALSE);

					if (mode == PACK_MODE_SERVER_SESSION)
					{
						update_imap_importance_level (en, FALSE);
					}
				}

				//
				// change side of keysite buildings
				//

				change_local_keysite_building_sides (en, (entity_sides) side);

				//
				// place into other sides force
				//

				delete_local_entity_from_parents_child_list (en, LIST_TYPE_KEYSITE_FORCE);

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_KEYSITE_FORCE, get_local_force_entity ((entity_sides) side), NULL);

				if (mode == PACK_MODE_SERVER_SESSION)
				{
					// Must be done after removed from force
					update_imap_distance_to_friendly_base ((entity_sides) raw->side);
				}

				//
				// switch sides
				//

				raw->side = side;

				//
				// update I-Maps (add new side)
				//

				if (raw->in_use)
				{
					update_imap_sector_side (en, TRUE);

					if (mode == PACK_MODE_SERVER_SESSION)
					{
						update_imap_importance_level (en, TRUE);

						update_keysite_distance_to_friendly_base (en, (entity_sides) side);
					}
				}
			}








			if (raw->in_use != in_use)
			{
				//
				// Update I-Maps (in use flag changed)
				//

				raw->in_use = in_use;

				update_imap_sector_side (en, in_use);

				if (mode == PACK_MODE_SERVER_SESSION)
				{
					update_imap_importance_level (en, in_use);

					if (in_use)
					{
						update_keysite_distance_to_friendly_base (en, (entity_sides) side);
					}
					else
					{
						update_imap_distance_to_friendly_base ((entity_sides) side);
					}
				}
			}





















































































		}

		en = get_local_entity_succ (en);
	}
}




