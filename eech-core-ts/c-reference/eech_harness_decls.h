/*
 * Declarations the harness adds after the original headers.
 */

#ifndef EECH_HARNESS_DECLS_H
#define EECH_HARNESS_DECLS_H

/* cmndline.h (the EECH.INI "entity update frame rate" setting) */
extern int
	command_line_entity_update_frame_rate;

/* cmndline.h (EECH.INI "downwash"): selects the local-only entity heap; off */
extern int
	command_line_downwash;

/*
 * Session state that the original expresses as macros over globals
 * (ui_menu/session/session.h :: get_valid_current_game_session,
 * get_current_game_session_type). It is not part of the port, and the adopted
 * paths never read it: sc_msgs.c reaches it only for aircraft and vehicles.
 * Declared here as functions so that any use fails loudly (harness.c) instead
 * of reading an invented value. (get_game_status is the original global.h
 * macro over game_status since Slice 4, which reads it: see harness.c.)
 */
extern int get_valid_current_game_session (void);
extern session_list_types get_current_game_session_type (void);

/* functions extracted into eech_extracted.c and called from harness.c */
extern void harness_overload_group_link_parent_responses (void);

extern void harness_default_update_link_responses (void);

extern void harness_overload_aircraft_link_parent_responses (entity_types type);

extern void harness_overload_keysite_link_child_responses (void);

extern void harness_overload_group_link_child_response (void);

extern void harness_overload_task_link_parent_response (void);

extern int (*harness_default_get_entity_int_value) (entity *en, int_types type);

extern void (*harness_default_set_entity_int_value) (entity *en, int_types type, int value);

extern void (*harness_default_set_entity_float_value) (entity *en, float_types type, float value);

extern float (*harness_default_get_entity_float_value) (entity *en, float_types type);

extern void update_client_server_entities (void);

extern float get_2d_range (const vec3d *v1, const vec3d *v2);

extern float get_approx_2d_range (const vec3d *v1, const vec3d *v2);

extern void assess_group_supplies (entity *en);

#endif
