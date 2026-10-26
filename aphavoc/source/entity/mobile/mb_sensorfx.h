//
// TEST-ONLY sensor-detection observer hooks (EEDCS-43).
//
// Without EECH_SENSOR_FIXTURE every macro below expands to nothing, so the four AI target-scan group functions
// (aircraft/ac_tgt.c, vehicle/vh_tgt.c) preprocess to the original text. With it (cargo feature `sensor-fixture`, never
// shipping), each hook only READS values the scan has already computed (or recomputes the same expression on a local copy)
// and folds them into the trace in csrc/eech_sensor_fixture.c. Nothing a hook does writes campaign state, draws from the
// game's random stream, or changes a branch.
//

#ifndef MB_SENSORFX_H
#define MB_SENSORFX_H

#ifdef EECH_SENSOR_FIXTURE

enum eech_sensor_function
{
	EECH_SENSOR_AIR_TO_AIR,
	EECH_SENSOR_AIR_TO_SURFACE,
	EECH_SENSOR_SURFACE_TO_AIR,
	EECH_SENSOR_SURFACE_TO_SURFACE,
	EECH_SENSOR_FUNCTIONS
};

//
// campaign eligibility filters: counted, never physical facts
//

enum eech_sensor_skip
{
	EECH_SENSOR_SKIP_SELF,
	EECH_SENSOR_SKIP_SIDE,
	EECH_SENSOR_SKIP_IDENTIFY,
	EECH_SENSOR_SKIP_AIRBORNE,
	EECH_SENSOR_SKIP_TARGET_TYPE,
	EECH_SENSOR_SKIP_NO_GROUP,
	EECH_SENSOR_SKIP_BEST_GROUP,
	EECH_SENSOR_SKIP_DEAD,
	EECH_SENSOR_SKIPS
};

//
// the outcome of a candidate's physical predicates
//

enum eech_sensor_verdict
{
	EECH_SENSOR_VERDICT_ACCEPT,
	EECH_SENSOR_VERDICT_RANGE,
	EECH_SENSOR_VERDICT_RANGE_ADJUSTED,
	EECH_SENSOR_VERDICT_ALTITUDE,
	EECH_SENSOR_VERDICT_TOLERANCE,
	EECH_SENSOR_VERDICT_LINE_OF_SIGHT,
	EECH_SENSOR_VERDICTS
};

enum eech_sensor_tag
{
	EECH_SENSOR_TAG_SOURCE_X,
	EECH_SENSOR_TAG_SOURCE_Y,
	EECH_SENSOR_TAG_SOURCE_Z,
	EECH_SENSOR_TAG_SCAN_RANGE,
	EECH_SENSOR_TAG_SCAN_RANGE_SQUARED,
	EECH_SENSOR_TAG_SCAN_CEILING,
	EECH_SENSOR_TAG_SCAN_FLOOR_INITIAL,
	EECH_SENSOR_TAG_SECTOR_X_MIN,
	EECH_SENSOR_TAG_SECTOR_Z_MIN,
	EECH_SENSOR_TAG_SECTOR_X_MAX,
	EECH_SENSOR_TAG_SECTOR_Z_MAX,
	EECH_SENSOR_TAG_TARGET_RANGE,
	EECH_SENSOR_TAG_TARGET_PLAYER,
	EECH_SENSOR_TAG_RADAR_ON,
	EECH_SENSOR_TAG_RADAR_JAMMER_ON,
	EECH_SENSOR_TAG_INFRA_RED_JAMMER_ON,
	EECH_SENSOR_TAG_ADJUSTED_RANGE,
	EECH_SENSOR_TAG_STEALTH_FACTOR,
	EECH_SENSOR_TAG_STEALTH_LIMIT,
	EECH_SENSOR_TAG_FLOOR,
	EECH_SENSOR_TAG_FLOOR_INDEPENDENT,
	EECH_SENSOR_TAG_DIFFICULTY_FACTOR,
	EECH_SENSOR_TAG_RADAR_ALTITUDE,
	EECH_SENSOR_TAG_TOLERANCE_DRAW,
	EECH_SENSOR_TAG_TOLERANCE_LIMIT,
	EECH_SENSOR_TAG_COS_ANGLE,
	EECH_SENSOR_TAG_RESULT,
	EECH_SENSOR_TAG_CRITERIA,
	EECH_SENSOR_TAG_REPLACED,
	EECH_SENSOR_TAGS
};

extern void eech_sensor_begin (int function, entity *source);
extern void eech_sensor_end (int target_count, entity *best_group, float best_score);
extern void eech_sensor_skip (int reason);
extern void eech_sensor_candidate (void);
extern void eech_sensor_f (int tag, float value);
extern void eech_sensor_i (int tag, int value);
extern void eech_sensor_dec_f (int tag, float value);
extern void eech_sensor_dec_i (int tag, int value);
extern void eech_sensor_verdict (int verdict);
extern void eech_sensor_not_perceived (void);
extern void eech_sensor_floor (float floor_after, float scan_range, float target_range);
extern void eech_sensor_floor_factor (float factor);
extern void eech_sensor_tolerance (double draw, double limit, float radar_altitude);

#define SENSOR_FX_BEGIN(FUNCTION,SOURCE)		eech_sensor_begin ((FUNCTION), (SOURCE));
#define SENSOR_FX_END(COUNT,GROUP,SCORE)		eech_sensor_end ((COUNT), (GROUP), (SCORE));
#define SENSOR_FX_SKIP(REASON)					eech_sensor_skip ((REASON));
#define SENSOR_FX_CANDIDATE()					eech_sensor_candidate ();
#define SENSOR_FX_F(TAG,VALUE)					eech_sensor_f ((TAG), (VALUE));
#define SENSOR_FX_I(TAG,VALUE)					eech_sensor_i ((TAG), (VALUE));
#define SENSOR_FX_DEC_F(TAG,VALUE)				eech_sensor_dec_f ((TAG), (VALUE));
#define SENSOR_FX_DEC_I(TAG,VALUE)				eech_sensor_dec_i ((TAG), (VALUE));
#define SENSOR_FX_VERDICT(VERDICT)				eech_sensor_verdict ((VERDICT));
#define SENSOR_FX_NOT_PERCEIVED()				eech_sensor_not_perceived ();
#define SENSOR_FX_FLOOR(FLOOR,RANGE,TARGET)	eech_sensor_floor ((FLOOR), (RANGE), (TARGET));
#define SENSOR_FX_FLOOR_FACTOR(FACTOR)			eech_sensor_floor_factor ((FACTOR));
#define SENSOR_FX_TOLERANCE(DRAW,LIMIT,ALT)	eech_sensor_tolerance ((DRAW), (LIMIT), (ALT));

#else

#define SENSOR_FX_BEGIN(FUNCTION,SOURCE)
#define SENSOR_FX_END(COUNT,GROUP,SCORE)
#define SENSOR_FX_SKIP(REASON)
#define SENSOR_FX_CANDIDATE()
#define SENSOR_FX_F(TAG,VALUE)
#define SENSOR_FX_I(TAG,VALUE)
#define SENSOR_FX_DEC_F(TAG,VALUE)
#define SENSOR_FX_DEC_I(TAG,VALUE)
#define SENSOR_FX_VERDICT(VERDICT)
#define SENSOR_FX_NOT_PERCEIVED()
#define SENSOR_FX_FLOOR(FLOOR,RANGE,TARGET)
#define SENSOR_FX_FLOOR_FACTOR(FACTOR)
#define SENSOR_FX_TOLERANCE(DRAW,LIMIT,ALT)

#endif

#endif
