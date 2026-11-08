/* TEST-ONLY read-only normal-fire correlation. No argument is evaluated in an
 * ordinary build. No launch operation, physical answer or reset is exposed. */
#ifndef MB_NORMALFIREFX_H
#define MB_NORMALFIREFX_H
#ifdef EECH_NORMAL_FIRE_CORRELATION
void eech_nf_fire (entity *source, entity *target, int weapon, int ready,
    const vec3d *actual, const vec3d *intercept, const float *angle, const float *maximum);
void eech_nf_fire_end (entity *source);
void eech_nf_detach_begin (entity *source, int weapon, int burst);
void eech_nf_package (int config, int package, const weapon_package_status *status);
void eech_nf_device (int device);
void eech_nf_station (const object_3d_sub_instance *station);
void eech_nf_flash (const float *timer);
void eech_nf_fallback (int kind);
void eech_nf_detach_end (entity *source, const viewpoint *vp);
#define NF_FIRE(S,T,W,R,A,I,E,M) eech_nf_fire ((S),(T),(W),(R),(A),(I),(E),(M));
#define NF_FIRE_END(S) eech_nf_fire_end ((S));
#define NF_DETACH_BEGIN(S,W,B) eech_nf_detach_begin ((S),(W),(B));
#define NF_PACKAGE(C,P,S) eech_nf_package ((C),(P),(S));
#define NF_DEVICE(D) eech_nf_device ((D));
#define NF_STATION(S) eech_nf_station ((S));
#define NF_FLASH(T) eech_nf_flash ((T));
#define NF_FALLBACK(K) eech_nf_fallback ((K));
#define NF_DETACH_END(S,V) eech_nf_detach_end ((S),(V));
#else
#define NF_FIRE(S,T,W,R,A,I,E,M)
#define NF_FIRE_END(S)
#define NF_DETACH_BEGIN(S,W,B)
#define NF_PACKAGE(C,P,S)
#define NF_DEVICE(D)
#define NF_STATION(S)
#define NF_FLASH(T)
#define NF_FALLBACK(K)
#define NF_DETACH_END(S,V)
#endif
#endif
