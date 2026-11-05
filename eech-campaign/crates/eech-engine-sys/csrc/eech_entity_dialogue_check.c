/*
 * TEST-ONLY (EEDCS-49, feature `entity-dialogue-check`): the isolated control of the campaign's half of the host-owned entity collision narrow phase (aphavoc/.../mb_hostentity.h).
 *
 * It includes the REAL dialogue (the header the shipping mb_hostmodel.c includes) and binds it to a scripted host, so what is checked is the shipped code's behaviour at the
 * boundary: what it sends (the segment and the pose, never the caller's outputs), what it writes into the caller's buffers (only the components the host reports the original
 * assigned, and nothing at all on a refusal) and when it stops the engine.  It calls no original code, loads no model and says nothing about the host's kernel (eech-world).
 *
 * THE PRECONDITION THIS EXISTS FOR.  A real caller (answer_weapon_step_lookup in wn_move.c) passes an automatic face normal it never initialised, so the intercept point and normal
 * are outputs, not inputs.  Where the platform has memory protection the buffers are placed on pages that cannot be read: a dialogue that read, loaded, copied or forwarded a word
 * of a buffer the original had not assigned would fault.  Elsewhere the same cases run on ordinary buffers and the request is compared across two different buffer contents.
 *
 * eech_entity_dialogue_check_run returns the number of failed expectations (0: all held) and writes one line per failure; *checks is the number of expectations made.
 */

#ifdef EECH_ENTITY_DIALOGUE_CHECK

#include <setjmp.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#if defined(__linux__)
#include <sys/mman.h>
#define HAVE_GUARD_PAGES 1
#else
#define HAVE_GUARD_PAGES 0
#endif

#include "entity/mobile/mb_hostmodel.h"

#define FALSE 0
#define TRUE 1

/* ---- the scripted host ---- */

#define SCENES 100
#define SCENE_NAME "CANDIDATE"

struct scripted_answer
{
	int result;
	int write_mask;
	uint32_t words[6];
	const char *name;
};

static struct scripted_answer answer;
static campaign_model_request seen;
static int asks;
static int refuse_the_exchange;

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

static void scripted_ask (campaign_model_request *request, campaign_model_reply *reply)
{
	memset (reply, 0, sizeof (*reply));

	seen = *request;
	asks++;

	if (refuse_the_exchange)
	{
		debug_fatal ("host model: [model-provider:callback] the scripted host refused");
	}

	reply->result = answer.result;
	reply->write_mask = answer.write_mask;
	memcpy (reply->points, answer.words, sizeof (answer.words));
	strncpy (reply->name, answer.name, sizeof (reply->name) - 1);
}

static void scripted_scene (int scene)
{
	if ((scene < 1) || (scene >= SCENES))
	{
		debug_fatal ("host model: [model-provider:scene.index] scene %d is outside 1 to %d", scene, SCENES - 1);
	}
}

static void scripted_identity (const campaign_model_reply *reply, int scene)
{
	if (strcmp (reply->name, SCENE_NAME) != 0)
	{
		debug_fatal ("host model: [model-provider:entity_collision.identity] the host named scene %d \"%.60s\"", scene, reply->name);
	}
}

#define CAMPAIGN_ENTITY_REQUIRE_SCENE(SCENE) scripted_scene (SCENE)
#define CAMPAIGN_ENTITY_ASK(REQUEST,REPLY) scripted_ask ((REQUEST), (REPLY))
#define CAMPAIGN_ENTITY_IDENTITY(REPLY,SCENE) scripted_identity ((REPLY), (SCENE))

#include "entity/mobile/mb_hostentity.h"

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
		if ((n > 0) && (n < t->capacity - t->used))
		{
			t->used += n;
		}
	}
}

#define EXPECT(T,C) tc_expect ((T), (C), #C, __LINE__)

/* words that must stay exactly, or arrive exactly: a signed zero, a subnormal, a signalling NaN, an infinity, a quiet NaN with a payload */
static const uint32_t SENTINEL_POINT[3] = { 0x80000000u, 0x00000001u, 0x7fa00001u };
static const uint32_t SENTINEL_NORMAL[3] = { 0x7f800000u, 0xffc00123u, 0x80000001u };
/* computed words an answer carries: they include a signed zero and a NaN payload, which are legitimate results of arithmetic */
static const uint32_t COMPUTED[6] = { 0x40490fdbu, 0x80000000u, 0x7fa00002u, 0x00000000u, 0x3f800000u, 0xffc00124u };

/* a buffer of three floats on a page of its own, optionally unreadable and unwritable */
struct buffer
{
	float *floats;
	void *page;
	size_t length;
	int guarded;
};

static struct buffer buffer_make (const uint32_t *initial, int guarded)
{
	struct buffer b;

	memset (&b, 0, sizeof (b));

#if HAVE_GUARD_PAGES
	b.length = 4096;
	b.page = mmap (NULL, b.length, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
	if (b.page == MAP_FAILED)
	{
		b.page = NULL;
	}
	else
	{
		b.floats = (float *) b.page;
		if (initial)
		{
			memcpy (b.floats, initial, 3 * sizeof (float));
		}
		if (guarded)
		{
			if (mprotect (b.page, b.length, PROT_NONE) == 0)
			{
				b.guarded = TRUE;
			}
		}
	}
#else
	(void) guarded;
	b.page = NULL;
#endif

	if (!b.page)
	{
		static float fallback[8][3];
		static int next;

		b.floats = fallback[next++ % 8];
		if (initial)
		{
			memcpy (b.floats, initial, 3 * sizeof (float));
		}
		else
		{
			memset (b.floats, 0x5a, 3 * sizeof (float));
		}
	}

	return (b);
}

static void buffer_free (struct buffer *b)
{
#if HAVE_GUARD_PAGES
	if (b->page)
	{
		munmap (b->page, b->length);
	}
#else
	(void) b;
#endif
}

/* a guarded buffer is made readable again only for the check's own comparison, after the dialogue has finished */
static int buffer_is (struct buffer *b, const uint32_t *words)
{
#if HAVE_GUARD_PAGES
	if (b->page && b->guarded)
	{
		mprotect (b->page, b->length, PROT_READ);
	}
#endif

	return (memcmp (b->floats, words, 3 * sizeof (float)) == 0);
}

static const float SEGMENT_OLD[3] = { 1.0f, 2.0f, 3.0f };
static const float SEGMENT_NEW[3] = { 4.0f, 5.0f, 6.0f };

static void candidate_words (float *candidate, int count)
{
	static const uint32_t odd[4] = { 0x80000000u, 0x00000001u, 0x7fa00003u, 0x7f800000u };
	int i;

	for (i = 0; i < count; i++)
	{
		memcpy (&candidate[i], &odd[i % 4], sizeof (float));
	}
}

struct outcome
{
	int stopped, hit;
};

/* one exchange with the scripted answer; a stop is caught and its text kept */
static struct outcome ask_once (int scene, int candidate_count, struct buffer *point, struct buffer *normal)
{
	struct outcome o;
	float candidate[16];

	memset (&o, 0, sizeof (o));
	candidate_words (candidate, 16);
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) == 0)
	{
		o.hit = campaign_entity_collision_exchange (scene, SEGMENT_OLD, SEGMENT_NEW, candidate, candidate_count, point->floats, normal->floats);
	}
	else
	{
		o.stopped = TRUE;
	}

	return (o);
}

static void script (int result, int mask, const uint32_t *words, const char *name)
{
	memset (&answer, 0, sizeof (answer));
	answer.result = result;
	answer.write_mask = mask;
	if (words)
	{
		memcpy (answer.words, words, sizeof (answer.words));
	}
	answer.name = name;
}

/* an answer whose six words are COMPUTED for the components the mask names and zero for the others, as the host gives it */
static void script_masked (int result, int mask)
{
	uint32_t words[6];
	int i;

	for (i = 0; i < 6; i++)
	{
		words[i] = (mask & (1 << i)) ? COMPUTED[i] : 0;
	}

	script (result, mask, words, SCENE_NAME);
}

/* a refusal: the dialogue stopped with `marker`, asked the host once or not at all, and left BOTH buffers byte for byte as they were */
static void expect_refusal (struct tc *t, const char *label, const char *marker, int scene, int candidate_count, int line)
{
	struct buffer point = buffer_make (SENTINEL_POINT, FALSE), normal = buffer_make (SENTINEL_NORMAL, FALSE);
	struct outcome o;
	char what[160];

	o = ask_once (scene, candidate_count, &point, &normal);

	snprintf (what, sizeof (what), "%s: stopped", label);
	tc_expect (t, o.stopped, what, line);
	snprintf (what, sizeof (what), "%s: stopped at %s (said: %.80s)", label, marker, fatal_text);
	tc_expect (t, strstr (fatal_text, marker) != NULL, what, line);
	snprintf (what, sizeof (what), "%s: the intercept point is untouched", label);
	tc_expect (t, buffer_is (&point, SENTINEL_POINT), what, line);
	snprintf (what, sizeof (what), "%s: the face normal is untouched", label);
	tc_expect (t, buffer_is (&normal, SENTINEL_NORMAL), what, line);

	buffer_free (&point);
	buffer_free (&normal);
}

int eech_entity_dialogue_check_run (char *text, int capacity, int *checks)
{
	struct tc t;
	struct buffer point, normal, other_point, other_normal;
	struct outcome o;
	float expected[22];

	t.text = text;
	t.capacity = capacity;
	t.used = 0;
	t.failures = 0;
	t.checks = 0;
	text[0] = '\0';

	/* ---- the request: the segment then the candidate, bit for bit, and nothing of the caller's outputs ---- */
	candidate_words (expected, 16);

	/* a MISS that assigned nothing (not a state the original reaches, but the dialogue's domain allows it): both buffers are on pages that cannot be read or written, and the
	   exchange completes: no word of a buffer the original did not assign is read, loaded, copied or forwarded */
	point = buffer_make (NULL, TRUE);
	normal = buffer_make (NULL, TRUE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0);
	asks = 0;
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit && asks == 1);
	EXPECT (&t, seen.op == CAMPAIGN_MODEL_ENTITY_COLLISION && seen.scene == 7 && seen.pose_count == CAMPAIGN_MODEL_ENTITY_STATIONARY_POSE);
	EXPECT (&t, memcmp (&seen.pose[0], SEGMENT_OLD, sizeof (SEGMENT_OLD)) == 0 && memcmp (&seen.pose[3], SEGMENT_NEW, sizeof (SEGMENT_NEW)) == 0);
	EXPECT (&t, memcmp (&seen.pose[6], expected, 12 * sizeof (float)) == 0);
	buffer_free (&point);
	buffer_free (&normal);

	/* a moving candidate: 16 candidate floats and a pose of 22 */
	point = buffer_make (SENTINEL_POINT, FALSE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	o = ask_once (7, 16, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit && seen.pose_count == CAMPAIGN_MODEL_ENTITY_MOVING_POSE);
	EXPECT (&t, memcmp (&seen.pose[6], expected, 16 * sizeof (float)) == 0);
	buffer_free (&point);
	buffer_free (&normal);

	/* the request does not depend on what the caller's buffers hold: two different contents give the same words */
	{
		static const uint32_t other_a[3] = { 0x11111111u, 0x22222222u, 0x33333333u };
		static const uint32_t other_b[3] = { 0xffc00001u, 0x80000000u, 0x00000000u };
		campaign_model_request first;

		point = buffer_make (other_a, FALSE);
		normal = buffer_make (other_a, FALSE);
		other_point = buffer_make (other_b, FALSE);
		other_normal = buffer_make (other_b, FALSE);
		script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
		(void) ask_once (7, 12, &point, &normal);
		first = seen;
		(void) ask_once (7, 12, &other_point, &other_normal);
		EXPECT (&t, memcmp (&first, &seen, sizeof (first)) == 0);
		/* the unassigned point of both stays exactly as it was */
		EXPECT (&t, buffer_is (&point, other_a) && buffer_is (&other_point, other_b));
		buffer_free (&point);
		buffer_free (&normal);
		buffer_free (&other_point);
		buffer_free (&other_normal);
	}

	/* ---- a HIT: both vectors assigned, written with the computed words (a signed zero and NaN payloads are results like any other) ---- */
	point = buffer_make (SENTINEL_POINT, FALSE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_FOUND, CAMPAIGN_ENTITY_EVERYTHING_WRITTEN);
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && o.hit);
	EXPECT (&t, buffer_is (&point, &COMPUTED[0]) && buffer_is (&normal, &COMPUTED[3]));
	buffer_free (&point);
	buffer_free (&normal);

	/* ---- a MISS that crossed a plane outside the face: both vectors assigned and written, and it is still a miss ---- */
	point = buffer_make (SENTINEL_POINT, FALSE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_EVERYTHING_WRITTEN);
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit);
	EXPECT (&t, buffer_is (&point, &COMPUTED[0]) && buffer_is (&normal, &COMPUTED[3]));
	buffer_free (&point);
	buffer_free (&normal);

	/* ---- a MISS that never crossed a plane: only the normal was assigned. The point's page cannot be read or written, and its initialised signed-zero, subnormal and NaN words
	   stay exactly (checked on a second, readable buffer) ---- */
	point = buffer_make (NULL, TRUE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit);
	EXPECT (&t, buffer_is (&normal, &COMPUTED[3]));
	buffer_free (&point);
	buffer_free (&normal);

	point = buffer_make (SENTINEL_POINT, FALSE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit);
	EXPECT (&t, buffer_is (&point, SENTINEL_POINT));
	EXPECT (&t, buffer_is (&normal, &COMPUTED[3]));
	buffer_free (&point);
	buffer_free (&normal);

	/* a computed zero is a written word, not an unassigned one: a degenerate face assigns a zero normal */
	{
		static const uint32_t zeros[6] = { 0, 0, 0, 0, 0, 0 };

		point = buffer_make (SENTINEL_POINT, FALSE);
		normal = buffer_make (SENTINEL_NORMAL, FALSE);
		script (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN, zeros, SCENE_NAME);
		o = ask_once (7, 12, &point, &normal);
		EXPECT (&t, !o.stopped && !o.hit && buffer_is (&point, SENTINEL_POINT) && buffer_is (&normal, &zeros[0]));
		buffer_free (&point);
		buffer_free (&normal);
	}

	/* ---- refusals before any caller commit: each leaves both initialised buffers byte for byte as they were ---- */
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	refuse_the_exchange = TRUE;
	expect_refusal (&t, "the host refuses the exchange", "model-provider:callback", 7, 12, __LINE__);
	refuse_the_exchange = FALSE;

	expect_refusal (&t, "scene 0", "scene.index", 0, 12, __LINE__);
	expect_refusal (&t, "a scene beyond the catalogue", "scene.index", SCENES, 12, __LINE__);
	expect_refusal (&t, "a candidate of 13 floats", "entity_collision.pose", 7, 13, __LINE__);
	expect_refusal (&t, "a candidate of no floats", "entity_collision.pose", 7, 0, __LINE__);

	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	answer.name = "OTHER";
	expect_refusal (&t, "another scene's name", "entity_collision.identity", 7, 12, __LINE__);

	script_masked (0, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	expect_refusal (&t, "a result that is neither a hit nor a miss (0)", "entity_collision.result", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_ABSENT, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	expect_refusal (&t, "a result of the other kind (absent)", "entity_collision.result", 7, 12, __LINE__);

	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 64);
	expect_refusal (&t, "write mask 64", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, -1);
	expect_refusal (&t, "write mask -1", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0x01);
	expect_refusal (&t, "a point written in part", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0x0b);
	expect_refusal (&t, "a point component and a normal component", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0x08);
	expect_refusal (&t, "a normal written in part", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_POINT_WRITTEN);
	expect_refusal (&t, "a point without the normal", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_FOUND, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	expect_refusal (&t, "a hit that assigned only the normal", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_FOUND, 0);
	expect_refusal (&t, "a hit that assigned nothing", "entity_collision.write_mask", 7, 12, __LINE__);

	/* a word for a component the mask says was not assigned: even a signed zero is a word */
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	answer.words[1] = 0x80000000u;
	expect_refusal (&t, "a negative zero for an unassigned point component", "entity_collision.unwritten", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0);
	answer.words[5] = 0x00000001u;
	expect_refusal (&t, "a word for an unassigned normal component", "entity_collision.unwritten", 7, 12, __LINE__);

	/* every refusal above stopped before a commit even where the first words were fine: the normal (written first) is untouched when the POINT part is the defect */
	script_masked (CAMPAIGN_MODEL_TRIANGLE_FOUND, CAMPAIGN_ENTITY_EVERYTHING_WRITTEN);
	answer.write_mask = 0x3b;
	expect_refusal (&t, "a point with one component missing", "entity_collision.write_mask", 7, 12, __LINE__);

	if (checks)
	{
		*checks = t.checks;
	}

	return (t.failures);
}

#endif
