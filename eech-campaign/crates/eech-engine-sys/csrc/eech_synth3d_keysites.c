/*
 * EECH headless build: keysite scenes of the synthetic 3D database (see
 * eech_synth3d.c): airports and FARPs.
 *
 * popread.c finds a keysite's routes as named sub-objects of its scene
 * (FIXED_WING_LANDING_ROUTE, HELI_TAKEOFF_ROUTE, ...) and reads each one's
 * raw object as a line mesh (routegen.c :: parse_waypoint_routes_from_object):
 *
 *   - a route is one tree per slot; a tree starts at a node that ends a black
 *     line and is referenced once, and grows along the other lines;
 *   - the green lines are the primary route (the first slot's tree);
 *   - depth 0 is the far end (the approach, or the point a takeoff reaches);
 *     the deepest level is the slots on the ground (landing sites: their
 *     number is the keysite's landing-site count);
 *   - positions are metres relative to the keysite, y up.
 *
 * A takeoff route's deepest nodes must be the landing route's slots
 * (routegen.c :: match_end_slots). Scene links place the keysite's buildings;
 * the fixed object database (fx_objdb.c) makes hangars and towers important.
 */

#include "project.h"

#include "eech_synth3d.h"

/* depth, slot -> position */
typedef vec3d (*route_point) (int depth, int slot);

/* builds a route object: `slots` chains of `depths` nodes (depths >= 3) */
static int route_object (int slots, int depths, route_point at)
{
	struct synth_object o;
	int n = slots * depths;
	memset (&o, 0, sizeof (o));
	o.number_of_points = n;
	o.points = calloc ((size_t) n, sizeof (vec3d));
	/* node (slot s, depth d) = s * depths + d */
	for (int s = 0; s < slots; s++)
	{
		for (int d = 0; d < depths; d++)
		{
			o.points[s * depths + d] = at (d, s);
		}
	}
	/* surface 0: black start lines; 1: green primary route; 2: blue other routes */
	o.number_of_surfaces = 3;
	o.surfaces[0].red = o.surfaces[0].green = o.surfaces[0].blue = 0;
	o.surfaces[1].green = 255;
	o.surfaces[2].blue = 255;
	for (int k = 0; k < 3; k++)
	{
		o.surfaces[k].lines = calloc ((size_t) n, sizeof (struct line));
	}
	for (int s = 0; s < slots; s++)
	{
		int base = s * depths;
		o.surfaces[0].lines[o.surfaces[0].number_of_lines++] = (struct line) { base, base + 1 };
		for (int d = 1; d + 1 < depths; d++)
		{
			int k = s == 0 ? 1 : 2;
			o.surfaces[k].lines[o.surfaces[k].number_of_lines++] = (struct line) { base + d, base + d + 1 };
		}
	}
	if (o.surfaces[2].number_of_lines == 0)
	{
		o.number_of_surfaces = 2;
	}
	/* bounding box around the points */
	o.box.xmin = o.box.ymin = o.box.zmin = 1e9f;
	o.box.xmax = o.box.ymax = o.box.zmax = -1e9f;
	for (int i = 0; i < n; i++)
	{
		o.box.xmin = fminf (o.box.xmin, o.points[i].x);
		o.box.xmax = fmaxf (o.box.xmax, o.points[i].x);
		o.box.ymin = fminf (o.box.ymin, o.points[i].y);
		o.box.ymax = fmaxf (o.box.ymax, o.points[i].y);
		o.box.zmin = fminf (o.box.zmin, o.points[i].z);
		o.box.zmax = fmaxf (o.box.zmax, o.points[i].z);
	}
	o.radius = sqrtf (fmaxf (o.box.xmax * o.box.xmax, o.box.xmin * o.box.xmin) + fmaxf (o.box.zmax * o.box.zmax, o.box.zmin * o.box.zmin));
	return eech_synth3d_add_object (&o);
}

static vec3d v (float x, float y, float z)
{
	vec3d p = { x, y, z };
	return p;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* airport: runway along z (x = 0) from z = -1200 to 1200, apron east of it */

#define AIRPORT_PARKING 16

static vec3d airport_parking (int slot)
{
	return v (300.0f + 80.0f * (float) (slot / 8), 0.0f, 500.0f - 70.0f * (float) (slot % 8));
}

static vec3d fixed_wing_landing (int depth, int slot)
{
	float lateral = 5.0f * (float) (slot % 4);
	switch (depth)
	{
		case 0: return v (lateral, 300.0f, -7000.0f - 150.0f * (float) slot);	/* approach */
		case 1: return v (lateral, 90.0f, -3000.0f - 150.0f * (float) slot);	/* final */
		case 2: return v (lateral, 0.0f, -1100.0f);				/* touchdown */
		case 3: return v (lateral, 0.0f, 900.0f);				/* rollout */
		case 4: return v (150.0f, 0.0f, 900.0f - 70.0f * (float) slot);	/* taxi */
		default: return airport_parking (slot);					/* parked */
	}
}

static vec3d fixed_wing_takeoff (int depth, int slot)
{
	float lateral = 5.0f * (float) (slot % 4);
	switch (depth)
	{
		case 0: return v (lateral, 300.0f, 7000.0f + 150.0f * (float) slot);	/* climb out */
		case 1: return v (lateral, 60.0f, 2500.0f + 150.0f * (float) slot);
		case 2: return v (lateral, 0.0f, 1100.0f);				/* lift off */
		case 3: return v (lateral, 0.0f, -1100.0f);				/* line up */
		case 4: return v (150.0f, 0.0f, -900.0f - 70.0f * (float) slot);	/* taxi */
		default: return airport_parking (slot);
	}
}

/* helicopter pads west of the runway */
#define HELI_PADS 12

static vec3d airport_heli_pad (int slot)
{
	return v (-300.0f - 40.0f * (float) (slot / 6), 0.0f, 300.0f - 40.0f * (float) (slot % 6));
}

static vec3d airport_heli_landing (int depth, int slot)
{
	vec3d pad = airport_heli_pad (slot);
	switch (depth)
	{
		case 0: return v (pad.x - 1500.0f, 90.0f, pad.z);
		case 1: return v (pad.x - 600.0f, 35.0f, pad.z);
		case 2: return v (pad.x - 30.0f, 6.0f, pad.z);
		default: return pad;
	}
}

static vec3d airport_heli_takeoff (int depth, int slot)
{
	vec3d pad = airport_heli_pad (slot);
	switch (depth)
	{
		case 0: return v (pad.x - 1500.0f, 90.0f, pad.z - 400.0f);
		case 1: return v (pad.x - 600.0f, 35.0f, pad.z - 150.0f);
		case 2: return v (pad.x, 8.0f, pad.z);
		default: return pad;
	}
}

/* vehicles park south-east of the apron and leave towards the east */
#define VEHICLE_SLOTS 8

static vec3d airport_vehicle_slot (int slot)
{
	return v (450.0f + 15.0f * (float) (slot % 4), 0.0f, -300.0f - 15.0f * (float) (slot / 4));
}

static vec3d airport_vehicle_landing (int depth, int slot)
{
	vec3d p = airport_vehicle_slot (slot);
	switch (depth)
	{
		case 0: return v (1200.0f, 0.0f, p.z - 200.0f);
		case 1: return v (700.0f, 0.0f, p.z - 100.0f);
		default: return p;
	}
}

static vec3d airport_vehicle_takeoff (int depth, int slot)
{
	vec3d p = airport_vehicle_slot (slot);
	switch (depth)
	{
		case 0: return v (1200.0f, 0.0f, p.z + 200.0f);
		case 1: return v (700.0f, 0.0f, p.z + 100.0f);
		default: return p;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* FARP: four pads in a square */

#define FARP_PADS 8

static vec3d farp_pad (int slot)
{
	return v (-45.0f + 30.0f * (float) (slot % 4), 0.0f, (slot < 4) ? 25.0f : -25.0f);
}

static vec3d farp_landing (int depth, int slot)
{
	vec3d pad = farp_pad (slot);
	switch (depth)
	{
		case 0: return v (pad.x - 1200.0f, 80.0f, pad.z);
		case 1: return v (pad.x - 400.0f, 30.0f, pad.z);
		case 2: return v (pad.x - 20.0f, 5.0f, pad.z);
		default: return pad;
	}
}

static vec3d farp_takeoff (int depth, int slot)
{
	vec3d pad = farp_pad (slot);
	switch (depth)
	{
		case 0: return v (pad.x + 1200.0f, 80.0f, pad.z);
		case 1: return v (pad.x + 400.0f, 30.0f, pad.z);
		case 2: return v (pad.x, 8.0f, pad.z);
		default: return pad;
	}
}

static vec3d farp_vehicle_slot (int slot)
{
	return v (60.0f + 12.0f * (float) slot, 0.0f, -60.0f);
}

static vec3d farp_vehicle_landing (int depth, int slot)
{
	vec3d p = farp_vehicle_slot (slot);
	switch (depth)
	{
		case 0: return v (p.x + 600.0f, 0.0f, p.z - 300.0f);
		case 1: return v (p.x + 200.0f, 0.0f, p.z - 100.0f);
		default: return p;
	}
}

static vec3d farp_vehicle_takeoff (int depth, int slot)
{
	vec3d p = farp_vehicle_slot (slot);
	switch (depth)
	{
		case 0: return v (p.x + 600.0f, 0.0f, p.z + 300.0f);
		case 1: return v (p.x + 200.0f, 0.0f, p.z + 100.0f);
		default: return p;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/*
 * holding routes: one chain around a circuit (popread.c makes the first node
 * the HOLDING_LOOP entry, pushed outwards, and the rest HOLDING points)
 */

#define HOLDING_POINTS 8

static vec3d circuit (int depth, float cx, float cz, float radius, float altitude, float start)
{
	float a = start + (float) depth * (2.0f * PI / HOLDING_POINTS);
	return v (cx + radius * sinf (a), altitude, cz + radius * cosf (a));
}

static vec3d fixed_wing_landing_holding (int depth, int slot) { (void) slot; return circuit (depth, 0.0f, -2000.0f, 3000.0f, 600.0f, 0.0f); }
static vec3d fixed_wing_takeoff_holding (int depth, int slot) { (void) slot; return circuit (depth, 0.0f, 2000.0f, 3000.0f, 600.0f, PI); }
static vec3d heli_landing_holding (int depth, int slot) { (void) slot; return circuit (depth, -300.0f, 0.0f, 900.0f, 150.0f, 0.0f); }
static vec3d heli_takeoff_holding (int depth, int slot) { (void) slot; return circuit (depth, -300.0f, 0.0f, 900.0f, 150.0f, PI); }
static vec3d farp_landing_holding (int depth, int slot) { (void) slot; return circuit (depth, 0.0f, 0.0f, 700.0f, 120.0f, 0.0f); }
static vec3d farp_takeoff_holding (int depth, int slot) { (void) slot; return circuit (depth, 0.0f, 0.0f, 700.0f, 120.0f, PI); }
static vec3d vehicle_landing_holding (int depth, int slot) { (void) slot; return circuit (depth, 600.0f, -400.0f, 150.0f, 0.0f, 0.0f); }
static vec3d vehicle_takeoff_holding (int depth, int slot) { (void) slot; return circuit (depth, 600.0f, -400.0f, 150.0f, 0.0f, PI); }
static vec3d farp_vehicle_holding (int depth, int slot) { (void) slot; return circuit (depth, 150.0f, -150.0f, 100.0f, 0.0f, 0.0f); }

/* ---------------------------------------------------------------------------------------------------------------------------- */

struct route_spec
{
	int named_index, slots, depths;
	route_point at;
};

static void set_routes (struct synth_scene *scene, const struct route_spec *routes, int n)
{
	scene->number_of_sub_objects = n;
	scene->sub_objects = calloc ((size_t) n, sizeof (struct synth_sub_object));
	for (int i = 0; i < n; i++)
	{
		scene->sub_objects[i].named_index = routes[i].named_index;
		scene->sub_objects[i].object = route_object (routes[i].slots, routes[i].depths, routes[i].at);
	}
}

static void set_links (struct synth_scene *scene, const struct synth_link *links, int n)
{
	scene->number_of_links = n;
	scene->links = calloc ((size_t) n, sizeof (struct synth_link));
	memcpy (scene->links, links, sizeof (struct synth_link) * (size_t) n);
}

void eech_synth3d_keysites (struct synth_scene *scenes)
{
	static const struct route_spec airport_routes[] =
	{
		{ OBJECT_3D_SUB_OBJECT_FIXED_WING_LANDING_ROUTE, AIRPORT_PARKING, 6, fixed_wing_landing },
		{ OBJECT_3D_SUB_OBJECT_FIXED_WING_TAKEOFF_ROUTE, AIRPORT_PARKING, 6, fixed_wing_takeoff },
		{ OBJECT_3D_SUB_OBJECT_HELI_LANDING_ROUTE, HELI_PADS, 4, airport_heli_landing },
		{ OBJECT_3D_SUB_OBJECT_HELI_TAKEOFF_ROUTE, HELI_PADS, 4, airport_heli_takeoff },
		{ OBJECT_3D_SUB_OBJECT_ROUTED_VEHICLE_LANDING_ROUTE, VEHICLE_SLOTS, 3, airport_vehicle_landing },
		{ OBJECT_3D_SUB_OBJECT_ROUTED_VEHICLE_TAKEOFF_ROUTE, VEHICLE_SLOTS, 3, airport_vehicle_takeoff },
		{ OBJECT_3D_SUB_OBJECT_FIXED_WING_LANDING_HOLDING_ROUTE, 1, HOLDING_POINTS, fixed_wing_landing_holding },
		{ OBJECT_3D_SUB_OBJECT_FIXED_WING_TAKEOFF_HOLDING_ROUTE, 1, HOLDING_POINTS, fixed_wing_takeoff_holding },
		{ OBJECT_3D_SUB_OBJECT_HELI_LANDING_HOLDING_ROUTE, 1, HOLDING_POINTS, heli_landing_holding },
		{ OBJECT_3D_SUB_OBJECT_HELI_TAKEOFF_HOLDING_ROUTE, 1, HOLDING_POINTS, heli_takeoff_holding },
		{ OBJECT_3D_SUB_OBJECT_ROUTED_VEHICLE_LANDING_HOLDING_ROUTE, 1, HOLDING_POINTS, vehicle_landing_holding },
		{ OBJECT_3D_SUB_OBJECT_ROUTED_VEHICLE_TAKEOFF_HOLDING_ROUTE, 1, HOLDING_POINTS, vehicle_takeoff_holding },
	};
	static const struct route_spec farp_routes[] =
	{
		{ OBJECT_3D_SUB_OBJECT_HELI_LANDING_ROUTE, FARP_PADS, 4, farp_landing },
		{ OBJECT_3D_SUB_OBJECT_HELI_TAKEOFF_ROUTE, FARP_PADS, 4, farp_takeoff },
		{ OBJECT_3D_SUB_OBJECT_ROUTED_VEHICLE_LANDING_ROUTE, 4, 3, farp_vehicle_landing },
		{ OBJECT_3D_SUB_OBJECT_ROUTED_VEHICLE_TAKEOFF_ROUTE, 4, 3, farp_vehicle_takeoff },
		{ OBJECT_3D_SUB_OBJECT_HELI_LANDING_HOLDING_ROUTE, 1, HOLDING_POINTS, farp_landing_holding },
		{ OBJECT_3D_SUB_OBJECT_HELI_TAKEOFF_HOLDING_ROUTE, 1, HOLDING_POINTS, farp_takeoff_holding },
		{ OBJECT_3D_SUB_OBJECT_ROUTED_VEHICLE_LANDING_HOLDING_ROUTE, 1, HOLDING_POINTS, farp_vehicle_holding },
		{ OBJECT_3D_SUB_OBJECT_ROUTED_VEHICLE_TAKEOFF_HOLDING_ROUTE, 1, HOLDING_POINTS, farp_vehicle_holding },
	};
	const struct synth_link american_buildings[] =
	{
		{ OBJECT_3D_AMERICAN_CONTROL_TOWER01, { 250.0f, 0.0f, 0.0f }, 0.0f },
		{ OBJECT_3D_AMERICAN_HANGAR01, { 420.0f, 0.0f, 450.0f }, 0.0f },
		{ OBJECT_3D_AMERICAN_HANGAR02, { 420.0f, 0.0f, 300.0f }, 0.0f },
		{ OBJECT_3D_AMERICAN_HANGAR03, { 420.0f, 0.0f, 150.0f }, 0.0f },
	};
	const struct synth_link russian_buildings[] =
	{
		{ OBJECT_3D_RUSSIAN_CONTROL_TOWER01, { 250.0f, 0.0f, 0.0f }, 0.0f },
		{ OBJECT_3D_RUSSIAN_HANGAR01, { 420.0f, 0.0f, 450.0f }, 0.0f },
		{ OBJECT_3D_RUSSIAN_HANGAR02, { 420.0f, 0.0f, 300.0f }, 0.0f },
		{ OBJECT_3D_RUSSIAN_HANGAR03, { 420.0f, 0.0f, 150.0f }, 0.0f },
	};
	static const int american_airports[] = { OBJECT_3D_AMERICAN_AIRPORT01, OBJECT_3D_AMERICAN_AIRPORT02, OBJECT_3D_AMERICAN_AIRPORT03 };
	static const int russian_airports[] =
	{
		OBJECT_3D_RUSSIAN_AIRPORT01, OBJECT_3D_RUSSIAN_AIRPORT02, OBJECT_3D_RUSSIAN_AIRPORT03, OBJECT_3D_RUSSIAN_AIRPORT04, OBJECT_3D_RUSSIAN_AIRPORT05,
		OBJECT_3D_RUSSIAN_AIRPORT06, OBJECT_3D_RUSSIAN_AIRPORT07, OBJECT_3D_RUSSIAN_AIRPORT09, OBJECT_3D_RUSSIAN_AIRPORT10, OBJECT_3D_RUSSIAN_AIRPORT11,
		OBJECT_3D_RUSSIAN_AIRPORT12, OBJECT_3D_RUSSIAN_AIRPORT13
	};
	static const int farps[] =
	{
		OBJECT_3D_AMERICAN_FARP01, OBJECT_3D_AMERICAN_FARP02, OBJECT_3D_AMERICAN_FARP03, OBJECT_3D_AMERICAN_FARP04, OBJECT_3D_AMERICAN_FARP05,
		OBJECT_3D_AMERICAN_FARP06, OBJECT_3D_AMERICAN_FARP07, OBJECT_3D_AMERICAN_FARP08, OBJECT_3D_AMERICAN_FARP09, OBJECT_3D_AMERICAN_FARP10,
		OBJECT_3D_AMERICAN_FARP11, OBJECT_3D_AMERICAN_FARP12, OBJECT_3D_AMERICAN_FARP13,
		OBJECT_3D_RUSSIAN_FARP01, OBJECT_3D_RUSSIAN_FARP02, OBJECT_3D_RUSSIAN_FARP03, OBJECT_3D_RUSSIAN_FARP04, OBJECT_3D_RUSSIAN_FARP05,
		OBJECT_3D_RUSSIAN_FARP06, OBJECT_3D_RUSSIAN_FARP07, OBJECT_3D_RUSSIAN_FARP08, OBJECT_3D_RUSSIAN_FARP09, OBJECT_3D_RUSSIAN_FARP10,
		OBJECT_3D_RUSSIAN_FARP11, OBJECT_3D_RUSSIAN_FARP12, OBJECT_3D_RUSSIAN_FARP13
	};
	size_t i;

	for (i = 0; i < ARRAY_LENGTH (american_airports); i++)
	{
		set_routes (&scenes[american_airports[i]], airport_routes, ARRAY_LENGTH (airport_routes));
		set_links (&scenes[american_airports[i]], american_buildings, ARRAY_LENGTH (american_buildings));
	}
	for (i = 0; i < ARRAY_LENGTH (russian_airports); i++)
	{
		set_routes (&scenes[russian_airports[i]], airport_routes, ARRAY_LENGTH (airport_routes));
		set_links (&scenes[russian_airports[i]], russian_buildings, ARRAY_LENGTH (russian_buildings));
	}
	for (i = 0; i < ARRAY_LENGTH (farps); i++)
	{
		set_routes (&scenes[farps[i]], farp_routes, ARRAY_LENGTH (farp_routes));
	}
}
