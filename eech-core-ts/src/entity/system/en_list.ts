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

import { ASSERT, UnportedBehaviourError } from "../../core/assert";
import { EntityType, ListType } from "../../generated/c-enums";
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
