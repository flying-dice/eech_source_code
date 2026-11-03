

































































#include "project.h"

#define DEBUG_MODULE 0




































































































































































































































































































































int vehicle_critically_damaged (entity *en)
{
	vehicle
		*raw;

	int
		percent_damaged;

	if (get_local_entity_type (en) == ENTITY_TYPE_PERSON)
		return FALSE;
	
	raw = (vehicle *) get_local_entity_data (en);

	percent_damaged = 100 * raw->damage_level / vehicle_database[raw->mob.sub_type].initial_damage_level;

	if (percent_damaged < VEHICLE_CRITICAL_DAMAGE_LEVEL)
		return TRUE;
	else
		return FALSE;
}

void assess_vehicle_damage_level (entity *en, int old_damage_level)
{
	vehicle
		*raw;
	float
		initial_damage,
		old_percent_damaged,
		new_percent_damaged;
	int
		explosive_quality = get_local_entity_int_value (en, INT_TYPE_EXPLOSIVE_QUALITY);

	raw = (vehicle *) get_local_entity_data (en);

	if (old_damage_level == raw->damage_level)
		return;

	if ((explosive_quality & EXPLOSIVE_QUALITY_EXPLOSIVE || explosive_quality & EXPLOSIVE_QUALITY_FLAMMABLE) && get_comms_model () == COMMS_MODEL_SERVER)
	{
		initial_damage = (float)(vehicle_database[raw->mob.sub_type].initial_damage_level);

		old_percent_damaged = (100.0 * (float)old_damage_level) / initial_damage;
		new_percent_damaged = (100.0 * (float)raw->damage_level) / initial_damage;





		// create smoke effect

		if(new_percent_damaged < VEHICLE_CRITICAL_DAMAGE_LEVEL && old_percent_damaged >= VEHICLE_CRITICAL_DAMAGE_LEVEL)
		{




			if (get_local_entity_int_value (en, INT_TYPE_EXPLOSIVE_POWER) > 2)
			{

				set_client_server_entity_float_value(en, FLOAT_TYPE_SLEEP, 6000);
			}
			else
			{

				set_client_server_entity_float_value(en, FLOAT_TYPE_SLEEP, 300);
			}

			set_client_server_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_STOPPED);
		}
		


	}
}
