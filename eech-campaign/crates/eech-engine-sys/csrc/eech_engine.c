/*
 * EECH headless engine: boot and frame.
 *
 * The boot follows EECH's own dedicated-server path, calling the same
 * functions in the same order:
 *   WinMain (startup.c)              event, timer and file systems
 *   application_main (project.c)     options, command line, EECH.INI
 *   brief_initialise_game (init.c)   comms model, events, UI system, language
 *   full_initialise_game (init.c)    maths, comms, sound, 3D objects and terrain,
 *                                    tags, entities, AI, players, ballistics
 *   process_game_initialisation_phases (gameflow.c), dedicated server:
 *     NONE / GAME_TYPE / SETUP / GUNSHIP_TYPE (the campaign is created)
 *   flight (flight.c)                up to its frame loop
 * and a frame is one iteration of flight ()'s loop without drawing.
 */

#define _GNU_SOURCE
#include <fenv.h>
#include <unistd.h>
#include <sys/stat.h>

#include "project.h"

#include "ai/ai_misc/ai_dbase.h"
#include "ai/faction/faction.h"
#include "ai/highlevl/setup.h"

#include "eech_engine.h"
#include "eech_headless.h"
#include "eech_synth3d.h"

extern void eech_run_exit_functions (void);

/* project.c: set from the registry's "Installation Path" in the game */
extern char comanche_hokum_installation_path[];

/* flight.c globals */
extern int game_update_time;

enum engine_state { ENGINE_NEW, ENGINE_RUNNING };

static enum engine_state state;

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* entry discipline: FPU environment and fatal-error unwinding */

#define ENTER(RESULT) \
	fenv_t caller_fenv; \
	jmp_buf fatal_jmp; \
	jmp_buf *saved_target = eech_fatal_target; \
	if (eech_engine_poisoned) \
	{ \
		return EECH_ENGINE_POISONED; \
	} \
	fegetenv (&caller_fenv); \
	fesetround (FE_TOWARDZERO); \
	eech_fatal_target = &fatal_jmp; \
	if (setjmp (fatal_jmp)) \
	{ \
		eech_fatal_target = saved_target; \
		fesetenv (&caller_fenv); \
		return EECH_ENGINE_FATAL; \
	}

#define LEAVE(RESULT) \
	eech_fatal_target = saved_target; \
	fesetenv (&caller_fenv); \
	return (RESULT)

const char *eech_engine_fatal_message (void)
{
	return eech_fatal_message;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */

static void boot (const struct eech_engine_config *config)
{
	char *argv[1024];
	int argc = 0;
	session_list_data_type *session;

	/* startup.c :: WinMain */
	GetCurrentDirectory (1024, application_current_directory);
	set_fpu_rounding_mode_zero ();
	initialise_event_system ();
	initialise_timers_system ();
	initialise_file_system ();

	/* project.c :: set_comanche_hokum_installation_path: the installation root */
	snprintf (comanche_hokum_installation_path, 1024, "%s", config->install_root);

	/* project.c :: application_main */
	argv[argc++] = "eech";
	if (config->arguments)
	{
		for (int i = 0; config->arguments[i] && argc < 63; i++)
		{
			argv[argc++] = (char *) config->arguments[i];
		}
	}
	argv[argc] = NULL;

	initialise_global_options_data ();
	load_global_options_data ();
	initialize_options ();
	process_command_line (argc, argv);
	process_ini_file ();
	process_command_line (argc, argv);
	debug_fatal_warning_tone = FALSE;
	initialise_debug_system (TRUE);

	/*
	 * The dedicated server: no player gunship is flown, the campaign runs as
	 * the server. It is switched on after the init screen, whose function
	 * would otherwise run the game phases, flight () included, itself.
	 */
	command_line_comms_dedicated_server = FALSE;
	command_line_game_initialisation_phase_game_type = GAME_TYPE_CAMPAIGN;
	command_line_game_initialisation_phase_gunship_type = config->gunship_type;
	snprintf (command_line_game_initialisation_phase_path, sizeof (command_line_game_initialisation_phase_path), "%s", config->map_path);
	snprintf (command_line_game_initialisation_phase_directory, sizeof (command_line_game_initialisation_phase_directory), "%s", config->campaign_directory);
	snprintf (command_line_game_initialisation_phase_filename, sizeof (command_line_game_initialisation_phase_filename), "%s", config->campaign_filename);

	/* project.c: the resolution the 3D and UI systems are sized for (no display) */
	set_global_3d_visual_screen_width (640.0);
	set_global_3d_visual_screen_height (480.0);

	/* project.c: graphics system and the video screen (memory surfaces, no display) */
	initialise_graphics_system (NULL);
	ddraw_set_display_resolution (get_global_3d_visual_screen_width (), get_global_3d_visual_screen_height ());
	initialise_graphics_rendering_system ();
	set_active_screen (video_screen);

	/* init.c :: brief_initialise_game, full_initialise_game */
	brief_initialise_game ();
	/* the title screen pushes the init screen, whose function runs full_initialise_game */
	push_ui_screen (init_screen);
	command_line_comms_dedicated_server = TRUE;

	set_random_number_seed (config->random_seed);

	/* gameflow.c :: process_game_initialisation_phases, dedicated server */
	set_comms_model (COMMS_MODEL_SERVER);
	set_game_type (GAME_TYPE_CAMPAIGN);
	set_global_gunship_type ((gunship_types) config->gunship_type);
	set_global_gunship_side (gunship_sides[get_global_gunship_type ()]);
	load_side_dependant_application_sound_samples (NUM_ENTITY_SIDES);

	session = (session_list_data_type *) malloc_heap_mem (sizeof (session_list_data_type));
	memset (session, 0, sizeof (session_list_data_type));
	session->title = (char *) malloc_heap_mem (strlen ("Dedicated Server") + 2);
	sprintf (session->title, "Dedicated Server");
	session->type = SESSION_LIST_TYPE_HOST;
	session->list_id = 1;
	session->type_id = 1;
	session->warzone_name = "Headless";
	snprintf (session->data_path, sizeof (session->data_path), "%s", config->map_path);
	snprintf (session->campaign_directory, sizeof (session->campaign_directory), "%s", config->campaign_directory);
	snprintf (session->campaign_filename, sizeof (session->campaign_filename), "%s", config->campaign_filename);
	set_current_game_session (session);

	/* GAME_TYPE */
	if (get_pack_buffer ())
	{
		close_pack_buffer ();
	}
	set_game_status (GAME_STATUS_UNINITIALISED);
	open_pack_buffer (tx_pack_buffer, command_line_comms_pack_buffer_size);
	reset_comms_data ();
	comms_clear_data_record ();

	/* SETUP (server) */
	set_game_status (GAME_STATUS_INITIALISING);
	initialise_group_callsign_database ();
	initialise_message_log ();
	initialise_sound_channels ();
	initialise_valid_warzone_bridge_database ();
	reset_delta_time ();
	if (command_line_wut)
	{
		parse_WUT_file (WUT_filename);
	}
	session_planner_goto_button = command_line_planner_goto_button;
	session_vector_flight_model = command_line_vector_flight_model;
	session_ground_radar_ignores_infantry = command_line_ground_radar_ignores_infantry;
	session_camcom = command_line_camcom;
	session_campaign_map_update_interval = command_line_campaign_map_update_interval;
	session_russian_nvg_no_ir = command_line_russian_nvg_no_ir;
	session_cloud_puffs = command_line_cloud_puffs;
	session_comms_packet_data_size = command_line_comms_packet_data_size;
	set_valid_combat_zone (TRUE);
	load_3d_terrain_game_data ();
	initialise_population_name_database ();
	reinitialise_entity_system ();

	/* GUNSHIP_TYPE (host) */
	create_campaign (get_current_game_session ());
	setup_campaign ();
	create_server_pilot ();
	process_host_session_setup_options ();
	initialise_regen_queues ();

	/* flight.c :: flight, up to the frame loop */
	set_game_exit_type (GAME_EXIT_INVALID);
	set_exit_flight_loop (FALSE);
	push_event_stop ();
	push_event (flight_events, "flight_events");
	initialise_views ();
	set_raw_in_flight_game_mode (IN_FLIGHT_GAME_MODE_PLANNER);
	start_ai_system ();
	start_ingame_ui_system ();
	game_update_time = TIME_1_SECOND / command_line_max_game_update_rate;
	set_raw_time_acceleration (get_min_time_acceleration ());
	reset_delta_time ();
	server_create_session ();
	set_game_status (GAME_STATUS_INITIALISED);
	reset_receive_packet_list_time_stamps ();
}

int eech_engine_boot (const struct eech_engine_config *config)
{
	ENTER (0);
	if (state != ENGINE_NEW)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!config || !config->install_root || !config->map_path || !config->campaign_directory || !config->campaign_filename)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	char cohokum[4096];
	snprintf (cohokum, sizeof (cohokum), "%s/cohokum", config->install_root);
	if (chdir (cohokum) != 0)
	{
		eech_log (0, "install root %s has no cohokum directory", config->install_root);
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	boot (config);
	state = ENGINE_RUNNING;
	LEAVE (EECH_ENGINE_OK);
}

/* flight.c :: flight, one iteration of the frame loop, without drawing */
static void frame (uint32_t milliseconds)
{
	int count = get_time_acceleration ();

	if (count == 0)
	{
		update_client_server_entity (get_camera_entity ());
	}
	else
	{
		while (count--)
		{
			receive_comms_data ();
			if ((!get_session_entity ()) || (get_game_exit_type () == GAME_EXIT_KICKOUT))
			{
				set_exit_flight_loop (TRUE);
				break;
			}
			update_client_server_entities ();
			reset_interpolation_timer ();
			update_update_functions ();
			/* flight.c :: update_in_flight_timers (static there) */
			update_speech_buffers ();
			update_mobile_sprite_light_timers ();
			update_message_log ();
		}
	}
	reset_local_entity_drawn_flags ();

	/* limit_frame_rate / set_delta_time: the host's clock */
	eech_headless_advance_time_ms (milliseconds);
	set_delta_time ();
	process_events ();
}

int eech_engine_frame (uint32_t milliseconds)
{
	ENTER (0);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	frame (milliseconds);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_prepare_installation (const char *root)
{
	ENTER (0);
	char path[4096];
	int ok;
	snprintf (path, sizeof (path), "%s/cohokum", root);
	mkdir (path, 0755);
	snprintf (path, sizeof (path), "%s/cohokum/3ddata", root);
	mkdir (path, 0755);
	ok = eech_synth3d_write (path);
	snprintf (path, sizeof (path), "%s/common", root);
	mkdir (path, 0755);
	snprintf (path, sizeof (path), "%s/common/data", root);
	mkdir (path, 0755);
	ok = ok && eech_synth_write_briefings (path);
	LEAVE (ok ? EECH_ENGINE_OK : EECH_ENGINE_BAD_ARGUMENT);
}

/* diagnostics: every keysite, with its side, type, name and landing types */
void eech_engine_debug_keysites (void)
{
	entity *force = get_session_entity () ? get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE) : NULL;
	while (force)
	{
		entity *ks = get_local_entity_first_child (force, LIST_TYPE_KEYSITE_FORCE);
		while (ks)
		{
			vec3d *p = get_local_entity_vec3d_ptr (ks, VEC3D_TYPE_POSITION);
			eech_log (1, "keysite %s side=%d type=%s landing_types=%x in_use=%d at %.0f,%.0f,%.0f",
				get_local_entity_string (ks, STRING_TYPE_KEYSITE_NAME), get_local_entity_int_value (ks, INT_TYPE_SIDE),
				entity_sub_type_keysite_names[get_local_entity_int_value (ks, INT_TYPE_ENTITY_SUB_TYPE)],
				get_local_entity_int_value (ks, INT_TYPE_LANDING_TYPES), get_local_entity_int_value (ks, INT_TYPE_IN_USE), p->x, p->y, p->z);
			ks = get_local_entity_child_succ (ks, LIST_TYPE_KEYSITE_FORCE);
		}
		force = get_local_entity_child_succ (force, LIST_TYPE_FORCE);
	}
}

/* eech_observe.c, behind the entry discipline */
extern int eech_observe_objects (eech_object_callback callback, void *user);
extern int eech_observe_clock (struct eech_clock *clock);

int eech_engine_objects (eech_object_callback callback, void *user, int *count)
{
	ENTER (0);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	*count = eech_observe_objects (callback, user);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_clock (struct eech_clock *clock)
{
	ENTER (0);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	int result = eech_observe_clock (clock);
	LEAVE (result);
}
