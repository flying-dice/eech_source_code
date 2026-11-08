/* Private dedicated-process sociable world. Real vehicle readiness/system/
 * vector/fire, selection, create, model detach and host D2 birth. Environmental
 * inputs: one compiled Bradley TOW package still loaded, a stationary Abrams
 * target 1 km ahead, clear LOS/flat terrain, initial ready state and timer.
 * No fire decision, angle, selection or result is scripted. Three fixed update
 * intervals accommodate the original previous-frame vectors/readiness order;
 * this is not native/campaign engagement or natural-moving evidence. */
#ifdef EECH_NORMAL_FIRE_CHECK
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
#include "project.h"
#include "eech_headless.h"
#include "entity/mobile/mb_hostaim.h"
#include "entity/mobile/mb_hostlos.h"
extern int eech_nf_check(const char *,char *,int);
extern int eech_wpn_events_get(char *,int);
static unsigned next_id;
static int bind(const campaign_member_descriptor *d,char *id,size_t n,char *e,size_t en,void *u) {
    (void)d;(void)e;(void)en;(void)u;snprintf(id,n,"eech-world:%u",++next_id);return 0;
}
static int clear_los(const campaign_los_step *q,campaign_los_result *r,char *e,size_t n,void *u) {
    (void)q;(void)e;(void)n;(void)u;memset(r,0,sizeof(*r));r->state=CAMPAIGN_LOS_DONE;r->visible=1;return 0;
}
static entity *slot(int index,entity_types type,size_t size) {
    entity *e=get_free_entity(index);if(!e) abort();e->type=type;set_local_entity_data(e,calloc(1,size));return e;
}
static routed_vehicle *vehicle_at(entity *e,int subtype,float z) {
    routed_vehicle *r=(routed_vehicle *)get_local_entity_data(e);
    r->vh.mob.sub_type=subtype;r->vh.mob.alive=TRUE;r->vh.mob.side=ENTITY_SIDE_BLUE_FORCE;
    r->vh.mob.position.x=500;r->vh.mob.position.y=100;r->vh.mob.position.z=z;
    get_identity_matrix3x3(r->vh.mob.attitude);
    r->vh.object_3d_shape=vehicle_database[subtype].default_3d_shape;
    r->vh.inst3d=construct_3d_object(r->vh.object_3d_shape);
    r->vh.weapon_config_type=vehicle_database[subtype].default_weapon_config_type;
    r->vh.damage_level=vehicle_database[subtype].initial_damage_level;
    r->vh.weapon_package_status_array=calloc(NUM_WEAPON_PACKAGES,sizeof(weapon_package_status));
    return r;
}
int eech_nf_normal_check(const char *directory,campaign_aim_fn callback,campaign_env_fn environment,void *user,char *out,int cap) {
    jmp_buf fatal; jmp_buf *previous=eech_fatal_target;
    entity *source,*target,*g,*s,*up,*sess,*born,*view;
    routed_vehicle *raw,*traw;int i,package=-1,before,after,base,used=0;
    campaign_env_host env_host={environment,NULL};
    campaign_aim_host aim_host={callback,user};campaign_los_host los_host={clear_los,NULL};
    campaign_roster_host roster={bind,NULL,NULL};char events[30000];
    base=eech_nf_check(directory,out,cap);if(base) return base;
    used=(int)strlen(out);eech_fatal_target=&fatal;
    if(setjmp(fatal)) {campaign_aim_set_host(NULL);snprintf(out+used,(size_t)(cap-used),"SETUP normal fatal: %s\n",eech_fatal_message);eech_fatal_target=previous;return -1;}
    initialise_target_point_offsets();
    initialise_entity_heap(4096);campaign_roster_reset();initialise_entity_message_responses();
    initialise_smoke_list_database();initialise_meta_smoke_list_database();
    set_comms_model(COMMS_MODEL_SERVER);set_comms_data_flow(COMMS_DATA_FLOW_RX);
    system_delta_time=0.1f;
    source=slot(40,ENTITY_TYPE_ROUTED_VEHICLE,sizeof(routed_vehicle));target=slot(41,ENTITY_TYPE_ROUTED_VEHICLE,sizeof(routed_vehicle));
    g=slot(400,ENTITY_TYPE_GROUP,sizeof(group));s=slot(404,ENTITY_TYPE_SECTOR,sizeof(sector));
    up=slot(405,ENTITY_TYPE_UPDATE,sizeof(update));sess=slot(406,ENTITY_TYPE_SESSION,sizeof(session));
    view=slot(407,ENTITY_TYPE_CAMERA,sizeof(camera));set_camera_entity(view);
    set_update_entity(up);set_session_entity(sess);
    world_map.sector_side_length=4096;world_map.num_map_x_sectors=1;world_map.num_map_z_sectors=1;world_map.num_map_sectors=1;
    world_map.min_map_x=world_map.min_map_z=0;world_map.max_map_x=world_map.max_map_z=4095;
    world_map.min_map_y=-100;world_map.max_map_y=10000;
    entity_sector_map=calloc(1,sizeof(*entity_sector_map));entity_sector_map[0]=s;
    raw=vehicle_at(source,ENTITY_SUB_TYPE_VEHICLE_M2A2_BRADLEY,500);traw=vehicle_at(target,ENTITY_SUB_TYPE_VEHICLE_M1A2_ABRAMS,1500);
    raw->vh.mob.sector_link.parent=s;traw->vh.mob.sector_link.parent=s;traw->vh.mob.side=ENTITY_SIDE_RED_FORCE;
    raw->vh.mob.target_link.parent=target;
    raw->vh.weapon_system_ready_state=2;raw->vh.target_fire_timer=0.01f;
    for(i=0;i<NUM_WEAPON_PACKAGES;i++) if(weapon_config_database[raw->vh.weapon_config_type][i].sub_type==ENTITY_SUB_TYPE_WEAPON_BGM_71D_TOW_2) {package=i;break;}
    if(package<0) {snprintf(out+used,(size_t)(cap-used),"SETUP no compiled TOW package\n");eech_fatal_target=previous;return -1;}
    raw->vh.selected_weapon=ENTITY_SUB_TYPE_WEAPON_BGM_71D_TOW_2;
    /* The compiled defaults are not a flight-ready configuration (zero motor
     * powers/mass). Reuse the accepted D2 aim.rs::flying_gun motor-domain
     * fixture exactly, in BOTH C's figures and the reference host. These are
     * explicit synthetic physical inputs, never retail-effective GWUT facts. */
    {weapon_data *f=&weapon_database[ENTITY_SUB_TYPE_WEAPON_BGM_71D_TOW_2];
     f->start_weight=5;f->final_weight=5;f->diameter=0.1f;
     f->boost_time=1;f->sustain_time=2;f->boost_power=1;f->sustain_power=1;}
    raw->vh.weapon_package_status_array[package].number=weapon_config_database[raw->vh.weapon_config_type][package].number;
    campaign_roster_set_host(&roster);campaign_roster_link_member(g,source);campaign_roster_link_member(g,target);
    campaign_aim_set_host(&aim_host);campaign_los_set_host(&los_host);
    campaign_env_set_host(&env_host);
    /* One private physical sample descriptor; no retail/audio asset or backend.
     * The real sound entity constructor and reference lifetime kernel use it. */
    {int sample=weapon_database[ENTITY_SUB_TYPE_WEAPON_BGM_71D_TOW_2].exterior_launch_sound_effect;
     application_sound_effects[sample].rate=application_sound_effects[sample].original_rate=22050;
     application_sound_effects[sample].size=22050;}
    supply_weapon_figures();
    before=raw->vh.weapon_package_status_array[package].number;
    for(i=0;i<3;i++) {
        update_vehicle_weapon_system_ready(source);
        update_entity_weapon_systems(source);
        update_entity_weapon_system_weapon_and_target_vectors(source);
        update_vehicle_weapon_fire(source);
    }
    after=raw->vh.weapon_package_status_array[package].number;
    born=get_local_entity_first_child(source,LIST_TYPE_LAUNCHED_WEAPON);
    /* Two real D2 steps of the actual normal-born weapon: original unviewed
     * state, then the existing camera child relation. This private physical
     * control exercises both precision paths; it is not a native policy input. */
    if(born) {
        weapon_movement(born);
        insert_local_entity_into_parents_child_list(born,LIST_TYPE_VIEW_WEAPON,view,NULL);
        weapon_movement(born);
    }
    campaign_aim_set_host(NULL);
    eech_wpn_events_get(events,sizeof(events));
    snprintf(out+used,(size_t)(cap-used),"normal source=%s target=%s package=%d ammo=%d->%d ready=%d vectors=%d birth=%d\n%s",
        campaign_roster_host_id(source),campaign_roster_host_id(target),package,before,after,raw->vh.selected_weapon_system_ready,raw->vh.weapon_and_target_vectors_valid,born!=NULL,events);
    eech_fatal_target=previous;
    if(!born || before<=after || get_local_entity_parent(born,LIST_TYPE_TARGET)!=target ||
        get_local_entity_int_value(born,INT_TYPE_ENTITY_SUB_TYPE)!=ENTITY_SUB_TYPE_WEAPON_BGM_71D_TOW_2 ||
        !strstr(events,"kind=normal_fire") || !strstr(events,"kind=normal_birth") || !strstr(events,"matched=1") || strstr(events,"kind=normal_error")) return 1;
    return 0;
}
#endif
