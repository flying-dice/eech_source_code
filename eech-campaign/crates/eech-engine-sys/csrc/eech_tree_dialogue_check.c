/*
 * TEST-ONLY (EEDCS-49, feature `tree-dialogue-check`): the isolated control of the campaign's half of the host-owned tree segment query (aphavoc/.../mb_hosttree.h).
 *
 * It includes the REAL dialogue (the header the shipping mb_hostterrain.c includes) and binds it to a scripted host and a recording terrain call, so what is checked is the shipped
 * code's behaviour at the boundary: which terrain elevations it answers, in which order, with which exact words, what it sends back to the host, what it leaves in the caller's
 * buffers and when it stops the engine.  It calls no original code, loads no terrain and no model, and says nothing about the host's tree kernel (that is eech-world's exact replay).
 *
 * The original reads the elevation of the OLD end only when the old end is inside the map and of the NEW end only when the new end is: so a query asks for none, OLD only, NEW only
 * (a segment that enters the map), or OLD then NEW.  Anything else (an end twice, NEW before OLD, a third question, a non-finite point, an unknown state, a hit that is not a boolean)
 * is a stop, and a stop writes nothing into the caller's buffers.
 *
 * eech_tree_dialogue_check_run returns the number of failed expectations (0: all held) and writes one line per failure; *checks is the number of expectations made.
 */

#ifdef EECH_TREE_DIALOGUE_CHECK

#include <math.h>
#include <setjmp.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "entity/mobile/mb_hostterrain.h"

#define FALSE 0
#define TRUE 1
#define ASSERT(X) ((void) 0)

struct VEC3D
{
	float x, y, z;
};

typedef struct VEC3D vec3d;

/* ---- the scripted host and the recording terrain call ---- */

struct scripted_reply
{
	int state, end, hit;
	uint32_t x, z, point[3], normal[3];
};

#define MAX_STEPS 8

static struct scripted_reply script[MAX_STEPS];
static int script_length, script_at;

static int sent_ops[MAX_STEPS];
static uint32_t sent_elevation[MAX_STEPS];
static campaign_terrain_request first_request;
static int asks;

static uint32_t looked_x[MAX_STEPS], looked_z[MAX_STEPS];
static int lookups;

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

static void scripted_ask (campaign_terrain_request *request, campaign_terrain_reply *reply, const char *what)
{
	const struct scripted_reply *step;

	(void) what;

	memset (reply, 0, sizeof (*reply));

	if (asks < MAX_STEPS)
	{
		sent_ops[asks] = request->op;
		sent_elevation[asks] = request->tree_elevation;
	}

	if (asks == 0)
	{
		first_request = *request;
	}

	asks++;

	if (script_at >= script_length)
	{
		debug_fatal ("the control's scripted host was asked more often than it was given replies");
	}

	step = &script[script_at++];

	reply->tree_state = step->state;
	reply->tree_end = step->end;
	reply->tree_hit = step->hit;
	reply->tree_x = step->x;
	reply->tree_z = step->z;
	memcpy (reply->tree_result_point, step->point, sizeof (step->point));
	memcpy (reply->tree_result_normal, step->normal, sizeof (step->normal));
}

static float word_float (uint32_t word)
{
	float value;

	memcpy (&value, &word, sizeof (value));

	return (value);
}

static uint32_t float_word (float value)
{
	uint32_t word;

	memcpy (&word, &value, sizeof (word));

	return (word);
}

/* a distinct, exactly representable answer for each point */
static float scripted_elevation (float x, float z)
{
	if (lookups < MAX_STEPS)
	{
		looked_x[lookups] = float_word (x);
		looked_z[lookups] = float_word (z);
	}

	lookups++;

	return (x * 0.5f + z * 0.25f);
}

#define CAMPAIGN_TREE_ASK(REQUEST,REPLY,WHAT) scripted_ask ((REQUEST), (REPLY), (WHAT))
#define CAMPAIGN_TREE_ELEVATION(X,Z) scripted_elevation ((X), (Z))

#include "entity/mobile/mb_hosttree.h"

/* ---- the control ---- */

struct tc
{
	char *text;
	int capacity, used, failures, checks;
};

static void tc_expect (struct tc *t, int condition, const char *label, int line)
{
	t->checks++;

	if (!condition)
	{
		int n;

		t->failures++;
		n = snprintf (t->text + t->used, (size_t) (t->capacity - t->used), "FAIL line %d: %s\n", line, label);
		if (n > 0 && n < t->capacity - t->used)
		{
			t->used += n;
		}
	}
}

#define EXPECT(T,C) tc_expect ((T), (C), #C, __LINE__)

#define NEED(END,X,Z) { CAMPAIGN_TERRAIN_TREE_NEED_TERRAIN, (END), 0, float_word (X), float_word (Z), { 0, 0, 0 }, { 0, 0, 0 } }

/* the caller's buffers: a signalling NaN and a signed zero that must come back or stay exactly */
static const uint32_t CALLER_POINT[3] = { 0x7fa00001u, 0x80000000u, 0x00000007u };
static const uint32_t CALLER_NORMAL[3] = { 0xffc00123u, 0x00000001u, 0x80000001u };
static const uint32_t FINAL_POINT[3] = { 0x40490fdbu, 0x80000000u, 0x7fa00002u };
static const uint32_t FINAL_NORMAL[3] = { 0x00000000u, 0x3f800000u, 0xffc00124u };

static struct scripted_reply done_reply (int hit)
{
	struct scripted_reply done;

	memset (&done, 0, sizeof (done));
	done.state = CAMPAIGN_TERRAIN_TREE_DONE;
	done.hit = hit;
	memcpy (done.point, FINAL_POINT, sizeof (done.point));
	memcpy (done.normal, FINAL_NORMAL, sizeof (done.normal));

	return (done);
}

struct outcome
{
	int stopped, hit;
	vec3d point, normal;
};

static void words_set (vec3d *vector, const uint32_t *words)
{
	memcpy (&vector->x, &words[0], sizeof (uint32_t));
	memcpy (&vector->y, &words[1], sizeof (uint32_t));
	memcpy (&vector->z, &words[2], sizeof (uint32_t));
}

static int words_equal (const vec3d *vector, const uint32_t *words)
{
	return (memcmp (&vector->x, &words[0], 4) == 0 && memcmp (&vector->y, &words[1], 4) == 0 && memcmp (&vector->z, &words[2], 4) == 0);
}

/* one query against the script; the old end is (100, 50, 200), the new end (300, 60, 400) */
static struct outcome run_query (const struct scripted_reply *steps, int count)
{
	struct outcome outcome;
	vec3d old_position, new_position;

	memset (&outcome, 0, sizeof (outcome));
	memcpy (script, steps, (size_t) count * sizeof (*steps));
	script_length = count;
	script_at = 0;
	asks = 0;
	lookups = 0;
	tree_query_open = FALSE;
	fatal_text[0] = '\0';

	old_position.x = 100.0f;
	old_position.y = 50.0f;
	old_position.z = 200.0f;
	new_position.x = 300.0f;
	new_position.y = 60.0f;
	new_position.z = 400.0f;
	words_set (&outcome.point, CALLER_POINT);
	words_set (&outcome.normal, CALLER_NORMAL);

	if (setjmp (fatal_jump) == 0)
	{
		outcome.hit = campaign_tree_dialogue (&old_position, &new_position, &outcome.point, &outcome.normal, TRUE, FALSE);
	}
	else
	{
		outcome.stopped = TRUE;
	}

	return (outcome);
}

/* a query that ends: the lookups it made are exactly `ends` (a string of 'o' and 'n'), each at that end's exact coordinates, and the host was answered with each elevation's pattern */
static void expect_complete (struct tc *t, const char *ends, int hit, int line)
{
	struct scripted_reply steps[MAX_STEPS];
	struct outcome outcome;
	size_t count = strlen (ends), i;

	for (i = 0; i < count; i++)
	{
		steps[i] = (struct scripted_reply) NEED ((ends[i] == 'o') ? CAMPAIGN_TERRAIN_TREE_END_OLD : CAMPAIGN_TERRAIN_TREE_END_NEW, (ends[i] == 'o') ? 100.0f : 300.0f, (ends[i] == 'o') ? 200.0f : 400.0f);
	}

	steps[count] = done_reply (hit);
	outcome = run_query (steps, (int) count + 1);

	tc_expect (t, !outcome.stopped, ends[0] ? ends : "(no lookup)", line);
	tc_expect (t, !outcome.stopped && outcome.hit == hit, "the hit is the host's", line);
	tc_expect (t, lookups == (int) count, "one terrain call for each question, and none that was not asked", line);
	tc_expect (t, asks == (int) count + 1, "one begin, then one answer for each question", line);

	for (i = 0; i < count && i < MAX_STEPS; i++)
	{
		const int old_end = ends[i] == 'o';

		tc_expect (t, looked_x[i] == float_word (old_end ? 100.0f : 300.0f) && looked_z[i] == float_word (old_end ? 200.0f : 400.0f), "the lookup is at the asked end's exact coordinates", line);
		tc_expect (t, sent_ops[i + 1] == CAMPAIGN_TERRAIN_TREE_TERRAIN, "the answer is a tree terrain exchange", line);
		tc_expect (t, sent_elevation[i + 1] == float_word (word_float (looked_x[i]) * 0.5f + word_float (looked_z[i]) * 0.25f), "the host gets the elevation the terrain call returned, as its pattern", line);
	}

	tc_expect (t, words_equal (&outcome.point, FINAL_POINT) && words_equal (&outcome.normal, FINAL_NORMAL), "the caller's buffers hold the host's final words, patterns exact", line);
}

/* a query that must stop: at this marker, with the caller's buffers exactly as they were */
static void expect_stop (struct tc *t, const struct scripted_reply *steps, int count, const char *marker, const char *label, int line)
{
	struct outcome outcome = run_query (steps, count);

	tc_expect (t, outcome.stopped, label, line);
	tc_expect (t, outcome.stopped && strstr (fatal_text, marker) != NULL, marker, line);
	tc_expect (t, words_equal (&outcome.point, CALLER_POINT) && words_equal (&outcome.normal, CALLER_NORMAL), "a stopped query writes nothing into the caller's buffers", line);
}

int eech_tree_dialogue_check_run (char *text, int capacity, int *checks)
{
	struct tc t;
	struct scripted_reply s[MAX_STEPS];
	const campaign_terrain_request *begin;

	t.text = text;
	t.capacity = capacity;
	t.used = 0;
	t.failures = 0;
	t.checks = 0;
	text[0] = '\0';

	/* every valid subsequence of the two ends: none, OLD only, NEW only (the segment enters the map), OLD then NEW; a hit and a miss */
	expect_complete (&t, "", FALSE, __LINE__);
	expect_complete (&t, "", TRUE, __LINE__);
	expect_complete (&t, "o", FALSE, __LINE__);
	expect_complete (&t, "n", FALSE, __LINE__);
	expect_complete (&t, "n", TRUE, __LINE__);
	expect_complete (&t, "on", FALSE, __LINE__);
	expect_complete (&t, "on", TRUE, __LINE__);

	/* what the host is sent first: the caller's exact words, the flags, no elevation */
	begin = &first_request;
	EXPECT (&t, begin->op == CAMPAIGN_TERRAIN_TREE_BEGIN);
	EXPECT (&t, begin->tree_old[0] == float_word (100.0f) && begin->tree_old[1] == float_word (50.0f) && begin->tree_old[2] == float_word (200.0f));
	EXPECT (&t, begin->tree_new[0] == float_word (300.0f) && begin->tree_new[1] == float_word (60.0f) && begin->tree_new[2] == float_word (400.0f));
	EXPECT (&t, memcmp (begin->tree_point, CALLER_POINT, sizeof (CALLER_POINT)) == 0 && memcmp (begin->tree_normal, CALLER_NORMAL, sizeof (CALLER_NORMAL)) == 0);
	EXPECT (&t, begin->tree_ignore_forest_edge_trees == 1 && begin->tree_skip_tree_top_test == 0 && begin->tree_elevation == 0);

	/* the dialogue stops, and writes nothing, on: an end twice, NEW before OLD, a third question, a non-finite point, an unknown state, a hit that is not a boolean, a failed host */
	s[0] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_OLD, 100.0f, 200.0f);
	s[1] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_OLD, 100.0f, 200.0f);
	s[2] = done_reply (FALSE);
	expect_stop (&t, s, 3, "tree.lookups", "OLD twice", __LINE__);

	s[0] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_NEW, 300.0f, 400.0f);
	s[1] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_NEW, 300.0f, 400.0f);
	s[2] = done_reply (FALSE);
	expect_stop (&t, s, 3, "tree.lookups", "NEW twice", __LINE__);

	s[0] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_NEW, 300.0f, 400.0f);
	s[1] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_OLD, 100.0f, 200.0f);
	s[2] = done_reply (FALSE);
	expect_stop (&t, s, 3, "tree.lookups", "NEW then OLD", __LINE__);

	s[0] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_OLD, 100.0f, 200.0f);
	s[1] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_NEW, 300.0f, 400.0f);
	s[2] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_NEW, 300.0f, 400.0f);
	s[3] = done_reply (FALSE);
	expect_stop (&t, s, 4, "tree.lookups", "a third question", __LINE__);

	s[0] = (struct scripted_reply) NEED (CAMPAIGN_TERRAIN_TREE_END_OLD, 100.0f, 200.0f);
	s[0].x = 0x7f800000u;
	s[1] = done_reply (FALSE);
	expect_stop (&t, s, 2, "tree.lookups", "a non-finite x", __LINE__);

	s[0] = (struct scripted_reply) NEED (7, 100.0f, 200.0f);
	s[1] = done_reply (FALSE);
	expect_stop (&t, s, 2, "tree.lookups", "an end that is neither OLD nor NEW", __LINE__);

	s[0] = done_reply (FALSE);
	s[0].state = 9;
	expect_stop (&t, s, 1, "tree.answer", "an unknown state", __LINE__);

	s[0] = done_reply (2);
	expect_stop (&t, s, 1, "tree.answer", "a hit that is not a boolean", __LINE__);

	/* a stopped query is not restartable by accident: the next query after the harness reset behaves as a fresh one, and a nested one is refused */
	run_query (s, 0);
	tree_query_open = TRUE;
	{
		vec3d a, b, p, n;

		memset (&a, 0, sizeof (a));
		b = p = n = a;
		fatal_text[0] = '\0';

		if (setjmp (fatal_jump) == 0)
		{
			(void) campaign_tree_dialogue (&a, &b, &p, &n, FALSE, FALSE);
			tc_expect (&t, 0, "a query started inside another must stop", __LINE__);
		}
		else
		{
			tc_expect (&t, strstr (fatal_text, "tree.query") != NULL, "a query started inside another stops at tree.query", __LINE__);
		}
	}

	if (checks)
	{
		*checks = t.checks;
	}

	return (t.failures);
}

#endif
