/* Private dedicated-process collaborator control. The real synthetic model
 * writer/loader, constructor, search, viewpoint and detach bodies run. Only the
 * environment (one launcher, model device availability and flat terrain) is
 * controlled. This is not a campaign boot, normal-fire or native eligibility
 * claim. No campaign policy or fire gate is replaced. */
#ifdef EECH_NORMAL_FIRE_CHECK
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
#include "project.h"
#include "eech_headless.h"
#include "eech_synth3d.h"
#include "entity/mobile/mb_hostterrain.h"

extern int eech_wpn_events_get(char *,int);
static int terrain_queries;
static int flat(const campaign_terrain_request *q,campaign_terrain_reply *r,char *e,size_t n,void *u) {
    (void)e;(void)n;(void)u;
    if(q->op!=CAMPAIGN_TERRAIN_POINT) return 1;
    memset(r,0,sizeof(*r)); r->normal[1]=1; r->type=TERRAIN_TYPE_LAND;
    ++terrain_queries;return 0;
}
/* Device availability is an environmental property of the actual model DB.
 * The original search/constructor code remains the collaborator, including
 * the original fallback when one named device is absent. */
static void named(object_3d_scene_database_entry *s,int index,int present) {
    int i;for(i=0;i<s->number_of_sub_object_table_entries;i++) {
        int v=s->sub_object_indices_table[i].sub_object_index;
        if(v==index || v==-index) s->sub_object_indices_table[i].sub_object_index=present?index:-index;
    }
}
int eech_nf_check(const char *directory,char *out,int cap) {
    jmp_buf fatal; jmp_buf *previous=eech_fatal_target;
    entity *pool; routed_vehicle *raw; weapon_package_status *status;
    const vehicle_data *v=&vehicle_database[ENTITY_SUB_TYPE_VEHICLE_M1A2_ABRAMS];
    object_3d_scene_database_entry *scene;
    int package,weapon_type,i,k,used=0,failures=0;
    campaign_terrain_host host={flat,NULL};
    char events[24000];
    static const int missing[]={0,OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_HEADING,OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_PITCH,
        OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_WEAPON,OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_MUZZLE};
    eech_fatal_target=&fatal;
    if(setjmp(fatal)) {snprintf(out,cap,"SETUP fatal: %s\n",eech_fatal_message);eech_fatal_target=previous;return -1;}
    if(!eech_synth3d_write(directory)) {snprintf(out,cap,"SETUP synthetic write failed\n");eech_fatal_target=previous;return -1;}
    initialise_file_system();
    load_texturemap_data(directory);
    initialise_3d_objects(directory);
    initialise_entity_functions();
    pool=calloc(MAX_NUM_ENTITIES,sizeof(*pool));raw=calloc(1,sizeof(*raw));status=calloc(NUM_WEAPON_PACKAGES,sizeof(*status));
    if(!pool||!raw||!status) abort();
    entities=pool;number_of_entities=MAX_NUM_ENTITIES;
    pool[40].type=ENTITY_TYPE_ROUTED_VEHICLE;set_local_entity_data(&pool[40],raw);
    raw->vh.mob.sub_type=ENTITY_SUB_TYPE_VEHICLE_M1A2_ABRAMS;
    raw->vh.mob.position.x=500;raw->vh.mob.position.y=100;raw->vh.mob.position.z=500;
    get_3d_transformation_matrix(raw->vh.mob.attitude,0,0,0);
    raw->vh.weapon_config_type=v->default_weapon_config_type;
    raw->vh.object_3d_shape=v->default_3d_shape;
    raw->vh.weapon_package_status_array=status;
    world_map.min_map_x=world_map.min_map_z=0;world_map.max_map_x=world_map.max_map_z=999;
    world_map.min_map_y=-100;world_map.max_map_y=1000;
    campaign_terrain_set_host(&host);
    package=-1;
    for(i=0;i<NUM_WEAPON_PACKAGES;i++) if(weapon_config_database[v->default_weapon_config_type][i].sub_type==v->default_weapon_type) {package=i;break;}
    if(package<0) {snprintf(out,cap,"SETUP no default package\n");eech_fatal_target=previous;return -1;}
    weapon_type=v->default_weapon_type;
    scene=&objects_3d_scene_database[v->default_3d_shape];
    for(k=0;k<5;k++) {
        viewpoint vp;int before,after;int same;
        for(i=0;i<5;i++) if(missing[i]) named(scene,missing[i],1);
        if(missing[k]) named(scene,missing[k],0);
        memset(status,0,NUM_WEAPON_PACKAGES*sizeof(*status));
        status[package].number=weapon_config_database[v->default_weapon_config_type][package].number;
        before=status[package].number;
        raw->vh.inst3d=construct_3d_object(v->default_3d_shape);
        detach_local_entity_weapon(&pool[40],weapon_type,1,&vp);
        after=status[package].number;
        same=memcmp(&vp.position,&raw->vh.mob.position,sizeof(vp.position))==0;
        if(before-after!=1 || (k==1||k==2? !same:same)) ++failures;
        used+=snprintf(out+used,(size_t)(cap-used),"detach case=%d ammo=%d->%d origin_fallback=%d position=%.9g,%.9g,%.9g\n",k,before,after,same,(double)vp.x,(double)vp.y,(double)vp.z);
        destruct_3d_object(raw->vh.inst3d);raw->vh.inst3d=NULL;
    }
    for(i=0;i<5;i++) if(missing[i]) named(scene,missing[i],1);
#ifdef EECH_WEAPON_FIXTURE
    eech_wpn_events_get(events,sizeof(events));
#else
    events[0]=0;
#endif
    used+=snprintf(out+used,(size_t)(cap-used),"collaborator_checks=5 terrain_queries=%d failures=%d\n%s",terrain_queries,failures,events);
    /* This is deliberately required even without the observer: the same real
     * behavior can pass while missing correlation is a meaningful RED. */
    if(!strstr(events,"kind=normal_detach")) {snprintf(out+strlen(out),(size_t)(cap-strlen(out)),"MISSING_OBSERVATION normal_detach\n");++failures;}
    eech_fatal_target=previous;
    return failures;
}

/* Only the NEW observer's own noninterference, not a replay of D2 math or a
 * whole-entry FPU guarantee. Four caller modes with nonzero status flags. */
#if defined(EECH_NORMAL_FIRE_CORRELATION) && (defined(__i386__) || defined(__x86_64__))
#include <fenv.h>
#include <errno.h>
#include "entity/mobile/mb_normalfirefx.h"
struct nf_fp {unsigned short control,status;unsigned mxcsr;};
static void nf_fp_read(struct nf_fp *f) {
    __asm__ __volatile__("fnstcw %0":"=m"(f->control));
    __asm__ __volatile__("fnstsw %0":"=m"(f->status));
    __asm__ __volatile__("stmxcsr %0":"=m"(f->mxcsr));
}
int eech_nf_observer_check(char *out,int cap) {
    fenv_t saved;unsigned mx;int mode,fail=0,terrain_before=terrain_queries;vec3d v={0,0,1};float angle=0,maximum=1;
    char empty[32000];viewpoint pose;weapon_package_status status;object_3d_sub_instance station;float flash;uint32_t flash_word;
    memset(&pose,0,sizeof(pose));get_identity_matrix3x3(pose.attitude);
    memset(&station,0,sizeof(station));memset(&status,0,sizeof(status));
    fegetenv(&saved);__asm__ __volatile__("stmxcsr %0":"=m"(mx));
    /* The previous drain consumed its records, not capture ownership. */
    if(eech_wpn_events_get(empty,sizeof(empty)) || empty[0]) ++fail;
    for(mode=0;mode<4;mode++) {
        unsigned short cw,sw;unsigned csr;unsigned char env[28];struct nf_fp before,after;int seed=get_random_number_seed();
        __asm__ __volatile__("fnstcw %0":"=m"(cw));cw=(unsigned short)((cw&~0x0c00u)|((unsigned)mode<<10)|0x3fu);__asm__ __volatile__("fldcw %0"::"m"(cw));
        __asm__ __volatile__("stmxcsr %0":"=m"(csr));csr=(csr&~0x6000u)|((unsigned)mode<<13)|0x1f80u|0x34u;__asm__ __volatile__("ldmxcsr %0"::"m"(csr));
        __asm__ __volatile__("fnstenv %0":"=m"(env));memcpy(&sw,env+4,2);sw|=0x34u;memcpy(env+4,&sw,2);__asm__ __volatile__("fldenv %0"::"m"(env));
        errno=EDOM;nf_fp_read(&before);
        /* Unrostered heap slot: a lazy identity/host request would fail here.
         * These direct hook calls are observer controls, never fire evidence. */
        status.number=2;station.visible_object=1;flash_word=0;memcpy(&flash,&flash_word,sizeof(flash));
        eech_nf_fire(&entities[42],NULL,1,1,&v,&v,&angle,&maximum);
        eech_nf_detach_begin(&entities[42],1,1);
        eech_nf_package(WEAPON_CONFIG_TYPE_M1A2_ABRAMS_1,1,&status);
        eech_nf_device(15);eech_nf_station(&station);eech_nf_flash(&flash);
        /* These integer/raw stores model effects only for the hook's own
         * noninterference control; the real effects run in the five cases. */
        status.number=1;station.visible_object=0;flash_word=0xbf800000u;memcpy(&flash,&flash_word,sizeof(flash));
        eech_nf_fallback(1);eech_nf_fallback(2);
        eech_nf_detach_end(&entities[42],&pose);
        eech_nf_fire_end(&entities[42]);
        nf_fp_read(&after);
        if(before.control!=after.control || before.status!=after.status || before.mxcsr!=after.mxcsr || errno!=EDOM || seed!=get_random_number_seed()) ++fail;
    }
    fesetenv(&saved);__asm__ __volatile__("ldmxcsr %0"::"m"(mx));
    /* A drained stream still records the later calls, exactly once. */
    eech_wpn_events_get(empty,sizeof(empty));
    {int n=0;const char *p=empty;while((p=strstr(p,"kind=normal_fire"))) {++n;++p;}if(n!=4) ++fail;}
    if(eech_wpn_events_get(empty,sizeof(empty)) || empty[0]) ++fail;
    if(terrain_queries!=terrain_before) ++fail;
    snprintf(out,cap,"observer_checks=4 rounding_modes nonzero_x87_mxcsr_flags errno shared_rng no_aim_provider_no_terrain_traffic_and_drain failures=%d\n",fail);
    return fail;
}
#else
int eech_nf_observer_check(char *out,int cap) {snprintf(out,cap,"observer check requires correlation and x86\n");return -1;}
#endif
#endif
