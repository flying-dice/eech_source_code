//
// TEST-ONLY physical combat-resolution observer hooks (EEDCS-46).
//
// Without EECH_WEAPON_FIXTURE every macro below expands to nothing, so the sources that carry them (entity/system/en_msgs/en_msgs.c,
// entity/mobile/weapon/damage.c, entity/special/effect/explosn/explosn.c, entity/mobile/mb_msgs.c) preprocess to the original text. With it (cargo feature
// `weapon-fixture`, never shipping) each hook only READS values the original has already computed, or entity fields it has already stored, and folds them into
// the report of csrc/eech_cmb_fixture.c. Nothing a hook does writes campaign state, draws from the game's random stream, evaluates a campaign function the original
// did not evaluate on the same path, or changes a branch. The shared random stream is observed only by the number of draws between two points (stepping a local
// copy of the generator), never its value. No entity pointer, index or seed leaves the fixture.
//
// The collision hooks WRAP every registered ENTITY_MESSAGE_COLLISION handler once, when the message responses are initialised (CMB_FX_INSTALL), so all thirteen
// collision handlers (six mobile, seven fixed) are observed without a hook in each, and notify_local_entity itself (which the eech-sys crate extracts verbatim) is
// untouched: the receiver is read before the handler runs and again after it (an entity the handler destroyed is reported as destroyed; its slot is not read for
// anything else).
//

#ifndef MB_CMBFX_H
#define MB_CMBFX_H

#ifdef EECH_WEAPON_FIXTURE

struct ENTITY;

extern void eech_cmb_install (void *default_response);
extern void eech_cmb_capability_begin (void);
extern void eech_cmb_capability_path (int path);
extern void eech_cmb_capability_raw (int result);
extern void eech_cmb_capability_end (struct ENTITY *wpn, struct ENTITY *target, float modifier, int weapon_type, int warhead_type, float penetration, float armour, float random_modifier, int result);
extern void eech_cmb_damage_invulnerable (struct ENTITY *en);
extern void eech_cmb_damage_written (struct ENTITY *en, int new_level);
extern void eech_cmb_damage_end (struct ENTITY *en, struct ENTITY *weapon, float modifier, int capability);
extern void eech_cmb_blast_begin (struct ENTITY *en, struct ENTITY *target, int capability, float radius);
extern void eech_cmb_blast_candidate (struct ENTITY *object, float range, float radius);
extern void eech_cmb_blast_notify (struct ENTITY *object, float modifier);
extern void eech_cmb_blast_rocket (struct ENTITY *object, float modifier);
extern void eech_cmb_blast_end (void);
extern void eech_cmb_repair (struct ENTITY *en, int old_level, int new_level);

// wraps every collision handler that is not the default (do nothing) response
#define CMB_FX_INSTALL(default_response) eech_cmb_install ((void *) default_response);

#define CMB_FX_CAPABILITY_BEGIN() eech_cmb_capability_begin ();
#define CMB_FX_CAPABILITY_PATH(path) eech_cmb_capability_path (path);
#define CMB_FX_CAPABILITY_RAW(result) eech_cmb_capability_raw (result);
#define CMB_FX_CAPABILITY_END(wpn, target, modifier, weapon_type, warhead_type, penetration, armour, random_modifier, result) \
	eech_cmb_capability_end (wpn, target, modifier, weapon_type, warhead_type, penetration, armour, random_modifier, result);

#define CMB_FX_DAMAGE_INVULNERABLE(en) eech_cmb_damage_invulnerable (en);
#define CMB_FX_DAMAGE_WRITTEN(en, level) eech_cmb_damage_written (en, level);
#define CMB_FX_DAMAGE_END(en, weapon, modifier, capability) eech_cmb_damage_end (en, weapon, modifier, capability);

#define CMB_FX_BLAST_BEGIN(en, target, capability, radius) eech_cmb_blast_begin (en, target, capability, radius);
#define CMB_FX_BLAST_CANDIDATE(object, range, radius) eech_cmb_blast_candidate (object, range, radius);
#define CMB_FX_BLAST_NOTIFY(object, modifier) eech_cmb_blast_notify (object, modifier);
#define CMB_FX_BLAST_ROCKET(object, modifier) eech_cmb_blast_rocket (object, modifier);
#define CMB_FX_BLAST_END() eech_cmb_blast_end ();

#define CMB_FX_REPAIR(en, old_level, new_level) eech_cmb_repair (en, old_level, new_level);

#else

#define CMB_FX_INSTALL(default_response)
#define CMB_FX_CAPABILITY_BEGIN()
#define CMB_FX_CAPABILITY_PATH(path)
#define CMB_FX_CAPABILITY_RAW(result)
#define CMB_FX_CAPABILITY_END(wpn, target, modifier, weapon_type, warhead_type, penetration, armour, random_modifier, result)
#define CMB_FX_DAMAGE_INVULNERABLE(en)
#define CMB_FX_DAMAGE_WRITTEN(en, level)
#define CMB_FX_DAMAGE_END(en, weapon, modifier, capability)
#define CMB_FX_BLAST_BEGIN(en, target, capability, radius)
#define CMB_FX_BLAST_CANDIDATE(object, range, radius)
#define CMB_FX_BLAST_NOTIFY(object, modifier)
#define CMB_FX_BLAST_ROCKET(object, modifier)
#define CMB_FX_BLAST_END()
#define CMB_FX_REPAIR(en, old_level, new_level)

#endif

#endif
