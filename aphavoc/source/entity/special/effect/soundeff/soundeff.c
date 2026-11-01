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

#include "entity/special/session/ss_envfx.h"

#ifdef EECH_CAMPAIGN_HOST_ENV

#include <string.h>

#include "entity/special/session/ss_hostenv.h"

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE 0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static void set_local_sound_effect_sample_indices (entity *en, int count, sound_sample_indices *indices);

static float adjust_radio_message_amplification (float amp, vec3d *pos);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int pause_local_entity_sound_type (entity *en, entity_sub_types type, float delay)
{
	entity
		*spec;

	sound_effect
		*raw;

	int
		count;

	count = 0;

	spec = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);
	
	while (spec)
	{
		if (get_local_entity_type (spec) == ENTITY_TYPE_SOUND_EFFECT)
		{
			raw = (sound_effect *) get_local_entity_data (spec);

			if (raw->eff.sub_type == type)
			{
				#ifdef EECH_CAMPAIGN_HOST_ENV

				//
				// EECH_CAMPAIGN_HOST_ENV (EEDCS-51, D3): whether this effect is paused, what that does to its validity and valid lifetime, the delay copy and whether it counts are the HOST's
				// (aphavoc/source/entity/special/session/ss_hostenv.h SOUND_PAUSE_RESUME).  The walk of the children and the sub type match stay the campaign's; the stored fields are handed over
				// by value and written from the checked answer.  The original follows in the #else, for the original game's build only.
				//

				{
					int
						valid,
						play,
						relink;

					valid = raw->valid_sound_effect;

					if (campaign_sound_pause_resume (FALSE, delay, (get_local_entity_parent (spec, LIST_TYPE_UPDATE) != NULL), &valid, &raw->valid_effect_lifetime, &play, &relink))
					{
						raw->valid_sound_effect = valid;

						count ++;
					}
				}

				#else

				//
				// "pause" sound
				//

				if (raw->valid_sound_effect)
				{
					raw->valid_sound_effect = FALSE;

					raw->valid_effect_lifetime = delay;

					count ++;
				}

				#endif
			}
		}

		spec = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);
	}

	return count;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_CAMPAIGN_HOST_ENV

//
// the parent's playback request of a resumed effect.  The session plays with the viewpoint and no range, as the original did inline (below, in the #else); the HOST decides only whether to ask.
//
// Any other parent: the original took the range from the viewpoint to the parent's position and called fn_play_local_entity_sound[type] with it.  The running original (the 8e90ab42e native, 1200 natural frames,
// the controlled fixed-wing and AI-helicopter pause and resume, and a deliberate recording overload) reached only the table's own default no-op from here, so for a parent whose entry IS that default function
// the request is empty and neither the range nor the call is made (the range is computed for nothing else): the elision is observed by ENV_FXP_ELIDED.  A parent whose entry is anything else would be a live
// playback consumer, which this campaign route no longer supports and does not serve silently: it stops here, by name, BEFORE any range is computed or any playback function is called.  What the entry is
// is asked of the table's real default function (campaign_play_entity_sound_is_default, an entity type outside the table is not default), never of another entry; a non-default entry that the table merely holds
// (for instance the sound effect type's own) is legal, only a live request through this route is unsupported.  The original game's build keeps the range and the call below.
//

static void play_resumed_parent_sound (entity *en)
{
	if (en == get_session_entity ())
	{
		play_local_entity_sound (en, &main_vp, 0);
	}
	else
	{
		int
			type;

		type = get_local_entity_type (en);

		if (!campaign_play_entity_sound_is_default (type))
		{
			debug_fatal ("host environment: [env-play:unsupported_live_playback] entity_type=%d entity_index=%d: the playback dispatch entry of the resumed parent is not the default no-op; live playback from a resumed parent is not supported (no range was computed and no playback function was called)", type, (int) get_local_entity_index (en));
		}

		ENV_FXP (ENV_FXP_ELIDED, en, 0)
	}
}

#endif

int resume_local_entity_sound_type (entity *en, entity_sub_types type)
{
	entity
		*spec;

	sound_effect
		*raw;

	int
		count;

	count = 0;

	spec = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);
	
	while (spec)
	{
		if (get_local_entity_type (spec) == ENTITY_TYPE_SOUND_EFFECT)
		{
			raw = (sound_effect *) get_local_entity_data (spec);

			if (raw->eff.sub_type == type)
			{
				#ifdef EECH_CAMPAIGN_HOST_ENV

				//
				// EECH_CAMPAIGN_HOST_ENV (EEDCS-51, D3): whether this effect is resumed, what that does to its validity, whether its parent is asked to play and whether it is put back in the update list,
				// and whether it counts, are the HOST's (aphavoc/source/entity/special/session/ss_hostenv.h SOUND_PAUSE_RESUME).  The walk of the children and the sub type match stay the campaign's, and so
				// does HOW the parent is asked to play (play_resumed_parent_sound above: the session's default no-op; any other parent only while its dispatch entry is the default no-op, else a named stop).
				// The answer is checked whole, then applied in the original order: the validity, the playback request, the relink, the count.  The original follows in the #else, for the original game's build only.
				//

				{
					int
						valid,
						play,
						relink;

					valid = raw->valid_sound_effect;

					if (campaign_sound_pause_resume (TRUE, 0.0f, (get_local_entity_parent (spec, LIST_TYPE_UPDATE) != NULL), &valid, &raw->valid_effect_lifetime, &play, &relink))
					{
						raw->valid_sound_effect = valid;

						if (play)
						{
							play_resumed_parent_sound (en);
						}

						if (relink)
						{
							if (get_local_entity_parent (spec, LIST_TYPE_UPDATE))
							{
								debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the host relinks an effect that the parent's playback left in the update list");
							}

							insert_local_entity_into_parents_child_list (spec, LIST_TYPE_UPDATE, get_update_entity (), NULL);
						}

						count ++;
					}
				}

				#else

				//
				// "unpause" sound
				//

				if (!raw->valid_sound_effect)
				{
					//
					// set flag and start playing
					//

					raw->valid_sound_effect = TRUE;
	
					if (en == get_session_entity ())
					{
						play_local_entity_sound (en, &main_vp, 0);
					}
					else
					{
						vec3d
							*position;
	
						viewpoint
							*vp;
	
						float
							range;
	
						vp = &main_vp;
	
						position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
	
						range = get_approx_3d_range (&vp->position, position);
	
						play_local_entity_sound (en, vp, range);
					}

					if (!get_local_entity_parent (spec, LIST_TYPE_UPDATE))
					{
						insert_local_entity_into_parents_child_list (spec, LIST_TYPE_UPDATE, get_update_entity (), NULL);
					}

					count ++;
				}

				#endif
			}
		}

		spec = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);
	}

	return count;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int pause_client_server_entity_sound_type (entity *en, entity_sub_types type, float delay)
{
	int
		count;
		
	count = pause_local_entity_sound_type (en, type, delay);

	transmit_entity_comms_message (ENTITY_COMMS_PAUSE_SOUND_TYPE, en, type, delay);

	return count;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int resume_client_server_entity_sound_type (entity *en, entity_sub_types type)
{
	int
		count;

	count = resume_local_entity_sound_type (en, type);

	transmit_entity_comms_message (ENTITY_COMMS_RESUME_SOUND_TYPE, en, type);

	return count;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void kill_local_entity_sound_type (entity *en, entity_sub_types type)
{
	entity
		*spec,
		*next;

	sound_effect
		*raw;

	spec = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);
	
	while (spec)
	{
		next = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);

		if (get_local_entity_type (spec) == ENTITY_TYPE_SOUND_EFFECT)
		{
			raw = (sound_effect *) get_local_entity_data (spec);

			if (raw->eff.sub_type == type)
			{
				//
				// "kill" sound
				//

				kill_local_entity (spec);
			}
		}

		spec = next;
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void kill_client_server_entity_sound_type (entity *en, entity_sub_types type)
{
	entity
		*spec,
		*next;

	sound_effect
		*raw;

	spec = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);
	
	while (spec)
	{
		next = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);

		if (get_local_entity_type (spec) == ENTITY_TYPE_SOUND_EFFECT)
		{
			raw = (sound_effect *) get_local_entity_data (spec);

			if (raw->eff.sub_type == type)
			{
				//
				// "kill" sound
				//

				kill_client_server_entity (spec);
			}
		}

		spec = next;
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void destroy_local_sound_effects (entity *en)
{
	entity
		*spec,
		*next;

	spec = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);
	
	while (spec)
	{
		next = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);

		if (get_local_entity_type (spec) == ENTITY_TYPE_SOUND_EFFECT)
		{
			destroy_local_entity (spec);
		}

		spec = next;
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void destroy_client_server_sound_effects (entity *en)
{
	entity
		*spec,
		*next;

	spec = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);
	
	while (spec)
	{
		next = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);

		if (get_local_entity_type (spec) == ENTITY_TYPE_SOUND_EFFECT)
		{
			destroy_client_server_entity_family (spec);
		}

		spec = next;
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void pause_local_continuous_weapon_sound_effect (entity *en, entity_sub_types weapon_sub_type)
{
	entity_sub_types
		effect_sub_type;

	ASSERT (en);

	//
	// reset burst timer so that it can fire next time round
	//
	
	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		set_local_entity_float_value (en, FLOAT_TYPE_WEAPON_BURST_TIMER, 0.0);
	}

	effect_sub_type = weapon_database [weapon_sub_type].launch_sound_effect_sub_type;

	if (effect_sub_type != ENTITY_SUB_TYPE_EFFECT_SOUND_MISC)
	{
		pause_local_entity_sound_type (en, effect_sub_type, 0.5);
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void pause_client_server_continuous_weapon_sound_effect (entity *en, entity_sub_types weapon_sub_type)
{
	ASSERT (en);

	ASSERT ((get_comms_model () == COMMS_MODEL_SERVER) || (en == get_gunship_entity ()));

	if (weapon_sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		pause_local_continuous_weapon_sound_effect (en, weapon_sub_type);

		transmit_entity_comms_message (ENTITY_COMMS_PAUSE_WEAPON_SOUND_TYPE, en, weapon_sub_type);
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int resume_client_server_continuous_weapon_sound_effect (entity *en, entity_sub_types weapon_sub_type)
{
	entity_sub_types
		effect_sub_type;

	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	effect_sub_type = weapon_database [weapon_sub_type].launch_sound_effect_sub_type;

	if (effect_sub_type == ENTITY_SUB_TYPE_EFFECT_SOUND_MISC)
	{
		return 0;
	}
	else
	{
		return (resume_client_server_entity_sound_type (en, effect_sub_type));
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int get_local_entity_sound_type_valid (entity *en, entity_sub_types type)
{
	entity
		*spec;

	sound_effect
		*raw;

	spec = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);
	
	while (spec)
	{
		if (get_local_entity_type (spec) == ENTITY_TYPE_SOUND_EFFECT)
		{
			raw = (sound_effect *) get_local_entity_data (spec);

			if (raw->eff.sub_type == type)
			{
				return (raw->valid_sound_effect);
			}
		}

		spec = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);
	}

	return FALSE;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void set_local_sound_effect_sample_indices (entity *en, int count, sound_sample_indices *indices)
{
	sound_effect
		*raw;

	int
		loop;

	ASSERT (en);

	ASSERT (count > 0);

	ASSERT (indices);

	raw = (sound_effect *) get_local_entity_data (en);

	ASSERT (!raw->effect_index);

	//
	// copy sample indices
	//

	raw->sound_effect_sequence_count = count;

	raw->effect_index = (sound_sample_indices *) malloc_fast_mem (sizeof (sound_sample_indices) * count);	
	
	memcpy (raw->effect_index, indices, sizeof (sound_sample_indices) * count);

	//
	// determine playing time
	//


	#ifdef EECH_CAMPAIGN_HOST_ENV

	//
	// EECH_CAMPAIGN_HOST_ENV (EEDCS-51, D3): the physical lifetime of the effect is the HOST's (aphavoc/source/entity/special/session/ss_hostenv.h SOUND_LIFETIME_INIT): the sum of the samples'
	// durations, the 40 s cap (300 s for MUSIC) and the looping case.  What the campaign still holds is the sound database: it reads each sample's rate and size from the descriptor (only for a timed effect,
	// as the original did) and hands them over by value.  The original arithmetic follows in the #else, for the original game's build only.
	//

	{
		int
			rate [CAMPAIGN_SOUND_SAMPLES],
			size [CAMPAIGN_SOUND_SAMPLES];

		memset (rate, 0, sizeof (rate));

		memset (size, 0, sizeof (size));

		ENV_FXA (ENV_FXA_CTOR, count, raw->looping, 0, 0)

		if (!raw->looping)
		{
			for (loop = 0; (loop < count) && (loop < CAMPAIGN_SOUND_SAMPLES); loop ++)
			{
				ASSERT (indices [loop] != SOUND_SAMPLE_INDEX_NONE);

				rate [loop] = application_sound_effects [indices [loop]].rate;
				size [loop] = application_sound_effects [indices [loop]].size;

				ENV_FXA (ENV_FXA_CTOR_SAMPLE, indices [loop], rate [loop], size [loop], application_sound_effects [indices [loop]].original_rate)
			}
		}

		raw->effect_lifetime = campaign_sound_lifetime_init (raw->looping, raw->eff.sub_type == ENTITY_SUB_TYPE_EFFECT_SOUND_MUSIC, count, rate, size);
	}

	#else

	raw->effect_lifetime = 0;

	if (!raw->looping)
	{
		float
			rate,
			size;

		for (loop = 0; loop < count; loop ++)
		{
			ASSERT (indices [loop] != SOUND_SAMPLE_INDEX_NONE);

			rate = (float)(application_sound_effects [indices [loop]].rate);
			size = (float)(application_sound_effects [indices [loop]].size);

			if (rate <= 0.0)
			{
				debug_log ("SOUNDEFF : Sound effect rate invalid ( index = %s, rate = %d )", application_sound_effects [indices [loop]].name, rate);
			}
			else
			{
				raw->effect_lifetime += (size / rate);
			}

			if (raw->effect_lifetime >= (raw->eff.sub_type != ENTITY_SUB_TYPE_EFFECT_SOUND_MUSIC ? 40.0f : 300.0f))
			{
				debug_log ("SOUNDEFF : Sound effect too long ( index = %s, %f s )", application_sound_effects [indices [loop]].name, raw->effect_lifetime);
			}
		}

		raw->effect_lifetime = min (raw->effect_lifetime, raw->eff.sub_type != ENTITY_SUB_TYPE_EFFECT_SOUND_MUSIC ? 40.0f : 300.0f);
	}

	#endif
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

entity *create_local_sound_effect_entity
			(
				int index,
				entity *parent,
				entity_sides side,
				entity_sub_types sub_type,
				sound_channel_types channel,
				sound_locality_types locality,
				vec3d *position,
				float amp,
				float pitch,
				int valid,
				int looping,
				int sample_count,
				sound_sample_indices *sample_indices
			)
{
	entity
		*en;

	vec3d
		pos;

	int
		panning,
		create_stack_attributes;

	ASSERT (parent);

	create_stack_attributes = force_local_entity_create_stack_attributes;

	if (pitch == 0.0)   //Werewolf 4 Feb 2006
	{
		pitch = 1.0;  // If pitch is unset, use a sane default value
	}

	if (get_comms_data_flow () == COMMS_DATA_FLOW_RX)
	{
		set_force_local_entity_create_stack_attributes (TRUE);
	}

	panning = TRUE;

	if (position)
	{
		pos = *position;
	}
	else
	{
		position = get_local_entity_vec3d_ptr (parent, VEC3D_TYPE_POSITION);

		if (position)
		{
			pos = *position;
		}
		else
		{
			pos.x = MID_MAP_X;
			pos.y = MID_MAP_Y;
			pos.z = MID_MAP_Z;

			panning = FALSE;
		}
	}

	//
	// special cases for speech
	//
	
	switch (sub_type)
	{
		case ENTITY_SUB_TYPE_EFFECT_SOUND_RADIO_MESSAGE:
		{
			if (get_global_gunship_side () == side)
			{
				amp = adjust_radio_message_amplification (amp, &pos);

				if (command_line_radio_msgs_noise)
					pitch -= 0.05 * frand1();
			}
			else
			{	
				amp = 0.0;
			}

			panning = FALSE;

			break;
		}

		case ENTITY_SUB_TYPE_EFFECT_SOUND_CPG_MESSAGE:
		case ENTITY_SUB_TYPE_EFFECT_SOUND_WARNING_MESSAGE:
		{
			if (parent == get_gunship_entity ())
			{
				amp = 1.0;
			}
			else
			{
				amp = 0.0;
			}

			panning = FALSE;

			break;
		}
	}

	//
	// create sound
	//

	en = create_local_entity
	(
		ENTITY_TYPE_SOUND_EFFECT,
		index,
		ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, sub_type),
		ENTITY_ATTR_PARENT (LIST_TYPE_SPECIAL_EFFECT, parent),
		ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, pos.x, pos.y, pos.z),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_SOUND_CHANNEL, channel),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_SOUND_EFFECT_LOOPING, looping),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_SOUND_EFFECT_PANNING, panning),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_SOUND_LOCALITY, locality),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_VALID_SOUND_EFFECT, valid),
		ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE_AMPLIFICATION, amp),
		ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE_SOUNDPITCH, pitch),
		ENTITY_ATTR_END
	);

	set_local_sound_effect_sample_indices (en, sample_count, sample_indices);

	#if DEBUG_MODULE

	debug_log ("SOUNDEFF : created effect %s (%d), parent %s (%d), next %d, looping %d, valid %d",
							application_sound_effects [sample_indices [0]].name,
							get_local_entity_index (en),
							get_local_entity_type_name (parent),
							get_local_entity_index (parent),
							looping,
							valid);

	#endif

	set_force_local_entity_create_stack_attributes (create_stack_attributes);

	ENV_FXE (ENV_FXE_CREATED, en)

	return en;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

entity *create_client_server_sound_effect_entity
			(
				entity *parent,
				entity_sides side,
				entity_sub_types sub_type,
				sound_channel_types channel,
				sound_locality_types locality,
				vec3d *position,
				float amp,
				float pitch,
				int valid,
				int looping,
				int sample_count,
				sound_sample_indices *sample_indices
			)
{
	entity
		*en;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (side < NUM_ENTITY_SIDES);

	en = create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		parent,
		side,
		sub_type,
		channel,
		locality,
		position,
		amp,
		pitch,
		valid,
		looping,
		sample_count,
		sample_indices
	);

	transmit_entity_comms_message
	(
		ENTITY_COMMS_CREATE_SOUND_EFFECT,
		NULL,
		get_local_entity_safe_index (en),
		parent,
		side,
		sub_type,
		channel,
		locality,
		position,
		amp,
		pitch,
		valid,
		looping,
		sample_count,
		sample_indices
	);

	return en;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

float adjust_radio_message_amplification (float amp, vec3d *pos)
{
	float
		d,
		range;

	if (!get_gunship_entity ())
	{
		return amp;
	}

	ASSERT (pos);

	range = get_approx_3d_range (get_local_entity_vec3d_ptr (get_gunship_entity (), VEC3D_TYPE_POSITION), pos);

	if (range > 120 * KILOMETRE)
	{
		return 0.0;
	}

	if (range < 1 * KILOMETRE)
	{
		return amp;
	}

	d = ((120 * KILOMETRE) - range) / ((120 * KILOMETRE) - (1 * KILOMETRE));

	return (d * amp);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////




