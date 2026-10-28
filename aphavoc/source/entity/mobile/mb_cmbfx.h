//
// TEST-ONLY physical combat-resolution observer hooks (EEDCS-46).
//
// Without EECH_WEAPON_FIXTURE every macro below expands to nothing, so the sources that carry them preprocess to the original text and no hook argument is
// evaluated. With it (cargo feature `weapon-fixture`, never shipping) each hook only READS values the original has already computed, or entity fields it has already
// stored, and folds them into the report of csrc/eech_cmb_fixture.c. Nothing a hook does writes campaign state, draws from the game's random stream, evaluates a campaign
// function the original did not evaluate on the same path, or changes a branch. The shared random stream is observed only by the number of draws between two points
// (stepping a local copy of the generator), never its value. No entity pointer, index or seed leaves the fixture.
//
// Collisions: every registered ENTITY_MESSAGE_COLLISION handler is WRAPPED once, when the message responses are initialised (CMB_FX_INSTALL), so notify_local_entity itself
// (which the eech-sys crate extracts verbatim) is untouched. The wrapper reads the message's damage modifier (the double after the sender) and the receiver's health ONLY
// for the thirteen kinds whose handlers read that argument and which hold a damage level (the six mobile and seven fixed kinds); a bridge is notified with NO fourth
// argument by its segments, and a cargo or a weapon holds no damage level, so for those the wrapper only counts the delivery and passes the arguments through untouched.
// The receiver is identified across the handler by a fixture-private GENERATION of its slot, advanced by CMB_FX_RELEASE when the entity heap frees the slot: a slot that was
// freed (and possibly reused) during the handler is reported as destroyed and read for nothing else.
//
// Timed health: the six per-frame updates that lose health outside a collision (fire, bleeding) are observed by CMB_FX_TIMED_*: the attempt, the draws the original's
// condition took, and when it fired the new health, in the original's order.
//

#ifndef MB_CMBFX_H
#define MB_CMBFX_H

#ifdef EECH_WEAPON_FIXTURE

struct ENTITY;

enum eech_cmb_timed
{
	EECH_CMB_TIMED_HELICOPTER = 1,
	EECH_CMB_TIMED_FIXED_WING,
	EECH_CMB_TIMED_PERSON_BLEEDING,
	EECH_CMB_TIMED_ANTI_AIRCRAFT_FIRE,
	EECH_CMB_TIMED_ROUTED_FIRE,
	EECH_CMB_TIMED_SHIP_FIRE,
	EECH_CMB_TIMEDS
};

enum eech_cmb_damage_exit
{
	EECH_CMB_DAMAGE_EXIT_NO_WEAPON = 1,
	EECH_CMB_DAMAGE_EXIT_INVULNERABLE
};

enum eech_cmb_repair_source
{
	EECH_CMB_REPAIR_RESUPPLY = 1,		/* mb_msgs.c: a mobile landed and was repaired */
	EECH_CMB_REPAIR_FIXED_RESTORE		/* fixed.c: a fixed structure restored after a model restore */
};

extern void eech_cmb_install (void *default_response);
extern void eech_cmb_release (struct ENTITY *en);
extern void eech_cmb_capability_begin (void);
extern void eech_cmb_capability_path (int path);
extern void eech_cmb_capability_raw (int result);
struct campaign_aim_event;

extern void eech_cmb_capability_forecast (struct ENTITY *target, int weapon_type, const struct campaign_aim_event *events, int count, int capability);
extern void eech_cmb_capability_end (struct ENTITY *wpn, struct ENTITY *target, float modifier, int weapon_type, int warhead_type, float penetration, float armour, float random_modifier, int result);
extern void eech_cmb_damage_begin (struct ENTITY *en, struct ENTITY *weapon);
extern void eech_cmb_damage_exit (int reason);
extern void eech_cmb_damage_written (struct ENTITY *en, int new_level);
extern void eech_cmb_damage_end (struct ENTITY *en, struct ENTITY *weapon, float modifier, int capability);
extern void eech_cmb_explosion_begin (int kill_code);
extern void eech_cmb_explosion_end (void);
extern void eech_cmb_blast_begin (struct ENTITY *en, struct ENTITY *target, int capability, float radius);
extern void eech_cmb_blast_candidate (struct ENTITY *object, float range, float radius);
extern void eech_cmb_blast_notify (struct ENTITY *object, float modifier);
extern void eech_cmb_blast_rocket (struct ENTITY *object, float modifier);
extern void eech_cmb_blast_end (void);
extern void eech_cmb_repair (struct ENTITY *en, int old_level, int new_level, int source);
extern void eech_cmb_timed_begin (int kind);
extern void eech_cmb_timed_hit (struct ENTITY *en, int new_level);
extern void eech_cmb_timed_end (void);

// wraps every collision handler that is not the default (do nothing) response
#define CMB_FX_INSTALL(default_response) eech_cmb_install ((void *) default_response);
#define CMB_FX_RELEASE(en) eech_cmb_release (en);

#define CMB_FX_CAPABILITY_BEGIN() eech_cmb_capability_begin ();
#define CMB_FX_CAPABILITY_PATH(path) eech_cmb_capability_path (path);
#define CMB_FX_CAPABILITY_RAW(result) eech_cmb_capability_raw (result);
#define CMB_FX_CAPABILITY_END(wpn, target, modifier, weapon_type, warhead_type, penetration, armour, random_modifier, result) \
	eech_cmb_capability_end (wpn, target, modifier, weapon_type, warhead_type, penetration, armour, random_modifier, result);

// the forecast is the host's (EEDCS-46): the fixture build asks the reference host to echo the original's hooks of it, and replays them into the observer in their original order
#define CMB_FX_FORECAST_ECHO 1
#define CMB_FX_CAPABILITY_FORECAST(target, weapon_type, result) \
	eech_cmb_capability_forecast (target, weapon_type, (result).echo, (result).has_echo ? (result).echo_count : 0, (result).forecast_capability);

#define CMB_FX_DAMAGE_BEGIN(en, weapon) eech_cmb_damage_begin (en, weapon);
#define CMB_FX_DAMAGE_EXIT(reason) eech_cmb_damage_exit (reason);
#define CMB_FX_DAMAGE_WRITTEN(en, level) eech_cmb_damage_written (en, level);
#define CMB_FX_DAMAGE_END(en, weapon, modifier, capability) eech_cmb_damage_end (en, weapon, modifier, capability);

#define CMB_FX_EXPLOSION_BEGIN(kill_code) eech_cmb_explosion_begin (kill_code);
#define CMB_FX_EXPLOSION_END() eech_cmb_explosion_end ();
#define CMB_FX_BLAST_BEGIN(en, target, capability, radius) eech_cmb_blast_begin (en, target, capability, radius);
#define CMB_FX_BLAST_CANDIDATE(object, range, radius) eech_cmb_blast_candidate (object, range, radius);
#define CMB_FX_BLAST_NOTIFY(object, modifier) eech_cmb_blast_notify (object, modifier);
#define CMB_FX_BLAST_ROCKET(object, modifier) eech_cmb_blast_rocket (object, modifier);
#define CMB_FX_BLAST_END() eech_cmb_blast_end ();

#define CMB_FX_REPAIR(en, old_level, new_level, source) eech_cmb_repair (en, old_level, new_level, source);

#define CMB_FX_TIMED_BEGIN(kind) eech_cmb_timed_begin (kind);
#define CMB_FX_TIMED_HIT(en, new_level) eech_cmb_timed_hit (en, new_level);
#define CMB_FX_TIMED_END() eech_cmb_timed_end ();

#else

#define CMB_FX_INSTALL(default_response)
#define CMB_FX_RELEASE(en)
#define CMB_FX_CAPABILITY_BEGIN()
#define CMB_FX_CAPABILITY_PATH(path)
#define CMB_FX_CAPABILITY_RAW(result)
#define CMB_FX_CAPABILITY_END(wpn, target, modifier, weapon_type, warhead_type, penetration, armour, random_modifier, result)
#define CMB_FX_FORECAST_ECHO 0
#define CMB_FX_CAPABILITY_FORECAST(target, weapon_type, result)
#define CMB_FX_DAMAGE_BEGIN(en, weapon)
#define CMB_FX_DAMAGE_EXIT(reason)
#define CMB_FX_DAMAGE_WRITTEN(en, level)
#define CMB_FX_DAMAGE_END(en, weapon, modifier, capability)
#define CMB_FX_EXPLOSION_BEGIN(kill_code)
#define CMB_FX_EXPLOSION_END()
#define CMB_FX_BLAST_BEGIN(en, target, capability, radius)
#define CMB_FX_BLAST_CANDIDATE(object, range, radius)
#define CMB_FX_BLAST_NOTIFY(object, modifier)
#define CMB_FX_BLAST_ROCKET(object, modifier)
#define CMB_FX_BLAST_END()
#define CMB_FX_REPAIR(en, old_level, new_level, source)
#define CMB_FX_TIMED_BEGIN(kind)
#define CMB_FX_TIMED_HIT(en, new_level)
#define CMB_FX_TIMED_END()

#endif

#endif
