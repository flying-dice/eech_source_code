/* Scratch control (EEDCS-49 D3, i686 enumeration pose): the ORIGINAL face-normal placement text (extracted by function name from modules/maths, see pose-control.sh)
   in a surrogate compilation context, driven with raw words, under round-toward-zero as the host's guard sets it. Prints one line per input: the four input words, then the nine
   words of the matrix. Inputs are made from integers and exact power-of-two scalings only, so every target is given the same words. */
#include <fenv.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define ASSERT(x)
#ifdef PRODUCTION_TEXT
#include "entity_candidate_step.c"
#else
typedef struct vec3d { float x, y, z; } vec3d;
typedef float matrix3x3[3][3];
#include "original_extract.h"
#endif

static uint64_t state = 0x5eed1234abcdef01ull;
static uint32_t next(void) { state ^= state << 13; state ^= state >> 7; state ^= state << 17; return (uint32_t) (state >> 16); }

int main(int argc, char **argv)
{
	int count = argc > 1 ? atoi(argv[1]) : 2000;
	fesetround(FE_TOWARDZERO);
	for (int i = 0; i < count; i++)
	{
		vec3d normal;
		matrix3x3 result;
		float heading;
		uint32_t words[9], in[4];
		/* exact: 16-bit integers scaled by powers of two */
		normal.x = ((int) (next() & 0xffff) - 32768) / 131072.0f;
		normal.z = ((int) (next() & 0xffff) - 32768) / 131072.0f;
		normal.y = 0.5f + (next() & 0xffff) / 131072.0f;
		heading = (next() & 0x7fffff) / 1048576.0f;
		memset(result, 0, sizeof result);
		get_3d_transformation_matrix_from_face_normal_and_heading(result, &normal, heading);
		memcpy(words, result, sizeof words);
		memcpy(in, &normal, 12); memcpy(in + 3, &heading, 4);
		printf("%08x %08x %08x %08x", in[0], in[1], in[2], in[3]);
		for (int k = 0; k < 9; k++) printf(" %08x", words[k]);
		printf("\n");
	}
	return 0;
}
