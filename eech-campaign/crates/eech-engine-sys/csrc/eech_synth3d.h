/* EECH headless build: the synthetic 3D object database (eech_synth3d.c, eech_synth3d_keysites.c) */
#ifndef EECH_SYNTH3D_H
#define EECH_SYNTH3D_H

/* writes bininfo.bin, 3dobjs.bin (.pts .sp .spn .spt) and 3dobjdb.bin into directory; 1 on success */
int eech_synth3d_write (const char *directory);

struct synth_scene;

/* airports and FARPs: routes and buildings (eech_synth3d_keysites.c) */
void eech_synth3d_keysites (struct synth_scene *scenes);

#endif
