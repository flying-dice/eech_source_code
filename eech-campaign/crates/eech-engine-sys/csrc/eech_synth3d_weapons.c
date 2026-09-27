/*
 * EECH headless build: weapon systems in the synthetic 3D database (see
 * eech_synth3d.c).
 *
 * EECH aims and fires through 3D sub-objects: weapon.c finds a launcher's
 * WEAPON_SYSTEM_HEADING and, below it, WEAPON_SYSTEM_PITCH (by the weapon
 * package's heading_depth and pitch_depth: the n-th such sub-object), turns
 * them to the target and computes the weapon and target vectors from the
 * pitch device. Without them INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID is
 * never set and no AI launcher ever fires (aircraft_fire_weapon returns
 * WEAPON_SYSTEM_NOT_READY; vehicles the same).
 *
 * Every aircraft and vehicle scene therefore gets a weapon-system tree deep
 * enough for every weapon configuration its type can carry: H heading
 * devices at the scene root, each with P pitch devices, each with M muzzles
 * and W weapon mounts, where H, P, M and W are the largest depths and counts
 * in weapon_config_database for the type's configurations.
 */

#include "project.h"

#include "eech_synth3d.h"

struct extent
{
	int headings, pitches, muzzles, weapons;
};

static void add_config (struct extent *e, int config)
{
	if (!weapon_config_type_valid (config))
	{
		return;
	}
	for (int package = 0; package < NUM_WEAPON_PACKAGES; package++)
	{
		const weapon_package *p = &weapon_config_database[config][package];
		if (p->sub_type == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			continue;
		}
		e->headings = max (e->headings, p->heading_depth + 1);
		e->pitches = max (e->pitches, p->pitch_depth + 1);
		e->muzzles = max (e->muzzles, p->muzzle_depth + 1);
		e->weapons = max (e->weapons, p->number);
	}
}

static void add_configs (struct extent *e, int min, int max, int extra1, int extra2, int extra3, int extra4)
{
	if (weapon_config_type_valid (min) && weapon_config_type_valid (max))
	{
		for (int c = min; c <= max; c++)
		{
			add_config (e, c);
		}
	}
	add_config (e, extra1);
	add_config (e, extra2);
	add_config (e, extra3);
	add_config (e, extra4);
}

/* a sub-object that can turn freely (the weapon package's own limits govern) */
static struct synth_sub_object device (int object, int named_index)
{
	struct synth_sub_object s;
	memset (&s, 0, sizeof (s));
	s.object = object;
	s.named_index = named_index;
	s.limits = 1;
	s.heading_minimum = -PI;
	s.heading_maximum = PI;
	s.pitch_minimum = -PI / 2;
	s.pitch_maximum = PI / 2;
	return s;
}

static void attach (struct synth_scene *scene, const struct extent *e, int object)
{
	int n = scene->number_of_sub_objects + e->headings;
	scene->sub_objects = realloc (scene->sub_objects, sizeof (struct synth_sub_object) * (size_t) n);
	for (int h = 0; h < e->headings; h++)
	{
		struct synth_sub_object *heading = &scene->sub_objects[scene->number_of_sub_objects + h];
		*heading = device (object, OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_HEADING);
		heading->number_of_children = e->pitches;
		heading->children = calloc ((size_t) e->pitches, sizeof (struct synth_sub_object));
		for (int p = 0; p < e->pitches; p++)
		{
			struct synth_sub_object *pitch = &heading->children[p];
			*pitch = device (object, OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_PITCH);
			pitch->number_of_children = e->muzzles + e->weapons;
			pitch->children = calloc ((size_t) pitch->number_of_children, sizeof (struct synth_sub_object));
			for (int m = 0; m < e->muzzles; m++)
			{
				pitch->children[m] = device (object, OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_MUZZLE);
				pitch->children[m].position.z = 2.0f;
			}
			for (int w = 0; w < e->weapons; w++)
			{
				pitch->children[e->muzzles + w] = device (object, OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_WEAPON);
				pitch->children[e->muzzles + w].position.z = 1.0f;
			}
		}
	}
	scene->number_of_sub_objects = n;
}

void eech_synth3d_weapon_systems (struct synth_scene *scenes)
{
	int object = eech_synth3d_box_object (0.2f, 0.2f, 0.2f);
	struct extent *extents = calloc (OBJECT_3D_LAST, sizeof (struct extent));

	for (int t = 0; t < NUM_ENTITY_SUB_TYPE_AIRCRAFT; t++)
	{
		const aircraft_data *a = &aircraft_database[t];
		if (a->default_3d_shape > OBJECT_3D_INVALID_OBJECT_INDEX && a->default_3d_shape < OBJECT_3D_LAST)
		{
			add_configs (&extents[a->default_3d_shape], a->min_weapon_config_type, a->max_weapon_config_type, a->default_weapon_config_type,
				a->air_to_air_weapon_config_type, a->air_to_surface_weapon_config_type, a->scout_weapon_config_type);
		}
	}
	for (int t = 0; t < NUM_ENTITY_SUB_TYPE_VEHICLES; t++)
	{
		const vehicle_data *v = &vehicle_database[t];
		if (v->default_3d_shape > OBJECT_3D_INVALID_OBJECT_INDEX && v->default_3d_shape < OBJECT_3D_LAST)
		{
			add_configs (&extents[v->default_3d_shape], v->min_weapon_config_type, v->max_weapon_config_type, v->default_weapon_config_type, -1, -1, -1);
		}
	}
	for (int s = 1; s < OBJECT_3D_LAST; s++)
	{
		if (extents[s].headings > 0)
		{
			attach (&scenes[s], &extents[s], object);
		}
	}
	free (extents);
}
