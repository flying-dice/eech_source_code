

































































#include "project.h"





#define DEBUG_MODULE 0





void initialise_entity_functions (void)
{
	//
	// set default functions
	//

	initialise_entity_attitude_default_functions ();

	initialise_entity_char_value_default_functions ();

	initialise_entity_create_default_functions ();

	initialise_entity_draw_default_functions ();

	initialise_entity_destroy_default_functions ();

	initialise_entity_float_value_default_functions ();

	initialise_entity_int_value_default_functions ();

	initialise_entity_pack_default_functions ();

	initialise_entity_ptr_value_default_functions ();

	initialise_entity_list_default_functions ();

	initialise_entity_sound_default_functions ();

	initialise_entity_string_default_functions ();

	initialise_entity_update_default_functions ();

	initialise_entity_vec3d_default_functions ();

	//
	// overload default functions
	//

	overload_anti_aircraft_functions ();

	overload_bridge_functions ();

	overload_camera_functions ();

	overload_cargo_functions ();

	overload_city_functions ();

	overload_city_building_functions ();

	overload_crater_functions ();

	overload_division_functions ();

	overload_explosion_functions ();

	overload_fixed_wing_functions ();

	overload_force_functions ();

	overload_group_functions ();

	overload_guide_functions ();

	overload_helicopter_functions ();

	overload_keysite_functions ();

	overload_landing_functions ();

	overload_object_functions ();

	overload_particle_functions ();

	overload_person_functions ();

	overload_pilot_functions ();

	overload_pylon_functions ();

	overload_regen_functions ();

	overload_routed_vehicle_functions ();

	overload_sector_functions ();

	overload_segment_functions ();

	overload_session_functions ();

	overload_scenic_functions ();

	overload_ship_vehicle_functions ();

	overload_site_functions ();

	overload_site_updatable_functions ();

	overload_smoke_list_functions ();

	overload_sound_effect_functions ();

	overload_sprite_functions ();

	overload_task_functions ();

	overload_update_functions ();

	overload_waypoint_functions ();

	overload_weapon_functions ();
}




