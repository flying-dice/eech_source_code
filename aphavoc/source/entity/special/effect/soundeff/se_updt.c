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

#include "entity/special/session/ss_hostenv.h"

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE 0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_CAMPAIGN_HOST_ENV

// the sound system's answer to the host's one question about a server's expired effect: is the handle still playing (the handle and the device are the campaign's)
static int host_handle_playing (void *handle)
{
	return get_system_sound_effect_playing ((system_sound_effect *) handle);
}

#endif

static void update_server (entity *en)
{

	sound_effect
		*raw;

	raw = (sound_effect *) get_local_entity_data (en);

	ENV_FXE (ENV_FXE_SERVER_ENTER, en)

	ASSERT (raw->effect_index);

	ASSERT (raw->eff.special_effect_link.parent);

	#if DEBUG_MODULE >= 2

	debug_log ("SE_UPDT : %s - lifetime %f, valid %d, valid lifetime %f",
	 								application_sound_effects [raw->effect_index [0]].name,
									raw->effect_lifetime,
									raw->valid_sound_effect,
									raw->valid_effect_lifetime);
	#endif

	#ifdef EECH_CAMPAIGN_HOST_ENV

	//
	// EECH_CAMPAIGN_HOST_ENV (EEDCS-51, D3): the physical decisions of this update are the HOST's (aphavoc/source/entity/special/session/ss_hostenv.h SOUND_LIFETIME_UPDATE): the aging of a valid
	// effect by the frame delta, whether its lifetime went below zero, whether the server kills the entity (no handle, or a handle that is not playing), and the countdown and unlink of an invalid effect.
	// The campaign hands over the stored fields by value (and the frame delta where the original read it), writes both lifetimes from the checked answer and then does what the host decided, in the original
	// order and at the original points: the kill, the unlink from the update list, the release of the handle.  The handle and the sound system stay the campaign's; the original follows in the #else.
	//

	{
		int
			expired,
			action;

		campaign_sound_lifetime_update (TRUE, get_delta_time (), raw->valid_sound_effect, raw->looping, &raw->effect_lifetime, &raw->valid_effect_lifetime, raw->sound_effect_data, host_handle_playing, &expired, &action);

		if (expired)
		{
			if (raw->sound_effect_data)
			{
				if (action == CAMPAIGN_SOUND_ACTION_KILL)
				{
					ENV_FXE (ENV_FXE_SERVER_EXPIRED_IDLE, en)

					kill_client_server_entity (en);

					ENV_FXE (ENV_FXE_SERVER_KILLED, en)
				}
				else
				{
					ENV_FXE (ENV_FXE_SERVER_EXPIRED_PLAYING, en)
				}
			}
			else
			{
				ENV_FXE (ENV_FXE_SERVER_EXPIRED_NO_HANDLE, en)

				if (action == CAMPAIGN_SOUND_ACTION_KILL)
				{
					kill_client_server_entity (en);

					ENV_FXE (ENV_FXE_SERVER_KILLED, en)
				}
			}
		}
		else if (action == CAMPAIGN_SOUND_ACTION_UNLINK)
		{
			delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);

			if (raw->sound_effect_data)
			{
				destroy_system_sound_effect (raw->sound_effect_data);

				raw->sound_effect_data = NULL;
			}
		}
	}

	#else

	if (raw->valid_sound_effect)
	{
		//
		// effect_lifetime
		//

		if (!raw->looping)
		{
			raw->effect_lifetime -= get_delta_time ();
		}
	
		if (raw->effect_lifetime < 0.0)
		{

			#if DEBUG_MODULE

			debug_log ("SE_UPDT : Expired ( index = %s, pos = %f, %f, lifetime %f)", application_sound_effects [raw->effect_index [0]].name, raw->eff.position.x, raw->eff.position.z, raw->effect_lifetime);
	
			#endif

			if (raw->sound_effect_data)
			{
				if (!get_system_sound_effect_playing (raw->sound_effect_data))
				{
					ENV_FXE (ENV_FXE_SERVER_EXPIRED_IDLE, en)

					kill_client_server_entity (en);

					ENV_FXE (ENV_FXE_SERVER_KILLED, en)
				}
				else
				{
					ENV_FXE (ENV_FXE_SERVER_EXPIRED_PLAYING, en)

					#if DEBUG_MODULE

					debug_log ("SE_UPDT : Not destroying because it's still playing ( index = %s, pos = %f, %f )", application_sound_effects [raw->effect_index [0]].name, raw->eff.position.x, raw->eff.position.z);
	
					#endif
				}
			}
			else
			{
				ENV_FXE (ENV_FXE_SERVER_EXPIRED_NO_HANDLE, en)

				kill_client_server_entity (en);

				ENV_FXE (ENV_FXE_SERVER_KILLED, en)
			}
		}
	}
	else
	{
		if (raw->valid_effect_lifetime > 0.0)
		{
			raw->valid_effect_lifetime = max (raw->valid_effect_lifetime - get_delta_time (), 0.0f);
		}
		else
		{
			delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);

			if (raw->sound_effect_data)
			{
				destroy_system_sound_effect (raw->sound_effect_data);

				raw->sound_effect_data = NULL;
			}
		}
	}
	#endif

}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static void update_client (entity *en)
{
	sound_effect
		*raw;

	raw = (sound_effect *) get_local_entity_data (en);

	ENV_FXE (ENV_FXE_CLIENT_ENTER, en)

	#ifdef EECH_CAMPAIGN_HOST_ENV

	//
	// EECH_CAMPAIGN_HOST_ENV (EEDCS-51, D3): as update_server, for the client routine: the host ages a valid effect and clamps its lifetime at zero (the client never kills), and counts an invalid effect
	// down and decides its unlink; the campaign writes the checked answer and unlinks and releases the handle as decided.  The original follows in the #else.
	//

	{
		int
			expired,
			action;

		campaign_sound_lifetime_update (FALSE, get_delta_time (), raw->valid_sound_effect, raw->looping, &raw->effect_lifetime, &raw->valid_effect_lifetime, raw->sound_effect_data, host_handle_playing, &expired, &action);

		if (action == CAMPAIGN_SOUND_ACTION_UNLINK)
		{
			delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);

			if (raw->sound_effect_data)
			{
				destroy_system_sound_effect (raw->sound_effect_data);

				raw->sound_effect_data = NULL;
			}
		}
	}

	#else

	if (raw->valid_sound_effect)
	{
		if (!raw->looping)
		{
			raw->effect_lifetime -= get_delta_time ();

			if (raw->effect_lifetime < 0.0)
			{
				raw->effect_lifetime = 0.0;
			}
		}
	}
	else
	{
		if (raw->valid_effect_lifetime > 0.0)
		{
			raw->valid_effect_lifetime = max (raw->valid_effect_lifetime - get_delta_time (), 0.0f);
		}
		else
		{
			delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);

			if (raw->sound_effect_data)
			{
				destroy_system_sound_effect (raw->sound_effect_data);

				raw->sound_effect_data = NULL;
			}
		}
	}

	#endif

	ENV_FXE (ENV_FXE_CLIENT_EXIT, en)
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void overload_sound_effect_update_functions (void)
{
	fn_update_client_server_entity[ENTITY_TYPE_SOUND_EFFECT][COMMS_MODEL_SERVER]  	= update_server;

	fn_update_client_server_entity[ENTITY_TYPE_SOUND_EFFECT][COMMS_MODEL_CLIENT]  	= update_client;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
