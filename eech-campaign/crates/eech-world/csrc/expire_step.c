/*
 * The physical lifetime of a dead mobile's wreck (mobile.c: calculate_mobile_death_timer_value and the dead-frame test in
 * rv_updt.c, ps_updt.c, fw_updt.c and hc_updt.c).  The campaign supplies the squared range to its nearest keysite (the
 * keysite list is campaign state); the host owns the wreck's age and the curve that says when it ends.  The macros are the
 * engine's, unparenthesised exactly as written there, so the integer division and the double-precision result are the
 * same bits as the reference.
 */
#include "eech_arith.h"

#define ONE_MINUTE (60)
#define ONE_HOUR (60 * 60)

#define MIN_DEATH_TIMER_RANGE	1000 * 1000
#define MAX_DEATH_TIMER_RANGE	10000 * 10000

#define MIN_DEATH_TIMER_VALUE	5.0 * ONE_MINUTE
#define MAX_DEATH_TIMER_VALUE	3.0 * ONE_HOUR

float eech_expire_threshold(float range)
{
	float result;

	if (range <= MIN_DEATH_TIMER_RANGE)
	{
		result = MIN_DEATH_TIMER_VALUE;
	}
	else if (range >= MAX_DEATH_TIMER_RANGE)
	{
		result = MAX_DEATH_TIMER_VALUE;
	}
	else
	{
		float temp;

		temp = ((range - MIN_DEATH_TIMER_RANGE) / (MAX_DEATH_TIMER_RANGE / MIN_DEATH_TIMER_RANGE));

		result = MIN_DEATH_TIMER_VALUE + ((MAX_DEATH_TIMER_VALUE - MIN_DEATH_TIMER_VALUE) * temp);
	}

	return result;
}

/* the dead-frame gate: the threshold is only tested when (int)(timer + 1) is a multiple of 300 */
int eech_expire_gate(float timer)
{
	return !((int)(timer + 1) % 300);
}
