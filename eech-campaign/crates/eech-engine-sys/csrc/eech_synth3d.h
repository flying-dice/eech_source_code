/* EECH headless build: the synthetic 3D object database (eech_synth3d.c, eech_synth3d_keysites.c) */
#ifndef EECH_SYNTH3D_H
#define EECH_SYNTH3D_H

/* writes bininfo.bin, 3dobjs.bin (.pts .sp .spn .spt), 3dobjdb.bin, displace.bin, stars.bin into directory; 1 on success */
int eech_synth3d_write (const char *directory);

/* writes brief_en.dat into an installation's common/data; 1 on success */
int eech_synth_write_briefings (const char *common_data_dir);

struct line
{
	int a, b;
};

/* an object: bounding box, and for routes, points and coloured line surfaces */
struct synth_object
{
	object_3d_bounds box;
	float radius;
	int number_of_points;
	vec3d *points;
	int number_of_surfaces;
	struct
	{
		unsigned char red, green, blue;
		int number_of_lines;
		struct line *lines;
	} surfaces[4];
};

struct synth_sub_object
{
	int object;
	int named_index;	/* OBJECT_3D_SUB_OBJECT_* or 0 */
	vec3d position;
	float heading;
	int number_of_children;
	struct synth_sub_object *children;
};

struct synth_link
{
	int scene;
	vec3d position;
	float heading;
};

struct synth_scene
{
	int object;
	int number_of_sub_objects;
	struct synth_sub_object *sub_objects;
	int number_of_links;
	struct synth_link *links;
};

/* adds an object (ids from 1); the database keeps the pointers it is given */
int eech_synth3d_add_object (const struct synth_object *o);
int eech_synth3d_box_object (float x, float y, float z);

/* airports and FARPs: routes and buildings (eech_synth3d_keysites.c) */
void eech_synth3d_keysites (struct synth_scene *scenes);

#endif
