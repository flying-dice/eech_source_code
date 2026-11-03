

































































#include "project.h"

#include "gunships/dynamics/common/co_undercarriage.h"





#define DEBUG_MODULE 0





void initialise_entity_databases (void)
{
	initialise_bridge_database ();

	initialise_segment_database ();

	initialise_smoke_list_database ();

	initialise_meta_smoke_list_database ();

	initialise_explosion_database ();

	initialise_weapon_loading_gunship_database ();

	initialise_destroyed_object_database ();

	initialise_speech_index_table ();

	initialise_fixed_3d_object_database ();

	initialise_undercarriage_database();
	
// Xhit: added for downwash effect (030328)
	//VJ 030508 if downwash 
	if (command_line_downwash)
		initialise_downwash_database ();
}





void deinitialise_entity_databases (void)
{
	deinitialise_bridge_database ();

	deinitialise_segment_database ();

	deinitialise_explosion_database ();

	deinitialise_meta_smoke_list_database ();

	deinitialise_smoke_list_database ();

	deinitialise_weapon_loading_gunship_database ();

	deinitialise_destroyed_object_database ();

	deinitialise_speech_index_table ();

	report_non_entries_in_fixed_3d_object_database ();

	deinitialise_undercarriage_database();

// Xhit: added for downwash effect (030328)
	//VJ 030508 if downwash 
	if (command_line_downwash)
		deinitialise_downwash_database ();

}




