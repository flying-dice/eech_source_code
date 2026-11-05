/*
 * TEST-ONLY (EEDCS-52, D3; feature `target-point-fixture`): the observer hooks the target-point-fixture build's patched sources call (build/patches.rs,
 * TARGET_POINT_FIXTURE_PATCHES): aphavoc/source/entity/en_misc/en_maths/en_maths.c (get_local_entity_target_point), aphavoc/source/entity/mobile/weapon/wn_dbase.c
 * (initialise_target_point_offsets), weapon.c (fill_aim_target) and wn_move.c (the weapon step's target-vector lookup). This header is included by those staged files ONLY
 * when the feature is on; with it off none of the hooks, the patches or the definitions (eech_tp_fixture.c) exist in the build, and the in-tree sources are never edited.
 *
 * Every hook only RECORDS a value the original statement has just read or produced, in bounded private tables (eech_tp_fixture.c). None allocates, frees, writes a campaign
 * or model byte, draws a random number, reads the clock, changes an entity or does floating-point arithmetic: floats cross as pointers and are copied as bit patterns, and no
 * pointer is ever emitted. The one campaign function the fixture itself calls is campaign_object_host_id, which only READS an identity the host already issued (it never asks the host).
 */
#ifndef EECH_TP_HOOKS_H
#define EECH_TP_HOOKS_H

/* the caller context: immediately before an original call of get_local_entity_target_point. `site` 1 is weapon.c fill_aim_target (`detail` is the step's CAMPAIGN_AIM mode),
   2 is the weapon step's target-vector lookup in wn_move.c (`detail` is the lookup's question number). The next eech_tp_begin consumes it; a call with none is unattributed. */
extern void eech_tp_caller (int site, int detail);

/* get_local_entity_target_point begins, right after the original read the entity's 3D instance (NULL when it has none) */
extern void eech_tp_begin (const void *en, const void *instance);

/* the instance holds a TARGET_POINT sub-object: the original is about to overwrite the instance viewpoint from the entity pose (branch 1) */
extern void eech_tp_instance_target_point (void);

/* branch 1, immediately before the original calls get_3d_sub_object_world_viewpoint: the sub-object its search found and the instance it searched; the hook reads the parent chain the original is about to walk */
extern void eech_tp_instance_chain (const void *result_sub_object, const void *instance);

/* the instance holds no TARGET_POINT sub-object: the original answers the entity position (branch 2) */
extern void eech_tp_instance_position (void);

/* no instance, and the entity is a fixed object */
extern void eech_tp_fixed (void);

/* no instance, fixed object: the shape index the original read (OBJECT_3D_INVALID_OBJECT_INDEX answers the position alone, branch 4) */
extern void eech_tp_shape (int index);

/* no instance, valid shape: the offset has a non-zero x or z */
extern void eech_tp_horizontal (void);

/* the heading the original read for that rotation */
extern void eech_tp_heading (const float *heading);

/* the original is about to add the rotated x/z offsets (it enters this only when the heading is non-zero) */
extern void eech_tp_xz_applied (void);

/* branches 2, 3 and 4: right after the original's get_local_entity_vec3d has copied the entity position into the result, before any physical addition */
extern void eech_tp_base_position (const void *target_point);

/* get_local_entity_target_point ends: the target point it returned */
extern void eech_tp_end (const void *target_point);

/* initialise_target_point_offsets: the census of the per-model offset table. begin is the first statement; invalid is the table's entry for the invalid object index (zero); one
   of sub_object or bbox follows each constructed model's entry (the TARGET_POINT sub-object's world position, or the bounding-box fallback with its six bounds); end is the last. */
extern void eech_tp_census_begin (void);
extern void eech_tp_census_invalid (const void *offset);
extern void eech_tp_census_sub_object (int index, const void *offset);
extern void eech_tp_census_bbox (int index, const void *offset, const void *bounds);
extern void eech_tp_census_end (void);

/* random.c set_random_number_seed: right before the original's srand (seed) */
extern void eech_tp_seed (int seed);

#endif
