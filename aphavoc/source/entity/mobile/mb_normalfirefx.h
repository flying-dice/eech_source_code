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
void eech_nf_step_begin (entity *en);
void eech_nf_step_end (entity *en);
void eech_nf_query_begin (entity *en, int which, const vec3d *p, const vec3d *q);
void eech_nf_query_end (entity *en, entity *hit);
void eech_nf_model_entry (int op);
#define NF_FIRE(S,T,W,R,A,I,E,M) eech_nf_fire ((S),(T),(W),(R),(A),(I),(E),(M));
#define NF_FIRE_END(S) eech_nf_fire_end ((S));
#define NF_DETACH_BEGIN(S,W,B) eech_nf_detach_begin ((S),(W),(B));
#define NF_PACKAGE(C,P,S) eech_nf_package ((C),(P),(S));
#define NF_DEVICE(D) eech_nf_device ((D));
#define NF_STATION(S) eech_nf_station ((S));
#define NF_FLASH(T) eech_nf_flash ((T));
#define NF_FALLBACK(K) eech_nf_fallback ((K));
#define NF_DETACH_END(S,V) eech_nf_detach_end ((S),(V));
#define NF_STEP_BEGIN(E) eech_nf_step_begin ((E));
#define NF_STEP_END(E) eech_nf_step_end ((E));
#define NF_QUERY_BEGIN(E,W,P,Q) eech_nf_query_begin ((E),(W),(P),(Q));
#define NF_QUERY_END(E,H) eech_nf_query_end ((E),(H));
#define NF_MODEL_ENTRY(O) eech_nf_model_entry ((O));
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
#define NF_STEP_BEGIN(E)
#define NF_STEP_END(E)
#define NF_QUERY_BEGIN(E,W,P,Q)
#define NF_QUERY_END(E,H)
#define NF_MODEL_ENTRY(O)
#endif
#endif
