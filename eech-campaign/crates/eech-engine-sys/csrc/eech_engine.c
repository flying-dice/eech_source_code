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
enum engine_operation { OPERATION_NONE, OPERATION_BOOT, OPERATION_FRAME, OPERATION_OBSERVE, OPERATION_PREPARE };

static enum engine_state state;
static enum engine_operation operation;

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* entry discipline: FPU environment and fatal-error unwinding */

#define ENTER(OPERATION,HOST) \
	fenv_t caller_fenv; \
	jmp_buf fatal_jmp; \
	jmp_buf *saved_target = eech_fatal_target; \
	const struct eech_engine_host *entry_host = (HOST); \
	campaign_roster_host roster_host; \
	campaign_supply_arrival_host arrival_host; \
	campaign_movement_host movement_host; \
	campaign_los_host los_host; \
	campaign_sense_host sense_host; \
	campaign_aim_host aim_host; \
	campaign_terrain_host terrain_host; \
	if (eech_engine_poisoned) \
	{ \
		return EECH_ENGINE_POISONED; \
	} \
	if (operation != OPERATION_NONE) \
	{ \
		return EECH_ENGINE_STATE; \
	} \
	operation = (OPERATION); \
	roster_host.bind_member = entry_host ? entry_host->bind_mobile : NULL; \
	roster_host.observe_loss = entry_host ? entry_host->observe_loss : NULL; \
	roster_host.user = entry_host ? entry_host->user : NULL; \
	arrival_host.observe_reach = entry_host ? entry_host->supply_reached : NULL; \
	arrival_host.user = entry_host ? entry_host->user : NULL; \
	movement_host.move_vehicle = entry_host ? entry_host->move_vehicle : NULL; \
	movement_host.user = entry_host ? entry_host->user : NULL; \
	los_host.line_of_sight = entry_host ? entry_host->line_of_sight : NULL; \
	los_host.user = entry_host ? entry_host->user : NULL; \
	sense_host.sense = entry_host ? entry_host->sense : NULL; \
	sense_host.user = entry_host ? entry_host->user : NULL; \
	aim_host.aim = entry_host ? entry_host->aim : NULL; \
	aim_host.user = entry_host ? entry_host->user : NULL; \
	terrain_host.terrain = entry_host ? entry_host->terrain : NULL; \
	terrain_host.user = entry_host ? entry_host->user : NULL; \
	campaign_roster_set_host (entry_host ? &roster_host : NULL); \
	campaign_supply_arrival_set_host (entry_host ? &arrival_host : NULL); \
	campaign_movement_set_host (entry_host ? &movement_host : NULL); \
	campaign_los_set_host (entry_host ? &los_host : NULL); \
	campaign_sense_set_host (entry_host ? &sense_host : NULL); \
	campaign_aim_set_host (entry_host ? &aim_host : NULL); \
	/* the terrain host is installed by the first entry that carries one and kept for the process: LEAVE does not clear it */ \
	campaign_terrain_set_host (&terrain_host); \
	fegetenv (&caller_fenv); \
	fesetround (FE_TOWARDZERO); \
	eech_fatal_target = &fatal_jmp; \
	if (setjmp (fatal_jmp)) \
	{ \
		campaign_roster_set_host (NULL); \
		campaign_supply_arrival_set_host (NULL); \
		campaign_movement_set_host (NULL); \
		campaign_los_set_host (NULL); \
		campaign_sense_set_host (NULL); \
		campaign_aim_set_host (NULL); \
		operation = OPERATION_NONE; \
		eech_fatal_target = saved_target; \
		fesetenv (&caller_fenv); \
		return EECH_ENGINE_FATAL; \
	}

#define LEAVE(RESULT) \
	campaign_roster_set_host (NULL); \
	campaign_supply_arrival_set_host (NULL); \
	campaign_movement_set_host (NULL); \
	campaign_los_set_host (NULL); \
	campaign_sense_set_host (NULL); \
	campaign_aim_set_host (NULL); \
	operation = OPERATION_NONE; \
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
	/* the season is read by load_3d_terrain_game_data when it picks the terrain files (terrdata.c); native sets it earlier (sparm_sc.c notify_session_parameters) */
	if (config->winter)
	{
		set_global_season (SESSION_SEASON_WINTER);
	}
	load_3d_terrain_game_data ();
	/* EEDCS-47: the extents and heights the core reads are the host's, before anything reads them */
	campaign_terrain_install_map_facts ();
	initialise_population_name_database ();
	reinitialise_entity_system ();
	campaign_roster_reset ();
	campaign_supply_arrival_reset ();
	campaign_object_reset ();

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

int eech_engine_boot (const struct eech_engine_config *config, const struct eech_engine_host *host)
{
	ENTER (OPERATION_BOOT, host);
	if (state != ENGINE_NEW)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!config || !config->install_root || !config->map_path || !config->campaign_directory || !config->campaign_filename || !host || !host->bind_mobile || !host->observe_loss || !host->supply_reached || !host->move_vehicle || !host->line_of_sight || !host->sense || !host->aim || !host->terrain)
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
	/*
	 * EECH seeds its random numbers from the system clock when the session
	 * starts (session.c :: get_session_random_start_time_of_day), after any
	 * seed set here. The simulated clock's epoch is the seed's (seed 1: 0),
	 * so a run's randomness follows the seed as a real run's follows the
	 * wall clock.
	 */
	eech_headless_set_time_ms (((config->random_seed - 1u) % 1000000u) * 1000u);
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

int eech_engine_frame (uint32_t milliseconds, const struct eech_engine_host *host)
{
	ENTER (OPERATION_FRAME, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!host || !host->bind_mobile || !host->observe_loss || !host->supply_reached || !host->move_vehicle || !host->line_of_sight || !host->sense || !host->aim || !host->terrain)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	frame (milliseconds);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_report_loss (const char *host_id, struct eech_loss_report_receipt *receipt)
{
	campaign_loss_report_receipt source;
	ENTER (OPERATION_OBSERVE, NULL);
	if ((state != ENGINE_RUNNING) || !host_id || !host_id[0] || !receipt)
	{
		LEAVE (host_id && host_id[0] && receipt ? EECH_ENGINE_STATE : EECH_ENGINE_BAD_ARGUMENT);
	}
	campaign_roster_report_loss (host_id, &source);
	receipt->landing_reserved_before = source.landing_reserved_before;
	receipt->landing_reserved_after = source.landing_reserved_after;
	receipt->group_losses_before = source.group_losses_before;
	receipt->group_losses_after = source.group_losses_after;
	receipt->primary_task_losses_before = source.primary_task_losses_before;
	receipt->primary_task_losses_after = source.primary_task_losses_after;
	receipt->group_logical_members_before = source.group_logical_members_before;
	receipt->group_logical_members_after = source.group_logical_members_after;
	receipt->group_physical_members_before = source.group_physical_members_before;
	receipt->tasks_retired_expire = source.tasks_retired_expire;
	receipt->tasks_completed_on_report = source.tasks_completed_on_report;
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_report_object (const struct eech_object_report *report, struct eech_object_report_receipt *receipt, const struct eech_engine_host *host)
{
	campaign_object_report source;
	campaign_object_report_receipt applied;
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!report || !receipt || !host || !host->bind_mobile || !host->observe_loss || !host->supply_reached || !host->move_vehicle || !host->line_of_sight || !host->sense || !host->aim || !host->terrain)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	memset (&source, 0, sizeof (source));
	memcpy (source.host_id, report->host_id, sizeof (source.host_id));
	source.has_damage_level = report->has_damage_level;
	source.damage_level = report->damage_level;
	source.destroyed = report->destroyed;
	campaign_object_report_apply (&source, &applied);
	receipt->entity_type = applied.entity_type;
	receipt->rostered = applied.rostered;
	receipt->damage_level_before = applied.damage_level_before;
	receipt->damage_level_after = applied.damage_level_after;
	receipt->killed = applied.killed;
	LEAVE (EECH_ENGINE_OK);
}

struct structure_callback_context
{
	eech_structure_callback callback;
	void *user;
};

static void copy_structure (const campaign_object_snapshot *source, void *user)
{
	struct structure_callback_context *context = (struct structure_callback_context *) user;
	struct eech_structure structure;
	structure.host_id = source->host_id;
	switch (source->entity_type)
	{
	case ENTITY_TYPE_CITY_BUILDING: structure.kind = EECH_STRUCTURE_CITY_BUILDING; break;
	case ENTITY_TYPE_PYLON: structure.kind = EECH_STRUCTURE_PYLON; break;
	case ENTITY_TYPE_SCENIC: structure.kind = EECH_STRUCTURE_SCENIC; break;
	case ENTITY_TYPE_SEGMENT: structure.kind = EECH_STRUCTURE_SEGMENT; break;
	case ENTITY_TYPE_SITE: structure.kind = EECH_STRUCTURE_SITE; break;
	default: structure.kind = EECH_STRUCTURE_SITE_UPDATABLE; break;
	}
	structure.entity_type = source->entity_type;
	structure.sub_type = source->sub_type;
	structure.side = source->side;
	structure.alive = source->alive;
	structure.damage_level = source->damage_level;
	structure.initial_damage_level = source->initial_damage_level;
	memcpy (structure.position, source->position, sizeof (structure.position));
	context->callback (&structure, context->user);
}

int eech_engine_structures (eech_structure_callback callback, void *user, int *count, const struct eech_engine_host *host)
{
	struct structure_callback_context context;
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!callback || !count || !host || !host->bind_mobile || !host->observe_loss || !host->supply_reached || !host->move_vehicle || !host->line_of_sight || !host->sense || !host->aim || !host->terrain)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	context.callback = callback;
	context.user = user;
	*count = campaign_object_structures (copy_structure, &context);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_prepare_installation (const char *root)
{
	ENTER (OPERATION_PREPARE, NULL);
	char path[4096];
	int ok;
	snprintf (path, sizeof (path), "%s/cohokum", root);
	eech_mkdir (path);
	snprintf (path, sizeof (path), "%s/cohokum/3ddata", root);
	eech_mkdir (path);
	{
		/* a retail 3D database (its texture palette is retail only) is kept as it is */
		char retail[4200];
		struct stat st;
		snprintf (retail, sizeof (retail), "%s/textures.pal", path);
		ok = stat (retail, &st) == 0 ? 1 : eech_synth3d_write (path);
	}
	snprintf (path, sizeof (path), "%s/common", root);
	eech_mkdir (path);
	snprintf (path, sizeof (path), "%s/common/data", root);
	eech_mkdir (path);
	ok = ok && eech_synth_write_briefings (path);
	LEAVE (ok ? EECH_ENGINE_OK : EECH_ENGINE_BAD_ARGUMENT);
}

/* diagnostics: every keysite, with its side, type, name and landing types */
static void force_data_dump (entity *force_en)
{
	force *raw = (force *) get_local_entity_data (force_en);
	int side = get_local_entity_int_value (force_en, INT_TYPE_SIDE);
	char line[1024];
	int n = 0;
	for (int c = 0; c < NUM_FORCE_INFO_CATAGORIES; c++)
	{
		n += snprintf (line + n, sizeof (line) - (size_t) n, " %s=%d/%d", force_info_catagory_names[c], raw->force_info_current_hardware[c], raw->force_info_reserve_hardware[c]);
	}
	eech_log (1, "force side=%d current/reserve:%s", side, line);
	n = 0;
	for (int r = 0; r < NUM_ENTITY_SUB_TYPE_REGENS; r++)
	{
		n += snprintf (line + n, sizeof (line) - (size_t) n, " %s=%d", entity_sub_type_regen_names[r], regen_manager[side][r].count);
	}
	eech_log (1, "  regen queues:%s", line);
	for (entity *ks = get_local_entity_first_child (force_en, LIST_TYPE_KEYSITE_FORCE); ks; ks = get_local_entity_child_succ (ks, LIST_TYPE_KEYSITE_FORCE))
	{
		int regens = 0, regens_alive = 0;
		for (entity *rg = get_local_entity_first_child (ks, LIST_TYPE_REGEN); rg; rg = get_local_entity_child_succ (rg, LIST_TYPE_REGEN))
		{
			entity *building = get_local_entity_first_child (rg, LIST_TYPE_MEMBER);
			regens++;
			regens_alive += building && get_local_entity_int_value (building, INT_TYPE_ALIVE);
		}
		eech_log (1, "  keysite %s %s usable=%s efficiency=%.2f ammo=%.1f fuel=%.1f regen sites %d (%d alive)",
			get_local_entity_string (ks, STRING_TYPE_KEYSITE_NAME), entity_sub_type_keysite_names[get_local_entity_int_value (ks, INT_TYPE_ENTITY_SUB_TYPE)],
			keysite_usable_state_names[get_local_entity_int_value (ks, INT_TYPE_KEYSITE_USABLE_STATE)], get_local_entity_float_value (ks, FLOAT_TYPE_EFFICIENCY),
			get_local_entity_float_value (ks, FLOAT_TYPE_AMMO_SUPPLY_LEVEL), get_local_entity_float_value (ks, FLOAT_TYPE_FUEL_SUPPLY_LEVEL), regens, regens_alive);
		for (entity *grp = get_local_entity_first_child (ks, LIST_TYPE_KEYSITE_GROUP); grp; grp = get_local_entity_child_succ (grp, LIST_TYPE_KEYSITE_GROUP))
		{
			int type = get_local_entity_int_value (grp, INT_TYPE_ENTITY_SUB_TYPE);
			entity *task = get_local_group_primary_task (grp);
			entity *member = get_local_entity_first_child (grp, LIST_TYPE_MEMBER);
			if (!group_database[type].default_entity_type || (group_database[type].default_entity_type != ENTITY_TYPE_HELICOPTER && group_database[type].default_entity_type != ENTITY_TYPE_FIXED_WING))
			{
				continue;
			}
			eech_log (1, "    group %s %s mode=%d sleep=%.0f members=%d state=%s task=%s ammo=%.0f fuel=%.0f first=%s",
				get_local_entity_string (grp, STRING_TYPE_GROUP_CALLSIGN), entity_sub_type_group_names[type],
				get_local_entity_int_value (grp, INT_TYPE_GROUP_MODE), ((group *) get_local_entity_data (grp))->sleep,
				get_local_entity_int_value (grp, INT_TYPE_MEMBER_COUNT),
				verbose_operational_state_names[get_local_entity_int_value (grp, INT_TYPE_VERBOSE_OPERATIONAL_STATE)],
				task ? entity_sub_type_task_names[get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE)] : "-",
				get_local_entity_float_value (grp, FLOAT_TYPE_AMMO_SUPPLY_LEVEL), get_local_entity_float_value (grp, FLOAT_TYPE_FUEL_SUPPLY_LEVEL),
				member ? get_local_entity_string (member, STRING_TYPE_FULL_NAME) : "-");
		}
	}
}

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

int eech_engine_diagnostics (void)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	for (entity *force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE); force; force = get_local_entity_child_succ (force, LIST_TYPE_FORCE))
	{
		force_data_dump (force);
	}
	LEAVE (EECH_ENGINE_OK);
}

/* eech_observe.c, behind the entry discipline */
extern int eech_observe_objects (eech_object_callback callback, void *user);
extern int eech_observe_clock (struct eech_clock *clock);

struct member_callback_context
{
	eech_member_callback callback;
	void *user;
};

int eech_engine_objects (eech_object_callback callback, void *user, int *count)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	*count = eech_observe_objects (callback, user);
	LEAVE (EECH_ENGINE_OK);
}

static void copy_member (const campaign_member_snapshot *source, void *user)
{
	struct member_callback_context *context = (struct member_callback_context *) user;
	struct eech_member member;
	member.member_key = source->member_key;
	member.group_key = source->group_key;
	member.host_id = source->host_id;
	member.origin_entity_id = source->origin_entity_index;
	member.current_entity_id = source->current_entity_index;
	member.entity_type = source->entity_type;
	member.sub_type = source->sub_type;
	member.side = source->side;
	member.initial_alive = source->initial_alive;
	member.physical_present = source->physical_present;
	member.in_group = source->in_group;
	context->callback (&member, context->user);
}

int eech_engine_members (eech_member_callback callback, void *user, int *count)
{
	struct member_callback_context context;
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!callback || !count)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	context.callback = callback;
	context.user = user;
	*count = campaign_roster_members (copy_member, &context);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_roster_stats (struct eech_roster_stats *stats)
{
	campaign_roster_stats source;
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!stats)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	campaign_roster_get_stats (&source);
	stats->duplicate_comparisons = source.duplicate_comparisons;
	stats->duplicate_scan_seconds = source.duplicate_scan_seconds;
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_supply_tasks (eech_supply_task_callback callback, void *user, int *count)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!callback || !count)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	*count = campaign_supply_task_snapshots ((campaign_supply_task_snapshot_fn) callback, user);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_clock (struct eech_clock *clock)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	int result = eech_observe_clock (clock);
	LEAVE (result);
}

/* diagnostics: air groups, their tasks and states */
void eech_engine_debug_air_groups (void)
{
	entity *force = get_session_entity () ? get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE) : NULL;
	for (; force; force = get_local_entity_child_succ (force, LIST_TYPE_FORCE))
	{
		entity *group = get_local_entity_first_child (force, LIST_TYPE_AIR_REGISTRY);
		for (; group; group = get_local_entity_child_succ (group, LIST_TYPE_AIR_REGISTRY))
		{
			entity *task = get_local_group_primary_task (group);
			entity *member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);
			eech_log (1, "group %s side=%d %s state=%s task=%s members=%d first=%s op=%s",
				get_local_entity_string (group, STRING_TYPE_GROUP_CALLSIGN), get_local_entity_int_value (group, INT_TYPE_SIDE),
				entity_sub_type_group_names[get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)],
				verbose_operational_state_names[get_local_entity_int_value (group, INT_TYPE_VERBOSE_OPERATIONAL_STATE)],
				task ? entity_sub_type_task_names[get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE)] : "-",
				get_local_entity_int_value (group, INT_TYPE_MEMBER_COUNT),
				member ? aircraft_database[get_local_entity_int_value (member, INT_TYPE_ENTITY_SUB_TYPE)].full_name : "-",
				member ? operational_state_names[get_local_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE)] : "-");
		}
	}
}

/*
 * diagnostics: for every airborne tasked aircraft, the nearest enemy ground
 * unit, and EECH's weapon choice for it with and without range and LOS checks
 */
void eech_engine_debug_engagements (void)
{
	entity *en, *other;
	for (en = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE); en; en = get_local_entity_child_succ (en, LIST_TYPE_UPDATE))
	{
		int type = get_local_entity_type (en), side;
		entity *group, *task, *best = NULL;
		float best_d = 1e9f;
		vec3d *p;
		if (type != ENTITY_TYPE_HELICOPTER && type != ENTITY_TYPE_FIXED_WING)
		{
			continue;
		}
		if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_LANDED)
		{
			continue;
		}
		group = get_local_entity_parent (en, LIST_TYPE_MEMBER);
		task = group ? get_local_group_primary_task (group) : NULL;
		side = get_local_entity_int_value (en, INT_TYPE_SIDE);
		p = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
		for (other = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE); other; other = get_local_entity_child_succ (other, LIST_TYPE_UPDATE))
		{
			int ot = get_local_entity_type (other);
			if ((ot == ENTITY_TYPE_ROUTED_VEHICLE || ot == ENTITY_TYPE_ANTI_AIRCRAFT) && get_local_entity_int_value (other, INT_TYPE_SIDE) != side
				&& get_local_entity_int_value (other, INT_TYPE_ALIVE))
			{
				float d = get_2d_range (p, get_local_entity_vec3d_ptr (other, VEC3D_TYPE_POSITION));
				if (d < best_d)
				{
					best_d = d;
					best = other;
				}
			}
		}
		if (!best)
		{
			continue;
		}
		{
			entity_sub_types w0 = get_best_weapon_for_target (en, best, BEST_WEAPON_CRITERIA_MINIMAL);
			entity_sub_types w1 = get_best_weapon_for_target (en, best, BEST_WEAPON_RANGE_CHECK);
			entity_sub_types w2 = get_best_weapon_for_target (en, best, BEST_WEAPON_RANGE_CHECK | BEST_WEAPON_LOS_CHECK);
			int los = check_entity_line_of_sight (en, best, MOBILE_LOS_CHECK_ALL);
			eech_log (1, "%s side=%d task=%s state=%s alt=%.0f nearest enemy %s at %.0f m: best weapon minimal=%s range=%s range+los=%s los=%d target=%s",
				aircraft_database[get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE)].full_name, side,
				task ? entity_sub_type_task_names[get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE)] : "-",
				group ? verbose_operational_state_names[get_local_entity_int_value (group, INT_TYPE_VERBOSE_OPERATIONAL_STATE)] : "-",
				p->y - get_3d_terrain_elevation (p->x, p->z),
				vehicle_database[get_local_entity_int_value (best, INT_TYPE_ENTITY_SUB_TYPE)].full_name, best_d,
				weapon_database[w0].full_name, weapon_database[w1].full_name, weapon_database[w2].full_name, los,
				get_local_entity_parent (en, LIST_TYPE_TARGET) ? "yes" : "none");
		}
	}
}

#if defined(EECH_DEPENDENT_FIXTURE) || defined(EECH_LOSS_FIXTURE)

static entity *fixture_hosted_entity (int index)
{
	entity *en;

	if ((index < 0) || (index >= MAX_NUM_ENTITIES))
	{
		return NULL;
	}
	en = get_local_entity_ptr (index);
	if (!en || (get_local_entity_type (en) == ENTITY_TYPE_UNKNOWN) || !campaign_roster_host_id (en))
	{
		return NULL;
	}
	return en;
}

#endif

#ifdef EECH_DEPENDENT_FIXTURE

/* ---------------------------------------------------------------------------------------------------------------------------- */
/*
 * TEST-ONLY dependent-positioning fixture (EEDCS-40). Compiled only with the `dependent-fixture` feature; never part of the
 * shipping module. It constructs no entity and replaces no handler: it names two live, hosted campaign entities by index and
 * either re-links one as a movement dependent of the other (the same list insert mb_msgs.c uses for cargo and keysite
 * receivers) or moves the parent through the campaign's own setter, so EECH's genuine notify_local_entity handlers (rv_msgs.c,
 * aa_msgs.c, ps_msgs.c, hc_msgs.c) run and call campaign_movement_dependent_carry themselves.
 */

int eech_engine_dependent_fixture (int op, int parent_id, int child_id, const double position[3], const struct eech_engine_host *host)
{
	entity *parent, *child;
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!host || !host->bind_mobile || !host->observe_loss || !host->supply_reached || !host->move_vehicle || !host->line_of_sight || !host->sense || !host->aim || !host->terrain)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	parent = fixture_hosted_entity (parent_id);
	if (!parent)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	switch (op)
	{
	case EECH_FIXTURE_LINK:
		child = fixture_hosted_entity (child_id);
		if (!child || (child == parent))
		{
			LEAVE (EECH_ENGINE_BAD_ARGUMENT);
		}
		/* the child's current movement parent (a keysite or carrier, if any) is replaced */
		delete_local_entity_from_parents_child_list (child, LIST_TYPE_MOVEMENT_DEPENDENT);
		insert_local_entity_into_parents_child_list (child, LIST_TYPE_MOVEMENT_DEPENDENT, parent, NULL);
		break;
	case EECH_FIXTURE_MOVE:
		if (!position)
		{
			LEAVE (EECH_ENGINE_BAD_ARGUMENT);
		}
		{
			vec3d v;
			v.x = position[0];
			v.y = position[1];
			v.z = position[2];
			/* the genuine setter: notifies each movement dependent, then writes and places the parent */
			set_local_entity_vec3d (parent, VEC3D_TYPE_POSITION, &v);
		}
		break;
	case EECH_FIXTURE_SINK:
		if (get_local_entity_type (parent) != ENTITY_TYPE_SHIP_VEHICLE)
		{
			LEAVE (EECH_ENGINE_BAD_ARGUMENT);
		}
		/* the campaign's own sinking step: the host answers it, and the position it applies reaches the setter */
		ship_vehicle_death_movement (parent);
		break;
	default:
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

#endif

#ifdef EECH_LOSS_FIXTURE

/* ---------------------------------------------------------------------------------------------------------------------------- */
/*
 * TEST-ONLY all-family loss trigger (EEDCS-41). Compiled only with the `loss-fixture` feature; never part of the shipping
 * module. It names one live, hosted campaign member by index and ends it through the campaign's own kill_client_server_entity,
 * so the genuine kill_local handler of its family runs, asks the host observe_loss callback, and either applies the
 * consequences inline (forward) or leaves them pending for engine:report_loss (withhold). It constructs no entity and replaces
 * no handler. It stands in only for the combat that would otherwise end the member at a time the scenario cannot choose.
 */
int eech_engine_loss_fixture (int entity_id, const struct eech_engine_host *host)
{
	entity *en;
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!host || !host->bind_mobile || !host->observe_loss || !host->supply_reached || !host->move_vehicle || !host->line_of_sight || !host->sense || !host->aim || !host->terrain)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	en = fixture_hosted_entity (entity_id);
	if (!en)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	kill_client_server_entity (en);
	LEAVE (EECH_ENGINE_OK);
}

/*
 * TEST-ONLY setup step (EEDCS-41): ends the engagements that keep one hosted member's group out of the plain shape, using only
 * the campaign's own lifecycle operations, without ending the group.  (1) Tasks that target the group, and then those that
 * target the member itself, are notified the way group.c does when it kills the group (ENTITY_MESSAGE_TASK_COMPLETED with
 * TASK_TERMINATED_OBJECTIVE_MESSAGE; the task's own handlers end and unlink it).  (2) Engage guides, which hold a per-member valid-member mask and are what the bounded WITHHOLD
 * counts as restricted guides on a fixed-wing or ship group in flight, are ended the way the campaign ends an engagement:
 * terminate_all_engage_tasks (group) for the not yet assigned ones, as when the group is sent home or lands, and
 * terminate_entity_current_engage_task (member) for each member's own, as when a wingman is told to break off
 * (msg_in.c) and the member is reassigned to a valid task.  Takeoff guides are never touched: they belong to a member that has
 * not completed takeoff, which the guard refuses for its own reasons.  It changes no guard and no physics, constructs nothing,
 * and `*released` is the number of tasks notified plus the number of guides that left the group's guide stack.
 */
int eech_engine_loss_release (int entity_id, int *released, const struct eech_engine_host *host)
{
	entity *en, *group, *task, *next;
	int n = 0;
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!released || !host || !host->bind_mobile || !host->observe_loss || !host->supply_reached || !host->move_vehicle || !host->line_of_sight || !host->sense || !host->aim || !host->terrain)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	en = fixture_hosted_entity (entity_id);
	group = en ? get_local_entity_parent (en, LIST_TYPE_MEMBER) : NULL;
	if (!group)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	task = get_local_entity_first_child (group, LIST_TYPE_TASK_DEPENDENT);
	while (task)
	{
		next = get_local_entity_child_succ (task, LIST_TYPE_TASK_DEPENDENT);
		if (task->type == ENTITY_TYPE_TASK)
		{
			notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, task, group, TASK_TERMINATED_OBJECTIVE_MESSAGE);
			n++;
		}
		task = next;
	}
	task = get_local_entity_first_child (en, LIST_TYPE_TASK_DEPENDENT);
	while (task)
	{
		next = get_local_entity_child_succ (task, LIST_TYPE_TASK_DEPENDENT);
		if (task->type == ENTITY_TYPE_TASK)
		{
			notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, task, en, TASK_TERMINATED_OBJECTIVE_MESSAGE);
			n++;
		}
		task = next;
	}
	{
		entity *guide, *member;
		int before = 0, after = 0, round;
		for (guide = get_local_entity_first_child (group, LIST_TYPE_GUIDE_STACK); guide; guide = get_local_entity_child_succ (guide, LIST_TYPE_GUIDE_STACK))
		{
			before++;
		}
		terminate_all_engage_tasks (group);
		/* reassignment can give a member another engage guide, so repeat until a pass ends none (bounded) */
		for (round = 0; round < 8; round++)
		{
			int ended = 0;
			member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);
			while (member)
			{
				entity *next_member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
				entity *current = get_local_entity_parent (member, LIST_TYPE_FOLLOWER);
				entity *owner = current ? get_local_entity_parent (current, LIST_TYPE_GUIDE) : NULL;
				if (owner && (get_local_entity_int_value (owner, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_ENGAGE))
				{
					terminate_entity_current_engage_task (member);
					ended++;
				}
				member = next_member;
			}
			if (!ended)
			{
				break;
			}
		}
		terminate_all_engage_tasks (group);
		for (guide = get_local_entity_first_child (group, LIST_TYPE_GUIDE_STACK); guide; guide = get_local_entity_child_succ (guide, LIST_TYPE_GUIDE_STACK))
		{
			after++;
		}
		if (before > after)
		{
			n += before - after;
		}
	}
	*released = n;
	LEAVE (EECH_ENGINE_OK);
}

/*
 * TEST-ONLY read-only eligibility probe (EEDCS-41): runs, without ending anything, the guard the bounded WITHHOLD itself runs on
 * the descriptor the host would be shown, and returns its refusal text ("" when the loss would be withheld).  It lets the witness
 * choose a supported member instead of spending real kills on unsupported ones.
 */
int eech_engine_loss_shape (int entity_id, char *reason, int capacity)
{
	entity *en;
	char buffer[256];
	const char *refusal;
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!reason || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	en = fixture_hosted_entity (entity_id);
	if (!en)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	refusal = campaign_roster_withhold_refusal (en, buffer, sizeof (buffer));
	strncpy (reason, refusal, (size_t) capacity - 1);
	reason[capacity - 1] = ' ';
	LEAVE (EECH_ENGINE_OK);
}

/*
 * TEST-ONLY read-only diagnostic (EEDCS-41): describes why a hosted member's group is, or is not, a plain shape for the bounded
 * WITHHOLD: the member's group number and task dependents of the group (entity type and sub type of each) and each guide on its
 * guide stack (valid-member mask, 1 when it is not "all members", and the sub type of the task owning it).  Nothing is modified.
 */
int eech_engine_loss_group (int entity_id, char *text, int capacity)
{
	entity *en, *group, *child;
	char line[96];
	int used = 0;
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	en = fixture_hosted_entity (entity_id);
	group = en ? get_local_entity_parent (en, LIST_TYPE_MEMBER) : NULL;
	if (!group)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	text[0] = 0;
#define LOSS_GROUP_APPEND(...) \
	do \
	{ \
		int n_ = snprintf (line, sizeof (line), __VA_ARGS__); \
		if ((n_ > 0) && (used + n_ < capacity - 1)) \
		{ \
			memcpy (text + used, line, (size_t) n_ + 1); \
			used += n_; \
		} \
	} while (0)
	LOSS_GROUP_APPEND ("member_number=%d", get_local_entity_int_value (en, INT_TYPE_GROUP_MEMBER_NUMBER));
	child = get_local_entity_first_child (group, LIST_TYPE_TASK_DEPENDENT);
	while (child)
	{
		LOSS_GROUP_APPEND (" dep=%d/%d", (int) child->type, get_local_entity_int_value (child, INT_TYPE_ENTITY_SUB_TYPE));
		child = get_local_entity_child_succ (child, LIST_TYPE_TASK_DEPENDENT);
	}
	child = get_local_entity_first_child (group, LIST_TYPE_GUIDE_STACK);
	while (child)
	{
		entity *task = get_local_entity_parent (child, LIST_TYPE_GUIDE);
		unsigned int mask = (unsigned int) get_local_entity_int_value (child, INT_TYPE_VALID_GUIDE_MEMBERS);
		LOSS_GROUP_APPEND (" guide=task%d,mask%08x%s", task ? get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) : -1, mask,
			(mask == 0xffffffffu) ? "" : ",restricted");
		child = get_local_entity_child_succ (child, LIST_TYPE_GUIDE_STACK);
	}
#undef LOSS_GROUP_APPEND
	LEAVE (EECH_ENGINE_OK);
}

/*
 * TEST-ONLY read-only view of one side's force-info and regeneration state (EEDCS-41), the state the force-info and regen loss
 * phases change: values[0] is the force's total losses, values[1] and values[2] the number of force-info categories (C) and of
 * regen queues (R), then the current hardware of each category (C values), then for each queue in turn its count (R), its front
 * (R), its size (R) and the type, sub type and group of the slot written last (3 x R, -1 when the queue is empty).  The queues are
 * rings: regen_queue_insert writes at (front + count) % size and, at capacity, advances the front and leaves the count, so the
 * count alone cannot show an insertion.  It reads the arrays the phases write; nothing is modified.
 */
int eech_engine_loss_state (int side, int *values, int capacity, int *count)
{
	entity *force_en;
	force *raw;
	int n = 0;
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!values || !count || side <= 0 || side >= NUM_ENTITY_SIDES || capacity < 3 + NUM_FORCE_INFO_CATAGORIES + 6 * NUM_ENTITY_SUB_TYPE_REGENS)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	force_en = get_local_force_entity ((entity_sides) side);
	if (!force_en)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	raw = (force *) get_local_entity_data (force_en);
	values[n] = 0;
	for (int g = 0; g < NUM_ENTITY_SUB_TYPE_GROUPS; g++)
	{
		values[n] += raw->losses[g];
	}
	n++;
	values[n++] = NUM_FORCE_INFO_CATAGORIES;
	values[n++] = NUM_ENTITY_SUB_TYPE_REGENS;
	for (int c = 0; c < NUM_FORCE_INFO_CATAGORIES; c++)
	{
		values[n++] = raw->force_info_current_hardware[c];
	}
	for (int r = 0; r < NUM_ENTITY_SUB_TYPE_REGENS; r++)
	{
		values[n++] = regen_manager[side][r].count;
	}
	for (int r = 0; r < NUM_ENTITY_SUB_TYPE_REGENS; r++)
	{
		values[n++] = regen_manager[side][r].front;
	}
	for (int r = 0; r < NUM_ENTITY_SUB_TYPE_REGENS; r++)
	{
		values[n++] = regen_manager[side][r].size;
	}
	for (int r = 0; r < NUM_ENTITY_SUB_TYPE_REGENS; r++)
	{
		const regen_management_element *m = &regen_manager[side][r];
		const regen_list_element *last = (m->count > 0 && m->size > 0 && regen_queue[side][r]) ? &regen_queue[side][r][(m->front + m->count - 1) % m->size] : NULL;
		values[n++] = last ? last->type : -1;
		values[n++] = last ? last->sub_type : -1;
		values[n++] = last ? last->group : -1;
	}
	*count = n;
	LEAVE (EECH_ENGINE_OK);
}

#endif

#ifdef EECH_LOS_FIXTURE

/* ---------------------------------------------------------------------------------------------------------------------------- */
/*
 * TEST-ONLY line-of-sight characterisation entries (EEDCS-42). Compiled only with the `los-fixture` feature; never part of the
 * shipping module. They read the run-wide trace the recorder in mb_tgt.c keeps, clear it, or run the deterministic probe sweep
 * of eech_los_fixture.c between two live entities. The probe runs inside the engine entry so it sees EECH's round-toward-zero
 * mode, as the campaign's own queries do. None of them constructs an entity or replaces a handler.
 */
extern void eech_los_trace_get (struct eech_los_trace *out);
extern void eech_los_trace_clear (void);
extern int eech_los_probe_run (int source_id, int target_id, uint32_t seed, int count, struct eech_los_trace *out);

int eech_engine_los_trace (struct eech_los_trace *out)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!out)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	eech_los_trace_get (out);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_los_trace_reset (void)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	eech_los_trace_clear ();
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_los_probe (int source_id, int target_id, uint32_t seed, int count, struct eech_los_trace *out, const struct eech_engine_host *host)
{
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!out || !host || !host->line_of_sight || (count < 0) || (count > 100000))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_los_probe_run (source_id, target_id, seed, count, out) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

#endif

#ifdef EECH_SENSOR_FIXTURE

/*
 * TEST-ONLY sensor-detection characterisation (EEDCS-43): the run-wide trace of the AI target-scan group functions and a read-only
 * probe that runs them for every eligible live source (eech_sensor_fixture.c). Both run inside the engine entry, so they see EECH's
 * round-toward-zero mode as the campaign's own scans do. None of them constructs an entity, engages a target or replaces a handler.
 * The report is text, `key=value` lines; nothing in it is an entity index or address.
 */
extern int eech_sensor_trace_get (char *text, int capacity);
extern void eech_sensor_trace_clear (void);
extern int eech_sensor_probe_run (char *text, int capacity);

int eech_engine_sensor_trace (char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_sensor_trace_get (text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_sensor_trace_reset (void)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	eech_sensor_trace_clear ();
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_sensor_probe (char *text, int capacity, const struct eech_engine_host *host)
{
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	/* the surface-to-air scan reaches the host-owned line of sight (EEDCS-42), so the probe needs the same complete host as a frame */
	if (!text || (capacity <= 0) || !host || !host->bind_mobile || !host->observe_loss || !host->supply_reached || !host->move_vehicle || !host->line_of_sight || !host->sense || !host->aim || !host->terrain)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_sensor_probe_run (text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

#endif

#ifdef EECH_TERRAIN_FIXTURE

/*
 * TEST-ONLY terrain surface characterisation (EEDCS-47): the run-wide trace of every terrain query the campaign's own update loop makes,
 * and a read-only, deterministic probe sweep over the loaded map (eech_terrain_fixture.c). Both run inside the engine entry, so they see
 * EECH's round-toward-zero mode as the campaign's own queries do. None of them constructs an entity or replaces a handler, and the probe
 * restores the shared find-fast record. The report is text, `key=value` lines; nothing in it is an entity index or an address.
 */
extern int eech_terrain_trace_get (char *text, int capacity);
extern void eech_terrain_trace_clear (void);
extern int eech_terrain_probe_run (uint32_t seed, int count, int emit, char *text, int capacity);

int eech_engine_terrain_trace (char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_terrain_trace_get (text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_terrain_trace_reset (void)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	eech_terrain_trace_clear ();
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_terrain_probe (uint32_t seed, int count, int emit, char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0) || (count < 0) || (count > 200000) || (emit < 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_terrain_probe_run (seed, count, emit, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

extern int eech_terrain_face_probe_run (float x, float z, char *text, int capacity);

int eech_engine_terrain_face_probe (float x, float z, char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_terrain_face_probe_run (x, z, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

#endif

#ifdef EECH_AIM_FIXTURE

/*
 * TEST-ONLY physical weapon-aiming characterisation (EEDCS-44): the run-wide trace of the aiming code the campaign's own update loop runs
 * (eech_aim_fixture.c). Reading or clearing it runs inside the engine entry, as the game's own calls do. There is no probe: nothing
 * here calls an aiming function. The report is text, `key=value` lines; nothing in it is an entity index, address or seed.
 */
extern int eech_aim_trace_get (char *text, int capacity);
extern void eech_aim_trace_clear (void);

int eech_engine_aim_trace (char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_aim_trace_get (text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_aim_trace_reset (void)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	eech_aim_trace_clear ();
	LEAVE (EECH_ENGINE_OK);
}

extern int eech_aim_person_query_run (const char *host_id, char *text, int capacity);
extern int eech_aim_person_case_run (const char *person_id, const char *target_id, char *text, int capacity);

/* controlled person case (EEDCS-44): the campaign's own target link set on one host-identified person, and a read of one entity's pose */
int eech_engine_aim_person_query (const char *host_id, char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!host_id || !text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_aim_person_query_run (host_id, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

extern int eech_aim_lead_probe_run (const char *source_id, const char *target_id, int weapon, float sentinel, char *text, int capacity);

int eech_engine_aim_lead_probe (const char *source_id, const char *target_id, int weapon, float sentinel, char *text, int capacity, const struct eech_engine_host *host)
{
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!source_id || !target_id || !text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_aim_lead_probe_run (source_id, target_id, weapon, sentinel, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_aim_person_case (const char *person_id, const char *target_id, char *text, int capacity, const struct eech_engine_host *host)
{
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!person_id || !target_id || !text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_aim_person_case_run (person_id, target_id, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

#endif

#ifdef EECH_WEAPON_FIXTURE

/*
 * TEST-ONLY physical weapon-execution characterisation (EEDCS-45): the run-wide trace of the weapon-creation, flight, update and contact code the campaign's own
 * update loop runs (eech_wpn_fixture.c), the ordered event log, and the controlled launch case. Reading runs inside the engine entry, as the game's own calls do.
 * Nothing here calls a weapon function except the controlled case, which calls launch_client_server_weapon once, as the family fire code does.
 */
extern int eech_cmb_trace_get (char *text, int capacity);
extern int eech_cmb_case_census_run (char *text, int capacity);
extern int eech_cmb_case_health_run (const char *kind, int ordinal, int percent, char *text, int capacity);
extern int eech_cmb_case_bridge_run (int op, int ordinal, int index, int criterion, char *text, int capacity);
extern int eech_cmb_case_collide_run (const char *kind, int ordinal, const char *launcher_id, int weapon, double modifier, int weaponless, char *text, int capacity);
extern int eech_cmb_case_blast_run (const char *kind, int ordinal, double offset, int kill_code, const char *launcher_id, int weapon, char *text, int capacity);
extern int eech_cmb_events_get (char *text, int capacity);
extern int eech_wpn_trace_get (char *text, int capacity);
extern int eech_wpn_events_get (char *text, int capacity);
extern int eech_wpn_case_query_run (const char *host_id, int weapon, char *text, int capacity);
extern int eech_wpn_case_kind_run (const char *host_id, char *text, int capacity);
extern int eech_wpn_case_launch_run (const char *launcher_id, const char *target_id, int weapon, int salvo_count, char *text, int capacity);

int eech_engine_cmb_case_state (int op, const char *kind, int ordinal, int percent, char *text, int capacity, const struct eech_engine_host *host)
{
	int rc;

	/* the health case sets a health through the campaign's own setter and asks the critical predicate, both physical questions the host answers: it runs in the host's scope */
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!kind || !text || (capacity <= 0) || (op < 0) || (op > 1))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	rc = (op == 0) ? eech_cmb_case_census_run (text, capacity) : eech_cmb_case_health_run (kind, ordinal, percent, text, capacity);
	if (rc != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_cmb_case_act (int op, const char *kind, int ordinal, const char *launcher_id, int weapon, double amount, int kill_code, int weaponless, char *text, int capacity, const struct eech_engine_host *host)
{
	int rc;

	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!kind || !launcher_id || !text || (capacity <= 0) || (op < 1) || (op > 2))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	rc = (op == 1) ? eech_cmb_case_collide_run (kind, ordinal, launcher_id, weapon, amount, weaponless, text, capacity) : eech_cmb_case_blast_run (kind, ordinal, amount, kill_code, launcher_id, weapon, text, capacity);
	if (rc != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

/* The controlled bridge case (EEDCS-46, R5; see eech_cmb_case.c): the survey and the state of the bridges, the declared structural failure of one segment, and the disarming of the observer.  Given the
   same complete host as a frame, because the failure runs the campaign's own kill, explosion effects and consequences. */
int eech_engine_cmb_case_bridge (int op, int ordinal, int index, int criterion, char *text, int capacity, const struct eech_engine_host *host)
{
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0) || (op < 0) || (op > 3))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_cmb_case_bridge_run (op, ordinal, index, criterion, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_cmb_trace (char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_cmb_trace_get (text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

/* returns EECH_ENGINE_OK with `more` set to 1 while undrained events remain */
int eech_engine_cmb_events (char *text, int capacity, int *more)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0) || !more)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	*more = eech_cmb_events_get (text, capacity);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_wpn_trace (char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_wpn_trace_get (text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

/* returns EECH_ENGINE_OK with `more` set to 1 while undrained events remain */
int eech_engine_wpn_events (char *text, int capacity, int *more)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0) || !more)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	*more = eech_wpn_events_get (text, capacity);
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_wpn_case_query (const char *host_id, int weapon, char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!host_id || !text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_wpn_case_query_run (host_id, weapon, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_wpn_case_kind (const char *host_id, char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!host_id || !text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_wpn_case_kind_run (host_id, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

int eech_engine_wpn_case_launch (const char *launcher_id, const char *target_id, int weapon, int salvo_count, char *text, int capacity, const struct eech_engine_host *host)
{
	ENTER (OPERATION_OBSERVE, host);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!launcher_id || !target_id || !text || (capacity <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_wpn_case_launch_run (launcher_id, target_id, weapon, salvo_count, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

#endif

#ifdef EECH_GROUND_FIXTURE

/*
 * TEST-ONLY ground-contact characterisation (EEDCS-48): one read-only probe of the original point_above_ground, point_below_ground and
 * get_ground_impact_point against the loaded terrain (eech_ground_fixture.c). It runs inside the engine entry, so it sees EECH's
 * round-toward-zero mode as the campaign's own queries do, and it takes no host argument: none of the three asks a dialogue host. (In the host-terrain module their terrain reads go to the terrain provider that boot installed for the process.) It snapshots the shared
 * terrain point record and writes it back byte for byte. The report is text, `key=value` lines; no pointer, address or entity index is in it.
 * A request the probe refuses (a precondition) or whose report does not fit writes `ground-refused reason=...` and is a bad argument.
 */
extern int eech_ground_probe_run (uint32_t seed, int points, int impacts, char *text, int capacity);

int eech_engine_ground_probe (uint32_t seed, int points, int impacts, char *text, int capacity)
{
	ENTER (OPERATION_OBSERVE, NULL);
	if (state != ENGINE_RUNNING)
	{
		LEAVE (EECH_ENGINE_STATE);
	}
	if (!text || (capacity <= 0) || (points < 0) || (points > 100000) || (impacts < 0) || (impacts > 100000) || (points + impacts <= 0))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (eech_ground_probe_run (seed, points, impacts, text, capacity) != 0)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	LEAVE (EECH_ENGINE_OK);
}

#endif
