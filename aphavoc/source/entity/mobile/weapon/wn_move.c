//
// 	 Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	 Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	 677 High Road, North Finchley, London N12 0DA
//
// 	 Please see the document LICENSE.TXT for the full licence agreement
//
// 2. LICENCE
//  2.1
//  	Subject to the provisions of this Agreement we now grant to you the
//  	following rights in respect of the Source Code:
//   2.1.1
//   	the non-exclusive right to Exploit  the Source Code and Executable
//   	Code on any medium; and
//   2.1.2
//   	the non-exclusive right to create and distribute Derivative Works.
//  2.2
//  	Subject to the provisions of this Agreement we now grant you the
// 	following rights in respect of the Object Code:
//   2.2.1
// 	the non-exclusive right to Exploit the Object Code on the same
// 	terms and conditions set out in clause 3, provided that any
// 	distribution is done so on the terms of this Agreement and is
// 	accompanied by the Source Code and Executable Code (as
// 	applicable).
//
// 3. GENERAL OBLIGATIONS
//  3.1
//  	In consideration of the licence granted in clause 2.1 you now agree:
//   3.1.1
// 	that when you distribute the Source Code or Executable Code or
// 	any Derivative Works to Recipients you will also include the
// 	terms of this Agreement;
//   3.1.2
// 	that when you make the Source Code, Executable Code or any
// 	Derivative Works ("Materials") available to download, you will
// 	ensure that Recipients must accept the terms of this Agreement
// 	before being allowed to download such Materials;
//   3.1.3
// 	that by Exploiting the Source Code or Executable Code you may
// 	not impose any further restrictions on a Recipient's subsequent
// 	Exploitation of the Source Code or Executable Code other than
// 	those contained in the terms and conditions of this Agreement;
//   3.1.4
// 	not (and not to allow any third party) to profit or make any
// 	charge for the Source Code, or Executable Code, any
// 	Exploitation of the Source Code or Executable Code, or for any
// 	Derivative Works;
//   3.1.5
// 	not to place any restrictions on the operability of the Source
// 	Code;
//   3.1.6
// 	to attach prominent notices to any Derivative Works stating
// 	that you have changed the Source Code or Executable Code and to
// 	include the details anddate of such change; and
//   3.1.7
//   	not to Exploit the Source Code or Executable Code otherwise than
// 	as expressly permitted by  this Agreement.
//



/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#include "project.h"
#include "entity/mobile/mb_aimfx.h"
#include "entity/mobile/mb_hostaim.h"
#include "entity/mobile/mb_wpnfx.h"

#include "entity/tacview/tacview.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE 								0

#define DEBUG_MODULE_DISABLE_ARTILLERY_ERROR	0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_3D_OBJECT_SCALE (0.5)

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//HELLFIRE//

#define HELLFIRE_LOBL_CLIMB_RATIO	(0.14)
#define HELLFIRE_LOBL_DIVE_RATIO	(0.31)
#define HELLFIRE_LOAL_CLIMB1_RATIO	(0.084)
#define HELLFIRE_LOAL_CLIMB2_RATIO	(0.22)
#define HELLFIRE_LOAL_DIVE_RATIO	(0.5)
#define HELLFIRE_LOAL_CLIMB1_TIME	(0.0)
#define HELLFIRE_LOAL_CLIMB2_TIME	(-10.0)
#define HELLFIRE_LOAL_XZ	(1000.0)
#define HELLFIRE_LOAL_CLIMB1_Y		(300.0)
#define HELLFIRE_LOAL_CLIMB2_Y		(40.0)
#define HELLFIRE_LOAL_DIVE_Y		(-400.0)

//HELLFIRE//

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int
	draw_weapon_terrain_los_markers = FALSE;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define LAST_WEAPON (NUM_ENTITY_SUB_TYPE_WEAPONS - 1)

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//
// THE WEAPON'S FLIGHT (EEDCS-45)
//
// The host owns a weapon's physical flight: the guidance it flies by (what the weapon locks on, tracks and steers toward), its movement and attitude, the lifetime that
// ends it, and the contact and terminal facts (what it hit, where, when it left the map or struck the ground).  The campaign hands the host the weapon as it holds it
// and answers, by value, the questions the host asks about its world (the terrain, the objects, a launcher's or a target's facts); the host answers with the state
// the weapon left and the actions the campaign applies to its own mirror, in the order the original wrote and called them: the weapon's state, its position, a change
// of its target link, the submunitions it spawned, the Tacview update and the ground effect.  The terrain, object and model lookups and the effects stay the campaign's
// (explicitly: they are not the host's to own yet).  What the campaign keeps is what it decides: it applies, never chooses, the physical result.
//
// The host's step is a pure function of the request, so a step that needs an answer or a random sample stops, and is asked again with it.  The campaign's shared random
// stream is the campaign's: a sample is taken here, when the host asks for one, in the order the host asks.  The entity-local stream (the weapon's own seed) never leaves
// the campaign: the first four outputs go with the request.
//

#define WEAPON_STEP_ECHO_MAX 64

typedef struct weapon_step_context
{
	entity
		*en,
		*launcher_target,
		*hit_target;

	weapon
		*raw;
} weapon_step_context;

// the target a lookup is about: which = 0 the weapon's own target as the step began, 1 the launcher's target (once a lookup has asked for it), -1 none
static entity *weapon_step_target (weapon_step_context *context, int which)
{
	entity
		*target;

	target = NULL;

	if (which == 0)
	{
		target = context->raw->mob.target_link.parent;
	}
	else if (which == 1)
	{
		target = context->launcher_target;
	}

	return target;
}

static entity *weapon_step_required_target (weapon_step_context *context, const campaign_aim_event *lookup)
{
	entity
		*target;

	target = weapon_step_target (context, lookup->a);

	if (!target)
	{
		debug_fatal ("host aim: a weapon step asked about a target the weapon does not have (lookup %d, target %d)", lookup->kind, lookup->a);
	}

	return target;
}

static entity *weapon_step_required_launcher (weapon_step_context *context, const campaign_aim_event *lookup)
{
	entity
		*launcher;

	launcher = context->raw->launched_weapon_link.parent;

	if (!launcher)
	{
		debug_fatal ("host aim: a weapon step asked about a launcher the weapon does not have (lookup %d)", lookup->kind);
	}

	return launcher;
}

static void weapon_step_vec3d_result (float *r, const vec3d *v)
{
	r[0] = v->x;
	r[1] = v->y;
	r[2] = v->z;
}

// answers one lookup of the host's step: the question repeated, and its answer in f[6..11]
static void answer_weapon_step_lookup (const campaign_aim_event *lookup, campaign_aim_event *answer, void *user)
{
	weapon_step_context
		*context;

	entity
		*launcher,
		*target,
		*hit;

	vec3d
		p,
		q,
		v,
		intercept,
		face_normal;

	float
		*r;

	int
		terrain_class;

	context = (weapon_step_context *) user;

	*answer = *lookup;

	r = &answer->f[6];

	memset (r, 0, 6 * sizeof (float));

	switch (lookup->kind)
	{
		case CAMPAIGN_STEP_LOOKUP_VIEW_WEAPON:
		{
			r[0] = (context->en == get_local_entity_first_child (get_camera_entity (), LIST_TYPE_VIEW_WEAPON)) ? 1.0 : 0.0;

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_LAUNCHER_EO:
		{
			launcher = weapon_step_required_launcher (context, lookup);

			get_local_entity_vec3d (launcher, VEC3D_TYPE_EO_TRACKING_POINT, &p);

			weapon_step_vec3d_result (r, &p);

			r[3] = eo_tracking_point_valid (&p) ? 1.0 : 0.0;

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_LAUNCHER_INT:
		{
			launcher = weapon_step_required_launcher (context, lookup);

			switch (lookup->a)
			{
				case 0:
				{
					r[0] = (get_local_entity_int_value (launcher, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI) ? 1.0 : 0.0;

					break;
				}
				case 1:
				{
					r[0] = get_local_entity_int_value (launcher, INT_TYPE_LASER_ON) ? 1.0 : 0.0;

					break;
				}
				case 2:
				{
					r[0] = get_local_entity_int_value (launcher, INT_TYPE_LOS_TO_TARGET) ? 1.0 : 0.0;

					break;
				}
				default:
				{
					debug_fatal ("host aim: a weapon step asked a launcher question %d the campaign does not have", lookup->a);
				}
			}

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_LAUNCHER_TARGET:
		{
			launcher = weapon_step_required_launcher (context, lookup);

			target = get_local_entity_parent (launcher, LIST_TYPE_TARGET);

			context->launcher_target = target;

			r[0] = target ? 1.0 : 0.0;

			r[1] = (target == context->raw->mob.target_link.parent) ? 1.0 : 0.0;

			if (target)
			{
				weapon_step_vec3d_result (&r[2], get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION));
			}

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_TARGET_INT:
		{
			target = weapon_step_required_target (context, lookup);

			switch (lookup->b)
			{
				case 0:
				{
					r[0] = get_local_entity_int_value (target, INT_TYPE_IDENTIFY_MOBILE) ? 1.0 : 0.0;

					break;
				}
				case 1:
				{
					r[0] = (float) get_local_entity_int_value (target, INT_TYPE_WEAPON_DECOY_TYPE);

					break;
				}
				case 2:
				{
					r[0] = get_local_entity_int_value (target, INT_TYPE_RADAR_JAMMER_ON) ? 1.0 : 0.0;

					break;
				}
				case 3:
				{
					r[0] = get_local_entity_int_value (target, INT_TYPE_INFRA_RED_JAMMER_ON) ? 1.0 : 0.0;

					break;
				}
				case 4:
				{
					r[0] = get_local_entity_int_value (target, INT_TYPE_AIRBORNE_AIRCRAFT) ? 1.0 : 0.0;

					break;
				}
				case 5:
				{
					r[0] = (get_local_entity_int_value (target, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI) ? 1.0 : 0.0;

					break;
				}
				default:
				{
					debug_fatal ("host aim: a weapon step asked a target question %d the campaign does not have", lookup->b);
				}
			}

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_TARGET_VEC:
		{
			target = weapon_step_required_target (context, lookup);

			switch (lookup->b)
			{
				case 0:
				{
					p = *get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

					break;
				}
				case 1:
				{
					get_local_entity_target_point (target, &p);

					break;
				}
				case 2:
				{
					get_local_entity_vec3d (target, VEC3D_TYPE_MOTION_VECTOR, &p);

					break;
				}
				default:
				{
					debug_fatal ("host aim: a weapon step asked a target vector %d the campaign does not have", lookup->b);
				}
			}

			weapon_step_vec3d_result (r, &p);

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_WIND:
		{
			p.x = lookup->f[0];
			p.y = lookup->f[1];
			p.z = lookup->f[2];

			get_session_wind_velocity_at_point (&p, &v);

			weapon_step_vec3d_result (r, &v);

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_GROUND_PROBE:
		{
			p.x = lookup->f[0];
			p.y = lookup->f[1];
			p.z = lookup->f[2];

			r[0] = point_below_ground (&p) ? 1.0 : 0.0;

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_GROUND_IMPACT:
		{
			p.x = lookup->f[0];
			p.y = lookup->f[1];
			p.z = lookup->f[2];

			q.x = lookup->f[3];
			q.y = lookup->f[4];
			q.z = lookup->f[5];

			terrain_class = get_ground_impact_point (&p, &q, &q);

			r[0] = (float) terrain_class;

			weapon_step_vec3d_result (&r[1], &q);

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_FACE_NORMAL:
		{
			get_3d_terrain_face_normal (&v, lookup->f[0], lookup->f[2]);

			weapon_step_vec3d_result (r, &v);

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_ELEVATION:
		{
			r[0] = get_3d_terrain_elevation (lookup->f[0], lookup->f[2]);

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_COLLISION:
		{
			p.x = lookup->f[0];
			p.y = lookup->f[1];
			p.z = lookup->f[2];

			q.x = lookup->f[3];
			q.y = lookup->f[4];
			q.z = lookup->f[5];

			// the first object the segment meets, other than the weapon's launcher: a geometry query of the world's objects
			memset (&intercept, 0, sizeof (intercept));

			hit = get_line_of_sight_collision_entity (get_local_entity_parent (context->en, LIST_TYPE_LAUNCHED_WEAPON), NULL, &p, &q, &intercept, &face_normal);

			context->hit_target = hit;

			r[0] = hit ? 1.0 : 0.0;

			r[1] = (hit != weapon_step_target (context, lookup->a)) ? 1.0 : 0.0;

			r[2] = hit ? (float) hit->type : 0.0;

			weapon_step_vec3d_result (&r[3], &intercept);

			break;
		}
		case CAMPAIGN_STEP_LOOKUP_TREES:
		{
			p.x = lookup->f[0];
			p.y = lookup->f[1];
			p.z = lookup->f[2];

			q.x = lookup->f[3];
			q.y = lookup->f[4];
			q.z = lookup->f[5];

			// the first tree the segment meets: a geometry query of the world's trees
			memset (&intercept, 0, sizeof (intercept));

			r[0] = get_line_of_sight_collision_tree (&p, &q, &intercept, &face_normal, FALSE, TRUE) ? 1.0 : 0.0;

			weapon_step_vec3d_result (&r[1], &intercept);

			break;
		}
		default:
		{
			debug_fatal ("host aim: a weapon step asked a lookup of kind %d the campaign does not have", lookup->kind);
		}
	}
}

// applies one action of the host's step to the campaign's mirror of the weapon
static void apply_weapon_step_action (weapon_step_context *context, const campaign_aim_event *action)
{
	weapon
		*raw;

	entity
		*en;

	vec3d
		position;

	en = context->en;

	raw = context->raw;

	switch (action->kind)
	{
		case CAMPAIGN_STEP_ACTION_STATE:
		{
			if ((action->a < MISSILE_PHASE1) || (action->a > MISSILE_FINAL_PHASE) || (action->b < 0) || (action->b > 1) || (action->c < 0) || (action->c >= NUM_WEAPON_KILL_CODES))
			{
				debug_fatal ("host aim: a weapon step left a state the campaign cannot hold (phase %d, on ground %d, kill code %d)", action->a, action->b, action->c);
			}

			raw->missile_phase = action->a;

			raw->weapon_on_ground = action->b;

			raw->kill_code = action->c;

			raw->mob.velocity = action->f[0];

			raw->mob.motion_vector.x = action->f[1];
			raw->mob.motion_vector.y = action->f[2];
			raw->mob.motion_vector.z = action->f[3];

			raw->weapon_lifetime = action->f[4];

			raw->decoy_timer = action->f[5];

			break;
		}
		case CAMPAIGN_STEP_ACTION_ATTITUDE:
		{
			memcpy (raw->mob.attitude, action->f, 9 * sizeof (float));

			break;
		}
		case CAMPAIGN_STEP_ACTION_SPAWN:
		{
			viewpoint
				vp;

			int
				count;

			if ((action->a < 0) || (action->a > 64))
			{
				debug_fatal ("host aim: a weapon step spawned %d submunitions", action->a);
			}

			count = action->a;

			memcpy (vp.attitude, raw->mob.attitude, sizeof (matrix3x3));

			vp.position = raw->mob.position;

			while (count--)
			{
				create_server_entity_submunition_weapon (raw->launched_weapon_link.parent, raw->mob.target_link.parent, raw->mob.sub_type + 1, &vp, action->f[0]);
			}

			break;
		}
		case CAMPAIGN_STEP_ACTION_TACVIEW:
		{
			// TODO check rotation
			if (tacview_is_logging())
				write_tacview_unit_update(en, TRUE, TRUE, FALSE);

			break;
		}
		case CAMPAIGN_STEP_ACTION_POSITION:
		{
			position.x = action->f[0];
			position.y = action->f[1];
			position.z = action->f[2];

			set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &position);

			break;
		}
		case CAMPAIGN_STEP_ACTION_TARGET_LINK:
		{
			if (action->a == 0)
			{
				set_client_server_entity_parent (en, LIST_TYPE_TARGET, NULL);
			}
			else if (action->a == 1)
			{
				if (!context->launcher_target)
				{
					debug_fatal ("host aim: a weapon step switched the weapon to its launcher's target without having asked for it");
				}

				set_client_server_entity_parent (en, LIST_TYPE_TARGET, context->launcher_target);
			}
			else if (action->a == 2)
			{
				if (!context->hit_target)
				{
					debug_fatal ("host aim: a weapon step switched the weapon to a target it did not hit");
				}

				//
				// switch target parent to new_target (on server only)
				//

				delete_local_entity_from_parents_child_list (en, LIST_TYPE_TARGET);

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_TARGET, context->hit_target, NULL);
			}
			else
			{
				debug_fatal ("host aim: a weapon step changed the weapon's target link in a way (%d) the campaign does not have", action->a);
			}

			break;
		}
		case CAMPAIGN_STEP_ACTION_SMOKE_EFFECT:
		{
			raw->kill_code = WEAPON_KILL_CODE_HIT_LAND;

			create_client_server_weapon_killed_explosion_effect (en);

			raw->kill_code = WEAPON_KILL_CODE_OK;

			break;
		}
		default:
		{
			debug_fatal ("host aim: a weapon step left an action of kind %d the campaign does not have", action->kind);
		}
	}
}

#ifdef EECH_WEAPON_FIXTURE
#define weapon_movement weapon_movement_original
#endif

void weapon_movement (entity *en)
{
	weapon
		*raw;

	weapon_step_context
		context;

	campaign_aim_step
		step;

	campaign_aim_result
		result;

	int
		i,
		seed;

	#ifdef EECH_WEAPON_FIXTURE

	campaign_aim_event
		echo[WEAPON_STEP_ECHO_MAX];

	int
		echo_count,
		echoed;

	#endif

	////////////////////////////////////////
	//
	// pre-amble
	//
	////////////////////////////////////////

	ASSERT (en);

	raw = (weapon *) get_local_entity_data (en);

	ASSERT (entity_sub_type_weapon_valid (raw->mob.sub_type));

	////////////////////////////////////////
	//
	// the weapon, as the campaign holds it
	//
	////////////////////////////////////////

	memset (&step, 0, sizeof (step));

	step.weapon = raw->mob.sub_type;

	step.delta_time = get_delta_time ();

	step.system_time = get_system_time ();

	step.map_min[0] = MIN_MAP_X;
	step.map_min[1] = MIN_MAP_Y;
	step.map_min[2] = MIN_MAP_Z;

	step.map_max[0] = MAX_MAP_X;
	step.map_max[1] = MAX_MAP_Y;
	step.map_max[2] = MAX_MAP_Z;

	step.weapon_state.position[0] = raw->mob.position.x;
	step.weapon_state.position[1] = raw->mob.position.y;
	step.weapon_state.position[2] = raw->mob.position.z;

	memcpy (step.weapon_state.attitude, raw->mob.attitude, 9 * sizeof (float));

	step.weapon_state.motion_vector[0] = raw->mob.motion_vector.x;
	step.weapon_state.motion_vector[1] = raw->mob.motion_vector.y;
	step.weapon_state.motion_vector[2] = raw->mob.motion_vector.z;

	step.weapon_state.velocity = raw->mob.velocity;
	step.weapon_state.weapon_lifetime = raw->weapon_lifetime;
	step.weapon_state.decoy_timer = raw->decoy_timer;
	step.weapon_state.missile_phase = raw->missile_phase;
	step.weapon_state.loal_mode = raw->loal_mode;
	step.weapon_state.weapon_on_ground = raw->weapon_on_ground;
	step.weapon_state.parachute_status = raw->parachute_status;
	step.weapon_state.kill_code = raw->kill_code;
	step.weapon_state.has_launcher = (raw->launched_weapon_link.parent != NULL);
	step.weapon_state.has_target = (raw->mob.target_link.parent != NULL);
	step.weapon_state.server = (get_comms_model () == COMMS_MODEL_SERVER);

	// the first four outputs of the weapon's own stream, which the original drew from a copy of its seed in each call
	seed = get_client_server_entity_random_number_seed (en);

	for (i = 0; i < 4; i++)
	{
		step.weapon_state.entity_samples[i] = rand16x (&seed);
	}

	#ifdef EECH_WEAPON_FIXTURE

	step.echo = TRUE;

	#endif

	memset (&context, 0, sizeof (context));

	context.en = en;

	context.raw = raw;

	////////////////////////////////////////
	//
	// the host's step, and what it left
	//
	////////////////////////////////////////

	result = campaign_aim_weapon_step (&step, answer_weapon_step_lookup, &context);

	#ifdef EECH_WEAPON_FIXTURE

	// the echo is the host's request buffer: keep it before anything the actions do asks the host again
	echo_count = result.has_echo ? result.echo_count : 0;

	if (echo_count > WEAPON_STEP_ECHO_MAX)
	{
		debug_fatal ("host aim: a weapon step echoed %d events, more than the %d it can carry", echo_count, WEAPON_STEP_ECHO_MAX);
	}

	if (echo_count > 0)
	{
		memcpy (echo, result.echo, echo_count * sizeof (campaign_aim_event));
	}

	echoed = 0;

	#endif

	for (i = 0; i < result.action_count; i++)
	{
		#ifdef EECH_WEAPON_FIXTURE

		// the hooks of the original the host's step passed before this action
		while (echoed < result.actions[i].d)
		{
			WPN_FX_STEP_ECHO (&echo[echoed])

			echoed++;
		}

		#endif

		apply_weapon_step_action (&context, &result.actions[i]);
	}

	#ifdef EECH_WEAPON_FIXTURE

	while (echoed < echo_count)
	{
		WPN_FX_STEP_ECHO (&echo[echoed])

		echoed++;
	}

	#endif
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_WEAPON_FIXTURE

#undef weapon_movement

void weapon_movement (entity *en)
{
	eech_wpn_move_begin (en);

	weapon_movement_original (en);

	eech_wpn_move_end (en);
}

#endif

/** The weapon's physical figures, mirrored from the weapon database for the host (EEDCS-45).  The host owns the projectile simulation: it simulates the weapon at every
 *  pitch and velocity test and keeps the ballistic table that results.  What the figures are is the database's; what to do with them is not decided here. */
static void fill_weapon_figures (campaign_weapon_figures *figures, entity_sub_types wpn_type)
{
	const weapon_data
		*data;

	const char
		*name;

	data = &weapon_database[wpn_type];

	name = data->full_name ? data->full_name : "(null)";

	if (strlen (name) > CAMPAIGN_WEAPON_NAME_MAX)
	{
		debug_fatal ("host aim: the name of weapon %d is longer than %d bytes", wpn_type, CAMPAIGN_WEAPON_NAME_MAX);
	}

	memset (figures, 0, sizeof (*figures));

	memcpy (figures->name, name, strlen (name) + 1);

	figures->start_weight = data->start_weight;
	figures->final_weight = data->final_weight;
	figures->max_range = data->max_range;
	figures->muzzle_velocity = data->muzzle_velocity;
	figures->cruise_velocity = data->cruise_velocity;
	figures->g_max = data->g_max;
	figures->boost_time = data->boost_time;
	figures->cruise_time = data->cruise_time;
	figures->inhibit_time = data->inhibit_time;
	figures->drag_coefficient = data->drag_coefficient;
	figures->flightpath_deviation = data->flightpath_deviation;
	figures->boost_power = data->boost_power;
	figures->sustain_power = data->sustain_power;
	figures->sustain_time = data->sustain_time;
	figures->diameter = data->diameter;
	figures->max_altitude = data->max_altitude;
	figures->ignore_gravity = data->ignore_gravity;
	figures->flight_profile_or_self_destr = data->flight_profile_or_self_destr;
	figures->parachute_type = (wpn_type == ENTITY_SUB_TYPE_WEAPON_HOKUM_PILOT) || (wpn_type == ENTITY_SUB_TYPE_WEAPON_HOKUM_CO_PILOT) || (wpn_type == ENTITY_SUB_TYPE_WEAPON_CRATE);
	figures->weapon_class = data->weapon_class;

	figures->cruise_time_max_error = data->cruise_time_max_error;
	figures->detonation_radius = data->detonation_radius;
	figures->max_seeker_limit = data->max_seeker_limit;
	figures->max_launch_angle_error = data->max_launch_angle_error;
	figures->survive_ground_impact = data->survive_ground_impact;
	figures->in_flight_collision_check = data->in_flight_collision_check;
	figures->guidance_type = data->guidance_type;
	figures->aiming_type = data->aiming_type;
	figures->decoy_type = data->decoy_type;
	figures->warhead_type = data->warhead_type;
	figures->hellfire_ii = (wpn_type == ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II);
	figures->smoke_grenade = (wpn_type == ENTITY_SUB_TYPE_WEAPON_SMOKE_GRENADE);

	if ((wpn_type + 1) < NUM_ENTITY_SUB_TYPE_WEAPONS)
	{
		figures->submunition_count = (int) weapon_database[wpn_type + 1].burst_duration;
		figures->submunition_velocity_factor = weapon_database[wpn_type + 1].acquire_parent_forward_velocity;
	}
}

/** Hands the host every weapon's figures once at boot, in database order, to keep: the host flies a weapon by them (the weapon step), whether or not the weapon also has a
 *  ballistic table or a range run.  Called before the tables are generated.
 */
void supply_weapon_figures(void)
{
	entity_sub_types
		wpn_type;

	campaign_weapon_figures
		figures;

	for (wpn_type = 1; wpn_type <= LAST_WEAPON; wpn_type++)
	{
		fill_weapon_figures (&figures, wpn_type);

		campaign_aim_supply_figures (wpn_type, &figures);
	}
}

/** generates the ballistics tables for all weapons with ballistic aiming (the host does: each weapon's figures are sent once, in database order, the first and the last
 *  flagged).  The test fixture build asks the host to echo what it generated and replays it into the observer in the order the original produced it: every weapon's
 *  flights, then (after the pass) every weapon's table.
 */
void generate_ballistics_tables(void)
{
	entity_sub_types
		wpn_type,
		last_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	campaign_weapon_figures
		figures;

	int
		first = TRUE;

	for (wpn_type = 1; wpn_type <= LAST_WEAPON; wpn_type++)
		if (weapon_database[wpn_type].aiming_type && !weapon_database[wpn_type].guidance_type)
			last_type = wpn_type;

	for (wpn_type = 1; wpn_type <= LAST_WEAPON; wpn_type++)
	{
		if (weapon_database[wpn_type].aiming_type && !weapon_database[wpn_type].guidance_type)
		{
			fill_weapon_figures (&figures, wpn_type);

			#ifdef EECH_WEAPON_FIXTURE

			{
				campaign_aim_result
					result;

				size_t
					count;

				float
					*cells;

				count = (size_t) CAMPAIGN_TABLE_VELOCITY_TESTS * CAMPAIGN_TABLE_PITCH_INDICES * (size_t) ((int) (figures.max_range / CAMPAIGN_TABLE_RANGE_STEP) + 2) * 2;

				cells = (float *) safe_malloc (sizeof (float) * count);

				result = campaign_aim_generate (CAMPAIGN_AIM_TABLE, wpn_type, &figures, first, wpn_type == last_type, cells, count);

				WPN_FX_TABLE_HOST (wpn_type, figures.max_range, result.echo, result.echo_count, cells, result.echo_cell_count)

				safe_free (cells);
			}

			#else

			campaign_aim_generate (CAMPAIGN_AIM_TABLE, wpn_type, &figures, first, wpn_type == last_type, NULL, 0);

			#endif

			first = FALSE;
		}
	}

	for (wpn_type = 1; wpn_type <= LAST_WEAPON; wpn_type++)
		if (weapon_database[wpn_type].aiming_type && !weapon_database[wpn_type].guidance_type)
		{
			WPN_FX_TABLE_SUPPLIED (wpn_type)
		}
}

void generate_guided_missiles_tables(void)
{
	entity_sub_types
		wpn_type,
		last_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	campaign_weapon_figures
		figures;

	int
		first = TRUE;

	for (wpn_type = 1; wpn_type <= LAST_WEAPON; wpn_type++)
		if (weapon_database[wpn_type].guidance_type)
			last_type = wpn_type;

	for (wpn_type = 1; wpn_type <= LAST_WEAPON; wpn_type++)
	{
		if (weapon_database[wpn_type].guidance_type)
		{
			fill_weapon_figures (&figures, wpn_type);

			#ifdef EECH_WEAPON_FIXTURE

			{
				campaign_aim_result
					result;

				float
					unused = 0.0;

				result = campaign_aim_generate (CAMPAIGN_AIM_RANGE, wpn_type, &figures, first, wpn_type == last_type, &unused, 0);

				WPN_FX_RANGE_HOST (wpn_type, result.echo, result.echo_count)
			}

			#else

			campaign_aim_generate (CAMPAIGN_AIM_RANGE, wpn_type, &figures, first, wpn_type == last_type, NULL, 0);

			#endif

			first = FALSE;
		}
	}
}

/**
 * Looks up om the ballistics tables how much drop and time of flight a projectile
 * of wpn_type will have at the given range.
 * As we probably don't have the value for the exact range/pitch requested it
 * will make a weighted average of the closeset values we have.
 */
int get_ballistic_pitch_deflection(entity_sub_types wpn_type, float range, float height_diff_or_pitch, float* aiming_pitch, float* time_of_flight, int simplified, int fixed_pitch, float parent_velocity)
{
	//
	// The table interpolation (the iteration, the weighting and the drop limit) is the host's (EEDCS-44, mb_hostaim.h).  The tables themselves are raw
	// weapon data: generate_ballistics_tables simulates each weapon's flight (the still-core projectile simulation) and hands the result to the host once.
	// The outputs are written only when the host finds a solution, as the original wrote them.
	//

	campaign_aim_step
		step;

	campaign_aim_result
		result;

	ASSERT(time_of_flight);
	ASSERT(aiming_pitch);

	if (wpn_type == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON || wpn_type >= NUM_ENTITY_SUB_TYPE_WEAPONS) // some bug should be fixed..
	{
		debug_log("weapon type %i is illegal!", wpn_type);

		#ifdef EECH_AIM_FIXTURE

		{
			float no_output = 0.0f;

			eech_aim_ballistic (wpn_type, range, height_diff_or_pitch, simplified, fixed_pitch, parent_velocity, FALSE, &no_output, &no_output);
		}

		#endif

		return FALSE;
	}

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_BALLISTIC;

	step.echo = AIM_FX_ECHO;

	step.weapon = wpn_type;

	step.range = range;

	step.height_diff_or_pitch = height_diff_or_pitch;

	step.simplified = simplified;

	step.fixed_pitch = fixed_pitch;

	step.parent_velocity = parent_velocity;

	result = campaign_aim_ask (&step, FALSE);

	if (result.result)
	{
		*aiming_pitch = result.angle;

		*time_of_flight = result.time_of_flight;
	}

	#ifdef EECH_AIM_FIXTURE

	eech_aim_ballistic (wpn_type, range, height_diff_or_pitch, simplified, fixed_pitch, parent_velocity, result.result, &result.angle, &result.time_of_flight);

	#endif

	return result.result;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

