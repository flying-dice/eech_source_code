

































































#include "project.h"

#include "ai_misc/ai_misc.h"
#include "taskgen/taskgen.h"
#include "faction/faction.h"
#include "highlevl/highlevl.h"
#include "parser/parsgen.h"





#define DEBUG_MODULE 0





int
	ai_memory_counter;











void initialise_ai_system (void)
{

	#if AI_DEBUG

	debug_log ("Initialise AI system");

	ai_memory_counter = 0;

	#endif

	initialise_ai_database ();

	initialise_route_data ();

	initialise_object_3d_regeneration_database ();

	initialise_high_level_ai ();


}





void start_ai_system (void)
{
	start_high_level_ai ();
}





void stop_ai_system (void)
{
	stop_high_level_ai ();
}





void deinitialise_ai_system (void)
{



	deinitialise_ai_database ();

	deinitialise_route_data ();

	deinitialise_formation_database ();

	deinitialise_formation_component_database ();

	deinitialise_high_level_ai ();





	#if AI_DEBUG

	debug_log ("Deinitialise AI system");

	debug_log ("AI Memory count = %d", ai_memory_counter);

	#endif
}





void reinitialise_client_ai_system (void)
{

	comms_model_types
		temp_comms_model;

	comms_data_flow_types
		temp_comms_data_flow;

	temp_comms_model = get_comms_model ();

	temp_comms_data_flow = get_comms_data_flow ();

	//
	// set comms model and data flow to SERVER/TX
	//

	set_comms_model (COMMS_MODEL_SERVER);

	set_comms_data_flow (COMMS_DATA_FLOW_TX);

	if (population_placement_filename)
	{

		read_population_placement_file (population_placement_filename);
	}

	set_comms_model (temp_comms_model);

	set_comms_data_flow (temp_comms_data_flow);

	initialise_keysite_farp_enable (get_local_force_entity (ENTITY_SIDE_BLUE_FORCE));

	initialise_keysite_farp_enable (get_local_force_entity (ENTITY_SIDE_RED_FORCE));



	set_assign_create_waypoint_route (TRUE);
}





void reinitialise_ai_system (void)
{

	ASSERT (get_current_game_session ());







	clear_road_route_data ();

	deinitialise_formation_database ();

	deinitialise_formation_component_database ();

	initialise_formation_database ();

	initialise_formation_component_database ();





	set_assign_create_waypoint_route (TRUE);

	#if AI_DEBUG

	debug_log ("Reinitialise AI system");

	#endif
}


















