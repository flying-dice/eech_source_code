

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_weapon_functions (void)
{
	overload_mobile_functions (ENTITY_TYPE_WEAPON);

	overload_weapon_create_functions ();

	overload_weapon_destroy_functions ();

	overload_weapon_draw_functions ();

	overload_weapon_float_value_functions ();

	overload_weapon_int_value_functions ();

	overload_weapon_list_functions ();

	overload_weapon_pack_functions ();

	overload_weapon_string_functions ();

	overload_weapon_update_functions ();

	overload_weapon_vec3d_functions ();

	validate_weapon_database ();

	initialise_target_point_offsets ();
}

float get_weapon_current_weight(float lifetime, entity_sub_types subtype)
{
	if (lifetime > 0)
	{
		float boost_consumption = weapon_database[subtype].boost_power * weapon_database[subtype].boost_time;
		float sustain_consumption = weapon_database[subtype].sustain_power * weapon_database[subtype].sustain_time;
		float boost_scale = boost_consumption / (boost_consumption + sustain_consumption);
		float sustain_scale = sustain_consumption / (boost_consumption + sustain_consumption);
		float sustain_time_scale = weapon_database[subtype].sustain_time / (weapon_database[subtype].boost_time + weapon_database[subtype].sustain_time);
		float boost_time_scale = weapon_database[subtype].boost_time / (weapon_database[subtype].boost_time + weapon_database[subtype].sustain_time);
		float time_scale = lifetime / (weapon_database[subtype].boost_time + weapon_database[subtype].sustain_time);
		float delta_weight = weapon_database[subtype].start_weight - weapon_database[subtype].final_weight;
		float boost_fuel_weight = delta_weight * boost_scale * max(0.0f, (time_scale - sustain_time_scale) / boost_time_scale);
		float sustain_fuel_weight = sustain_time_scale ? (delta_weight * sustain_scale * min(1.0f, time_scale / sustain_time_scale)) : 0;
		
		return weapon_database[subtype].final_weight + boost_fuel_weight + sustain_fuel_weight;
	}
	else
		return weapon_database[subtype].final_weight;
}


float get_weapon_drag(float speed, float alt, float diameter, float drag_coefficient)
{
	float
		l = 0.0065,
		t0 = 288.15, // standard temperature, K
		area = PI * diameter * diameter / 4, // projectile area, m^2
		t = t0 - l * alt, // actual temperature, K
		air_density = get_precise_air_density(alt), // kg/m^3
		mach = speed / (20.046 * sqrt(t)), // velocity, mach
		cd; // drag coefficient

	// get drag coefficient (air resistance law of 1943)
	
	if (mach <= 0.8)
		cd = 0.158;
	else if (mach <= 0.9)
		cd = 0.137 * mach * mach  + 0.0865 * mach + 0.000898;
	else if (mach <= 1.0)
		cd = 1.35 * mach - 1.025;
	else if (mach <= 1.4)
		cd = - 0.925 * mach * mach + 2.335 * mach - 1.085;
	else if (mach <= 4)
		cd = - 0.0917 * mach + 0.4993;
	else
		cd = 0.1325;

	return 0.5 * drag_coefficient * air_density * speed * speed * cd * area;
}




