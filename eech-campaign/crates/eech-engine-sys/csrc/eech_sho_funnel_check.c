/*
 * TEST-ONLY (EEDCS-49, D3, features `sector-height-observer` + `sector-height-funnel-check`): the control of the observer's JOIN against the REAL placement map.
 *
 * The defect this control protects against: the observer's record of a placement must exist BEFORE the host's first state statement of that placement reaches the observer. In the real
 * funnel (en_list.c insert_local_entity_into_parents_child_list) `campaign_place_insert (en)` is one call that registers the placement, declares it and then states the entity's fixed or
 * city state (mb_hostplacemap.h place_insert -> place_fixed_state), all BEFORE it returns; LINK_CHILD follows. A record opened only after that call returns meets the first statement with
 * no record. A city building is placed twice: in its city's list (a statement under a city-list key) and then in its sector (a statement under the sector key, which the placement map prefers
 * as soon as it exists, mb_hostplacemap.h place_fixed_state); only the sector-stage statement joins a sector record.
 *
 * What is REAL here: mb_hostplacemap.h itself, the STAGED copy (so its staged observer hooks SH8 and SH9, with the sector-placement flag they pass, are the ones compiled), and the recorder core
 * (eech_sho.c). What is SCRIPTED: the entities (plain structs), the host (a key counter) and the entity-reading bodies of the hooks (the real ones read the engine's entity getters, which
 * these structs do not have; eech_sho.c gates them out under this feature). The ORDER of the observer's insert hook against `campaign_place_insert` is a parameter, because it is a property
 * of the staged en_list.c, not of this file: tests/sector_height_funnel.rs extracts it from the actual staged patch text, so the control and the code cannot disagree about placement.
 *
 * Each scenario is one funnel action; the Rust test reads the observer's report through the real converters after each. The control returns 0 when the action completed and 1 when the placement
 * map stopped the engine (the message is kept).
 */
#if defined(EECH_SECTOR_HEIGHT_OBSERVER) && defined(EECH_SHO_FUNNEL_CHECK)

#include <setjmp.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "entity/mobile/mb_hostmodel.h"

#define FALSE 0
#define TRUE 1

extern void eech_sho_core_insert (int ent, int etype, int family, int sx, int sz);
extern void eech_sho_core_remove (int ent, int etype, int family, int fixed_or_city);
extern void eech_sho_core_release (int ent);
extern void eech_sho_core_link_begin (uint32_t before);
extern void eech_sho_core_link_end (int ent, int etype, int family, int fixed_or_city, int shape_state, int shape, const uint32_t *pos, uint32_t ymax, uint32_t after, int sx, int sz);
extern void eech_sho_core_statement (int ent, int etype, int family, int sector_placed, const char *key, const uint32_t *words, int count);

/* the scripted world: fixed family types 60 to 69, the city 70, a city building 71 (a fixed-family type too, as in the campaign) */
#define CHECK_CITY_TYPE 70
#define CHECK_BUILDING_TYPE 71
#define FAMILY_FIXED 0
#define FAMILY_CITY 1
#define FAMILY_BUILDING 2
#define SHAPE_VALID 1

struct pentity
{
	int slot, type, has_position;
	const char *name, *identity, *object;
	float position[3];
	float heading, shape, sub_type;
	float box_min[3], box_max[3];
};

struct psector
{
	int x, z;
	float word;
};

static struct pentity pheap[16];
static int pheap_size = 16, place_host_installed;
static int issued;
static jmp_buf fatal_jump;
static char fatal_text[512];

static void debug_fatal (const char *format, ...)
{
	va_list arguments;

	va_start (arguments, format);
	vsnprintf (fatal_text, sizeof (fatal_text), format, arguments);
	va_end (arguments);

	longjmp (fatal_jump, 1);
}

static int place_heap_check (void)
{
	return (pheap_size);
}

static int check_host_installed (void)
{
	return (place_host_installed);
}

/* the scripted host: a counter issues the keys; a removal answers no retained name; the later phases acknowledge */
static void check_roster_call (int phase, int entity_type, float side, const float *position, const char *identity, char *key)
{
	(void) entity_type;
	(void) side;
	(void) position;
	(void) identity;

	if (phase == CAMPAIGN_SCAN_PHASE_INSERT)
	{
		sprintf (key, "%d", ++issued);
	}
}

static void check_city_call (int phase, int entity_type, const char *identity, char *key)
{
	(void) entity_type;
	(void) identity;

	if (phase == CAMPAIGN_SCAN_PHASE_CHILD_INSERT)
	{
		sprintf (key, "%d", ++issued);
	}
}

static void check_fixed_call (int entity_type, const char *key, const float *state)
{
	(void) entity_type;
	(void) key;
	(void) state;
}

static void check_city_state (const char *key, const float *state)
{
	(void) key;
	(void) state;
}

static void check_city_expand (const char *city, const char *building, float *box)
{
	(void) city;
	(void) building;
	(void) box;
}

#define campaign_model_host_installed check_host_installed
#define campaign_model_entity_roster check_roster_call
#define campaign_model_entity_city check_city_call
#define campaign_model_entity_fixed check_fixed_call
#define campaign_model_entity_city_state(KEY,STATE) check_city_state ((KEY), (STATE))
#define campaign_model_entity_city_expand(CITY,BUILDING,BOX) check_city_expand ((CITY), (BUILDING), (BOX))
#define CAMPAIGN_PLACE_ENTITY struct pentity
#define CAMPAIGN_PLACE_HEAP() place_heap_check ()
#define CAMPAIGN_PLACE_SLOT(E) ((E)->slot)
#define CAMPAIGN_PLACE_AT(SLOT) (&pheap[(SLOT)])
#define CAMPAIGN_PLACE_TYPE(E) ((E)->type)
#define CAMPAIGN_PLACE_NAME(E) ((E)->name)
#define CAMPAIGN_PLACE_POSITION(E) ((E)->has_position ? (E)->position : NULL)
#define CAMPAIGN_PLACE_IDENTITY(E) ((E)->identity)
#define CAMPAIGN_PLACE_OBJECT_ID(E) ((E)->object)
#define CAMPAIGN_PLACE_SIDE() 100.0f
#define CAMPAIGN_PLACE_FIXED_FAMILY(TYPE) ((((TYPE) >= 60) && ((TYPE) < 70)) || ((TYPE) == CHECK_BUILDING_TYPE))
#define CAMPAIGN_PLACE_FIXED_READ(E,OUT) do { memcpy ((OUT), (E)->position, 3 * sizeof (float)); (OUT)[3] = (E)->heading; (OUT)[4] = (E)->shape; (OUT)[5] = (E)->sub_type; } while (0)
#define CAMPAIGN_PLACE_CITY(TYPE) ((TYPE) == CHECK_CITY_TYPE)
#define CAMPAIGN_PLACE_CITY_READ(E,OUT) do { memcpy ((OUT), (E)->position, 3 * sizeof (float)); memcpy ((OUT) + 3, (E)->box_min, 3 * sizeof (float)); memcpy ((OUT) + 6, (E)->box_max, 3 * sizeof (float)); (OUT)[9] = (E)->sub_type; } while (0)

/* the STAGED mb_hostplacemap.h: its observer hooks SH8 and SH9 (the statement hooks, with the sector-placement flag) are compiled into this include */
#include "entity/mobile/mb_hostplacemap.h"

#undef campaign_model_host_installed
#undef campaign_model_entity_roster
#undef campaign_model_entity_city
#undef campaign_model_entity_fixed
#undef campaign_model_entity_city_state
#undef campaign_model_entity_city_expand

/* ---- the entity-reading hooks of the staged patches, over the scripted entities (the real bodies are gated out of eech_sho.c under this feature) ---- */

static int family_of_type (int type)
{
	return type == CHECK_CITY_TYPE ? FAMILY_CITY : (type == CHECK_BUILDING_TYPE ? FAMILY_BUILDING : FAMILY_FIXED);
}

static uint32_t word_bits (const float *word)
{
	uint32_t value;

	memcpy (&value, word, sizeof (value));

	return value;
}

void eech_sho_sector_insert (const void *en, const void *sector)
{
	const struct pentity *e = en;
	const struct psector *s = sector;

	eech_sho_core_insert (e->slot, e->type, family_of_type (e->type), s->x, s->z);
}

void eech_sho_sector_remove (const void *en)
{
	const struct pentity *e = en;

	eech_sho_core_remove (e->slot, e->type, family_of_type (e->type), 1);
}

void eech_sho_link_begin (const void *sector, const void *sender, const float *word)
{
	(void) sector;
	(void) sender;

	eech_sho_core_link_begin (word_bits (word));
}

void eech_sho_link_end (const void *sector, const void *sender, const float *word)
{
	const struct pentity *e = sender;
	const struct psector *s = sector;
	uint32_t pos[3];
	float ymax = 5.0f;

	memcpy (pos, e->position, sizeof (pos));

	eech_sho_core_link_end (e->slot, e->type, family_of_type (e->type), 1, SHAPE_VALID, (int) e->shape, pos, word_bits (&ymax), word_bits (word), s->x, s->z);
}

void eech_sho_statement_fixed (const void *en, int type, int sector_placed, const char *key, const float *state)
{
	const struct pentity *e = en;
	uint32_t words[CAMPAIGN_FIXED_STATE_WORDS];
	int i;

	for (i = 0; i < CAMPAIGN_FIXED_STATE_WORDS; i++)
	{
		words[i] = word_bits (&state[i]);
	}

	eech_sho_core_statement (e->slot, type, family_of_type (type), sector_placed, key, words, CAMPAIGN_FIXED_STATE_WORDS);
}

void eech_sho_statement_city (const void *en, int sector_placed, const char *key, const float *city)
{
	const struct pentity *e = en;
	uint32_t words[CAMPAIGN_CITY_STATE_WORDS];
	int i;

	for (i = 0; i < CAMPAIGN_CITY_STATE_WORDS; i++)
	{
		words[i] = word_bits (&city[i]);
	}

	eech_sho_core_statement (e->slot, CHECK_CITY_TYPE, FAMILY_CITY, sector_placed, key, words, CAMPAIGN_CITY_STATE_WORDS);
}

/* ---- the funnel ---- */

static struct psector sector34 = { 3, 4, 0.0f };

/* en_list.c's insertion into a sector list in the order of the STAGED patches: `hook_first` is whether the observer's insert hook (SH1) stands BEFORE `campaign_place_insert`, as the staged text says;
   `with_hook` FALSE models a placement the observer never saw (the genuinely unjoined case). LINK_CHILD (SH2, SH3) follows. */
static void funnel_insert (struct pentity *e, int hook_first, int with_hook)
{
	float height = e->position[1] + 5.0f;

	if (with_hook && hook_first)
	{
		eech_sho_sector_insert (e, &sector34);
	}

	place_insert (e);

	if (with_hook && !hook_first)
	{
		eech_sho_sector_insert (e, &sector34);
	}

	eech_sho_link_begin (&sector34, e, &sector34.word);

	if (height > sector34.word)
	{
		sector34.word = height;
	}

	eech_sho_link_end (&sector34, e, &sector34.word);
}

static void setup (void)
{
	int i;

	memset (pheap, 0, sizeof (pheap));

	for (i = 0; i < 16; i++)
	{
		pheap[i].slot = i;
		pheap[i].name = "SCRIPTED";
		pheap[i].has_position = TRUE;
		pheap[i].position[0] = 310.0f + (float) i;
		pheap[i].position[1] = 20.0f + (float) i;
		pheap[i].position[2] = 410.0f;
		pheap[i].shape = 12.0f + (float) i;
		pheap[i].sub_type = 3.0f;
	}

	/* slots 1, 2 and 5: fixed family; 3: the city; 4: a city building */
	pheap[1].type = pheap[2].type = pheap[5].type = 60;
	pheap[3].type = CHECK_CITY_TYPE;
	pheap[4].type = CHECK_BUILDING_TYPE;
	pheap[3].box_max[0] = 50.0f;

	sector34.word = 0.0f;
	issued = 0;
	place_reset ();
	place_host_installed = TRUE;
}

/* scenario codes:
     0 setup (a fresh scripted world and placement map)
    10, 11      insert the fixed entity of slot 1, 2 into sector (3, 4)
    12          insert the city (slot 3) into the sector
    13          the city building (slot 4): listed in its city (child stage), then inserted into the sector (sector stage)
    14          restate the states of slot 2 then slot 1 then the building (interleaved, after placement)
    16          remove the entity of slot 1 (placement map, then the observer's remove hook), then insert it again
    17          the slot-2 entity's slot is freed WHILE STILL LISTED (the placement map removes it itself, as the heap release does; then the observer's release), then it is inserted again
    18          insert slot 5 WITHOUT the observer's insert hook: a placement the observer never saw
    19          restate slot 5's state (its sector-stage statement has no record)
   returns 0 when the action completed, 1 when the placement map stopped the engine (the text is kept) */
int eech_sho_funnel_run (int scenario, int hook_first)
{
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) != 0)
	{
		return 1;
	}

	switch (scenario)
	{
		case 0:
			setup ();
			break;
		case 10:
			funnel_insert (&pheap[1], hook_first, TRUE);
			break;
		case 11:
			funnel_insert (&pheap[2], hook_first, TRUE);
			break;
		case 12:
			funnel_insert (&pheap[3], hook_first, TRUE);
			break;
		case 13:
			place_child_insert (&pheap[4], &pheap[3], NULL);
			funnel_insert (&pheap[4], hook_first, TRUE);
			break;
		case 14:
			place_fixed_state (&pheap[2]);
			place_fixed_state (&pheap[1]);
			place_fixed_state (&pheap[4]);
			break;
		case 16:
			place_remove (&pheap[1]);
			eech_sho_sector_remove (&pheap[1]);
			funnel_insert (&pheap[1], hook_first, TRUE);
			break;
		case 17:
			place_release_entity (&pheap[2]);
			eech_sho_core_release (2);
			funnel_insert (&pheap[2], hook_first, TRUE);
			break;
		case 18:
			funnel_insert (&pheap[5], hook_first, FALSE);
			break;
		case 19:
			place_fixed_state (&pheap[5]);
			break;
		default:
			return 2;
	}

	return 0;
}

int eech_sho_funnel_text (char *text, int capacity)
{
	snprintf (text, (size_t) capacity, "%s", fatal_text);

	return 0;
}

#endif
