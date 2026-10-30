/* A stand-in for the engine's project.h, just enough for aphavoc/source/entity/mobile/mb_hostaim.c to compile on its own (forecast_checks.c, EEDCS-46); a copy of the weapon characterization's stub. */
#ifndef HOSTAIM_STUB_PROJECT_H
#define HOSTAIM_STUB_PROJECT_H

#include <math.h>
#include <setjmp.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#define TRUE 1
#define FALSE 0

typedef struct ENTITY entity;

/* debug_fatal in the engine unwinds to the entry point; here it unwinds to the check that expects it */
extern jmp_buf harness_fatal;
extern char harness_message[512];

void debug_fatal (const char *format, ...);
int rand16 (void);
float get_delta_time (void);
unsigned int get_system_time (void);
const char *campaign_roster_host_id (entity *en);
int get_local_entity_index (entity *en);

#endif
