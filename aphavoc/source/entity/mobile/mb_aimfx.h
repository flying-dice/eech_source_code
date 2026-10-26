//
// TEST-ONLY physical weapon-aiming observer hooks (EEDCS-44).
//
// Without EECH_AIM_FIXTURE every macro below expands to nothing, so the aiming sources (weapon/weapon.c, weapon/wn_move.c,
// aircraft/ac_wpn.c, vehicle/vh_wpn.c, vehicle/person/ps_anim.c) preprocess to the original text. With it (cargo feature `aim-fixture`,
// never shipping) each hook only READS values the original has already computed, or the entity fields it has already stored,
// and folds them into the trace in csrc/eech_aim_fixture.c. Nothing a hook does writes campaign state, draws from the game's random
// stream, evaluates a campaign getter that the original did not, or changes a branch. The shared random stream is observed only by its
// draw COUNT (stepping a local copy of the generator), never its value.
//
// Four solver entries are wrapped rather than edited: the original body is renamed behind a macro and a wrapper of the original name
// calls it, then reports the arguments and results. The remaining hooks are single statements placed between original statements.
//

#ifndef MB_AIMFX_H
#define MB_AIMFX_H

#ifdef EECH_AIM_FIXTURE

//
// which branch of the lead-and-ballistic solver ran (set by the inline hooks, read by the wrapper's report)
//

enum eech_aim_lead_path
{
	EECH_AIM_LEAD_PATH_NONE,
	EECH_AIM_LEAD_PATH_TINY_RANGE,
	EECH_AIM_LEAD_PATH_OFF_MAP,
	EECH_AIM_LEAD_PATH_STATIONARY,
	EECH_AIM_LEAD_PATH_MOVING,
	EECH_AIM_LEAD_PATHS
};

//
// the campaign's fire-decision sites that consume the physical readiness and launch-angle error
//

enum eech_aim_site
{
	EECH_AIM_SITE_AIRCRAFT_FIRE,
	EECH_AIM_SITE_AIRCRAFT_UPDATE,
	EECH_AIM_SITE_VEHICLE_UPDATE,
	EECH_AIM_SITES
};

enum eech_aim_reject
{
	EECH_AIM_REJECT_NOT_READY,
	EECH_AIM_REJECT_VECTORS_INVALID,
	EECH_AIM_REJECT_LAUNCH_ANGLE,
	EECH_AIM_REJECTS
};

//
// early exits of the weapon-system update (the original returns without finishing the call)
//

enum eech_aim_ws_exit
{
	EECH_AIM_WS_EXIT_NO_VIEWPOINT,
	EECH_AIM_WS_EXIT_NO_HEADING_DEVICE,
	EECH_AIM_WS_EXIT_NO_PITCH_DEVICE,
	EECH_AIM_WS_EXITS
};

//
// edges counted by the weapon-system update
//

enum eech_aim_ws_edge
{
	EECH_AIM_WS_EDGE_CLIMB_BUMP,
	EECH_AIM_WS_EDGE_HOLD_NOT_READY,
	EECH_AIM_WS_EDGE_NON_AI_PLAYER,
	EECH_AIM_WS_EDGE_PLAYER_LOCAL,
	EECH_AIM_WS_EDGE_POINT_LOCK,
	EECH_AIM_WS_EDGES
};

extern void eech_aim_vec_begin (entity *launcher);
extern void eech_aim_vec_stage (int stage);
extern void eech_aim_vec_targeted (int has_target);
extern void eech_aim_vec_tracking (void);
extern void eech_aim_vec_end (entity *launcher, const vec3d *weapon_vector, const vec3d *weapon_to_target_vector,
	const vec3d *weapon_to_intercept_point_vector, float weapon_to_intercept_point_range);

extern void eech_aim_pitch_solve_begin (void);
extern void eech_aim_pitch_solve (entity *source, entity *target, int weapon_sub_type, const vec3d *pitch_device_position,
	const vec3d *target_vector, const vec3d *intercept_point_vector, float intercept_point_range, int result);

extern void eech_aim_intercept_begin (const vec3d *pitch_device_position, float weapon_velocity, float target_true_velocity,
	const vec3d *target_point);
extern void eech_aim_intercept_end (const vec3d *intercept_point);

extern void eech_aim_lead_begin (void);
extern void eech_aim_lead_path (int path, float target_true_velocity, int iterations);
extern void eech_aim_lead_range (int phase, int iteration, float range);
extern void eech_aim_lead_iteration (int iteration, float target_move_distance, const vec3d *new_intercept_point);
extern void eech_aim_lead_triangulated (void);
extern void eech_aim_lead_end (const vec3d *pitch_device_position, int weapon_sub_type, float parent_velocity, entity *source,
	entity *target, const vec3d *intercept_point, const float *angle_of_projection, const float *time_of_flight, int result);

extern void eech_aim_ballistic (int weapon_sub_type, float range, float height_diff_or_pitch, int simplified, int fixed_pitch,
	float parent_velocity, int result, const float *aiming_pitch, const float *time_of_flight);

extern void eech_aim_ws_begin (entity *source);
extern void eech_aim_ws_stage (int stage);
extern void eech_aim_ws_ready (int found_device, int make_selected_ready, int selected_weapon_system_ready);
extern void eech_aim_ws_required (int package, int located, float required_heading, float required_pitch, float current_heading,
	float current_pitch);
extern void eech_aim_ws_step (int package, float delta_heading, float delta_pitch, float new_heading, float new_pitch, int heading_shared,
	int pitch_shared);
extern void eech_aim_ws_edge (int edge);
extern void eech_aim_ws_exit (entity *source, int reason, const weapon_package_status *package_status, int config_type);
extern void eech_aim_ws_end (entity *source, int selected_weapon_system_ready, const weapon_package_status *package_status, int config_type);

extern void eech_aim_ready_update (int vehicle, entity *en, float state_after, int state_before);
extern void eech_aim_ready_command (entity *en, int open, float new_state);
extern void eech_aim_angle (int site, entity *launcher, float launch_angle_error, float limit);
extern void eech_aim_reject (int site, int reason);

extern void eech_aim_person_branch (entity *person, int has_target_link);
extern void eech_aim_person_facing (entity *person, const vec3d *position, const vec3d *target_position, float heading);

#define AIM_FX_VEC_BEGIN(LAUNCHER)						eech_aim_vec_begin ((LAUNCHER));
#define AIM_FX_VEC_STAGE(STAGE)							eech_aim_vec_stage ((STAGE));
#define AIM_FX_VEC_TARGETED(HAS)						eech_aim_vec_targeted ((HAS));
#define AIM_FX_VEC_TRACKING()							eech_aim_vec_tracking ();
#define AIM_FX_VEC_END(L,WV,WT,WI,RANGE)				eech_aim_vec_end ((L), (WV), (WT), (WI), (RANGE));

#define AIM_FX_INTERCEPT_BEGIN(P,WV,TV,TP)				eech_aim_intercept_begin ((P), (WV), (TV), (TP));
#define AIM_FX_INTERCEPT_END(IP)						eech_aim_intercept_end ((IP));

#define AIM_FX_LEAD_PATH(PATH,TV,ITER)					eech_aim_lead_path ((PATH), (TV), (ITER));
#define AIM_FX_LEAD_RANGE(PHASE,ITER,RANGE)				eech_aim_lead_range ((PHASE), (ITER), (RANGE));
#define AIM_FX_LEAD_ITERATION(ITER,DISTANCE,POINT)		eech_aim_lead_iteration ((ITER), (DISTANCE), (POINT));
#define AIM_FX_LEAD_TRIANGULATED()						eech_aim_lead_triangulated ();

#define AIM_FX_WS_BEGIN(SOURCE)							eech_aim_ws_begin ((SOURCE));
#define AIM_FX_WS_STAGE(STAGE)							eech_aim_ws_stage ((STAGE));
#define AIM_FX_WS_READY(FOUND,MAKE,READY)				eech_aim_ws_ready ((FOUND), (MAKE), (READY));
#define AIM_FX_WS_REQUIRED(PKG,LOC,RH,RP,CH,CP)			eech_aim_ws_required ((PKG), (LOC), (RH), (RP), (CH), (CP));
#define AIM_FX_WS_STEP(PKG,DH,DP,NH,NP,HS,PS)			eech_aim_ws_step ((PKG), (DH), (DP), (NH), (NP), (HS), (PS));
#define AIM_FX_WS_EDGE(EDGE)							eech_aim_ws_edge ((EDGE));
#define AIM_FX_WS_EXIT(SOURCE,REASON,STATUS,CONFIG)		eech_aim_ws_exit ((SOURCE), (REASON), (STATUS), (CONFIG));
#define AIM_FX_WS_END(SOURCE,READY,STATUS,CONFIG)		eech_aim_ws_end ((SOURCE), (READY), (STATUS), (CONFIG));

#define AIM_FX_READY_UPDATE(VEHICLE,EN,AFTER,BEFORE)		eech_aim_ready_update ((VEHICLE), (EN), (AFTER), (BEFORE));
#define AIM_FX_READY_COMMAND(EN,OPEN,STATE)				eech_aim_ready_command ((EN), (OPEN), (STATE));
#define AIM_FX_ANGLE(SITE,LAUNCHER,ERROR,LIMIT)			eech_aim_angle ((SITE), (LAUNCHER), (ERROR), (LIMIT));
#define AIM_FX_REJECT(SITE,REASON)						eech_aim_reject ((SITE), (REASON));

#define AIM_FX_PERSON_BRANCH(PERSON,HAS)				eech_aim_person_branch ((PERSON), (HAS));
#define AIM_FX_PERSON_FACING(PERSON,POS,TARGET,HEADING)	eech_aim_person_facing ((PERSON), (POS), (TARGET), (HEADING));

#else

#define AIM_FX_VEC_BEGIN(LAUNCHER)
#define AIM_FX_VEC_STAGE(STAGE)
#define AIM_FX_VEC_TARGETED(HAS)
#define AIM_FX_VEC_TRACKING()
#define AIM_FX_VEC_END(L,WV,WT,WI,RANGE)

#define AIM_FX_INTERCEPT_BEGIN(P,WV,TV,TP)
#define AIM_FX_INTERCEPT_END(IP)

#define AIM_FX_LEAD_PATH(PATH,TV,ITER)
#define AIM_FX_LEAD_RANGE(PHASE,ITER,RANGE)
#define AIM_FX_LEAD_ITERATION(ITER,DISTANCE,POINT)
#define AIM_FX_LEAD_TRIANGULATED()

#define AIM_FX_WS_BEGIN(SOURCE)
#define AIM_FX_WS_STAGE(STAGE)
#define AIM_FX_WS_READY(FOUND,MAKE,READY)
#define AIM_FX_WS_REQUIRED(PKG,LOC,RH,RP,CH,CP)
#define AIM_FX_WS_STEP(PKG,DH,DP,NH,NP,HS,PS)
#define AIM_FX_WS_EDGE(EDGE)
#define AIM_FX_WS_EXIT(SOURCE,REASON,STATUS,CONFIG)
#define AIM_FX_WS_END(SOURCE,READY,STATUS,CONFIG)

#define AIM_FX_READY_UPDATE(VEHICLE,EN,AFTER,BEFORE)
#define AIM_FX_READY_COMMAND(EN,OPEN,STATE)
#define AIM_FX_ANGLE(SITE,LAUNCHER,ERROR,LIMIT)
#define AIM_FX_REJECT(SITE,REASON)

#define AIM_FX_PERSON_BRANCH(PERSON,HAS)
#define AIM_FX_PERSON_FACING(PERSON,POS,TARGET,HEADING)

#endif

#endif
