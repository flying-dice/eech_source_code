#!/bin/sh
# task-only: symbol checks of the shipping candidate (EEDCS-44), inside the repository build image (read-only use of the build volume)
# usage (in the image): symcheck-aim.sh   with the build volume at /target and the output directory at /out
NM=x86_64-w64-mingw32-nm
objdir=/target/x86_64-pc-windows-gnu/release/build
objects() { find $objdir -name "$1" -path '*eech-engine-sys*' | head -1; }
for name in mb_hostaim weapon wn_move en_anim ac_anim vh_anim ps_anim rv_hostmv; do
	o=$(objects "*$name*.o")
	echo "=== $name: $o"
	[ -n "$o" ] || continue
	echo "--- undefined math and solver symbols referenced by $name.o"
	$NM "$o" | grep " U " | sed 's/^ *U //' | grep -E "^(sqrt|atan2|asin|acos|sin|cos|pow|floor|fabs|normalise_3d_vector|get_3d_transformation|multiply_transpose|get_keyframed_animation_state|set_entity_keyframe_value|update_entity_reversable_keyframed_value)" | sort -u | tr '\n' ' '
	echo
done
echo "=== eech_dc.dll: the aiming kernel and the fixture are not in the shipping module (kernel symbols are in eech-world.exe)"
for n in eech_aim_ready_update eech_aim_ready_command eech_aim_required eech_aim_step eech_aim_ballistic eech_aim_lead eech_aim_vectors eech_aim_intercept eech_aim_person_face eech_aim_trace eech_aim_replay_echo ballistic_pitch_deflection get_intercept_point; do
	printf "%s: dll=%s exe=%s\n" $n "$($NM /out/eech_dc.dll 2>/dev/null | grep -c " $n\$")" "$($NM /out/eech-world.exe 2>/dev/null | grep -c " $n\$")"
done
echo "=== eech_dc.dll symbols named campaign_aim"
$NM /out/eech_dc.dll 2>/dev/null | grep -E "campaign_aim" | sed 's/^[0-9a-f]* //'
echo "=== eech_dc.dll: the original solvers by name (get_ballistic_pitch_deflection stays as the thin campaign-side call)"
$NM /out/eech_dc.dll 2>/dev/null | grep -E "get_ballistic_pitch_deflection|get_lead_and_ballistic|get_target_intercept_point|update_entity_weapon_systems|update_aircraft_weapon_system_ready|open_entity_weapon_system_ready" | sed 's/^[0-9a-f]* //'
