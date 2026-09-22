/*
 * Declarations the harness adds after the original headers.
 */

#ifndef EECH_HARNESS_DECLS_H
#define EECH_HARNESS_DECLS_H

/* cmndline.h (the EECH.INI "entity update frame rate" setting) */
extern int
	command_line_entity_update_frame_rate;

/* functions extracted into eech_extracted.c and called from harness.c */
extern void harness_overload_group_link_parent_responses (void);

extern void harness_default_update_link_responses (void);

extern void (*harness_default_set_entity_int_value) (entity *en, int_types type, int value);

extern void (*harness_default_set_entity_float_value) (entity *en, float_types type, float value);

extern void update_client_server_entities (void);

extern float get_2d_range (const vec3d *v1, const vec3d *v2);

extern float get_approx_2d_range (const vec3d *v1, const vec3d *v2);

extern void assess_group_supplies (entity *en);

#endif
