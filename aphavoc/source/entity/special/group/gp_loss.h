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

// Campaign work the world's kill and destroy functions used to do inline. They
// call these at the same points, so the world still drives them: this is the
// remaining world -> campaign coupling, kept explicit until a reviewed
// host-authority change. None of these checks the comms model; callers keep
// their own server checks.

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// The campaign consequences of one member's physical loss, as phases. Each family's
// kill_local names the phase at the point its original inline code ran, so the
// interleaving with the physical work is unchanged; the campaign decides here whether
// the phase runs now (the host forwarded the loss) or at the host's report (withheld).
// Phases of one member are applied at most once and in ascending order.

typedef enum campaign_loss_phase
{
	CAMPAIGN_LOSS_PHASE_FORCE_INFO = 1,
	CAMPAIGN_LOSS_PHASE_REGEN,
	CAMPAIGN_LOSS_PHASE_MEMBER_TASKS,
	CAMPAIGN_LOSS_PHASE_LANDING,
	CAMPAIGN_LOSS_PHASE_GROUP_LOSS,
	CAMPAIGN_LOSS_PHASE_TASK_LOSS,
	CAMPAIGN_LOSS_PHASE_MEMBER_KILLED,
	CAMPAIGN_LOSS_PHASE_OBJECTIVES
} campaign_loss_phase;

// the live-entity consequence of one phase; the immediate (forwarded) loss path

extern void apply_member_loss_phase (entity *member, entity *group, int phase);

// a lost member's consequences

extern void notify_member_task_dependents_completed (entity *member);

extern void increment_group_losses (entity *group);

extern void increment_group_primary_task_losses (entity *group);

extern void increment_task_losses (entity *task);

// must be called after the member is unlinked from the group's member list

extern void notify_group_member_killed (entity *group, entity *member);

extern int apply_group_member_killed (entity *group, unsigned int member_number);

extern void notify_forces_check_campaign_objectives (entity *member);

extern void check_all_force_campaign_objectives (void);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// detaching a member from the campaign's lists, on kill and on destroy

extern void unlink_member_task_dependents (entity *member);

extern void unlink_member_from_guide (entity *member);

extern void unlink_member_from_group (entity *member);

extern void unlink_member_from_takeoff_queue (entity *member);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
