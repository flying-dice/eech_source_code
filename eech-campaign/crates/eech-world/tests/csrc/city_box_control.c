/*
 * TEST-ONLY numeric control of the ORIGINAL city bounding-box expansion (Task49, D3): the arithmetic statements of ct_msgs.c update_city_bounding_box (the original-fixture build's body), the two maths routines they call
 * (modules/maths/matrix.c get_3d_transformation_matrix and multiply_matrix3x3_vec3d) and nothing else, in a surrogate of the engine's compilation context (the types the statements name). The statements are COPIED
 * VERBATIM by the generator recorded in the test (tests/model_target_point.rs, `the_controls_statements_are_the_originals`), which compares them with the original sources on every run: this file is no second
 * implementation, it is the original text under a test entry. The two reads of the entity (the building's heading and position) are the entry's arguments, as they are the caller's reads in the original.
 *
 * The entry runs the statements under round-toward-zero (the original ran inside the engine, whose entry sets it) with the six exception flags cleared first, and reports the complete outer state before and after:
 * the x87 control word and status word and MXCSR, so that what the ORIGINAL arithmetic does to the floating-point state is measured, not assumed.
 */
#include <fenv.h>
#include <math.h>
#include <stdint.h>
#include <string.h>

typedef struct vec3d
{
	float x, y, z;
} vec3d;

typedef float matrix3x3[3][3];

typedef struct object_3d_bounds
{
	float xmin, xmax, ymin, ymax, zmin, zmax;
} object_3d_bounds;

typedef struct fixed
{
	vec3d position;
} fixed;

typedef struct city
{
	fixed fix;
	vec3d bounding_box_min, bounding_box_max;
} city;

/* ---- modules/maths/matrix.c, verbatim ---- */

static void get_3d_transformation_matrix ( matrix3x3 m, float heading, float pitch, float roll )
{

	float
		sinh,
		cosh,
		sinp,
		cosp,
		sinr,
		cosr,
		sinrsinh,
		sinrcosh,
		cosrsinh,
		cosrcosh;

	sinh = sin ( heading );
	cosh = cos ( heading );

	sinp = sin ( pitch );
	cosp = cos ( pitch );

	sinr = sin ( roll );
	cosr = cos ( roll );

	sinrsinh = sinr * sinh;

	sinrcosh = sinr * cosh;

	cosrsinh = cosr * sinh;

	cosrcosh = cosr * cosh;

	m [ 0 ] [ 0 ] = cosrcosh + ( sinrsinh * sinp );

	m [ 0 ] [ 1 ] = -sinr * cosp;

	m [ 0 ] [ 2 ] = ( sinrcosh * sinp ) - cosrsinh;

	m [ 1 ] [ 0 ] = sinrcosh - ( cosrsinh * sinp );

	m [ 1 ] [ 1 ] = cosr * cosp;

	m [ 1 ] [ 2 ] = -sinrsinh - ( cosrcosh * sinp );

	m [ 2 ] [ 0 ] = cosp * sinh;

	m [ 2 ] [ 1 ] = sinp;

	m [ 2 ] [ 2 ] = cosp * cosh;
}

static void multiply_matrix3x3_vec3d ( vec3d *result, const matrix3x3 m, const vec3d *v )
{

	vec3d
		temp;

	temp.x = v->x * m[0][0];
	temp.x += v->y * m[1][0];
	temp.x += v->z * m[2][0];

	temp.y = v->x * m[0][1];
	temp.y += v->y * m[1][1];
	temp.y += v->z * m[2][1];

	temp.z = v->x * m[0][2];
	temp.z += v->y * m[1][2];
	temp.z += v->z * m[2][2];

	result->x = temp.x;
	result->y = temp.y;
	result->z = temp.z;
}

/* ---- ct_msgs.c update_city_bounding_box, the statements from the bounding box's corners to the last comparison, verbatim; the entity's two reads are the arguments ---- */

static void original_expansion (city *raw, object_3d_bounds *bounding_box, float heading, vec3d *building_position)
{
	vec3d
		position_offset,
		p1,
		p2;

	matrix3x3
		attitude;

	p1.x = bounding_box->xmin;
	p1.y = bounding_box->ymin;
	p1.z = bounding_box->zmin;

	p2.x = bounding_box->xmax;
	p2.y = bounding_box->ymax;
	p2.z = bounding_box->zmax;

	//
	// rotate minmax points (no longer minmax after rotate)
	//


	get_3d_transformation_matrix (attitude, heading, 0.0, 0.0);

	multiply_matrix3x3_vec3d (&p1, attitude, &p1);
	multiply_matrix3x3_vec3d (&p2, attitude, &p2);

	//
	// get building position offset from city centre
	//


	position_offset.x = building_position->x - raw->fix.position.x;
	position_offset.y = building_position->y - raw->fix.position.y;
	position_offset.z = building_position->z - raw->fix.position.z;

	//
	// offset p1 and p2
	//

	p1.x += position_offset.x;
	p1.y += position_offset.y;
	p1.z += position_offset.z;

	p2.x += position_offset.x;
	p2.y += position_offset.y;
	p2.z += position_offset.z;

	//
	// adjust city bounding box
	//

	if (p1.x < raw->bounding_box_min.x) raw->bounding_box_min.x = p1.x;
	if (p1.y < raw->bounding_box_min.y) raw->bounding_box_min.y = p1.y;
	if (p1.z < raw->bounding_box_min.z) raw->bounding_box_min.z = p1.z;
	if (p1.x > raw->bounding_box_max.x) raw->bounding_box_max.x = p1.x;
	if (p1.y > raw->bounding_box_max.y) raw->bounding_box_max.y = p1.y;
	if (p1.z > raw->bounding_box_max.z) raw->bounding_box_max.z = p1.z;

	if (p2.x < raw->bounding_box_min.x) raw->bounding_box_min.x = p2.x;
	if (p2.y < raw->bounding_box_min.y) raw->bounding_box_min.y = p2.y;
	if (p2.z < raw->bounding_box_min.z) raw->bounding_box_min.z = p2.z;
	if (p2.x > raw->bounding_box_max.x) raw->bounding_box_max.x = p2.x;
	if (p2.y > raw->bounding_box_max.y) raw->bounding_box_max.y = p2.y;
	if (p2.z > raw->bounding_box_max.z) raw->bounding_box_max.z = p2.z;
}

#if defined(__x86_64__) || defined(__i386__)
static uint32_t read_mxcsr(void)
{
	uint32_t value = 0;
	__asm__ __volatile__("stmxcsr %0" : "=m"(value));
	return value;
}

static void write_mxcsr(uint32_t value)
{
	__asm__ __volatile__("ldmxcsr %0" : : "m"(value));
}

static uint32_t read_x87_control(void)
{
	uint16_t value = 0;
	__asm__ __volatile__("fnstcw %0" : "=m"(value));
	return value;
}

static uint32_t read_x87_status(void)
{
	uint16_t value = 0;
	__asm__ __volatile__("fnstsw %0" : "=m"(value));
	return value;
}
#else
static uint32_t read_mxcsr(void) { return 0; }
static void write_mxcsr(uint32_t value) { (void) value; }
static uint32_t read_x87_control(void) { return 0; }
static uint32_t read_x87_status(void) { return 0; }
#endif

/*
 * bounds: xmin, xmax, ymin, ymax, zmin, zmax (the scene's bounding box without lines). `report` receives, in order: the x87 control word and the x87 status word and MXCSR before the statements (exception flags
 * cleared, round-toward-zero set), and the same three after them. The caller's own state is restored on return. The box words in `min` and `max` are updated in place, as the original updates the entity's.
 */
int eech_city_box_original(const float *bounds, float heading, const float *building, const float *city_position, float *min, float *max, uint32_t *report)
{
	fenv_t caller;
	uint32_t caller_mxcsr;
	object_3d_bounds box;
	vec3d building_position;
	city entity;

	memcpy(&box, bounds, sizeof(box));
	memcpy(&building_position, building, sizeof(vec3d));
	memset(&entity, 0, sizeof(entity));
	memcpy(&entity.fix.position, city_position, sizeof(vec3d));
	memcpy(&entity.bounding_box_min, min, sizeof(vec3d));
	memcpy(&entity.bounding_box_max, max, sizeof(vec3d));

	caller_mxcsr = read_mxcsr();
	if (fegetenv(&caller) != 0)
	{
		return -1;
	}

	fesetround(FE_TOWARDZERO);
	write_mxcsr((read_mxcsr() & ~0x6000u) | 0x6000u);
	feclearexcept(FE_ALL_EXCEPT);
	write_mxcsr(read_mxcsr() & ~0x3fu);

	report[0] = read_x87_control();
	report[1] = read_x87_status();
	report[2] = read_mxcsr();

	original_expansion(&entity, &box, heading, &building_position);

	report[3] = read_x87_control();
	report[4] = read_x87_status();
	report[5] = read_mxcsr();

	memcpy(min, &entity.bounding_box_min, sizeof(vec3d));
	memcpy(max, &entity.bounding_box_max, sizeof(vec3d));

	fesetenv(&caller);
	write_mxcsr(caller_mxcsr);

	return 0;
}
