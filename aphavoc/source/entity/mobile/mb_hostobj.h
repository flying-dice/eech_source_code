//
// 	Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	677 High Road, North Finchley, London N12 0DA
//
// 	Please see the document LICENSE.TXT for the full licence agreement
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
//  	following rights in respect of the Executable Code:
//   2.2.1
//   	the non-exclusive right to Exploit the Executable Code on any
//   	medium; and
//   2.2.2
//   	the non-exclusive right to distribute the Executable Code.
//  2.3
//  	You agree to Exploit the Source Code, Executable Code and Derivative Works
//  	only in accordance with the terms of this Agreement.
//

// The host-issued identity of the physical objects the host must act on or report that are not rostered mobiles (EEDCS-46, R3), and the physical facts the host reports about them.
//
// The roster (gp_roster.c) already gives every physical group member an opaque identity issued by the host.  A STRUCTURE (a site, a site's updatable, a city building, a pylon, a piece of scenery, a
// bridge segment, a bridge) had none: a blast or a bridge walk could only offer it by a handle that exists for that walk alone.  This module is the structures' roster: the host issues an identity
// (CAMPAIGN_AIM_IDENTIFY, the first time the campaign has to name the structure to it), the campaign copies it and never parses or derives it, and the identity ends with the physical incarnation, when
// the entity is released (set_free_entity).  The identity is not the entity's heap index: that index is reused, an identity never is.  An identity the host issued for one incarnation can therefore
// never name another object, and one that is unknown, ended or of a mobile is refused by name.
//
// What the host REPORTS (campaign_object_report_apply) is a physical fact about an object it has the identity of: its damage level is now N, and/or it is destroyed.  The campaign applies it with the
// same two steps the collision response of the object uses (set the damage level; kill the object when it is destroyed) and so every existing consequence follows from the real handlers: the kill of a
// rostered mobile goes through the loss observation and the host-ID loss transaction as any other kill, the kill of a bridge segment through the bridge's collision, the collapse questions and the
// vehicles walk.  A fact is applied once: a report about an object that is already destroyed, or one that would raise its damage level, is refused.  Who destroyed it (credit, awareness) is the
// campaign's own policy and is not something a report can claim.

#ifndef MB_HOSTOBJ_H
#define MB_HOSTOBJ_H

#include <stddef.h>

struct ENTITY;

#define CAMPAIGN_OBJECT_ID_MAX 128

typedef struct campaign_object_report
{
	char host_id[CAMPAIGN_OBJECT_ID_MAX + 1];
	/* the damage level the object now has (never above the level it had), when the host reports one */
	int has_damage_level;
	int damage_level;
	/* the host reports the object destroyed (it is killed once; a damage level of zero or below is destruction too) */
	int destroyed;
} campaign_object_report;

/* What one report applied, copied from the performed campaign steps: an observation, never an input */
typedef struct campaign_object_report_receipt
{
	int entity_type;
	int rostered;
	int damage_level_before;
	int damage_level_after;
	int killed;
} campaign_object_report_receipt;

typedef struct campaign_object_snapshot
{
	/* BORROWED from the registry: valid only inside the callback it is passed to (identifying the next structure may grow the registry and move it); the callback copies it */
	const char *host_id;
	int entity_type;
	int sub_type;
	int side;
	int alive;
	int damage_level;
	int initial_damage_level;
	float position[3];
} campaign_object_snapshot;

typedef void (*campaign_object_snapshot_fn) (const campaign_object_snapshot *object, void *user);

/* whether the kind of entity is a structure the host is given an identity for */
int campaign_object_is_structure (int entity_type);

/* The identity the host issued for the object, or NULL: a rostered mobile's (the roster's) or a structure's, never calling the host */
const char *campaign_object_host_id (struct ENTITY *en);

/* As above, but a structure that has none yet is identified now by asking the host (CAMPAIGN_AIM_IDENTIFY); NULL for an object that is not a structure or a rostered mobile */
const char *campaign_object_identify (struct ENTITY *en);

/* Whether the identity was ever issued for a structure, live or ended (never forgotten, like the roster's).  The host's identities are ONE namespace that is never reused: a group member bound later
   may not take one (gp_rosterid.c, before the roster commits the binding), and a structure may not take a member's (campaign_object_identify). */
int campaign_object_id_ever_issued (const char *host_id);

/* the entity is released: its identity ends with it (called by set_free_entity) */
void campaign_object_release_entity (struct ENTITY *en);

/* forget every identity (engine boot) */
void campaign_object_reset (void);

/* Apply a physical fact the host reports.  Stops with a clear error, before changing anything, for an identity that is unknown, ended, or not of a structure or a rostered mobile; for an object that
   is already destroyed; for a damage level above the current one or below zero; and for a report that says nothing */
void campaign_object_report_apply (const campaign_object_report *report, campaign_object_report_receipt *receipt);

/* Every structure that exists, identified (the host is asked for those that have no identity yet), in entity-list order, as copied values; returns how many */
int campaign_object_structures (campaign_object_snapshot_fn callback, void *user);

#endif
