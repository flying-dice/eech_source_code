/*
 * TEST-ONLY (EEDCS-50, D3; feature model-host-fixture): diagnostics of the host-owned model data (aphavoc/.../mb_hostmodel.h). Never part of the shipping module.
 *
 * The host-owned accessors are the SHIPPING code; this file adds three things to a build that has them, so a run can show where its answers came from. The entries that
 * call them are in eech_engine.c (they run inside the engine entry, as the campaign's own calls do: round-toward-zero, a fatal error unwinding to the entry).
 *
 *   counters   how many catalogue, scene and triangle questions the host was asked and how it answered them, and how often each bounds / radius table was read. A counter
 *              alone proves nothing about authority (it only counts calls); it is read together with
 *   poison     which overwrites every campaign-side array the ORIGINAL accessors read (the scene database's radius, both bounds and collision object index) with NaN and an
 *              out-of-table index. A campaign that goes on to give the same observations as an unpoisoned run has taken them from the host; a read of any of those arrays,
 *              by an accessor or by code that bypasses them, would change them.
 *   probe      one scene's radius and bounds, and one step of its collision-triangle cursor, through the REAL public accessors, in the format of the original fixture's
 *              rows (eech_model_fixture.c): float bit patterns, the return value, the number of the nine buffer words the call wrote (0 for a FALSE: the buffer is left
 *              untouched), the pointer-stability check of the bounds, and the rounding mode (fegetround) before and after the accessor call INSIDE the entry. That is the
 *              rounding mode only: it says nothing of the exception flags, the masks or MXCSR, and nothing about what the entry hands back to its CALLER.
 *   fpu        the COMPLETE outer floating-point state (x87 control and status words, MXCSR) around the REAL public entries, from the caller's side (eech_engine_model_host_fpu,
 *              below): the proof of what a caller sees, which no fegetround check is.
 *
 * None of them draws a random number, advances time or touches an entity.
 */

#ifdef EECH_MODEL_HOST_FIXTURE

#include <fenv.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

#ifdef EECH_CAMPAIGN_HOST_MODEL

struct campaign_model_counters
{
	long catalogue, scene, triangle, found, end, absent, bounds_reads, bounds_without_lines_reads, radius_reads;
	long presence_checked, presence_mismatches;
};

extern struct campaign_model_counters campaign_model_counts;
extern void campaign_model_poison_arrays (void);

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static void bounds_text (char *buffer, size_t capacity, const struct OBJECT_3D_BOUNDS *b)
{
	snprintf (buffer, capacity, "0x%08x,0x%08x,0x%08x,0x%08x,0x%08x,0x%08x", float_bits (b->xmin), float_bits (b->xmax), float_bits (b->ymin), float_bits (b->ymax),
		float_bits (b->zmin), float_bits (b->zmax));
}

int eech_model_host_counters_run (char *text, int capacity)
{
	int n = snprintf (text, (size_t) capacity,
		"model-host catalogue=%ld scene=%ld triangle=%ld found=%ld end=%ld absent=%ld bounds_reads=%ld bounds_without_lines_reads=%ld radius_reads=%ld"
		" presence_checked=%ld presence_mismatches=%ld\n",
		campaign_model_counts.catalogue, campaign_model_counts.scene, campaign_model_counts.triangle, campaign_model_counts.found, campaign_model_counts.end,
		campaign_model_counts.absent, campaign_model_counts.bounds_reads, campaign_model_counts.bounds_without_lines_reads, campaign_model_counts.radius_reads,
		campaign_model_counts.presence_checked, campaign_model_counts.presence_mismatches);

	return ((n < 0) || (n >= capacity)) ? 1 : 0;
}

int eech_model_host_poison_run (void)
{
	if (!objects_3d_scene_database)
	{
		return 1;
	}

	campaign_model_poison_arrays ();

	return 0;
}

int eech_model_host_probe_run (int scene, int depth, char *text, int capacity)
{
	object_3d_index_numbers index;
	const struct OBJECT_3D_BOUNDS *bb, *bb2, *bb_again, *bb2_again;
	float radius, radius_again;
	vec3d buffer[3];
	uint32_t words[9];
	char bb_text[160], bb2_text[160];
	int i, writes = 0, returned, round_before, round_after, used = 0, n;

	if ((scene < 1) || (scene >= OBJECT_3D_LAST) || (depth < 0))
	{
		return 1;
	}

	index = (object_3d_index_numbers) scene;

	/* the bounds and radius, twice: a caller keeps the bounds pointer across loops, so a second ask must give the same address and the same bits */
	bb = get_object_3d_bounding_box (index);
	bb2 = get_object_3d_bounding_box_without_lines (index);
	radius = get_object_3d_radius (index);
	bb_again = get_object_3d_bounding_box (index);
	bb2_again = get_object_3d_bounding_box_without_lines (index);
	radius_again = get_object_3d_radius (index);
	bounds_text (bb_text, sizeof (bb_text), bb);
	bounds_text (bb2_text, sizeof (bb2_text), bb2);

	n = snprintf (text + used, (size_t) (capacity - used), "model-host-scene scene=%d radius=0x%08x bb2=%s bb=%s stable=%d\n", scene, float_bits (radius), bb2_text, bb_text,
		(bb == bb_again) && (bb2 == bb2_again) && (float_bits (radius) == float_bits (radius_again)) && !memcmp (bb, bb_again, sizeof (*bb)) && !memcmp (bb2, bb2_again, sizeof (*bb2)));
	if ((n < 0) || (n >= capacity - used))
	{
		return 1;
	}
	used += n;

	/* the triangle step, into a buffer pre-filled with the original fixture's quiet-NaN sentinel */
	for (i = 0; i < 9; i++)
	{
		words[i] = 0x7fc0a5a5u;
	}
	memcpy (buffer, words, sizeof (buffer));
	round_before = fegetround ();
	returned = get_object_3d_collision_object_geometry_triangle (index, buffer, depth);
	round_after = fegetround ();
	memcpy (words, buffer, sizeof (words));
	for (i = 0; i < 9; i++)
	{
		writes += words[i] != 0x7fc0a5a5u;
	}
	n = snprintf (text + used, (size_t) (capacity - used), "model-host-tri scene=%d depth=%d return=%d writes=%d round_before=%d round_after=%d", scene, depth, returned, writes,
		round_before, round_after);
	if ((n < 0) || (n >= capacity - used))
	{
		return 1;
	}
	used += n;
	if (writes)
	{
		n = snprintf (text + used, (size_t) (capacity - used), " p=");
		if ((n < 0) || (n >= capacity - used))
		{
			return 1;
		}
		used += n;
		for (i = 0; i < 9; i++)
		{
			n = snprintf (text + used, (size_t) (capacity - used), "%s0x%08x", i ? "," : "", words[i]);
			if ((n < 0) || (n >= capacity - used))
			{
				return 1;
			}
			used += n;
		}
	}
	n = snprintf (text + used, (size_t) (capacity - used), "\n");
	return ((n < 0) || (n >= capacity - used)) ? 1 : 0;
}


/* ---------------------------------------------------------------------------------------------------------------------------- */
/* The COMPLETE outer floating-point state around a PUBLIC engine entry (EEDCS-50, D3).
 *
 * eech_engine_model_host_fpu is an OUTER harness, not an engine entry: it is the caller. It chooses the caller's floating-point state, calls a real public entry, and reads the
 * state back the moment the entry returns, so what it reports is what a caller of the shipped entries sees. The state is the raw words, not a summary:
 *
 *   x87 control word   precision, rounding and the six exception masks
 *   x87 status word    the six exception FLAGS, the stack top, the condition codes
 *   MXCSR              the SSE exception flags, the six masks, the rounding, flush-to-zero and denormals-are-zero
 *
 * Before the entry the caller state is set to: the requested rounding mode in BOTH units (0 nearest, 1 down, 2 up, 3 toward zero), every exception masked, and NONZERO
 * exception flags (divide-by-zero, underflow, inexact) in both the x87 status word and MXCSR, so that an entry that clears or overwrites flags is seen. The entries are the real ones:
 *
 *   kind 0   eech_engine_model_host_probe: the engine entry (ENTER/LEAVE), the campaign's public accessors, the host's model callback, the Lua provider and its bridge, the
 *            parsing and narrowing in Rust and back. A host that cannot answer takes the entry's fatal path (setjmp), which is the second path a caller must see restored.
 *   kind 1   eech_engine_model_host_counters: the engine entry (ENTER/LEAVE) alone, with no callback.
 *
 * The report gives the words before and after and whether each is identical. It does NOT decide what is acceptable and it repairs nothing: a difference is a measurement of the
 * shipped ENTER/LEAVE macros (which restore with fesetenv; whether that restores the SSE word on a given toolchain is exactly what this measures). Afterwards the harness puts the
 * process back into the environment it found, so the diagnostic itself leaves no mark. Rounding modes alone (fegetround) are NOT a proof of the complete state, and this report
 * does not rest on them.
 */
#if defined(__x86_64__) || defined(__i386__)

struct fpu_words
{
	uint32_t x87_control, x87_status, mxcsr;
};

static void fpu_capture (struct fpu_words *w)
{
	uint16_t control = 0, status = 0;
	uint32_t mxcsr = 0;

	__asm__ __volatile__ ("fnstcw %0" : "=m" (control));
	__asm__ __volatile__ ("fnstsw %0" : "=m" (status));
	__asm__ __volatile__ ("stmxcsr %0" : "=m" (mxcsr));
	w->x87_control = control;
	w->x87_status = status;
	w->mxcsr = mxcsr;
}

/* the rounding mode in both units, all exceptions masked, and the divide-by-zero (0x04), underflow (0x10) and inexact (0x20) flags raised in both */
static void fpu_prepare (int rounding)
{
	uint16_t control = 0;
	uint32_t mxcsr = 0;
	unsigned char environment[28];
	uint16_t status;

	__asm__ __volatile__ ("fnstcw %0" : "=m" (control));
	control = (uint16_t) ((control & ~0x0C00u) | ((unsigned) rounding << 10) | 0x3Fu);
	__asm__ __volatile__ ("fldcw %0" : : "m" (control));

	__asm__ __volatile__ ("stmxcsr %0" : "=m" (mxcsr));
	mxcsr = (mxcsr & ~0x6000u) | ((unsigned) rounding << 13) | 0x1F80u | 0x34u;
	__asm__ __volatile__ ("ldmxcsr %0" : : "m" (mxcsr));

	/* the x87 flags are written through the 28-byte protected-mode environment image (status word at offset 4) and loaded back; nothing is computed */
	__asm__ __volatile__ ("fnstenv %0" : "=m" (environment));
	memcpy (&status, environment + 4, sizeof (status));
	status = (uint16_t) (status | 0x34u);
	memcpy (environment + 4, &status, sizeof (status));
	__asm__ __volatile__ ("fldenv %0" : : "m" (environment));
}

static void fpu_write_mxcsr (uint32_t mxcsr)
{
	__asm__ __volatile__ ("ldmxcsr %0" : : "m" (mxcsr));
}

int eech_engine_model_host_fpu (int kind, int scene, int depth, int rounding, char *text, int capacity)
{
	struct fpu_words before, after;
	fenv_t original;
	uint32_t original_mxcsr = 0;
	char scratch[2048];
	int result, n, used;

	if (!text || (capacity <= 0) || (kind < 0) || (kind > 1) || (rounding < 0) || (rounding > 3) || (scene < 1) || (depth < 0))
	{
		return EECH_ENGINE_BAD_ARGUMENT;
	}

	/* the process's own environment, to be handed back after the diagnostic */
	fegetenv (&original);
	__asm__ __volatile__ ("stmxcsr %0" : "=m" (original_mxcsr));

	fpu_prepare (rounding);
	fpu_capture (&before);
	if (kind == 0)
	{
		result = eech_engine_model_host_probe (scene, depth, scratch, (int) sizeof (scratch));
	}
	else
	{
		result = eech_engine_model_host_counters (scratch, (int) sizeof (scratch));
	}
	fpu_capture (&after);

	fesetenv (&original);
	fpu_write_mxcsr (original_mxcsr);

	n = snprintf (text, (size_t) capacity,
		"model-host-fpu kind=%d rounding=%d scene=%d depth=%d result=%d before_x87_control=0x%04x before_x87_status=0x%04x before_mxcsr=0x%08x"
		" after_x87_control=0x%04x after_x87_status=0x%04x after_mxcsr=0x%08x same_x87_control=%d same_x87_status=%d same_mxcsr=%d\n",
		kind, rounding, scene, depth, result, before.x87_control, before.x87_status, before.mxcsr, after.x87_control, after.x87_status, after.mxcsr,
		before.x87_control == after.x87_control, before.x87_status == after.x87_status, before.mxcsr == after.mxcsr);
	if ((n < 0) || (n >= capacity))
	{
		return EECH_ENGINE_BAD_ARGUMENT;
	}
	used = n;
	if (result == EECH_ENGINE_FATAL)
	{
		/* the engine's stop, first line only: the host's refusal that took the entry's fatal path */
		const char *message = eech_engine_fatal_message ();
		char line[200];
		size_t length = 0;

		while (message && message[length] && (message[length] != '\n') && (length < sizeof (line) - 1))
		{
			line[length] = message[length];
			length++;
		}
		line[length] = 0;
		n = snprintf (text + used, (size_t) (capacity - used), "model-host-fpu-fatal %s\n", line);
		if ((n < 0) || (n >= capacity - used))
		{
			return EECH_ENGINE_BAD_ARGUMENT;
		}
	}

	return EECH_ENGINE_OK;
}

#else

int eech_engine_model_host_fpu (int kind, int scene, int depth, int rounding, char *text, int capacity)
{
	(void) kind;
	(void) scene;
	(void) depth;
	(void) rounding;
	(void) text;
	(void) capacity;

	return EECH_ENGINE_BAD_ARGUMENT;
}

#endif

#endif

#endif
