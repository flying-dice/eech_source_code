













































































static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*aggressor;

	int
		total_damage_level;

	float
		damage_modifier;







	damage_modifier = va_arg (pargs, double);

	if (sender)
	{
		if (get_local_entity_type (sender) == ENTITY_TYPE_WEAPON)
		{
			aggressor = get_local_entity_parent (sender, LIST_TYPE_LAUNCHED_WEAPON);
		}
		else
		{
			aggressor = sender;
		}

		if (aggressor)
		{
			notify_keysite_structure_under_attack (receiver, aggressor);
		}
	}
	else
	{
		aggressor = NULL;
	}

	damage_client_server_entity (receiver, sender, damage_modifier);

	total_damage_level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);

	if (total_damage_level <= 0)
	{
		if (aggressor)
		{
			if (get_local_entity_float_value (receiver, FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE) >= 0.5)
			{
				if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) == ENTITY_PLAYER_LOCAL)
				{
					inc_player_log_kills (get_global_gunship_side (), get_current_player_log (), receiver);
				}
				else if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) == ENTITY_PLAYER_REMOTE)
				{

				}
			}
		}

		kill_client_server_entity (receiver);
	}

	return (TRUE);
}










































































































































