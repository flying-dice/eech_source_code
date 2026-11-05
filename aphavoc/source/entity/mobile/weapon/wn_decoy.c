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
#include "entity/mobile/mb_hostaim.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE 0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//
// THE DECOY (EEDCS-45)
//
// A decoy's timer, and what a seeker locked on its launcher does when the decoy becomes active, are the host's (the timer restarts at a time drawn from the shared stream;
// a seeker is drawn to a decoy by the decoy's kind, its own and a sample of the same stream).  The campaign keeps who is locked on whom (it tells those seekers that the
// decoy became active), the player's difficulty (it goes in as a coefficient), and the link it makes when the host says a seeker goes for the decoy.
//

static void refuse_decoy_lookup (const campaign_aim_event *lookup, campaign_aim_event *answer, void *context)
{
	debug_fatal ("host aim: the host asked a decoy request a lookup (kind %d) it does not take", lookup->kind);
}

// the player's difficulty makes a decoy less effective (a setting of the campaign, given to the host as a figure; the AI has none)
float get_decoy_difficulty_coefficient (entity *sender)
{
	float
		coefficient;

	game_difficulty_settings
		difficulty;

	entity
		*launcher,
		*pilot;

	coefficient = 1.0;

	// get game difficulty level (use player's own level)

	launcher = get_local_entity_parent (sender, LIST_TYPE_LAUNCHED_WEAPON);

	if (launcher)
	{
		if (get_local_entity_int_value (launcher, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			pilot = get_local_entity_first_child (launcher, LIST_TYPE_AIRCREW);

			if (pilot)
			{
				difficulty = (game_difficulty_settings) get_local_entity_int_value (pilot, INT_TYPE_DIFFICULTY_LEVEL);

				if (difficulty == GAME_DIFFICULTY_EASY) {
					coefficient = 0.1;
				}
				else if (difficulty == GAME_DIFFICULTY_MEDIUM) {
					coefficient = 0.5;
				}
			}
		}
	}

	return (coefficient);
}

void update_decoy (entity *en)
{
	weapon
		*raw;

	entity
		*launcher,
		*locked_on_weapon,
		*succ;

	campaign_aim_step
		step;

	campaign_aim_result
		result;

	int
		i;

	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (weapon *) get_local_entity_data (en);

	ASSERT (weapon_database[raw->mob.sub_type].decoy_type != WEAPON_DECOY_TYPE_NONE);

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_WEAPON_DECOY;

	step.weapon = raw->mob.sub_type;

	step.delta_time = get_delta_time ();

	step.weapon_state.decoy_timer = raw->decoy_timer;

	step.weapon_state.server = TRUE;

	result = campaign_aim_weapon_step (&step, refuse_decoy_lookup, NULL);

	for (i = 0; i < result.action_count; i++)
	{
		if (result.actions[i].kind != CAMPAIGN_STEP_ACTION_DECOY)
		{
			debug_fatal ("host aim: a decoy update left an action of kind %d the campaign does not apply to a decoy", result.actions[i].kind);
		}

		raw->decoy_timer = result.actions[i].f[0];

		if (result.actions[i].a)
		{
			launcher = get_local_entity_parent (en, LIST_TYPE_LAUNCHED_WEAPON);

			if (launcher)
			{
				locked_on_weapon = get_local_entity_first_child (launcher, LIST_TYPE_TARGET);

				while (locked_on_weapon)
				{
					succ = get_local_entity_child_succ (locked_on_weapon, LIST_TYPE_TARGET);

					notify_local_entity (ENTITY_MESSAGE_DECOY_ACTIVE, locked_on_weapon, en);

					locked_on_weapon = succ;
				}
			}
		}
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

entity_sub_types get_decoy_type_for_weapon (entity *en)
{
	weapon
		*raw;

	entity_sub_types
		weapon_sub_type;

	ASSERT (en);

	raw = (weapon *) get_local_entity_data (en);

	switch (weapon_database[raw->mob.sub_type].guidance_type)
	{
		////////////////////////////////////////
		case WEAPON_GUIDANCE_TYPE_NONE:
		////////////////////////////////////////
		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

			break;
		}
		////////////////////////////////////////
		case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_RADAR:
		case WEAPON_GUIDANCE_TYPE_ACTIVE_RADAR:
		case WEAPON_GUIDANCE_TYPE_RADIO_COMMAND:
		////////////////////////////////////////
		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_CHAFF;

			break;
		}
		////////////////////////////////////////
		case WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED:
		////////////////////////////////////////
		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_FLARE;

			break;
		}
		////////////////////////////////////////
		case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER:
		case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER_BEAM_RIDING:
		case WEAPON_GUIDANCE_TYPE_ACTIVE_LASER:
		case WEAPON_GUIDANCE_TYPE_WIRE_GUIDED:
		////////////////////////////////////////
		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_SMOKE_GRENADE;

			break;
		}
		////////////////////////////////////////
		default:
		////////////////////////////////////////
		{
			debug_fatal ("Invalid weapon guidance type = %d", weapon_database[raw->mob.sub_type].guidance_type);

			break;
		}
	}

	return (weapon_sub_type);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
