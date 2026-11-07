/*
 * The arithmetic of a sector's high-water height raise (EEDCS-49, D3, Task49): the ONE physical expression the original keeps in sc_msgs.c, `response_to_link_child`, for a fixed sender with a valid shape:
 *
 *	height = position->y + bounding_box->ymax;
 *	if (height > raw->tallest_structure_height) raw->tallest_structure_height = height;
 *
 * a `float` local, one float addition and a strict greater-than against the word the sector ACTUALLY holds. Nothing here clamps, takes a maximum, orders a NaN or compares against zero: a negative, zero,
 * infinite or NaN sum is stored only if it is strictly greater than the held word, so over a zero history a negative or zero sum never stores, a restored negative word can be raised and a restored NaN word is
 * never exceeded. The word is held and handed back as a float in memory (the caller copies the bits in and out), never re-derived.
 *
 * It is a header so that the host's kernel (model_step.c, `eech_model_sector_height_raise`) and the real-engine control (eech-engine-sys, csrc/eech_sector_height_check.c, which runs the original response beside
 * this expression over the same inputs) run THIS text and not a copy. The includer provides the floating-point environment: the original runs under round-toward-zero, and model_step.c's entry sets it
 * (and requires every exception masked) around this body.
 */
#ifndef SECTOR_HEIGHT_RAISE_H
#define SECTOR_HEIGHT_RAISE_H

static void __attribute__((noinline)) sector_height_raise_body(const float *position_y, const float *box_ymax, float *tallest_structure_height)
{
	float
		height;

	height = *position_y + *box_ymax;

	if (height > *tallest_structure_height)
	{
		*tallest_structure_height = height;
	}
}

#endif
