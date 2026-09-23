//
// Entity list sorting.
//
// C provenance: entity/en_misc/en_misc.c :: qs, quicksort_entity_list
//
// The original quicksort, not a library sort: it orders by descending
// sort_order, is not stable, and its middle pivot fixes where equal keys end
// up. assign.c :: assign_keysite_tasks assigns tasks in exactly this order.
//

import { ASSERT } from "../../core/assert";
import { cIntDivide } from "../../core/cint";
import type { Entity } from "../system/entity";

// C provenance: en_misc.c :: static void qs (entity **en_list, float *sort_order, int left, int right)
function qs(en_list: Entity[], sort_order: number[], left: number, right: number): void {
	let i = left;
	let j = right;

	const x = sort_order[cIntDivide(left + right, 2)];

	do {
		while (sort_order[i] > x && i < right) i++;
		while (x > sort_order[j] && j > left) j--;

		if (i <= j) {
			const y = sort_order[i];
			sort_order[i] = sort_order[j];
			sort_order[j] = y;

			const temp = en_list[i];
			en_list[i] = en_list[j];
			en_list[j] = temp;

			i++;
			j--;
		}
	} while (i <= j);

	if (left < j) qs(en_list, sort_order, left, j);
	if (i < right) qs(en_list, sort_order, i, right);
}

// C provenance: en_misc.c :: quicksort_entity_list (en_list, count, sort_order)
export function quicksortEntityList(en_list: Entity[] | undefined, count: number, sort_order: number[]): void {
	ASSERT(en_list !== undefined, "en_list");

	qs(en_list, sort_order, 0, count - 1);
}
