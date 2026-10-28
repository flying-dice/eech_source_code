#!/bin/sh
# usage: inventory-kill-is-destroy.sh <aphavoc/source/entity dir>: a SOURCE inventory (not a run) for the vehicles-walk guard of sg_dstry.c (vehicle_resumable, EEDCS-46 R5).  For every entity type and
# both comms models it reads, from the *_dstry.c files, the function bound to fn_kill_client_server_entity and to fn_destroy_client_server_entity, and prints only the bindings where they are THE SAME
# function (the kill IS the destruction: the object is unlinked and freed, so a walk cannot go on through it), marking the kinds a blast window offers (explosn.c blast_readable).  A binding made any
# other way than a plain assignment in those files is not seen; the guard itself compares the tables at run time, so it does not depend on this list.
set -u
dir=${1:?usage: inventory-kill-is-destroy.sh <entity dir>}
offered=" ANTI_AIRCRAFT CITY_BUILDING FIXED_WING HELICOPTER PERSON PYLON ROUTED_VEHICLE SCENIC SEGMENT SHIP_VEHICLE SITE SITE_UPDATABLE WEAPON "
bindings=$(grep -rhE 'fn_(kill|destroy)_client_server_entity[[:space:]]*\[ENTITY_TYPE_[A-Z_]+\]\[COMMS_MODEL_(SERVER|CLIENT)\][[:space:]]*=' "$dir" --include=*_dstry.c |
  sed -E 's/^[[:space:]]*fn_(kill|destroy)_client_server_entity[[:space:]]*\[ENTITY_TYPE_([A-Z_]+)\]\[COMMS_MODEL_(SERVER|CLIENT)\][[:space:]]*=[[:space:]]*([A-Za-z_0-9]+).*/\1 \2 \3 \4/')
echo "bindings read: $(echo "$bindings" | wc -l)"
echo "$bindings" | awk '$1 == "kill" { print $2, $3, $4 }' | sort | while read -r type model kfn; do
  dfn=$(echo "$bindings" | awk -v t="$type" -v m="$model" '$1 == "destroy" && $2 == t && $3 == m { print $4 }')
  if [ "$kfn" = "$dfn" ]; then
    case "$offered" in *" $type "*) mark="  [offered by a blast window]: NOT RESUMABLE";; *) mark="";; esac
    printf '%s %-16s kill=destroy=%s%s\n' "$model" "$type" "$kfn" "$mark"
  fi
done
echo "kinds offered by a blast window:$(echo $offered | sed "s/^/ /")"
