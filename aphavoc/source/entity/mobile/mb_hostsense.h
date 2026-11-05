/*
 * Host-owned physical sensing (EEDCS-43, D2).  The host owns what a source's sensor physically perceives of a target: the range
 * between them (planar squared for the aircraft scans, approximate 3-D for the vehicle scans), the scan envelope's floor and ceiling
 * and the way the floor falls as a scan goes on, the random tolerance applied to the floor, the target's radar signature (stealth
 * factor) and the range adjustment for switched-off emitters.  Campaign code keeps what to ask and what it means: which side, group and
 * state make a target a candidate, in which order the candidates are asked, the ranking of the detected ones, line of sight, and
 * engagement.  The scan functions in ac_tgt.c and vh_tgt.c ask for each candidate and act on the answer.
 *
 * The exchange is a short synchronous dialogue on one host callback, with at most one scan open:
 *
 *   BEGIN      the source's position and the facts of its sensor: scan range (the unit's raw figure) and, for the surface-to-air scan,
 *              the group's raw ceiling and initial floor; the host identity of the source when the roster has one.  The host answers OPEN.
 *   CANDIDATE  one target, in the order the campaign enumerated it: its position, whether a human flies it, and for the surface-to-air
 *              scan its radar altitude (and the difficulty factor of a human pilot).  The host answers REJECT (with the predicate that
 *              failed), ACCEPT (with the measured range), or asks for a further fact: NEED_DRAW or NEED_SIGNATURE.
 *   DRAW       a random draw in [0, 1], taken by the campaign at the point the original scan took it (after the altitude test), from
 *              its own private source-derived generator; the host never sees the generator or its seed.  The host answers REJECT or
 *              NEED_SIGNATURE.
 *   SIGNATURE  the facts the target's radar signature and range adjustment depend on: the state of its emitters (human-flown targets
 *              only) and, for a helicopter, the configuration facts of its radar-stealth factor.  The host answers REJECT or ACCEPT.
 *   END        the scan is over.  The host answers CLOSED and drops what it kept.
 *
 * The host keeps the physical state of the scan between messages (the surface-to-air floor is lowered by each candidate the campaign
 * asks about, whether or not it is later detected, and the next candidate meets the lowered floor).  The positions, the radar altitude, the
 * helicopter configuration facts and the unit database figures (scan range, ceiling, floor) that the campaign supplies are transitional mirrors
 * of what a host that owns the world already knows: the reference (EECH) host measures with them as inputs of its own model; they do not make
 * the campaign the owner of a physical solver.  A host that owns its world (a DCS host, later) may decide at CANDIDATE from its own state and
 * answer REJECT with NOT_PERCEIVED, ignoring them.  Only the host's verdict returns to the campaign.  The four named predicates (range,
 * range_adjusted, altitude, tolerance) are what the reference model of the original EECH scans reports; a host with another model has no
 * obligation to map onto them.
 *
 * No EECH pointer, entity index or seed crosses the boundary; identities are the host's own, as for line of sight.  There is no campaign
 * fallback: a hosted engine with no sense provider stops with an error at the first scan.
 */
#ifndef MB_HOSTSENSE_H
#define MB_HOSTSENSE_H

#include <stddef.h>

#define CAMPAIGN_SENSE_HOST_ID_MAX 128
#define CAMPAIGN_SENSE_ERROR_MAX 256

enum campaign_sense_mode
{
	CAMPAIGN_SENSE_BEGIN = 1,
	CAMPAIGN_SENSE_CANDIDATE,
	CAMPAIGN_SENSE_DRAW,
	CAMPAIGN_SENSE_SIGNATURE,
	CAMPAIGN_SENSE_END
};

enum campaign_sense_scan
{
	CAMPAIGN_SENSE_AIR_TO_AIR = 1,
	CAMPAIGN_SENSE_AIR_TO_SURFACE,
	CAMPAIGN_SENSE_SURFACE_TO_AIR,
	CAMPAIGN_SENSE_SURFACE_TO_SURFACE
};

enum campaign_sense_state
{
	CAMPAIGN_SENSE_OPEN = 1,
	CAMPAIGN_SENSE_ACCEPT,
	CAMPAIGN_SENSE_REJECT,
	CAMPAIGN_SENSE_NEED_DRAW,
	CAMPAIGN_SENSE_NEED_SIGNATURE,
	CAMPAIGN_SENSE_CLOSED
};

/* why a candidate was not perceived.  RANGE, RANGE_ADJUSTED, ALTITUDE and TOLERANCE name the predicates of the original EECH sensing model, which the
   reference host reproduces and the characterisation records; each is legal only for the scans whose original had it.  NOT_PERCEIVED is the host's
   own verdict from whatever model it has (another simulator's radar, sight or visibility model): it names no predicate, so every scan accepts it
   and campaign code treats the candidate exactly as it treats any other rejected one. */
enum campaign_sense_reason
{
	CAMPAIGN_SENSE_REASON_NONE = 0,
	CAMPAIGN_SENSE_REASON_RANGE,
	CAMPAIGN_SENSE_REASON_RANGE_ADJUSTED,
	CAMPAIGN_SENSE_REASON_ALTITUDE,
	CAMPAIGN_SENSE_REASON_TOLERANCE,
	CAMPAIGN_SENSE_REASON_NOT_PERCEIVED
};

typedef struct campaign_sense_step
{
	int mode;
	int scan;
	/* BEGIN: the source's host identity (empty when the roster has none) and its sensor facts.  scan_ceiling and scan_floor are
	   meaningful for the surface-to-air scan only */
	char source_host_id[CAMPAIGN_SENSE_HOST_ID_MAX + 1];
	float source_position[3];
	float scan_range;
	float scan_ceiling;
	float scan_floor;
	/* CANDIDATE: the target's host identity (empty when none), position and player flag (aircraft-to-aircraft and surface-to-air scans);
	   the surface-to-air scan adds the radar altitude, and the difficulty factor when a human flies the target */
	char target_host_id[CAMPAIGN_SENSE_HOST_ID_MAX + 1];
	float target_position[3];
	int target_player;
	int has_difficulty;
	float difficulty_factor;
	float radar_altitude;
	/* DRAW */
	double draw;
	/* SIGNATURE: has_emitters when a human flies the target; helicopter targets carry the three configuration facts */
	int has_emitters;
	int radar_on;
	int radar_jammer_on;
	int infra_red_jammer_on;
	int target_helicopter;
	int weapon_bay_closed;
	int stub_wings_attached;
	int undercarriage_up;
} campaign_sense_step;

typedef struct campaign_sense_result
{
	int state;
	/* REJECT: the predicate; ACCEPT, and optionally REJECT, NEED_DRAW and NEED_SIGNATURE: the measured range the ranking uses */
	int reason;
	int has_range;
	float target_range;
	/* optional trace echo: the host's intermediate physical values, read only by the test fixture build; the shipping module ignores it */
	int has_echo;
	float echo_ceiling;
	float echo_floor;
	float echo_adjusted_range;
	float echo_stealth_factor;
	float echo_stealth_limit;
	double echo_tolerance_limit;
} campaign_sense_result;

typedef int (*campaign_sense_fn)
(
	const campaign_sense_step *step,
	campaign_sense_result *result,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_sense_host
{
	campaign_sense_fn sense;
	void *user;
} campaign_sense_host;

struct ENTITY;
struct VEC3D;

void campaign_sense_set_host (const campaign_sense_host *host);

/* Opens a scan.  scan is a campaign_sense_scan.  scan_ceiling and scan_floor matter for the surface-to-air scan only. */
void campaign_sense_begin (int scan, struct ENTITY *source, const struct VEC3D *source_position, float scan_range, float scan_ceiling, float scan_floor);

/* Asks the host about one candidate target of the open scan; the campaign has already applied its own eligibility filters.  Returns TRUE when the
   host reports the target detected, with the measured range, FALSE when it rejects it. */
int campaign_sense_candidate (struct ENTITY *source, struct ENTITY *target, const struct VEC3D *target_position, float *target_range);

/* Closes the open scan */
void campaign_sense_end (void);

#endif
