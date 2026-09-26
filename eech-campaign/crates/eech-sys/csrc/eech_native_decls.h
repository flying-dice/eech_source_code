/*
 * Declarations the native kernel adds after the original headers.
 * Provenance: eech-core-ts/c-reference/eech_harness_decls.h (81ed32e).
 */

#ifndef EECH_NATIVE_DECLS_H
#define EECH_NATIVE_DECLS_H

/* cmndline.h (the EECH.INI "entity update frame rate" setting) */
extern int
	command_line_entity_update_frame_rate;

/* cmndline.h (EECH.INI "downwash"): selects the local-only entity heap; off */
extern int
	command_line_downwash;

/* ui_menu/session/session.h macros over session state the kernel does not
   hold: declared as functions so any use fails loudly */
extern int get_valid_current_game_session (void);
extern session_list_types get_current_game_session_type (void);

/* functions extracted into eech_extracted*.c */
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
extern void eech_reset_extracted_statics (void);

extern void assign_keysite_tasks (entity *keysite, task_category_types category);
extern void update_client_server_entities (void);
extern float get_2d_range (const vec3d *v1, const vec3d *v2);
extern float get_approx_2d_range (const vec3d *v1, const vec3d *v2);
extern void assess_group_supplies (entity *en);

/* suitable.c: EECH's deinitialiser of the start-up suitability array */
extern void deinitialise_group_task_array (void);

/* ks_updt.c and ts_updt.c (NATIVE: compiled for step ()) */
extern void overload_keysite_update_functions (void);
extern void overload_task_update_functions (void);

/* patch P2: ks_updt.c's function-local task_timer, now resettable */
extern float eech_ks_updt_task_timer;

/* en_heap.c's get_free_entity, renamed by a controlled definition (spec.rs UNIT_DEFINES) */
extern entity *eech_original_get_free_entity (int index);

#endif
