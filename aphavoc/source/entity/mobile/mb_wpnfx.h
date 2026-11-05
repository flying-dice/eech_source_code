//
// TEST-ONLY physical weapon-execution observer hooks (EEDCS-45).
//
// Without EECH_WEAPON_FIXTURE every macro below expands to nothing, so the weapon-execution sources (weapon/wn_creat.c, weapon/wn_move.c,
// weapon/wn_updt.c, weapon/wn_cllsn.c) preprocess to the original text. With it (cargo feature `weapon-fixture`, never shipping) each hook only
// READS values the original has already computed, or entity fields it has already stored, and folds them into the trace in
// csrc/eech_wpn_fixture.c. Nothing a hook does writes campaign state, draws from the game's random stream, evaluates a campaign getter that
// the original did not on the same path, or changes a branch. The shared random stream is observed only by its draw COUNT (stepping a local
// copy of the generator), never its value, and no entity pointer, index or seed leaves the fixture.
//
// Two functions are WRAPPED (the original body is renamed behind a feature-only macro; a wrapper of the original name observes its
// arguments and the state it leaves): weapon_movement and create_client_server_entity_weapon. Everything
// else is a single statement placed between original statements. A hook never reads an output the original left unwritten on the path taken
// (the position of an invalid guided target position is never read).
//

#ifndef MB_WPNFX_H
#define MB_WPNFX_H

#ifdef EECH_WEAPON_FIXTURE

//
// the branch a weapon_movement call took (a call can take several: guided/unguided, then a terminal path)
//

enum eech_wpn_path
{
	EECH_WPN_PATH_FUZE_TIMER,				// decoy timer elapsed: submunitions and/or self destruct
	EECH_WPN_PATH_EXHAUSTED,				// lifetime ran out (kill code exhausted or self destruct)
	EECH_WPN_PATH_ON_GROUND,				// resting on the ground: no movement
	EECH_WPN_PATH_GUIDED,					// guided movement
	EECH_WPN_PATH_UNGUIDED,					// unguided (ballistic/rocket) movement
	EECH_WPN_PATH_LOAL_CLIMB_DIVE,			// hellfire lock-on-after-launch profile
	EECH_WPN_PATH_KILLED_BY_GUIDANCE,		// the guided step set a kill code (too close, overshot)
	EECH_WPN_PATH_CONTACT_TARGET,			// in-flight contact with a target
	EECH_WPN_PATH_CONTACT_TREES,			// in-flight contact with trees
	EECH_WPN_PATH_OUT_OF_MAP_AREA,
	EECH_WPN_PATH_ABOVE_CEILING,
	EECH_WPN_PATH_GROUND_WATER,
	EECH_WPN_PATH_GROUND_SURVIVED,
	EECH_WPN_PATH_GROUND_SMOKE_GRENADE,
	EECH_WPN_PATH_GROUND_LAND,
	EECH_WPN_PATHS
};

enum eech_wpn_guided
{
	EECH_WPN_GUIDED_TOO_CLOSE,
	EECH_WPN_GUIDED_OVERSHOT,
	EECH_WPN_GUIDED_TARGET_LOST,
	EECH_WPN_GUIDED_TURNED,
	EECH_WPN_GUIDED_KINDS
};

//
// how get_target_position produced its answer (a call can pass several)
//

enum eech_wpn_tgt
{
	EECH_WPN_TGT_TRACKING_POINT,
	EECH_WPN_TGT_MOBILE_INTERCEPT,
	EECH_WPN_TGT_JAMMER_CHAFF,
	EECH_WPN_TGT_JAMMER_IR,
	EECH_WPN_TGT_SMOKE_DISTRACT,
	EECH_WPN_TGT_DECOY_INTERCEPT,
	EECH_WPN_TGTS
};

// ballistic table generation (boot): the HOST simulates and keeps the tables; the observer is given the host's echo of what it generated (the flights it simulated and the table
// cells) and replays it, in the order the original produced it (every weapon's flights during the pass, every weapon's table after it, then the guided range runs)
struct campaign_aim_event;
extern void eech_wpn_table_host (int sub_type, float max_range, const struct campaign_aim_event *events, int event_count, const float *cells, unsigned long long cell_count);
extern void eech_wpn_table_supplied (int sub_type);
extern void eech_wpn_range_host (int sub_type, const struct campaign_aim_event *events, int event_count);

// launch
extern void eech_wpn_salvo (entity *launcher, int sub_type, int salvo_size, unsigned pods_firing, int ai_launcher);
extern void eech_wpn_launch_begin (entity *launcher, int sub_type);
extern void eech_wpn_launch_no_ammo (void);
extern void eech_wpn_launch_attempt (int current_count, int burst_size, int has_target);
extern void eech_wpn_launch_created (int created);
extern void eech_wpn_launch_fired_at (void);
extern void eech_wpn_launch_ammo (int current_count, int new_count, int decoy);
extern void eech_wpn_launch_end (void);
extern void eech_wpn_birth_begin (void);
extern void eech_wpn_fuze_begin (void);
extern void eech_wpn_lead_begin (void);
extern void eech_wpn_lead_end (void);
extern void eech_wpn_fuze_solved (float decoy_timer, int draws);
extern void eech_wpn_dispersion (float heading, float pitch);
extern void eech_wpn_birth_end (entity *en, const weapon *raw, int submunition);

// flight
extern void eech_wpn_move_begin (entity *en);
extern void eech_wpn_move_end (entity *en);
extern void eech_wpn_path (int path);
extern void eech_wpn_submunitions (int count);
extern void eech_wpn_tracking (int tracking_point);
extern void eech_wpn_guidance (int guidance_type, int intercept_point_valid);
extern void eech_wpn_guidance_source (int has_launcher, int target_changes, int new_target_present);
extern void eech_wpn_tgt_branch (int branch, int taken);
extern void eech_wpn_tgt_end (int valid, const vec3d *position);
extern void eech_wpn_intercept (float weapon_velocity, float target_true_velocity, const vec3d *intercept_point);
extern void eech_wpn_guided (int kind, float a, float b, float c, float d);
extern void eech_wpn_precision (int high_precision_collision);
extern void eech_wpn_ground (int hit_ground, int high_precision_collision);
extern void eech_wpn_impact (int terrain_class, int survive_ground_impact);

// the host's weapon step: the hooks of the original weapon_movement its step passed, replayed in order from the step's echo
extern void eech_wpn_step_echo (const struct campaign_aim_event *event);

// update order and terminal
extern void eech_wpn_update_stage (entity *en, int stage);
extern void eech_wpn_decoy_begin (float decoy_timer);
extern void eech_wpn_decoy (float decoy_timer);
extern void eech_wpn_terminal (entity *en, const weapon *raw);
extern void eech_wpn_destroy (entity *en);

#define WPN_FX_TABLE_HOST(T,MR,EV,N,CELLS,COUNT)				eech_wpn_table_host ((T), (MR), (EV), (N), (CELLS), (COUNT));
#define WPN_FX_TABLE_SUPPLIED(T)								eech_wpn_table_supplied ((T));
#define WPN_FX_RANGE_HOST(T,EV,N)								eech_wpn_range_host ((T), (EV), (N));

#define WPN_FX_SALVO(L,T,SIZE,PODS,AI)							eech_wpn_salvo ((L), (T), (SIZE), (PODS), (AI));
#define WPN_FX_LAUNCH_NO_AMMO()									eech_wpn_launch_no_ammo ();
#define WPN_FX_LAUNCH_ATTEMPT(CURRENT,BURST,TARGET)			eech_wpn_launch_attempt ((CURRENT), (BURST), (TARGET));
#define WPN_FX_LAUNCH_CREATED(CREATED)							eech_wpn_launch_created ((CREATED));
#define WPN_FX_LAUNCH_FIRED_AT()								eech_wpn_launch_fired_at ();
#define WPN_FX_LAUNCH_AMMO(CURRENT,NEW,DECOY)					eech_wpn_launch_ammo ((CURRENT), (NEW), (DECOY));
#define WPN_FX_BIRTH_BEGIN()									eech_wpn_birth_begin ();
#define WPN_FX_BIRTH_END(EN,RAW,SUB)							eech_wpn_birth_end ((EN), (RAW), (SUB));



#define WPN_FX_LEAD_BEGIN()										eech_wpn_lead_begin ();
#define WPN_FX_LEAD_END()											eech_wpn_lead_end ();
#define WPN_FX_STEP_ECHO(EVENT)									eech_wpn_step_echo ((EVENT));

#define WPN_FX_UPDATE_STAGE(EN,STAGE)							eech_wpn_update_stage ((EN), (STAGE));
#define WPN_FX_DECOY_BEGIN(TIMER)								eech_wpn_decoy_begin ((TIMER));
#define WPN_FX_DECOY(TIMER)										eech_wpn_decoy ((TIMER));
#define WPN_FX_TERMINAL(EN,RAW)									eech_wpn_terminal ((EN), (RAW));
#define WPN_FX_DESTROY(EN)										eech_wpn_destroy ((EN));

#else

#define WPN_FX_TABLE_HOST(T,MR,EV,N,CELLS,COUNT)
#define WPN_FX_TABLE_SUPPLIED(T)
#define WPN_FX_RANGE_HOST(T,EV,N)

#define WPN_FX_SALVO(L,T,SIZE,PODS,AI)
#define WPN_FX_LAUNCH_NO_AMMO()
#define WPN_FX_LAUNCH_ATTEMPT(CURRENT,BURST,TARGET)
#define WPN_FX_LAUNCH_CREATED(CREATED)
#define WPN_FX_LAUNCH_FIRED_AT()
#define WPN_FX_LAUNCH_AMMO(CURRENT,NEW,DECOY)
#define WPN_FX_BIRTH_BEGIN()
#define WPN_FX_BIRTH_END(EN,RAW,SUB)



#define WPN_FX_LEAD_BEGIN()
#define WPN_FX_LEAD_END()
#define WPN_FX_STEP_ECHO(EVENT)

#define WPN_FX_UPDATE_STAGE(EN,STAGE)
#define WPN_FX_DECOY_BEGIN(TIMER)
#define WPN_FX_DECOY(TIMER)
#define WPN_FX_TERMINAL(EN,RAW)
#define WPN_FX_DESTROY(EN)

#endif

#endif
