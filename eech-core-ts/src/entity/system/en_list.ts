//
// Entity lists.
//
// C provenance: entity/system/en_funcs/en_list.c, en_list/get_frst.h, get_prnt.h,
//               get_succ.h, get_pred.h and the per type xx_list.c files, which
//               select the list roots and links each entity type owns.
//
// A root holds `first_child`; a link holds `parent`, `child_succ` and
// `child_pred`. Several list types can share one link: a group's
// LIST_TYPE_BUILDING_GROUP, LIST_TYPE_INDEPENDENT_GROUP and
// LIST_TYPE_KEYSITE_GROUP all resolve to `group_link` (get_prnt.h), so
// get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP) returns the parent of
// whichever of those lists the group is in. That aliasing is preserved.
//

import { ASSERT, EechFatalError, UnportedBehaviourError } from "../../core/assert";
import { EntityMessage } from "../../generated/c-enums";
import { EntityType, ListType } from "../../generated/c-enums";
import { notifyLocalEntity } from "./en_msgs";
import type { Entity, EntityListLink, EntityListRoot } from "./entity";

interface ListLayout {
	roots: Record<number, string>;
	links: Record<number, string>;
}

const listLayouts: Record<number, ListLayout> = {};

function getListLayout(entityType: EntityType): ListLayout {
	let layout = listLayouts[entityType];

	if (layout === undefined) {
		layout = { roots: {}, links: {} };
		listLayouts[entityType] = layout;
	}

	return layout;
}

// Declares that entities of `entityType` own the list root `slot` for `listTypes`
// (C: #define LIST_TYPE_xxx_ROOT in xx_list.c).
export function overloadEntityListRoot(entityType: EntityType, slot: string, listTypes: ListType[]): void {
	const layout = getListLayout(entityType);

	for (const listType of listTypes) {
		layout.roots[listType] = slot;
	}
}

// Declares that entities of `entityType` own the list link `slot` for `listTypes`
// (C: #define LIST_TYPE_xxx_LINK in xx_list.c).
export function overloadEntityListLink(entityType: EntityType, slot: string, listTypes: ListType[]): void {
	const layout = getListLayout(entityType);

	for (const listType of listTypes) {
		layout.links[listType] = slot;
	}
}

function getRoot(en: Entity, type: ListType): EntityListRoot {
	const slot = getListLayout(en.type).roots[type];

	if (slot === undefined) {
		throw new UnportedBehaviourError(`list root [${EntityType[en.type]}] [${ListType[type]}]`);
	}

	let root = en.roots[slot];

	if (root === undefined) {
		root = { first_child: undefined };
		en.roots[slot] = root;
	}

	return root;
}

function getLink(en: Entity, type: ListType): EntityListLink {
	const slot = getListLayout(en.type).links[type];

	if (slot === undefined) {
		throw new UnportedBehaviourError(`list link [${EntityType[en.type]}] [${ListType[type]}]`);
	}

	let link = en.links[slot];

	if (link === undefined) {
		link = { parent: undefined, child_succ: undefined, child_pred: undefined };
		en.links[slot] = link;
	}

	return link;
}

// C provenance: en_list.c :: get_local_entity_first_child
export function getLocalEntityFirstChild(en: Entity, type: ListType): Entity | undefined {
	return getRoot(en, type).first_child;
}

// C provenance: en_list.c :: get_local_entity_parent
export function getLocalEntityParent(en: Entity, type: ListType): Entity | undefined {
	return getLink(en, type).parent;
}

// C provenance: en_list.c :: get_local_entity_child_succ
export function getLocalEntityChildSucc(en: Entity, type: ListType): Entity | undefined {
	return getLink(en, type).child_succ;
}

// C provenance: en_list.c :: get_local_entity_child_pred
export function getLocalEntityChildPred(en: Entity, type: ListType): Entity | undefined {
	return getLink(en, type).child_pred;
}

// The setters keep the ASSERT at the top of en_list/set_frst.h, set_prnt.h,
// set_succ.h and set_pred.h (included by every xx_list.c): an entity never
// links to itself.

function setLocalEntityFirstChild(en: Entity, type: ListType, first_child: Entity | undefined): void {
	ASSERT(en !== first_child, "en != first_child");

	getRoot(en, type).first_child = first_child;
}

// C provenance: en_list.c :: set_local_entity_parent
export function setLocalEntityParent(en: Entity, type: ListType, parent: Entity | undefined): void {
	ASSERT(en !== parent, "en != parent");

	getLink(en, type).parent = parent;
}

function setLocalEntityChildSucc(en: Entity, type: ListType, child_succ: Entity | undefined): void {
	ASSERT(en !== child_succ, "en != child_succ");

	getLink(en, type).child_succ = child_succ;
}

// C provenance: en_list.c :: set_local_entity_child_pred
export function setLocalEntityChildPred(en: Entity, type: ListType, child_pred: Entity | undefined): void {
	ASSERT(en !== child_pred, "en != child_pred");

	getLink(en, type).child_pred = child_pred;
}

//
// C provenance: en_list.c :: insert_local_entity_into_parents_child_list
//
// The #ifdef DEBUG list validation is kept, with EECH's debug-build meaning
// (as ASSERT is): inserting an entity into a list it is already in is fatal.
//
export function insertLocalEntityIntoParentsChildList(en: Entity, type: ListType, parent: Entity | undefined, pred: Entity | undefined): void {
	ASSERT(parent !== undefined, "parent");

	ASSERT(pred !== parent, "pred != parent");

	const current_parent = getLocalEntityParent(en, type);

	if (current_parent) {
		let item = getLocalEntityFirstChild(current_parent, type);

		while (item) {
			if (item === en) {
				throw new EechFatalError(
					"Entity already in list (entity type = %s, list type = %s)",
					`Entity already in list (entity type = ${EntityType[en.type]}, list type = ${ListType[type]})`,
				);
			}

			item = getLocalEntityChildSucc(item, type);
		}
	}

	const succ = pred ? getLocalEntityChildSucc(pred, type) : getLocalEntityFirstChild(parent, type);

	setLocalEntityChildSucc(en, type, succ);

	setLocalEntityChildPred(en, type, pred);

	setLocalEntityParent(en, type, parent);

	if (succ) {
		setLocalEntityChildPred(succ, type, en);
	}

	if (pred) {
		setLocalEntityChildSucc(pred, type, en);
	} else {
		setLocalEntityFirstChild(parent, type, en);
	}

	notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, parent, en, type);

	notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_PARENT, en, parent, type);
}

// C provenance: en_list.c :: delete_local_entity_from_parents_child_list
export function deleteLocalEntityFromParentsChildList(en: Entity, type: ListType): void {
	const parent = getLocalEntityParent(en, type);

	if (parent) {
		notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, parent, en, type);

		notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT, en, parent, type);

		const succ = getLocalEntityChildSucc(en, type);

		const pred = getLocalEntityChildPred(en, type);

		if (pred) {
			setLocalEntityChildSucc(pred, type, succ);
		} else {
			setLocalEntityFirstChild(parent, type, succ);
		}

		if (succ) {
			setLocalEntityChildPred(succ, type, pred);
		}

		setLocalEntityParent(en, type, undefined);

		setLocalEntityChildSucc(en, type, undefined);

		setLocalEntityChildPred(en, type, undefined);
	}
}

// C provenance: en_list.c :: unlink_local_entity_children
export function unlinkLocalEntityChildren(en: Entity, list: ListType): void {
	let child = getLocalEntityFirstChild(en, list);

	while (child) {
		deleteLocalEntityFromParentsChildList(child, list);

		child = getLocalEntityFirstChild(en, list);
	}
}

//
// Establishes list membership exactly as the pointer updates of
// en_list.c :: insert_local_entity_into_parents_child_list, but without its
// ENTITY_MESSAGE_LINK_CHILD / ENTITY_MESSAGE_LINK_PARENT notifications. This is
// the state a restored campaign has after en_pack.c unpacks list pointers.
// insert_local_entity_into_parents_child_list itself is unported: group,
// keysite, task, regen and waypoint respond to those messages.
//
export function insertLocalEntityIntoParentsChildListRaw(en: Entity, type: ListType, parent: Entity, pred: Entity | undefined): void {
	ASSERT(pred !== parent, "pred != parent");

	const link = getLink(en, type);

	ASSERT(link.parent === undefined, "entity not already in a list sharing this link");

	const succ = pred !== undefined ? getLocalEntityChildSucc(pred, type) : getLocalEntityFirstChild(parent, type);

	link.child_succ = succ;
	link.child_pred = pred;
	link.parent = parent;

	if (succ !== undefined) {
		getLink(succ, type).child_pred = en;
	}

	if (pred !== undefined) {
		getLink(pred, type).child_succ = en;
	} else {
		getRoot(parent, type).first_child = en;
	}
}
