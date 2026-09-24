/*
 * EECH headless build: a synthetic 3D object database.
 *
 * The retail 3D database (cohokum/3ddata: bininfo.bin, 3dobjs.bin and its
 * point/reference files, 3dobjdb.bin) is not part of the EECH source
 * repository, and the simulation cannot run without it: every entity
 * constructs a 3D instance, and keysites take their landing routes,
 * landing-site counts and buildings from 3D scenes.
 *
 * This writer produces a database in EECH's own formats, with the names the
 * engine compiled (3dmodels.h, textanim.h), so EECH's unmodified loaders
 * (3dobjid.c, 3dobjdb.c) read it. Every write mirrors a read in those
 * loaders. The content is synthetic:
 *
 *   - every scene has one object with no polygons and a bounding box sized
 *     for what the scene name says it is (aircraft, vehicle, building, ...);
 *   - keysite scenes (airports, FARPs) get route sub-objects built the way
 *     routegen.c reads them (line faces, black start edges, green primary
 *     route) and scene links to buildings;
 *   - cameras, lights, textures and animations are empty.
 */

#include <sys/stat.h>

#include "project.h"

#include "eech_headless.h"
#include "eech_synth3d.h"

extern const char *texture_animation_names_[];

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* output */

struct out
{
	FILE *fp;
	int failed;
};

static void put (struct out *o, const void *data, size_t size)
{
	if (!o->failed && fwrite (data, 1, size, o->fp) != size)
	{
		o->failed = 1;
	}
}

static void put_int (struct out *o, int v) { put (o, &v, 4); }
static void put_uint (struct out *o, unsigned int v) { put (o, &v, 4); }
static void put_float (struct out *o, float v) { put (o, &v, 4); }
static void put_u16 (struct out *o, unsigned short v) { put (o, &v, 2); }
static void put_u8 (struct out *o, unsigned char v) { put (o, &v, 1); }

static int open_out (struct out *o, const char *dir, const char *name)
{
	char path[4096];
	snprintf (path, sizeof (path), "%s/%s", dir, name);
	o->fp = fopen (path, "wb");
	o->failed = !o->fp;
	return !o->failed;
}

static int close_out (struct out *o)
{
	if (o->fp && fclose (o->fp) != 0)
	{
		o->failed = 1;
	}
	o->fp = NULL;
	return !o->failed;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* objects */

static struct synth_object *objects;
static int number_of_objects, objects_allocated;

int eech_synth3d_add_object (const struct synth_object *o)
{
	if (number_of_objects + 1 >= objects_allocated)
	{
		objects_allocated = objects_allocated ? objects_allocated * 2 : 4096;
		objects = realloc (objects, sizeof (*objects) * (size_t) objects_allocated);
	}
	/* object ids start at 1 */
	objects[++number_of_objects] = *o;
	return number_of_objects;
}

int eech_synth3d_box_object (float x, float y, float z)
{
	struct synth_object o;
	memset (&o, 0, sizeof (o));
	o.box.xmin = -x / 2;
	o.box.xmax = x / 2;
	o.box.ymin = 0;
	o.box.ymax = y;
	o.box.zmin = -z / 2;
	o.box.zmax = z / 2;
	o.radius = sqrtf (x * x + y * y + z * z) / 2;
	return eech_synth3d_add_object (&o);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* scene content */

static int contains (const char *name, const char *word)
{
	return strstr (name, word) != NULL;
}

/* bounding box from what the scene name says it is */
static void size_for_scene (const char *name, float *x, float *y, float *z)
{
	*x = 6; *y = 3; *z = 6;
	if (contains (name, "AIRPORT") || contains (name, "AIRFIELD")) { *x = 1500; *y = 10; *z = 1500; }
	else if (contains (name, "FARP")) { *x = 120; *y = 5; *z = 120; }
	else if (contains (name, "CARRIER") || contains (name, "KIEV") || contains (name, "TARAWA")) { *x = 40; *y = 40; *z = 250; }
	else if (contains (name, "SHIP") || contains (name, "FRIGATE") || contains (name, "BOAT") || contains (name, "LCAC") || contains (name, "LCU")) { *x = 15; *y = 12; *z = 80; }
	else if (contains (name, "HANGAR")) { *x = 40; *y = 15; *z = 40; }
	else if (contains (name, "TOWER") || contains (name, "PYLON") || contains (name, "AERIAL") || contains (name, "MAST")) { *x = 6; *y = 30; *z = 6; }
	else if (contains (name, "BUILDING") || contains (name, "HOUSE") || contains (name, "FACTORY") || contains (name, "OFFICE") || contains (name, "CHURCH")
		|| contains (name, "BARRACK") || contains (name, "WAREHOUSE") || contains (name, "BUNKER") || contains (name, "POWER") || contains (name, "REFINERY")
		|| contains (name, "SHED") || contains (name, "TANKGROUP") || contains (name, "STATION")) { *x = 20; *y = 10; *z = 20; }
	else if (contains (name, "BRIDGE")) { *x = 12; *y = 8; *z = 60; }
	else if (contains (name, "TREE") || contains (name, "PALM")) { *x = 4; *y = 12; *z = 4; }
	else if (contains (name, "TENT") || contains (name, "CRATE") || contains (name, "DRUM") || contains (name, "HESCO")) { *x = 4; *y = 3; *z = 4; }
	else if (contains (name, "MARINE") || contains (name, "INFANTRY") || contains (name, "SOLDIER") || contains (name, "PILOT") || contains (name, "PERSON")
		|| contains (name, "CREW") || contains (name, "_MAN")) { *x = 1; *y = 1.8f; *z = 1; }
	else if (contains (name, "MISSILE") || contains (name, "ROCKET") || contains (name, "BOMB") || contains (name, "SHELL") || contains (name, "ROUND")) { *x = 0.4f; *y = 0.4f; *z = 3; }
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* writers: every write mirrors a read in 3dobjid.c / 3dobjdb.c */

static int write_bininfo (const char *dir)
{
	struct out o;
	int i, total;
	if (!open_out (&o, dir, "bininfo.bin"))
	{
		return 0;
	}
	/* scene names 1 .. OLD_LAST-1, each with its maximum draw distance */
	put_int (&o, OBJECT_3D_OLD_LAST);
	for (i = 1, total = 0; i < OBJECT_3D_OLD_LAST; i++)
	{
		total += (int) strlen (object_3d_scene_names[i]) + 1;
	}
	put_int (&o, total);
	for (i = 1; i < OBJECT_3D_OLD_LAST; i++)
	{
		int n = (int) strlen (object_3d_scene_names[i]) + 1;
		put_int (&o, n);
		put (&o, object_3d_scene_names[i], (size_t) n);
		put_float (&o, 4000.0f);
	}
	/* sub-object names 1 .. OLD_LAST-1 */
	put_int (&o, OBJECT_3D_SUB_OBJECT_OLD_LAST);
	for (i = 1, total = 0; i < OBJECT_3D_SUB_OBJECT_OLD_LAST; i++)
	{
		total += (int) strlen (object_3d_subobject_names[i]) + 1;
	}
	put_int (&o, total);
	for (i = 1; i < OBJECT_3D_SUB_OBJECT_OLD_LAST; i++)
	{
		int n = (int) strlen (object_3d_subobject_names[i]) + 1;
		put_int (&o, n);
		put (&o, object_3d_subobject_names[i], (size_t) n);
	}
	/* cameras, camouflage sets: none */
	put_int (&o, 0);
	put_int (&o, 0);
	/* texture animations 0 .. LAST_OLD-1: one frame each, the first texture */
	put_int (&o, TEXTURE_ANIMATION_INDEX_LAST_OLD);
	for (i = 0, total = 0; i < TEXTURE_ANIMATION_INDEX_LAST_OLD; i++)
	{
		total += (int) strlen (texture_animation_names_[i]) + 1;
	}
	put_int (&o, total);
	for (i = 0; i < TEXTURE_ANIMATION_INDEX_LAST_OLD; i++)
	{
		int n = (int) strlen (texture_animation_names_[i]) + 1;
		put_int (&o, n);
		put (&o, texture_animation_names_[i], (size_t) n);
		put_int (&o, 1);
		put_int (&o, 0);
	}
	/* displacement animations: none */
	put_int (&o, 0);
	return close_out (&o);
}

/*
 * 3dobjs.bin and its side files. A route object has one point per node,
 * line surfaces (no polygons flag) whose surface point list is the object's
 * points in order, and 2 point references per line face.
 */
static int write_objects (const char *dir)
{
	struct out bin, pts, sp, spn, spt;
	int i, s, total_surfaces = 0, total_points = 0, total_plain = 0;

	for (i = 1; i <= number_of_objects; i++)
	{
		total_surfaces += objects[i].number_of_surfaces;
		total_points += objects[i].number_of_points;
		for (s = 0; s < objects[i].number_of_surfaces; s++)
		{
			total_plain += objects[i].surfaces[s].number_of_lines * 2;
		}
	}
	if (!open_out (&bin, dir, "3dobjs.bin") || !open_out (&pts, dir, "3dobjs.pts") || !open_out (&sp, dir, "3dobjs.sp")
		|| !open_out (&spn, dir, "3dobjs.spn") || !open_out (&spt, dir, "3dobjs.spt"))
	{
		return 0;
	}
	put_int (&bin, number_of_objects);
	put_int (&bin, total_surfaces);
	put_int (&bin, total_points);
	put_int (&bin, 0);		/* point normals */
	put_int (&bin, total_plain);	/* face point references */
	put_int (&bin, 0);		/* face normal references */
	put_int (&bin, 0);		/* polygons */

	for (i = 1; i <= number_of_objects; i++)
	{
		struct synth_object *ob = &objects[i];
		int surface_point_refs = 0, line_count = 0;
		for (s = 0; s < ob->number_of_surfaces; s++)
		{
			surface_point_refs += ob->number_of_points;
			line_count += ob->surfaces[s].number_of_lines;
		}
		put_int (&bin, 0);				/* pretend_null_object */
		put_int (&bin, ob->number_of_points);
		put_int (&bin, ob->number_of_surfaces);
		put_int (&bin, line_count);			/* faces */
		put_int (&bin, 0);				/* point normals */
		put_int (&bin, 0);				/* lighting normals */
		put_int (&bin, 0);				/* culling normals offset */
		put_float (&bin, ob->radius);
		put_int (&bin, surface_point_refs);		/* surface point references (.sp) */
		put_int (&bin, 0);				/* surface point normals (.spn) */
		put_int (&bin, 0);				/* texture points (.spt) */
		put (&bin, &ob->box, sizeof (object_3d_bounds));
		put (&bin, &ob->box, sizeof (object_3d_bounds));
		for (s = 0; s < ob->number_of_surfaces; s++)
		{
			face_surface_description d;
			memset (&d, 0, sizeof (d));
			d.polygons = 0;
			d.number_of_points = (unsigned char) (ob->number_of_points & 0xff);
			d.red = ob->surfaces[s].red;
			d.green = ob->surfaces[s].green;
			d.blue = ob->surfaces[s].blue;
			put_uint (&bin, d.surface_flags);
			put_u16 (&bin, 0);				/* texture */
			put_u16 (&bin, 0);				/* luminosity texture */
			put_int (&bin, ob->surfaces[s].number_of_lines);
			put_uint (&bin, d.colour);
			put_u8 (&bin, 0);				/* reflectivity */
			put_u8 (&bin, 0);				/* specularity */
		}
		/* point normals: none */
		put_int (&bin, line_count * 2);			/* face point references */
		put_int (&bin, 0);				/* face normal references */
		/* polygon faces: none (line surfaces carry no face sizes) */
		for (s = 0; s < ob->number_of_surfaces; s++)
		{
			for (int l = 0; l < ob->surfaces[s].number_of_lines; l++)
			{
				put_u8 (&bin, (unsigned char) ob->surfaces[s].lines[l].a);
				put_u8 (&bin, (unsigned char) ob->surfaces[s].lines[l].b);
			}
		}
		/* points: short, scaled to the bounding box's largest extent per axis */
		{
			float xm = fmaxf (fabsf (ob->box.xmin), fabsf (ob->box.xmax));
			float ym = fmaxf (fabsf (ob->box.ymin), fabsf (ob->box.ymax));
			float zm = fmaxf (fabsf (ob->box.zmin), fabsf (ob->box.zmax));
			for (int p = 0; p < ob->number_of_points; p++)
			{
				put_u16 (&pts, (unsigned short) (short) (xm > 0 ? ob->points[p].x * 32767.0f / xm : 0));
				put_u16 (&pts, (unsigned short) (short) (ym > 0 ? ob->points[p].y * 32767.0f / ym : 0));
				put_u16 (&pts, (unsigned short) (short) (zm > 0 ? ob->points[p].z * 32767.0f / zm : 0));
			}
		}
		for (s = 0; s < ob->number_of_surfaces; s++)
		{
			for (int p = 0; p < ob->number_of_points; p++)
			{
				put_u16 (&sp, (unsigned short) p);
			}
		}
	}
	/* the side files are memory mapped: never empty */
	put_u16 (&pts, 0); put_u16 (&pts, 0); put_u16 (&pts, 0);
	put_u16 (&sp, 0);
	put_u16 (&spn, 0);
	put_float (&spt, 0); put_float (&spt, 0);
	return close_out (&bin) & close_out (&pts) & close_out (&sp) & close_out (&spn) & close_out (&spt);
}

static void put_identity_keyframes (struct out *o, vec3d position, float heading)
{
	put_int (o, 1);
	put_int (o, 0);			/* index */
	put_int (o, 1);			/* linear */
	put_float (o, position.x);
	put_float (o, position.y);
	put_float (o, position.z);
	put_float (o, heading);
	put_float (o, 0);
	put_float (o, 0);
	put_float (o, 1);
	put_float (o, 1);
	put_float (o, 1);
	put_float (o, 0);
	put_float (o, 0);
	put_float (o, 0);
}

static int count_sub_objects (int n, const struct synth_sub_object *s, int *named)
{
	int total = n;
	for (int i = 0; i < n; i++)
	{
		if (s[i].named_index)
		{
			(*named)++;
		}
		total += count_sub_objects (s[i].number_of_children, s[i].children, named);
	}
	return total;
}

/* read_indices then read_subobjects, recursively (initialise_3d_sub_object) */
static void put_sub_objects (struct out *o, int n, const struct synth_sub_object *s)
{
	int named = 0;
	for (int i = 0; i < n; i++)
	{
		named += s[i].named_index != 0;
	}
	put_int (o, named);
	for (int i = 0; i < n; i++)
	{
		if (s[i].named_index)
		{
			put_int (o, s[i].named_index);
			put_int (o, i);
		}
	}
	put_int (o, n);
	for (int i = 0; i < n; i++)
	{
		put_int (o, s[i].object);
		put_int (o, 0);		/* contributes to collisions */
		put_int (o, 0);		/* approximation in level */
		put_int (o, 0);		/* approximation out level */
		put_int (o, 0);		/* no relative limits */
		put_identity_keyframes (o, s[i].position, s[i].heading);
		put_float (o, 0);	/* dissolve */
		put_int (o, 0);		/* dissolve keyframes */
		put_sub_objects (o, s[i].number_of_children, s[i].children);
	}
}

static int write_scenes (const char *dir, const struct synth_scene *scenes)
{
	struct out o;
	int i;
	vec3d origin = { 0, 0, 0 };
	if (!open_out (&o, dir, "3dobjdb.bin"))
	{
		return 0;
	}
	put_int (&o, OBJECT_3D_LAST - 1);
	for (i = 0; i < 9; i++)
	{
		put_int (&o, 0);	/* database totals (statistics only) */
	}
	for (i = 1; i < OBJECT_3D_LAST; i++)
	{
		const struct synth_scene *sc = &scenes[i];
		int named = 0, total;
		for (int k = 0; k < 16; k++)
		{
			put_int (&o, -1);
		}
		put_int (&o, i);		/* scene index */
		put_int (&o, 0);		/* self shadows */
		put_int (&o, 0);		/* lights */
		put_int (&o, 0);		/* cameras */
		put_int (&o, sc->number_of_links);
		for (int k = 0; k < sc->number_of_links; k++)
		{
			put_int (&o, sc->links[k].scene);
			put_float (&o, sc->links[k].position.x);
			put_float (&o, sc->links[k].position.y);
			put_float (&o, sc->links[k].position.z);
			put_float (&o, sc->links[k].heading);
			put_float (&o, 0);
			put_float (&o, 0);
		}
		put_int (&o, 0);		/* sprite lights */
		total = count_sub_objects (sc->number_of_sub_objects, sc->sub_objects, &named);
		put_int (&o, total);
		put_int (&o, named);
		put_int (&o, 0);		/* texture animations */
		put_int (&o, 0);		/* approximations */
		put_int (&o, sc->object);
		put_int (&o, 0);		/* shadow approximation */
		put_int (&o, 0);		/* shadow polygon object */
		put_float (&o, 1);
		put_float (&o, 1);
		put_float (&o, 1);
		put_int (&o, -1);		/* collision object */
		put_identity_keyframes (&o, origin, 0);
		put_float (&o, 0);		/* dissolve */
		put_int (&o, 0);		/* dissolve keyframes */
		put_int (&o, 0);		/* displacement sequence */
		put_int (&o, 0);		/* displacement keyframes */
		put_sub_objects (&o, sc->number_of_sub_objects, sc->sub_objects);
	}
	put_int (&o, 0);			/* cameras */
	put_int (&o, 0);			/* camera keyframes */
	return close_out (&o);
}

/*
 * rendering tables the 3D system loads at start: displacement maps
 * (3ddisp.c), none; the horizon directory, where 3dhorizn.c caches the
 * converted horizon images
 */
static int write_render_tables (const char *dir)
{
	struct out o;
	char horizon[4096];
	snprintf (horizon, sizeof (horizon), "%s/horizon", dir);
	mkdir (horizon, 0755);
	if (!open_out (&o, dir, "displace.bin"))
	{
		return 0;
	}
	put_int (&o, 0);
	if (!close_out (&o))
	{
		return 0;
	}
	/* stars (3dstars.c): one white star */
	if (!open_out (&o, dir, "stars.bin"))
	{
		return 0;
	}
	put_int (&o, 1);
	put_int (&o, 1);
	put_float (&o, 1); put_float (&o, 1); put_float (&o, 1); put_float (&o, 0);
	put_float (&o, -1); put_float (&o, -1); put_float (&o, -1);
	put_float (&o, 2.0f / 65535); put_float (&o, 2.0f / 65535); put_float (&o, 2.0f / 65535);
	put_u16 (&o, 32767); put_u16 (&o, 65535); put_u16 (&o, 32767); put_u16 (&o, 0);
	return close_out (&o);
}

/*
 * textures.bin (textuser.c :: load_texturemap_data): no palettes, then every
 * system texture as a reserved slot with its name. Reserved textures carry
 * no pixels; nothing headless draws them, and their names resolve.
 */
extern const char *const eech_texture_names[];
extern const int eech_number_of_texture_names;

static int write_textures (const char *dir)
{
	struct out o;
	if (!open_out (&o, dir, "textures.bin"))
	{
		return 0;
	}
	put_int (&o, 0);
	put_int (&o, eech_number_of_texture_names);
	for (int i = 0; i < eech_number_of_texture_names; i++)
	{
		int n = (int) strlen (eech_texture_names[i]);
		put_uint (&o, 1);	/* flags: reserved_texture */
		put_int (&o, n);
		put (&o, eech_texture_names[i], (size_t) n);
	}
	return close_out (&o);
}

/*
 * brief_en.dat (briefing.c): a briefing and debriefing text for every task
 * type. The retail texts are not in the repository.
 */
int eech_synth_write_briefings (const char *common_data_dir)
{
	struct out o;
	if (!open_out (&o, common_data_dir, "brief_en.dat"))
	{
		return 0;
	}
	fprintf (o.fp, ":START\n");
	for (int t = 0; t < NUM_ENTITY_SUB_TYPE_TASKS; t++)
	{
		fprintf (o.fp, ":TYPE %s\n", entity_sub_type_task_names[t]);
		fprintf (o.fp, ":TEXT1\nOrders received.\n:TEXT2\nProceed as tasked.\n:TEXT3\nReport on completion.\n:END\n");
		fprintf (o.fp, ":SUCCESS\nObjective achieved.\n:PARTIAL\nObjective partially achieved.\n:FAILURE\nObjective not achieved.\n:END\n");
	}
	fprintf (o.fp, ":END\n");
	return close_out (&o);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */

int eech_synth3d_write (const char *directory)
{
	struct synth_scene *scenes = calloc (OBJECT_3D_LAST, sizeof (*scenes));
	int ok;
	number_of_objects = 0;

	for (int i = 1; i < OBJECT_3D_LAST; i++)
	{
		float x, y, z;
		const char *name = object_3d_scene_names[i] ? object_3d_scene_names[i] : "";
		size_for_scene (name, &x, &y, &z);
		scenes[i].object = eech_synth3d_box_object (x, y, z);
	}
	eech_synth3d_keysites (scenes);

	ok = write_bininfo (directory) && write_objects (directory) && write_scenes (directory, scenes) && write_render_tables (directory)
		&& write_textures (directory);
	free (scenes);
	return ok;
}
