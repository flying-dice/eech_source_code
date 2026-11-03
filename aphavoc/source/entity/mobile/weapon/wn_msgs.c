

































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	weapon
		*raw;







	raw = (weapon *) get_local_entity_data (receiver);

	raw->mob.kill_next_update = TRUE;

	return (TRUE);
}





static int response_to_decoy_active (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	weapon
		*raw;

	weapon_decoy_types
		decoy_type;







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (weapon *) get_local_entity_data (receiver);

	decoy_type = (weapon_decoy_types) get_local_entity_int_value (sender, INT_TYPE_WEAPON_DECOY_TYPE);

	if (get_weapon_locked_onto_decoy (decoy_type, sender))
	{
		switch (weapon_database[raw->mob.sub_type].guidance_type)
		{

			case WEAPON_GUIDANCE_TYPE_NONE:

			{
				break;
			}

			case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_RADAR:
			case WEAPON_GUIDANCE_TYPE_ACTIVE_RADAR:

			{
				if (decoy_type == WEAPON_DECOY_TYPE_CHAFF)
				{
					set_client_server_entity_parent (receiver, LIST_TYPE_TARGET, sender);
				}

				break;
			}

			case WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED:

			{
				if (decoy_type == WEAPON_DECOY_TYPE_FLARE)
				{
					set_client_server_entity_parent (receiver, LIST_TYPE_TARGET, sender);
				}

				break;
			}

			case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER:
			case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER_BEAM_RIDING:
			case WEAPON_GUIDANCE_TYPE_ACTIVE_LASER:
			case WEAPON_GUIDANCE_TYPE_WIRE_GUIDED:
			case WEAPON_GUIDANCE_TYPE_RADIO_COMMAND:

			{
				if (decoy_type == WEAPON_DECOY_TYPE_SMOKE_GRENADE)
				{
					set_client_server_entity_parent (receiver, LIST_TYPE_TARGET, sender);
				}

				break;
			}

			default:

			{
				debug_fatal ("Invalid weapon guidance type = %d", weapon_database[raw->mob.sub_type].guidance_type);

				break;
			}
		}
	}

	return (TRUE);
}





static int response_to_decoy_expired (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{






	ASSERT (sender);

	delete_local_entity_from_parents_child_list (receiver, LIST_TYPE_TARGET);

	insert_local_entity_into_parents_child_list (receiver, LIST_TYPE_TARGET, sender, NULL);

	//
	// a comms message is not required as the unlinking will work on both client and server
	//

	return (TRUE);
}


















































static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	weapon
		*raw;

	list_types
		list_type;

	vec3d
		target_position;







	raw = (weapon *) get_local_entity_data (receiver);

	list_type = va_arg (pargs, list_types);

	if (list_type == LIST_TYPE_TARGET)
	{
		if (get_view_mode () == VIEW_MODE_EXTERNAL)
		{
			if (get_camera_entity ())
			{
				if (get_local_entity_int_value (get_camera_entity (), INT_TYPE_CAMERA_MODE) == CAMERA_MODE_WEAPON)
				{
					if (receiver == get_local_entity_first_child (get_camera_entity (), LIST_TYPE_VIEW_WEAPON))
					{
						if (sender)
						{
							if (!get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_WEAPON))
							{
								if (!get_local_entity_int_value (sender, INT_TYPE_ALIVE))
								{
									get_local_entity_target_point (sender, &target_position);

									if (get_3d_range (&raw->mob.position, &target_position) < 250.0)
									{
										notify_local_entity (ENTITY_MESSAGE_SET_CAMERA_ACTION, get_camera_entity (), NULL, CAMERA_ACTION_WEAPON_EXPLOSION);

										reset_weapon_explosion_camera_position (get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_POSITION));
									}
								}
							}
						}
					}
				}
			}
		}
	}

	return (TRUE);
}































